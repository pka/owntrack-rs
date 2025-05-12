use crate::db::Db;
use crate::meshtastic;
use crate::owntracks;
use gethostname::gethostname;
use rumqttc::{AsyncClient, Event, Incoming, MqttOptions, QoS};
use std::process;
use std::str::FromStr;
use std::time::Duration;
use tokio::time;

pub async fn subscribe(db: &Db) -> anyhow::Result<()> {
    let mqtt_url = match dotenvy::var("MQTT_URL") {
        Ok(url) if !url.is_empty() => url,
        Err(_) | Ok(_) => {
            log::info!("MQTT_URL not set, skipping MQTT client");
            return Ok(());
        }
    };
    let mqtt_user = dotenvy::var("MQTT_USER")?;
    let mqtt_password = dotenvy::var("MQTT_PASSWORD")?;
    let client_id = format!("{}-{}", gethostname().to_string_lossy(), process::id());

    let mut mqttoptions = MqttOptions::parse_url(format!("{mqtt_url}?client_id={client_id}"))?;
    mqttoptions.set_credentials(mqtt_user, mqtt_password);
    mqttoptions.set_keep_alive(Duration::from_secs(5));
    mqttoptions.set_clean_session(false);

    let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);
    client.subscribe("owntracks/#", QoS::AtMostOnce).await?;

    loop {
        let notification = eventloop.poll().await;
        log::debug!("Notification = {notification:?}");
        match notification {
            Ok(Event::Incoming(Incoming::Publish(packet))) => {
                log::info!(
                    "{}: {}",
                    packet.topic,
                    String::from_utf8_lossy(packet.payload.as_ref()),
                );
                if let Ok(msg) =
                    serde_json::from_slice::<owntracks::Message>(packet.payload.as_ref())
                {
                    log::debug!("{msg:?}");
                    if let owntracks::Message::Location(loc) = msg {
                        let Some((user, device)) = get_user_device_from_topic(&packet.topic) else {
                            log::error!("Unexpected topic `{}`", packet.topic);
                            continue;
                        };
                        if let Err(e) = db.insert_location(&user, &device, &loc).await {
                            log::error!("{e}");
                        }
                    }
                } else if let Ok(msg) =
                    serde_json::from_slice::<meshtastic::Message>(packet.payload.as_ref())
                {
                    log::debug!("{msg:?}");
                    if let meshtastic::Message::Position(pos) = msg {
                        // Skip other devices
                        if pos.hops_away > 0 {
                            continue;
                        }
                        let Some((user, device)) =
                            get_user_device_from_meshtastic_topic(&packet.topic)
                        else {
                            log::error!("Unexpected topic `{}`", packet.topic);
                            continue;
                        };
                        let mut lat_str = pos.payload.latitude_i.to_string();
                        lat_str.insert(lat_str.len() - 7, '.'); // 470401765 -> 47.0401765
                        let mut lon_str = pos.payload.longitude_i.to_string();
                        lon_str.insert(lon_str.len() - 7, '.');
                        let loc = owntracks::Location {
                            tid: "msh".to_string(),
                            ts: pos.payload.time,
                            velocity: pos.payload.ground_speed,
                            lat: f32::from_str(&lat_str).unwrap(),
                            lon: f32::from_str(&lon_str).unwrap(),
                            alt: pos.payload.altitude,
                            accuracy: None,
                            v_accuracy: None,
                            cog: None,
                            annotations: pos.payload.annotations,
                        };
                        log::debug!("{loc:?}");
                        if let Err(e) = db.insert_location(&user, &device, &loc).await {
                            log::error!("{e}");
                        }
                    }
                }
            }
            Ok(_ev) => {}
            Err(error) => {
                log::info!("MQTT error: {error}");
                // Avoid error flood
                time::sleep(Duration::from_millis(500)).await;
            }
        }
    }
}

pub fn get_user_device_from_topic(topic: &str) -> Option<(String, String)> {
    // topic: "onwntrack/{user}/{device}"
    let parts: Vec<&str> = topic.split('/').collect();
    if parts.len() != 3 {
        return None;
    }
    let user = parts[1].to_string();
    let device = parts[2].to_string();
    Some((user, device))
}

pub fn get_user_device_from_meshtastic_topic(topic: &str) -> Option<(String, String)> {
    // topic: "owntracks/{user}/msh/{device}/2/json/LongFast/!xxxx{device}"
    let parts: Vec<&str> = topic.split('/').collect();
    if parts.len() != 8 {
        return None;
    }
    let user = parts[1].to_string();
    let device = parts[3].to_string();
    Some((user, device))
}
