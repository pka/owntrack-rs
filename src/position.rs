use crate::db::{deserialize_dict_to_string, serialize_raw_json};
use serde::{Deserialize, Serialize};

/// OwnTracks compatible location with custom annotations
#[derive(Serialize, Deserialize, Debug)]
pub struct Position {
    /// Tracker ID used to display the initials of a user (iOS,Android/string/optional) required for http mode
    #[serde(default)] // Make optional regarding to spec
    pub tid: String,
    /// UNIX epoch timestamp in seconds of the location fix (iOS,Android/integer/epoch/required)
    #[serde(rename = "tst")]
    pub ts: i64,
    /// velocity (iOS,Android/integer/kmh/optional)
    #[serde(rename = "vel")]
    pub velocity: Option<u16>,
    /// latitude (iOS,Android/float/degree/required)
    pub lat: f32,
    /// longitude (iOS,Android/float/degree/required)
    pub lon: f32,
    /// Altitude measured above sea level (iOS,Android/integer/meters/optional)
    pub alt: Option<i16>,
    /// Accuracy of the reported location in meters without unit (iOS,Android/integer/meters/optional)
    #[serde(rename = "acc")]
    pub accuracy: Option<u32>,
    /// vertical accuracy of the alt element (iOS/integer/meters/optional)
    #[serde(rename = "vac")]
    pub v_accuracy: Option<i16>,
    /// Course over ground (iOS/integer/degree/optional)
    pub cog: Option<i16>,
    /// Additional parameters
    #[serde(
        flatten,
        serialize_with = "serialize_raw_json",
        deserialize_with = "deserialize_dict_to_string"
    )]
    pub annotations: String,
}
