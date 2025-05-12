use crate::db::{deserialize_dict_to_string, serialize_raw_json};
use serde::{Deserialize, Serialize};
use serde_json::json;

/// OwnTracks JSON message
/// <https://owntracks.org/booklet/tech/json/>
#[derive(Serialize, Deserialize, Debug)]
#[allow(clippy::large_enum_variant)]
#[serde(tag = "_type")]
#[serde(rename_all = "lowercase")]
pub enum Message {
    Beacon,
    Card,
    Cmd,
    Configuration,
    Encrypted,
    Location(Location),
    Lwt,
    Request,
    Status,
    Steps,
    Transition,
    Waypoint,
    Waypoints,
}

/// OwnTracks location
#[derive(Serialize, Deserialize, Debug)]
pub struct OtLocation {
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
    pub alt: Option<u16>,
    /// Accuracy of the reported location in meters without unit (iOS,Android/integer/meters/optional)
    #[serde(rename = "acc")]
    pub accuracy: Option<u32>,
    /// vertical accuracy of the alt element (iOS/integer/meters/optional)
    #[serde(rename = "vac")]
    pub v_accuracy: Option<i16>,
    /// Device battery level (iOS,Android/integer/percent/optional)
    #[serde(rename = "batt")]
    pub batt_level: Option<u8>,
    /// Battery Status 0=unknown, 1=unplugged, 2=charging, 3=full (iOS, Android)
    #[serde(rename = "bs")]
    pub batt_status: u8,
    /// Course over ground (iOS/integer/degree/optional)
    pub cog: Option<i16>,
    /// radius around the region when entering/leaving (iOS/integer/meters/optional)
    pub rad: Option<u32>,
    /// trigger for the location report (iOS,Android/string/optional)
    /// * `p`: ping issued randomly by background task (iOS,Android)
    /// * `c`: circular region enter/leave event (iOS,Android)
    /// * `C`: circular region enter/leave event for +follow regions (iOS)
    /// * `b`: beacon region enter/leave event (iOS)
    /// * `r`: response to a reportLocation cmd message (iOS,Android)
    /// * `u`: manual publish requested by the user (iOS,Android)
    /// * `t`: timer based publish in move move (iOS)
    /// * `v`: updated by Settings/Privacy/Locations Services/System Services/Frequent Locations monitoring (iOS)
    #[serde(rename = "t")]
    pub trigger: Option<String>,
    /// barometric pressure (iOS/float/kPa/optional/extended data)
    #[serde(rename = "p")]
    pub pressure: Option<f32>,
    /// point of interest name (iOS/string/optional)
    pub poi: Option<String>,
    /// Internet connectivity status (route to host) when the message is created (iOS,Android/string/optional/extended data)
    /// * `w`: phone is connected to a WiFi connection (iOS,Android)
    /// * `o`: phone is offline (iOS,Android)
    /// * `m`: mobile data (iOS,Android)
    #[serde(rename = "conn")]
    pub conn_status: Option<String>,
    /// name of the tag (iOS/string/optional)
    pub tag: Option<String>,
    /// (only in HTTP payloads) contains the original publish topic (e.g. owntracks/jane/phone). (iOS,Android >= 2.4,string)
    pub topic: Option<String>,
    /// contains a list of regions the device is currently in (e.g. ["Home","Garage"]). Might be empty. (iOS,Android/list of strings/optional)
    pub inregions: Option<String>,
    /// contains a list of region IDs the device is currently in (e.g. ["6da9cf","3defa7"]). Might be empty. (iOS,Android/list of strings/optional)
    pub inrids: Option<String>,
    /// unique name of the WLAN. (iOS,string/optional)
    #[serde(rename = "SSID")]
    pub ssid: Option<String>,
    /// identifies the access point. (iOS,string/optional)
    #[serde(rename = "BSSID")]
    pub bssid: Option<String>,
    /// identifies the time at which the message is constructed (if it differs from tst, which is the timestamp of the GPS fix) (iOS,Android/integer/epoch/optional)
    pub created_at: Option<i64>,
    /// identifies the monitoring mode at which the message is constructed (significant=1, move=2) (iOS/integer/optional)
    #[serde(rename = "m")]
    pub mmode: Option<u8>,
    /// random identifier to be used by consumers to correlate & distinguish send/return messages (Android/string)
    #[serde(rename = "_id")]
    pub msg_id: String,
}

/// OwnTracks compatible location with custom annotations
#[derive(Serialize, Deserialize, Debug)]
pub struct Location {
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

pub struct AppConfig {
    /// Owntracks/MQTT username
    pub username: String,
    /// Device identifier
    pub device_id: String,
    /// MQTT client ID
    pub client_id: String,
    /// Tracker (display) ID
    pub tid: String,
    /// MQTT password
    pub password: String,
    /// Use password authentication
    pub use_password: bool,
    /// Operating mode (0: MQTT, 3: HTTP)
    pub mode: u8,
    /// MQTT broker hostname
    pub mqtt_host: String,
    /// MQTT broker port
    pub mqtt_port: u16,
    /// Use WebSocket
    pub ws: bool,
    /// MQTT topic base
    pub topic_base: String,
    /// HTTP publish URL
    pub http_url: String,
    /// Enable TLS
    pub tls: bool,
}

impl AppConfig {
    pub fn from_env(req_url: Option<String>) -> Self {
        let username = dotenvy::var("OTRS_USERNAME").unwrap_or("me".to_string());
        let device_id = dotenvy::var("OTRS_DEVICE_ID").unwrap_or("mobile".to_string());
        let tid = dotenvy::var("OTRS_TID").unwrap_or(username.chars().take(2).collect::<String>());
        let http_address = dotenvy::var("HTTP_ADDRESS").unwrap_or("localhost".to_string());
        let http_url = dotenvy::var("OTRS_BASE_URL")
            .unwrap_or(req_url.unwrap_or(format!("https://{http_address}")))
            + &format!("/owntracks?u={username}&d={device_id}");
        let tls = http_url.starts_with("https://");
        AppConfig {
            username,
            device_id,
            client_id: dotenvy::var("OTRS_CLIENT_ID").unwrap_or("owntracks-app".to_string()),
            tid,
            use_password: dotenvy::var("OTRS_PASSWORD")
                .map(|s| !s.is_empty())
                .unwrap_or(false),
            password: dotenvy::var("OTRS_PASSWORD").unwrap_or("".to_string()),
            mode: 3,
            mqtt_host: dotenvy::var("MQTT_HOST").unwrap_or("localhost".to_string()),
            mqtt_port: dotenvy::var("MQTT_PORT")
                .unwrap_or("1883".to_string())
                .parse()
                .unwrap_or(1883),
            ws: dotenvy::var("MQTT_WS")
                .unwrap_or("false".to_string())
                .parse()
                .unwrap_or(false),
            topic_base: dotenvy::var("MQTT_TOPIC_BASE").unwrap_or("owntracks".to_string()),
            http_url,
            tls,
        }
    }
}
pub fn otrc_json(cfg: &AppConfig) -> serde_json::Value {
    json!({
            "_type": "configuration",
            "allowRemoteLocation": true,
            "auth": true,
            "cleanSession": false,
            "clientId": cfg.client_id,
            "cmd": true,
            "deviceId": cfg.device_id,
            "experimentalFeatures": [
                "bearingArrowFollowsDeviceOrientation",
                "showExperimentalPreferenceUI",
                "allowSmallKeepalive",
                "useOSMMap"
            ],
            "extendedData": true,
            "host": cfg.mqtt_host,
            "keepalive": 55,
            "mapLayerStyle": "OpenStreetMapNormal",
            "mode": cfg.mode,
            "mqttProtocolLevel": 4,
            "password": cfg.password,
            "port": cfg.mqtt_port,
            "positions": 200,
            "pubQos": 2,
            "pubRetain": true,
            "pubTopicBase": format!("{}/{}/{}", cfg.topic_base, cfg.username, cfg.device_id),
            "ranging": true,
            "sub": true,
            "subQos": 2,
            "subTopic": format!("{tb}/+/+ {tb}/+/+/event {tb}/+/+/info {tb}/{user}/{dev}/cmd",
                tb = cfg.topic_base, user=cfg.username, dev=cfg.device_id),
            "tid": cfg.tid,
            "tls": cfg.tls,
            "url": cfg.http_url,
            "usePassword": cfg.use_password,
            "username": cfg.username,
            "waypoints": [
                {
                    "_type": "waypoint",
                    "desc": "+follow",
                    "lat": 37.3323,
                    "lon": -122.031,
                    "rad": 50,
                    "rid": "QUICKSETUP-BA2BFB63-7C60",
                    "tst": 1610104383
                }
            ],
            "willQos": 1,
            "willRetain": false,
            "willTopic": "",
            "ws": cfg.ws
        }
    )
}
