use crate::position::Position;
use chrono::{DateTime, FixedOffset, NaiveDate};
use serde::{ser::Error, Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;
use sqlx::migrate::{MigrateDatabase, Migrator};
use sqlx::{AnyPool, Sqlite};

static MIGRATOR: Migrator = sqlx::migrate!();

/// Track identification
#[derive(sqlx::FromRow, Deserialize, Debug)]
pub struct TrackRef {
    pub device_id: i32,
    /// Track start in UTC, format `2025-02-19 06:46:54+00`
    pub ts_start: String,
    /// Query segmented track
    pub segmented: Option<bool>,
    /// Client's offset from UTC in minutes (e.g. 660 for Australia/Sydney). Defaults to UTC.
    pub tz: Option<i32>,
}

/// Epoch-second window `[start, end)` covering the local calendar day `date`,
/// where local time is `tz_minutes` east of UTC.
pub fn local_day_window(date: NaiveDate, tz_minutes: i32) -> (i64, i64) {
    let midnight_utc = date
        .and_hms_opt(0, 0, 0)
        .expect("00:00:00 is a valid time")
        .and_utc()
        .timestamp();
    let start = midnight_utc - i64::from(tz_minutes) * 60;
    (start, start + 86_400)
}

/// Local calendar date of a UTC timestamp such as `2025-02-19 06:46:54+00`.
pub fn local_date(utc_ts: &str, tz_minutes: i32) -> anyhow::Result<NaiveDate> {
    let utc = DateTime::<FixedOffset>::parse_from_str(utc_ts, "%F %T%#z")?;
    let local_tz = tz_minutes
        .checked_mul(60)
        .and_then(FixedOffset::east_opt)
        .ok_or_else(|| anyhow::anyhow!("timezone offset out of range: {tz_minutes} minutes"))?;
    Ok(utc.with_timezone(&local_tz).date_naive())
}

/// Format a UTC timestamp as `2025-02-19 06:46:54+00`, the format used by the API.
fn utc_stamp(ts: &str) -> String {
    if ts.ends_with("+00") {
        ts.to_string()
    } else {
        format!("{ts}+00")
    }
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
pub struct DevicePosition {
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

    pub async fn insert_position(
        &self,
        user: &str,
        device: &str,
        pos: &Position,
    ) -> anyhow::Result<()> {
        // Upsert device position
        let device_id: i64 = sqlx::query_scalar(r#"
            INSERT INTO devices (user_id, device, tid, ts, velocity, lat, lon, alt, accuracy, v_accuracy, cog)
            VALUES ($1, $2, $3, unixepoch($4, 'unixepoch'), $5, $6, $7, $8, $9, $10, $11)
            ON CONFLICT(user_id, device) DO UPDATE
            SET tid=$3, ts=unixepoch($4, 'unixepoch'), velocity=$5, lat=$6, lon=$7, alt=$8, accuracy=$8, v_accuracy=$10, cog=$11
            RETURNING id"#
        )
        .bind(user)
        .bind(device)
        .bind(&pos.tid)
        .bind(pos.ts)
        .bind(pos.velocity.map(|val| val as i32)) // u16 is not supported by Any driver
        .bind(pos.lat)
        .bind(pos.lon)
        .bind(pos.alt.map(|val| val as i32)) // u16 is not supported by Any driver
        .bind(pos.accuracy.map(|val| val as i64)) // u32 is not supported by Any driver
        .bind(pos.v_accuracy)
        .bind(pos.cog)
        .fetch_one(&self.pool)
        .await?;

        sqlx::query(
            r#"INSERT INTO positions
             (device_id, ts, velocity, lat, lon, alt, accuracy, v_accuracy, cog, annotations)
              VALUES ($1, unixepoch($2, 'unixepoch'), $3, $4, $5, $6, $7, $8, $9, $10)"#,
        )
        .bind(device_id)
        .bind(pos.ts)
        .bind(pos.velocity.map(|val| val as i32)) // u16 is not supported by Any driver
        .bind(pos.lat)
        .bind(pos.lon)
        .bind(pos.alt.map(|val| val as i32)) // u16 is not supported by Any driver
        .bind(pos.accuracy.map(|val| val as i64)) // u32 is not supported by Any driver
        .bind(pos.v_accuracy)
        .bind(pos.cog)
        .bind(&pos.annotations)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Return track infos of a given local calendar day.
    /// `tz_minutes` is the client's offset from UTC; the day is `[00:00, 24:00)` in that offset.
    pub async fn query_tracks_info(
        &self,
        date: NaiveDate,
        tz_minutes: i32,
    ) -> anyhow::Result<Vec<TrackInfo>> {
        let (start, end) = local_day_window(date, tz_minutes);
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
            WHERE positions.ts >= $1 AND positions.ts < $2
            GROUP BY device_id, user_id, device, devices.tid"#,
        )
        .bind(start)
        .bind(end)
        .fetch_all(&self.pool)
        .await?;

        for track in &mut tracks {
            track.ts_start = utc_stamp(&track.ts_start);
            track.ts_end = utc_stamp(&track.ts_end);
        }
        // Sort the tracks by ts_end in descending order
        tracks.sort_by(|a, b| b.ts_end.cmp(&a.ts_end));

        Ok(tracks)
    }

    /// Query a single track: the device's points on the local day its track starts in.
    pub async fn query_track(&self, track_ref: &TrackRef) -> anyhow::Result<TrackData> {
        let tz_minutes = track_ref.tz.unwrap_or(0);
        let date = local_date(&track_ref.ts_start, tz_minutes)?;
        let (start, end) = local_day_window(date, tz_minutes);
        let mut points: Vec<GpsPoint> = sqlx::query_as(
            r#"
                SELECT
                    lat as y,
                    lon as x,
                    datetime(ts, 'unixepoch') AS ts,
                    velocity as speed,
                    alt as elevation,
                    accuracy,
                    v_accuracy,
                    cog,
                    annotations
                FROM positions
                WHERE ts >= $1 AND ts < $2
                AND device_id = $3
                ORDER BY id
                "#,
        )
        .bind(start)
        .bind(end)
        .bind(track_ref.device_id)
        .fetch_all(&self.pool)
        .await?;
        for point in &mut points {
            point.ts = utc_stamp(&point.ts);
        }

        let track = TrackData {
            device_id: track_ref.device_id,
            date: date.to_string(),
            points,
        };

        Ok(track)
    }

    /// Return last device positions whose fix falls on the given local calendar day.
    pub async fn query_positions(
        &self,
        date: NaiveDate,
        tz_minutes: i32,
    ) -> anyhow::Result<Vec<DevicePosition>> {
        let (start, end) = local_day_window(date, tz_minutes);
        let mut positions: Vec<DevicePosition> = sqlx::query_as(
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
            WHERE ts >= $1 AND ts < $2
            "#,
        )
        .bind(start)
        .bind(end)
        .fetch_all(&self.pool)
        .await?;
        for position in &mut positions {
            position.ts = utc_stamp(&position.ts);
        }

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

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::any::AnyPoolOptions;

    fn epoch(rfc3339: &str) -> i64 {
        DateTime::parse_from_rfc3339(rfc3339).unwrap().timestamp()
    }

    fn day(s: &str) -> NaiveDate {
        NaiveDate::parse_from_str(s, "%F").unwrap()
    }

    #[test]
    fn local_day_window_follows_client_offset() {
        // Midnight on 22 Oct in Sydney (UTC+11) is 13:00 UTC on 21 Oct.
        let (start, end) = local_day_window(day("2026-10-22"), 660);
        assert_eq!(start, epoch("2026-10-21T13:00:00Z"));
        assert_eq!(end, epoch("2026-10-22T13:00:00Z"));
    }

    #[test]
    fn local_date_follows_client_offset() {
        // 20:00 UTC on 21 Oct is 07:00 on 22 Oct in Sydney.
        assert_eq!(
            local_date("2026-10-21 20:00:00+00", 660).unwrap(),
            day("2026-10-22")
        );
        assert_eq!(
            local_date("2026-10-21 20:00:00+00", 0).unwrap(),
            day("2026-10-21")
        );
    }

    #[test]
    fn utc_stamp_adds_offset_once() {
        assert_eq!(utc_stamp("2026-10-21 20:00:00"), "2026-10-21 20:00:00+00");
        assert_eq!(
            utc_stamp("2026-10-21 20:00:00+00"),
            "2026-10-21 20:00:00+00"
        );
    }

    const SCHEMA: &str = r#"
        CREATE TABLE devices (id INTEGER PRIMARY KEY, user_id TEXT, device TEXT, tid TEXT,
            ts INTEGER, lat REAL, lon REAL, velocity INTEGER, alt INTEGER, accuracy INTEGER,
            v_accuracy INTEGER, cog INTEGER);
        CREATE TABLE positions (id INTEGER PRIMARY KEY, device_id INTEGER, ts INTEGER,
            lat REAL, lon REAL, velocity INTEGER, alt INTEGER, accuracy INTEGER,
            v_accuracy INTEGER, cog INTEGER, annotations TEXT);
    "#;

    /// One device with fixes at 10:00 UTC 21 Oct (21 Oct in Sydney),
    /// 20:00 UTC 21 Oct and 03:00 UTC 22 Oct (both 22 Oct in Sydney).
    async fn test_db() -> Db {
        sqlx::any::install_default_drivers();
        let pool = AnyPoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();
        sqlx::raw_sql(SCHEMA).execute(&pool).await.unwrap();
        sqlx::query(
            "INSERT INTO devices (id, user_id, device, tid, ts, lat, lon) VALUES (1, 'alice', 'phone', 'AA', $1, -33.8, 151.2)",
        )
        .bind(epoch("2026-10-22T03:00:00Z"))
        .execute(&pool)
        .await
        .unwrap();
        for ts in [
            "2026-10-21T10:00:00Z",
            "2026-10-21T20:00:00Z",
            "2026-10-22T03:00:00Z",
        ] {
            sqlx::query(
                "INSERT INTO positions (device_id, ts, lat, lon, annotations) VALUES (1, $1, -33.8, 151.2, '{}')",
            )
            .bind(epoch(ts))
            .execute(&pool)
            .await
            .unwrap();
        }
        Db { pool }
    }

    #[tokio::test]
    async fn tracks_are_listed_on_their_local_day() {
        let db = test_db().await;

        // 22 Oct in Sydney covers the 20:00 UTC and 03:00 UTC fixes.
        let tracks = db.query_tracks_info(day("2026-10-22"), 660).await.unwrap();
        assert_eq!(tracks.len(), 1);
        assert_eq!(tracks[0].ts_start, "2026-10-21 20:00:00+00");
        assert_eq!(tracks[0].ts_end, "2026-10-22 03:00:00+00");

        // 21 Oct in Sydney only covers the 10:00 UTC fix.
        let tracks = db.query_tracks_info(day("2026-10-21"), 660).await.unwrap();
        assert_eq!(tracks.len(), 1);
        assert_eq!(tracks[0].ts_start, "2026-10-21 10:00:00+00");
        assert_eq!(tracks[0].ts_end, "2026-10-21 10:00:00+00");
    }

    #[tokio::test]
    async fn track_points_use_the_local_day() {
        let db = test_db().await;

        let sydney = TrackRef {
            device_id: 1,
            ts_start: "2026-10-21 20:00:00+00".into(),
            segmented: None,
            tz: Some(660),
        };
        let track = db.query_track(&sydney).await.unwrap();
        assert_eq!(track.date, "2026-10-22");
        let stamps: Vec<&str> = track.points.iter().map(|p| p.ts.as_str()).collect();
        assert_eq!(stamps, ["2026-10-21 20:00:00+00", "2026-10-22 03:00:00+00"]);

        // Without an offset the day is UTC, so the 10:00 UTC fix is included instead.
        let utc = TrackRef { tz: None, ..sydney };
        let track = db.query_track(&utc).await.unwrap();
        assert_eq!(track.date, "2026-10-21");
        let stamps: Vec<&str> = track.points.iter().map(|p| p.ts.as_str()).collect();
        assert_eq!(stamps, ["2026-10-21 10:00:00+00", "2026-10-21 20:00:00+00"]);
    }
}
