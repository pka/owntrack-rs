//! Meshtastic integration via MQTT: <https://meshtastic.org/docs/software/integrations/mqtt/>

pub(crate) mod protobufs;

use crate::db::Db;
use crate::owntracks::Location;
use prost::Message;
use std::str::FromStr;

pub async fn decode_packet(
    db: &Db,
    user: &str,
    device: &str, // TODO: get device name from User packet via mesh_packet.from
    envelope: &protobufs::ServiceEnvelope,
) -> Result<(), prost::DecodeError> {
    // log::debug!("{envelope:?}");
    // ServiceEnvelope { packet: Some(MeshPacket { from: 3257392698, to: 4294967295, channel: 0, id: 786780598, rx_time: 1748123191, rx_snr: 0.0, hop_limit: 3, want_ack: false, priority: Background, rx_rssi: 0, delayed: NoDelay, via_mqtt: false, hop_start: 3, public_key: [], pki_encrypted: false, next_hop: 0, relay_node: 58, tx_after: 0,
    //   payload_variant: Some(Decoded(Data { portnum: PositionApp, payload: [...], want_response: false, dest: 0, source: 0, request_id: 0, reply_id: 0, emoji: 0, bitfield: None })) }),
    //   channel_id: "Tracking", gateway_id: "!12341234" }
    if let Some(ref mesh_packet) = envelope.packet {
        if let Some(protobufs::mesh_packet::PayloadVariant::Decoded(ref packet_data)) =
            mesh_packet.payload_variant
        {
            match packet_data.portnum() {
                protobufs::PortNum::PositionApp => {
                    let position = protobufs::Position::decode(packet_data.payload.as_slice())?;
                    log::info!("<{device}> {position:?}");
                    // Position { latitude_i: Some(470400000), longitude_i: Some(94300000), altitude: Some(491), time: 1748123191, location_source: LocInternal, altitude_source: AltUnset, timestamp: 0, timestamp_millis_adjust: 0, altitude_hae: None, altitude_geoidal_separation: None, pdop: 149, hdop: 0, vdop: 0,
                    //  gps_accuracy: 0, ground_speed: Some(0), ground_track: Some(18928000), fix_quality: 0, fix_type: 0, sats_in_view: 10, sensor_id: 0, next_update: 0, seq_number: 0, precision_bits: 32 }
                    if let Some(loc) = position_to_location(device, position) {
                        if let Err(e) = db.insert_location(user, device, &loc).await {
                            log::error!("{e}");
                        }
                    }
                }
                protobufs::PortNum::NodeinfoApp => {
                    let user = protobufs::User::decode(packet_data.payload.as_slice())?;
                    log::info!("<{device}> {user:?}");
                    // User { id: "!12341234", long_name: "My Tracker", short_name: "1234", macaddr: [...], hw_model: TrackerT1000E, is_licensed: false, role: Client, public_key: [...], is_unmessagable: None }
                }
                protobufs::PortNum::TelemetryApp => {
                    let telemetry = protobufs::Telemetry::decode(packet_data.payload.as_slice())?;
                    log::info!("<{device}> {telemetry:?}");
                    // Telemetry { time: 1748124056, variant: Some(DeviceMetrics(DeviceMetrics { battery_level: Some(28), voltage: Some(3.788), channel_utilization: Some(8.636667), air_util_tx: Some(0.029166665), uptime_seconds: Some(66) })) }
                }
                _ => {}
            }
        }
    }
    Ok(())
}

fn position_to_location(device: &str, pos: protobufs::Position) -> Option<Location> {
    if let (Some(lat_i), Some(lon_i)) = (pos.latitude_i, pos.longitude_i) {
        let mut lat_str = lat_i.to_string();
        lat_str.insert(lat_str.len() - 7, '.'); // 470401765 -> 47.0401765
        let mut lon_str = lon_i.to_string();
        lon_str.insert(lon_str.len() - 7, '.');
        let loc = Location {
            tid: device.to_string(), // userid[userid.len() - 4..].to_string()
            ts: pos.time.into(),
            velocity: pos.ground_speed.map(|val| val as u16), // u32
            lat: f32::from_str(&lat_str).unwrap(),
            lon: f32::from_str(&lon_str).unwrap(),
            alt: pos.altitude.map(|val| val as i16), // i32
            accuracy: None,
            v_accuracy: None,
            cog: None,
            annotations: "{}".to_string(),
        };
        Some(loc)
    } else {
        None
    }
}
