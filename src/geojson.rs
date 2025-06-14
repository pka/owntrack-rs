use crate::db::{GpsPoint, Position, TrackData};
use crate::stats::{BboxStats, DistanceStats, ElevationDiffStats, TrackStats};
use geojson::{Feature, FeatureCollection, Geometry, JsonObject, JsonValue};

const MAX_ACCURACY: i32 = 200; // meters
const ANNOTATIONS_SKIP_LIST: &[&str] = &["_id", "m", "BSSID", "SSID", "created_at"];

fn point_properties(pt: &GpsPoint) -> JsonObject {
    let mut json = JsonObject::from_iter([
        ("time".to_string(), JsonValue::from(pt.ts.to_string())),
        ("speed".to_string(), JsonValue::from(pt.speed)),
        ("elevation".to_string(), JsonValue::from(pt.elevation)),
        ("accuracy".to_string(), JsonValue::from(pt.accuracy)),
        ("v_accuracy".to_string(), JsonValue::from(pt.v_accuracy)),
        ("cog".to_string(), JsonValue::from(pt.cog)),
    ]);
    let annotations: serde_json::Map<String, serde_json::Value> =
        serde_json::from_str(pt.annotations.as_str()).unwrap();
    json.extend(
        annotations
            .into_iter()
            .filter(|(key, _)| !ANNOTATIONS_SKIP_LIST.contains(&key.as_str())),
    );
    json
}

/// Build a GeoJSON LineString FeatureCollection
pub fn track(tracks: &[TrackData]) -> anyhow::Result<String> {
    let features: Vec<Feature> = tracks
        .iter()
        .map(|track| {
            let mut points = track.points.iter().filter(|point| {
                // keep only points within accuracy
                point.accuracy.unwrap_or(0) < MAX_ACCURACY
            });
            let geometry = Geometry::new(geojson::Value::LineString(
                points.clone().map(|pt| vec![pt.x, pt.y]).collect(),
            ));
            let bbox = BboxStats::from_xy_iter(points.clone().map(|pt| (pt.x, pt.y))).bbox();
            // Use properties of last point
            let properties = points.next_back().map(point_properties);
            Feature {
                geometry: Some(geometry),
                properties,
                bbox,
                ..Default::default()
            }
        })
        .collect();

    let geojson = FeatureCollection {
        features,
        ..Default::default()
    };
    Ok(geojson.to_string())
}

/// Build a GeoJSON FeatureCollection with segments containing speed, etc.
pub fn track_with_segments(tracks: &[TrackData]) -> anyhow::Result<String> {
    let features: Vec<Feature> = tracks
        .iter()
        .enumerate()
        .flat_map(|(no, track)| {
            let segments: Vec<Feature> = track
                .points
                .iter()
                .filter(|point| {
                    // keep only points within accuracy
                    point.accuracy.unwrap_or(0) < MAX_ACCURACY
                })
                .collect::<Vec<_>>()
                .windows(2)
                .map(|pts| {
                    let line = vec![vec![pts[0].x, pts[0].y], vec![pts[1].x, pts[1].y]];
                    let geometry = Geometry::new(geojson::Value::LineString(line));
                    let mut properties = point_properties(pts[0]);
                    properties.extend([("trackno".to_string(), JsonValue::from(no))]);
                    Feature {
                        geometry: Some(geometry),
                        properties: Some(properties),
                        ..Default::default()
                    }
                })
                .collect();
            segments
        })
        .collect();

    let geojson = FeatureCollection {
        features,
        ..Default::default()
    };
    Ok(geojson.to_string())
}

/// Build a GeoJSON Point FeatureCollection
pub fn track_points(tracks: &[TrackData]) -> anyhow::Result<String> {
    let feat_iter = tracks
        .iter()
        .flat_map(|track| track.points.iter())
        .filter(|point| {
            // keep only points within accuracy
            point.accuracy.unwrap_or(0) < MAX_ACCURACY
        });
    let features = feat_iter
        .clone()
        .enumerate()
        .map(|(idx, pt)| {
            let geometry = Geometry::new(geojson::Value::Point(vec![pt.x, pt.y]));
            let properties = point_properties(pt);
            Feature {
                id: Some(geojson::feature::Id::Number(serde_json::Number::from(idx))),
                geometry: Some(geometry),
                properties: Some(properties),
                ..Default::default()
            }
        })
        .collect();

    let bbox = BboxStats::from_xy_iter(feat_iter.clone().map(|pt| (pt.x, pt.y))).bbox();

    let mut stats = TrackStats::from_iter(feat_iter.clone()).as_properties();
    stats.extend(
        ElevationDiffStats::from_iter(feat_iter.clone().filter_map(|pt| pt.elevation))
            .as_properties(),
    );
    stats.extend(DistanceStats::from_xy_iter(feat_iter.map(|pt| (pt.x, pt.y))).as_properties());
    let stats_json = JsonObject::from_iter([("stats".to_string(), JsonValue::from(stats))]);

    let geojson = FeatureCollection {
        bbox,
        features,
        foreign_members: Some(stats_json),
    };
    Ok(geojson.to_string())
}

/// Build a GeoJSON Point FeatureCollection
pub fn positions(points: &[Position]) -> anyhow::Result<String> {
    let features = points
        .iter() // without accuracy filter
        .map(|pt| {
            let geometry = Geometry::new(geojson::Value::Point(vec![pt.x, pt.y]));
            let properties = JsonObject::from_iter([
                ("device_id".to_string(), JsonValue::from(pt.device_id)),
                ("time".to_string(), JsonValue::from(pt.ts.to_string())),
                ("tid".to_string(), JsonValue::from(pt.tid.clone())),
                ("speed".to_string(), JsonValue::from(pt.speed)),
                ("elevation".to_string(), JsonValue::from(pt.elevation)),
                ("accuracy".to_string(), JsonValue::from(pt.accuracy)),
                ("v_accuracy".to_string(), JsonValue::from(pt.v_accuracy)),
                ("cog".to_string(), JsonValue::from(pt.cog)),
            ]);
            Feature {
                id: Some(geojson::feature::Id::Number(serde_json::Number::from(
                    pt.device_id,
                ))),
                geometry: Some(geometry),
                properties: Some(properties),
                ..Default::default()
            }
        })
        .collect();
    let bbox = BboxStats::from_xy_iter(points.iter().map(|pt| (pt.x, pt.y))).bbox();

    let geojson = FeatureCollection {
        features,
        bbox,
        ..Default::default()
    };
    Ok(geojson.to_string())
}
