pub mod db;
mod geojson;
mod gpx;
mod http;
mod meshtastic;
mod mqtt;
mod owntracks;
mod stats;

use db::Db;
use env_logger::Env;

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    match dotenvy::dotenv() {
        Ok(_) | Err(dotenvy::Error::Io(_)) => {} // ignore missing .env file
        Err(err) => anyhow::bail!(err),
    }
    env_logger::init_from_env(Env::default().default_filter_or("info"));

    let db = Db::connect().await?;
    db.run_migrations().await?;
    let mqtt_db = db.clone();
    let _handler = tokio::spawn(async move {
        mqtt::subscribe(&mqtt_db).await.unwrap();
    });
    http::webserver(db).await?;
    Ok(())
}
