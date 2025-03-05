use crate::owntracks::Location;
//use chrono::{DateTime, FixedOffset, Local};
use serde::{ser::Error, Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;
use sqlx::migrate::{MigrateDatabase, Migrator};
use sqlx::{AnyPool, Sqlite};

static MIGRATOR: Migrator = sqlx::migrate!();

/// Track identification
#[derive(sqlx::FromRow, Deserialize, Debug)]
pub struct TrackId {
    pub user: String,
    pub device: String,
    pub ts_start: String,
}

#[derive(sqlx::FromRow, Debug)]
pub struct TrackData {
    pub user: String,
    pub device: String,
    pub date: String, // time::Date,
    pub points: Vec<GpsPoint>,
}

#[derive(sqlx::FromRow, Debug)]
pub struct GpsPoint {
    pub y: f64,
    pub x: f64,
    /// Timestamp in format 2025-02-19 06:46:54+00
    pub ts: String, // DateTime<FixedOffset> is not supported by Any driver
    pub speed: Option<i16>,
    pub elevation: Option<i16>,
    /// Accuracy in meters
    pub accuracy: Option<i32>, // owntracks: u32
    /// Vertical accuracy in meters
    pub v_accuracy: Option<i16>,
}

#[derive(sqlx::FromRow, Serialize, Debug)]
pub struct TrackInfo {
    pub user: String,
    pub device: String,
    pub date: String,     // time::Date is not supported by Any driver
    pub ts_start: String, // DateTime<FixedOffset> is not supported by Any driver
    pub ts_end: String,   // DateTime<FixedOffset> is not supported by Any driver
    pub speed_min: Option<i16>,
    pub speed_max: Option<i16>,
    pub elevation_min: Option<i16>,
    pub elevation_max: Option<i16>,
}

impl TrackId {
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
                CREATE SEQUENCE IF NOT EXISTS gpslog_id_seq;
                ALTER TABLE gpslog ALTER COLUMN id SET DEFAULT NEXTVAL ('gpslog_id_seq');
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
            r#"INSERT INTO gpslog
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
                user_id as "user",
                device,
                date(gpslog.ts, 'unixepoch') as date,
                datetime(min(gpslog.ts), 'unixepoch') as ts_start,
                datetime(max(gpslog.ts), 'unixepoch') as ts_end,
                min(gpslog.velocity) as speed_min,
                max(gpslog.velocity) as speed_max,
                min(gpslog.alt) as elevation_min,
                max(gpslog.alt) as elevation_max
            FROM gpslog
            JOIN devices ON gpslog.device_id = devices.id
            WHERE date(gpslog.ts, 'unixepoch') = $1
            GROUP BY device_id, user_id, device, date(gpslog.ts, 'unixepoch')"#,
        )
        .bind(date)
        .fetch_all(&self.pool)
        .await?;

        // Sort the tracks by ts_end in descending order
        tracks.sort_by(|a, b| b.ts_end.cmp(&a.ts_end));

        Ok(tracks)
    }

    /// Query a single track by its ID.
    pub async fn query_track(&self, track_id: &TrackId) -> anyhow::Result<TrackData> {
        let date = track_id.date();
        let points: Vec<GpsPoint> = sqlx::query_as(
            r#"
                SELECT
                    gpslog.lat as y,
                    gpslog.lon as x,
                    datetime(gpslog.ts, 'unixepoch') AS ts,
                    gpslog.velocity as speed,
                    gpslog.alt as elevation,
                    gpslog.accuracy,
                    gpslog.v_accuracy
                FROM gpslog
                JOIN devices ON gpslog.device_id = devices.id
                WHERE date(gpslog.ts, 'unixepoch') = $1
                AND user_id = $2
                AND device = $3
                ORDER BY gpslog.id
                "#,
        )
        .bind(&date)
        .bind(&track_id.user)
        .bind(&track_id.device)
        .fetch_all(&self.pool)
        .await?;

        let gps_points = points
            .into_iter()
            .map(|p| GpsPoint {
                y: p.y,
                x: p.x,
                ts: p.ts,
                speed: p.speed,
                elevation: p.elevation,
                accuracy: p.accuracy,
                v_accuracy: p.v_accuracy,
            })
            .collect();

        let track = TrackData {
            user: track_id.user.clone(),
            device: track_id.device.clone(),
            date,
            points: gps_points,
        };

        Ok(track)
    }

    /// Return tracks of a given date
    pub async fn query_tracks(&self, date: &str) -> anyhow::Result<Vec<TrackData>> {
        // First get the unique user/device combinations for the date
        // Alternative as single query:
        // SELECT user, device, ts::date, array_agg((lat, lon, ts, velocity, alt, accuracy, v_accuracy) ORDER BY id) AS points
        // WHERE ts::date = ?                                                                                                                                                                                                                                                                                                                                           ║
        // GROUP BY user, device, ts::date
        let user_devices: Vec<TrackId> = sqlx::query_as(
            r#"
            SELECT DISTINCT "user", device, datetime(min(ts), 'unixepoch') AS ts_start
            FROM gpslog
            WHERE date(ts, 'unixepoch') = date($1)
            GROUP BY "user", device
            "#,
        )
        .bind(date)
        .fetch_all(&self.pool)
        .await?;

        let mut tracks = Vec::new();

        for track_id in user_devices {
            let track = self.query_track(&track_id).await?;
            if !track.points.is_empty() {
                tracks.push(track);
            }
        }

        Ok(tracks)
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
