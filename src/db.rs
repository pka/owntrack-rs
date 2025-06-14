use crate::owntracks::Location;
use serde::{ser::Error, Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;
use sqlx::migrate::{MigrateDatabase, Migrator};
use sqlx::{AnyPool, Sqlite};

static MIGRATOR: Migrator = sqlx::migrate!();

/// Track identification
#[derive(sqlx::FromRow, Deserialize, Debug)]
pub struct TrackRef {
    pub device_id: i32,
    pub ts_start: String,
    /// Query segmented track
    pub segmented: Option<bool>,
}

#[derive(sqlx::FromRow, Debug)]
pub struct TrackData {
    pub device_id: i32,
    pub date: String, // time::Date,
    pub points: Vec<GpsPoint>,
}

#[derive(sqlx::FromRow, Debug)]
pub struct GpsPoint {
    pub y: f64,
    pub x: f64,
    /// Timestamp in format 2025-02-19 06:46:54+00
    pub ts: String, // DateTime<FixedOffset> is not supported by Any driver
    pub tid: String,
    pub speed: Option<i16>,
    pub elevation: Option<i16>,
    /// Accuracy in meters
    pub accuracy: Option<i32>, // owntracks: u32
    /// Vertical accuracy in meters
    pub v_accuracy: Option<i16>,
    pub cog: Option<i16>,
    pub annotations: String,
}

#[derive(sqlx::FromRow, Debug)]
pub struct Position {
    pub device_id: i32,
    pub y: f64,
    pub x: f64,
    /// Timestamp in format 2025-02-19 06:46:54+00
    pub ts: String, // DateTime<FixedOffset> is not supported by Any driver
    pub tid: String,
    pub speed: Option<i16>,
    pub elevation: Option<i16>,
    /// Accuracy in meters
    pub accuracy: Option<i32>, // owntracks: u32
    /// Vertical accuracy in meters
    pub v_accuracy: Option<i16>,
    pub cog: Option<i16>,
}

#[derive(sqlx::FromRow, Serialize, Debug)]
pub struct TrackInfo {
    pub device_id: i32,
    pub user_id: String,
    pub device: String,
    pub tid: String,
    pub ts_start: String, // DateTime<FixedOffset> is not supported by Any driver
    pub ts_end: String,   // DateTime<FixedOffset> is not supported by Any driver
}

impl TrackRef {
    pub fn date(&self) -> String {
        // from timestamp in format 2025-02-19 06:46:54+00
        self.ts_start.split(' ').next().unwrap().to_string()
    }
}

#[derive(Clone)]
pub struct Db {
    pool: AnyPool,
}

impl Db {
    pub async fn connect() -> anyhow::Result<Self> {
        let conn_str =
            dotenvy::var("DB_CONNECTION").unwrap_or("sqlite://tracking.sqlite".to_string());
        if conn_str.starts_with("sqlite:")
            && !Sqlite::database_exists(&conn_str).await.unwrap_or(false)
        {
            log::info!("Creating database {conn_str}");
            Sqlite::create_database(&conn_str).await?;
        }
        sqlx::any::install_default_drivers();
        log::info!("Connecting to database...");
        let pool = AnyPool::connect(&conn_str).await?;
        Ok(Db { pool })
    }

    pub async fn run_migrations(&self) -> anyhow::Result<()> {
        log::info!("Running database migrations...");
        MIGRATOR.run(&self.pool).await?;
        let is_pg = self.pool.acquire().await?.backend_name() == "PostgreSQL";
        if is_pg {
            let _result = sqlx::raw_sql(
                r#"
                CREATE SEQUENCE IF NOT EXISTS devices_id_seq;
                ALTER TABLE devices ALTER COLUMN id SET DEFAULT NEXTVAL ('devices_id_seq');
                CREATE SEQUENCE IF NOT EXISTS positions_id_seq;
                ALTER TABLE positions ALTER COLUMN id SET DEFAULT NEXTVAL ('positions_id_seq');
                -- SQLite comaptible date/time functions
                CREATE OR REPLACE FUNCTION unixepoch(bigint, varchar(20)) RETURNS TIMESTAMPTZ
                    AS 'select to_timestamp($1);'
                    LANGUAGE SQL
                    IMMUTABLE;
                CREATE OR REPLACE FUNCTION date(TIMESTAMPTZ, varchar(20)) RETURNS VARCHAR
                    AS 'select $1::DATE::VARCHAR;'
                    LANGUAGE SQL
                    IMMUTABLE;
                CREATE OR REPLACE FUNCTION datetime(TIMESTAMPTZ, varchar(20)) RETURNS VARCHAR
                    AS 'select $1::VARCHAR;'
                    LANGUAGE SQL
                    IMMUTABLE;
                "#,
            )
            .execute(&self.pool)
            .await?;
        }
        let _result = sqlx::raw_sql(
            r#"
            CREATE INDEX IF NOT EXISTS positions_date_device_idx
                ON positions (date(ts, 'unixepoch'), device_id);
            "#,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn insert_location(
        &self,
        user: &str,
        device: &str,
        loc: &Location,
    ) -> anyhow::Result<()> {
        // Upsert device location
        let device_id: i64 = sqlx::query_scalar(r#"
            INSERT INTO devices (user_id, device, tid, ts, velocity, lat, lon, alt, accuracy, v_accuracy, cog)
            VALUES ($1, $2, $3, unixepoch($4, 'unixepoch'), $5, $6, $7, $8, $9, $10, $11)
            ON CONFLICT(user_id, device) DO UPDATE
            SET tid=$3, ts=unixepoch($4, 'unixepoch'), velocity=$5, lat=$6, lon=$7, alt=$8, accuracy=$8, v_accuracy=$10, cog=$11
            RETURNING id"#
        )
        .bind(user)
        .bind(device)
        .bind(&loc.tid)
        .bind(loc.ts)
        .bind(loc.velocity.map(|val| val as i32)) // u16 is not supported by Any driver
        .bind(loc.lat)
        .bind(loc.lon)
        .bind(loc.alt.map(|val| val as i32)) // u16 is not supported by Any driver
        .bind(loc.accuracy.map(|val| val as i64)) // u32 is not supported by Any driver
        .bind(loc.v_accuracy)
        .bind(loc.cog)
        .fetch_one(&self.pool)
        .await?;

        sqlx::query(
            r#"INSERT INTO positions
             (device_id, tid, ts, velocity, lat, lon, alt, accuracy, v_accuracy, cog, annotations)
              VALUES ($1, $2, unixepoch($3, 'unixepoch'), $4, $5, $6, $7, $8, $9, $10, $11)"#,
        )
        .bind(device_id)
        .bind(&loc.tid)
        .bind(loc.ts)
        .bind(loc.velocity.map(|val| val as i32)) // u16 is not supported by Any driver
        .bind(loc.lat)
        .bind(loc.lon)
        .bind(loc.alt.map(|val| val as i32)) // u16 is not supported by Any driver
        .bind(loc.accuracy.map(|val| val as i64)) // u32 is not supported by Any driver
        .bind(loc.v_accuracy)
        .bind(loc.cog)
        .bind(&loc.annotations)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Return track infos of a given date
    pub async fn query_tracks_info(&self, date: &str) -> anyhow::Result<Vec<TrackInfo>> {
        let mut tracks: Vec<TrackInfo> = sqlx::query_as(
            r#"SELECT
                device_id,
                user_id,
                device,
                devices.tid,
                datetime(min(positions.ts), 'unixepoch') as ts_start,
                datetime(max(positions.ts), 'unixepoch') as ts_end
            FROM positions
            JOIN devices ON positions.device_id = devices.id
            WHERE date(positions.ts, 'unixepoch') = $1
            GROUP BY device_id, user_id, device, devices.tid"#,
        )
        .bind(date)
        .fetch_all(&self.pool)
        .await?;

        // Sort the tracks by ts_end in descending order
        tracks.sort_by(|a, b| b.ts_end.cmp(&a.ts_end));

        Ok(tracks)
    }

    /// Query a single track
    pub async fn query_track(&self, track_ref: &TrackRef) -> anyhow::Result<TrackData> {
        let date = track_ref.date();
        let points: Vec<GpsPoint> = sqlx::query_as(
            r#"
                SELECT
                    lat as y,
                    lon as x,
                    datetime(ts, 'unixepoch') AS ts,
                    tid,
                    velocity as speed,
                    alt as elevation,
                    accuracy,
                    v_accuracy,
                    cog,
                    annotations
                FROM positions
                WHERE date(ts, 'unixepoch') = $1
                AND device_id = $2
                ORDER BY id
                "#,
        )
        .bind(&date)
        .bind(track_ref.device_id)
        .fetch_all(&self.pool)
        .await?;

        let track = TrackData {
            device_id: track_ref.device_id,
            date,
            points,
        };

        Ok(track)
    }

    /// Return last device postitions
    pub async fn query_positions(&self, date: &str) -> anyhow::Result<Vec<Position>> {
        let positions: Vec<Position> = sqlx::query_as(
            r#"
            SELECT
                id as device_id,
                lat as y,
                lon as x,
                datetime(ts, 'unixepoch') AS ts,
                tid,
                velocity as speed,
                alt as elevation,
                accuracy,
                v_accuracy,
                cog
            FROM devices
            WHERE date(ts, 'unixepoch') = $1
            "#,
        )
        .bind(date)
        .fetch_all(&self.pool)
        .await?;

        Ok(positions)
    }

    /// Check validity of invite
    pub async fn is_valid_invite(&self) -> anyhow::Result<bool> {
        // Check URL token for friend invites (TODO)
        // For initial setup check existence of devices
        let initial_device = sqlx::query_scalar(
            r#"
                SELECT COUNT(*) = 0
                FROM devices
                "#,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(initial_device)
    }
}

pub fn serialize_raw_json<S: Serializer>(v: &str, s: S) -> Result<S::Ok, S::Error> {
    let v: serde_json::Value =
        serde_json::from_str(v).map_err(|_| Error::custom("error parsing serialized json"))?;
    v.serialize(s)
}

pub fn deserialize_dict_to_string<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<String, D::Error> {
    let dict = Value::deserialize(deserializer)?;
    match dict {
        Value::Object(_) => Ok(dict.to_string()),
        _ => Err(serde::de::Error::custom("expected a JSON object")),
    }
}
