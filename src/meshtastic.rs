use crate::db::{deserialize_dict_to_string, serialize_raw_json};
use serde::{Deserialize, Serialize};

/// Meshtastic JSON message
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
#[serde(rename_all = "lowercase")]
pub enum Message {
    // {
    //     "channel": 0,
    //     "from": 2224737852,
    //     "hop_start": 3,
    //     "hops_away": 0,
    //     "id": 362355262,
    //     "payload": {
    //         "hardware": 43,
    //         "id": "!849ace3c",
    //         "longname": "Sargans1",
    //         "role": 1,
    //         "shortname": "Sgn1"
    //     },
    //     "rssi": -128,
    //     "sender": "!da657310",
    //     "snr": -20.75,
    //     "timestamp": 1747084042,
    //     "to": 4294967295,
    //     "type": "nodeinfo"
    // }
    Nodeinfo,
    // {
    //     "channel": 0,
    //     "from": 3185021733,
    //     "hop_start": 3,
    //     "hops_away": 0,
    //     "id": 3863449354,
    //     "payload": {
    //         "PDOP": 324,
    //         "altitude": 469,
    //         "ground_speed": 1,
    //         "latitude_i": 470450176,
    //         "longitude_i": 94404608,
    //         "precision_bits": 16,
    //         "sats_in_view": 8,
    //         "time": 1747080713
    //     },
    //     "rssi": -118,
    //     "sender": "!da657310",
    //     "snr": -9.5,
    //     "timestamp": 1747080712,
    //     "to": 4294967295,
    //     "type": "position"
    // }
    Position(Position),
    // {
    //     "channel": 0,
    //     "from": 3185021733,
    //     "hop_start": 3,
    //     "hops_away": 0,
    //     "id": 808235794,
    //     "payload": {
    //         "air_util_tx": 1.68216681480408,
    //         "battery_level": 83,
    //         "channel_utilization": 2.9449999332428,
    //         "uptime_seconds": 111020,
    //         "voltage": 4.01300001144409
    //     },
    //     "rssi": -118,
    //     "sender": "!da657310",
    //     "snr": -8.5,
    //     "timestamp": 1747081165,
    //     "to": 4294967295,
    //     "type": "telemetry"
    // }
    Telemetry,
}

/// Meshtastic position with custom annotations
#[derive(Serialize, Deserialize, Debug)]
pub struct Position {
    pub hops_away: u8,
    pub payload: PositionPayload,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PositionPayload {
    pub time: i64,
    pub latitude_i: u64,
    pub longitude_i: u64,
    pub precision_bits: u8,
    pub altitude: Option<i16>,
    pub ground_speed: Option<u16>,
    /// Additional parameters
    #[serde(
        flatten,
        serialize_with = "serialize_raw_json",
        deserialize_with = "deserialize_dict_to_string"
    )]
    pub annotations: String,
}
