use crate::db::GpsPoint;
use chrono::{DateTime, FixedOffset, Local, NaiveDateTime};
use geo::algorithm::vincenty_distance::VincentyDistance;
use geojson::JsonValue;
use polars::prelude::*;
use polars::{frame::row::Row, functions::concat_df_horizontal, series::ops::NullBehavior};
use smartcore::ensemble::random_forest_classifier::RandomForestClassifier;
use smartcore::linalg::basic::matrix::DenseMatrix;

pub struct Classifier {
    tree: RandomForestClassifier<f64, u16, DenseMatrix<f64>, Vec<u16>>,
}

static MODEL_JSON: &str = include_str!("model.json");

impl Classifier {
    pub fn load() -> Self {
        let tree = serde_json::from_str(MODEL_JSON).unwrap();
        Self { tree }
    }
    pub fn predict(&self, x: &DataFrame) -> Vec<u16> {
        let matrix = matrix_from_df(x);
        self.tree.predict(&matrix).unwrap()
    }
    pub fn activity(idx: u16) -> &'static str {
        let lookup = [
            "stop",
            "car",
            "walk",
            "ski",
            "bike",
            "train",
            "bus",
            "paraglide",
        ];
        lookup.get(idx as usize).unwrap_or(&"unknown")
    }
}

/// Preprocess raw track
///
/// Returns a tuple of filtered track.
pub fn preprocess(track: &[GpsPoint]) -> PolarsResult<Vec<GpsPoint>> {
    let df = track_enriched(track)?;
    let y = classify(&df)?;

    let ys: Series = y.iter().map(|v| *v as u32).collect();
    let ydf = df!("activity" => ys)?;
    let df = concat_df_horizontal(&[df, ydf], true)?;

    let sequences = sequences(df.clone().lazy())?;
    // println!("{sequences}");

    let mut activity = df
        .column("activity")?
        .as_materialized_series()
        .u32()?
        .to_vec_null_aware()
        .left()
        .unwrap();
    let latitude = df
        .column("latitude")?
        .as_materialized_series()
        .f64()?
        .to_vec_null_aware()
        .left()
        .unwrap();
    let longitude = df
        .column("longitude")?
        .as_materialized_series()
        .f64()?
        .to_vec_null_aware()
        .left()
        .unwrap();

    for row in sequences
        .clone()
        .lazy()
        .filter(col("activity").eq(lit(0)).and(col("len").gt(lit(30)))) // stops
        .select([
            col("min_idx"),
            col("max_idx"),
            col("latitude"),
            col("longitude"),
        ])
        .collect()?
        .into_struct("struct".into())
        .into_series()
        .rechunk()
        .iter()
    {
        let row_values: Vec<_> = row._iter_struct_av().collect();
        let min_idx: usize = row_values[0].try_extract()?;
        let max_idx: usize = row_values[1].try_extract()?;
        let lat_stop: f64 = row_values[2].try_extract()?;
        let lon_stop: f64 = row_values[3].try_extract()?;
        // forward
        let mut idx = max_idx + 1;
        let mut is_close = true;
        while idx < activity.len() && is_close {
            let dist = distance([longitude[idx], latitude[idx]], [lon_stop, lat_stop]);
            is_close = dist < 50;
            if is_close {
                activity[idx] = u32::MAX; // Null
            }
            idx += 1;
        }
        #[allow(clippy::needless_range_loop)]
        for idx in min_idx..=max_idx {
            activity[idx] = u32::MAX; // Null
        }
        // backward
        let mut idx = min_idx;
        is_close = true;
        while idx > 0 && is_close {
            idx -= 1;
            let dist = distance([longitude[idx], latitude[idx]], [lon_stop, lat_stop]);
            is_close = dist < 50;
            if is_close {
                activity[idx] = u32::MAX; // Null
            }
        }
    }

    let act: Series = activity.iter().collect();
    let actdf = df!("act" => act)?;
    let df = concat_df_horizontal(&[df, actdf], true)?;

    let filtered: Vec<GpsPoint> = df
        .lazy()
        .select([col("act"), col("vcalc")])
        .collect()?
        .into_struct("struct".into())
        .into_series()
        .rechunk()
        .iter()
        .zip(track.iter())
        .filter_map(|(row, point)| {
            let row_values: Vec<_> = row._iter_struct_av().collect();
            let activity: u32 = row_values[0].try_extract().ok()?;
            let mut vcalc: f64 = row_values[1].try_extract().ok()?;
            vcalc *= 3.6;

            let mut pt = point.clone();
            let speed = point.speed.unwrap_or(0) as f64;
            let vfact = vcalc.max(speed).max(1.0) / vcalc.min(speed).max(1.0);
            // Collapse stops
            // TOOD: keep first & last point
            if activity != u32::MAX && vfact < 5.0 && point.accuracy.unwrap_or(0) < 1000 {
                let mut annotations: serde_json::Map<String, serde_json::Value> =
                    serde_json::from_str(pt.annotations.as_str()).unwrap();
                annotations.extend([
                    (
                        "activity".to_string(),
                        JsonValue::from(Classifier::activity(activity as u16)),
                    ),
                    ("vcalc".to_string(), JsonValue::from(vcalc)),
                    ("vfact".to_string(), JsonValue::from(vfact)),
                ]);
                pt.annotations = serde_json::to_string(&annotations).unwrap();
                Some(pt)
            } else {
                None
            }
        })
        .collect();

    let ndel = track.len() - filtered.len();
    log::debug!(
        "Stop detection: removed {ndel} points ({:.2}%)",
        ndel * 100 / track.len()
    );
    Ok(filtered)
}

/// Process track and classify activity
pub fn analyze(track: &[GpsPoint]) -> Vec<u16> {
    let df = track_enriched(track).unwrap();
    classify(&df).unwrap()
}

fn track_enriched(track: &[GpsPoint]) -> PolarsResult<DataFrame> {
    let df = df_from_track(track)?;
    let df = speed_loc_means(df.lazy());
    delta(df.lazy())
}

pub fn classify(df: &DataFrame) -> PolarsResult<Vec<u16>> {
    let classifier = Classifier::load();
    let x = df.select(["speed", "vmean5", "vmean10", "vcalc", "vzcalc", "vcenter10"])?;
    let y = classifier.predict(&x);
    Ok(y)
}

fn df_from_track(track: &[GpsPoint]) -> PolarsResult<DataFrame> {
    let schema = Schema::from_iter(vec![
        Field::new(
            "timestamp".into(),
            DataType::Datetime(TimeUnit::Microseconds, None),
        ),
        Field::new("latitude".into(), DataType::Float64),
        Field::new("longitude".into(), DataType::Float64),
        Field::new("speed".into(), DataType::Float64),
        Field::new("elevation".into(), DataType::Int64),
    ]);

    let tz = *Local::now().offset();
    let rows = track
        .iter()
        .map(|pt| {
            let dt = DateTime::<FixedOffset>::parse_from_str(&pt.ts, "%F %T%#z") // SQLite stores in UTC without TZ
                .or(NaiveDateTime::parse_from_str(&pt.ts, "%F %T")
                    .map(|utcts| utcts.and_local_timezone(tz).unwrap()))
                .unwrap();
            // log::debug!("Timestamp `{}` -> `{dt:?}`", &pt.ts);
            Row::new(vec![
                AnyValue::Datetime(dt.timestamp_micros(), TimeUnit::Microseconds, None),
                AnyValue::Float64(pt.y),
                AnyValue::Float64(pt.x),
                AnyValue::Float64(pt.speed.unwrap_or(0).into()),
                AnyValue::Int64(pt.elevation.unwrap_or(0).into()), // keep i16 ?
            ])
        })
        .collect::<Vec<_>>(); // TODO: from_rows_iter_and_schema

    DataFrame::from_rows_and_schema(&rows, &schema)
}

fn speed_loc_means(df: LazyFrame) -> LazyFrame {
    let win5 = RollingOptionsFixedWindow {
        window_size: 5,
        min_periods: 1,
        ..Default::default()
    };
    let win10 = RollingOptionsFixedWindow {
        window_size: 10,
        min_periods: 1,
        ..Default::default()
    };
    df.with_columns([
        col("speed").rolling_mean(win5).alias("vmean5"),
        col("speed").rolling_mean(win10.clone()).alias("vmean10"),
        // Calculate mean positions for detecting oscilating GPS positions, especially during stops
        col("latitude")
            .rolling_mean(win10.clone())
            .alias("lat_mean10"),
        col("longitude")
            .rolling_mean(win10.clone())
            .alias("lon_mean10"),
    ])
}

fn delta(df: LazyFrame) -> PolarsResult<DataFrame> {
    // Difference between two rows
    let deltas = df
        .lazy()
        .with_columns([
            col("timestamp")
                .shift(lit(-1))
                .fill_null(col("timestamp"))
                .alias("ts2"),
            col("latitude")
                .shift(lit(-1))
                .fill_null(col("latitude"))
                .alias("lat2"),
            col("longitude")
                .shift(lit(-1))
                .fill_null(col("longitude"))
                .alias("lon2"),
            col("elevation")
                .diff(lit(1), NullBehavior::Ignore)
                .shift(lit(-1))
                .fill_null(0)
                .alias("ele_delta"),
            col("lat_mean10")
                .shift(lit(-1))
                .fill_null(col("lat_mean10"))
                .alias("lat_mean10_shift"),
            col("lon_mean10")
                .shift(lit(-1))
                .fill_null(col("lon_mean10"))
                .alias("lon_mean10_shift"),
        ])
        .with_columns([
            // Python map_rows lambda:
            // (t[n]-t[0]).total_seconds(), # duration = ts2 - timestamp
            // distance.distance((t[1], t[2]), (t[n+1], t[n+2])).meters, # dist = lat,lon <-> lat2,lon2
            // distance.distance((t[1], t[2]), (t[n-2], t[n-1])).meters, # dist_center10 = lat,lon <-> lat_mean10,lon_mean10
            // t[n+3]-t[4], # ele_delta = ele2 - elevation
            // For single column operations see also https://stackoverflow.com/a/71638091
            as_struct(vec![col("timestamp"), col("ts2")])
                .map(
                    |s| {
                        let ca = s.struct_()?;
                        let timestamp_s = ca.field_by_name("timestamp")?;
                        let timestamp = timestamp_s.datetime()?;
                        let ts2_s = ca.field_by_name("ts2")?;
                        let ts2 = ts2_s.datetime()?;

                        let out: Int64Chunked = timestamp
                            .as_datetime_iter()
                            .zip(ts2.as_datetime_iter())
                            .map(|(opt_a, opt_b)| match (opt_a, opt_b) {
                                (Some(a), Some(b)) => Some((b - a).num_seconds()),
                                _ => None,
                            })
                            .collect();

                        Ok(Some(out.into_series().into()))
                    },
                    GetOutput::from_type(DataType::Int64),
                )
                .alias("duration"),
            as_struct(vec![
                col("latitude"),
                col("longitude"),
                col("lat2"),
                col("lon2"),
            ])
            .map(
                |s| {
                    let ca = s.struct_()?;
                    let latitude_s = ca.field_by_name("latitude")?;
                    let latitude = latitude_s.f64()?;
                    let longitude_s = ca.field_by_name("longitude")?;
                    let longitude = longitude_s.f64()?;
                    let lat2_s = ca.field_by_name("lat2")?;
                    let lat2 = lat2_s.f64()?;
                    let lon2_s = ca.field_by_name("lon2")?;
                    let lon2 = lon2_s.f64()?;

                    let out: Int64Chunked = latitude
                        .into_iter()
                        .zip(longitude.into_iter())
                        .zip(lat2.into_iter())
                        .zip(lon2.into_iter())
                        .map(|(((opt_a, opt_b), opt_c), opt_d)| {
                            match (opt_a, opt_b, opt_c, opt_d) {
                                (Some(a), Some(b), Some(c), Some(d)) => {
                                    Some(distance([a, b], [c, d]))
                                }
                                _ => None,
                            }
                        })
                        .collect();

                    Ok(Some(out.into_series().into()))
                },
                GetOutput::from_type(DataType::Int64),
            )
            .alias("dist"),
            as_struct(vec![
                col("lat_mean10"),
                col("lon_mean10"),
                col("lat_mean10_shift"),
                col("lon_mean10_shift"),
            ])
            .map(
                |s| {
                    let ca = s.struct_()?;
                    let lat_mean10_s = ca.field_by_name("lat_mean10")?;
                    let lat_mean10 = lat_mean10_s.f64()?;
                    let lon_mean10_s = ca.field_by_name("lon_mean10")?;
                    let lon_mean10 = lon_mean10_s.f64()?;
                    let lat_mean10_shift_s = ca.field_by_name("lat_mean10_shift")?;
                    let lat_mean10_shift = lat_mean10_shift_s.f64()?;
                    let lon_mean10_shift_s = ca.field_by_name("lon_mean10_shift")?;
                    let lon_mean10_shift = lon_mean10_shift_s.f64()?;

                    let out: Int64Chunked = lat_mean10
                        .into_iter()
                        .zip(lon_mean10.into_iter())
                        .zip(lat_mean10_shift.into_iter())
                        .zip(lon_mean10_shift.into_iter())
                        .map(|(((opt_a, opt_b), opt_c), opt_d)| {
                            match (opt_a, opt_b, opt_c, opt_d) {
                                (Some(a), Some(b), Some(c), Some(d)) => {
                                    Some(distance([a, b], [c, d]))
                                }
                                _ => None,
                            }
                        })
                        .collect();

                    Ok(Some(out.into_series().into()))
                },
                GetOutput::from_type(DataType::Int64),
            )
            .alias("dist_center10"),
        ])
        .with_columns([
            (col("dist").cast(DataType::Float64) / col("duration"))
                .fill_nan(0)
                .alias("vcalc"),
            (col("ele_delta").cast(DataType::Float64) / col("duration"))
                .fill_nan(0)
                .alias("vzcalc"),
            (col("dist_center10").cast(DataType::Float64) / col("duration"))
                .fill_nan(0)
                .alias("vcenter10"),
        ])
        .collect()?;
    Ok(deltas)
}

fn distance(p1: [f64; 2], p2: [f64; 2]) -> i64 {
    let p1 = geo::Point::from(p1);
    let p2 = geo::Point::from(p2);
    let distance = p1.vincenty_distance(&p2).unwrap();
    distance.round() as i64
}

fn sequences(df: LazyFrame) -> PolarsResult<DataFrame> {
    let sequences = df
        .clone()
        // Count sequence lengths
        .select([col("activity").rle()])
        .unnest(Selector::Wildcard) // -> len, value
        .with_columns([
            (col("len").cum_sum(false) - col("len")).alias("min_idx"),
            (col("len").cum_sum(false) - lit(1) - col("len").floor_div(lit(2))).alias("mid_idx"),
            (col("len").cum_sum(false) - lit(1)).alias("max_idx"),
        ]);
    let joined = df
        .with_row_index("index", None)
        .join(
            sequences,
            [col("index")],
            [col("mid_idx")],
            JoinArgs::new(JoinType::Inner),
        )
        .collect()?;

    Ok(joined)
}

fn matrix_from_df(df: &DataFrame) -> DenseMatrix<f64> {
    let nrows = df.height();
    let ncols = df.width();
    let mut m_values = Vec::with_capacity(nrows * ncols);

    for col in df.iter() {
        let mut colvec = col.f64().unwrap().to_vec_null_aware().left().unwrap();
        m_values.append(&mut colvec);
    }

    DenseMatrix::new(nrows, ncols, m_values, true).unwrap()
}
