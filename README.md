<div class="oranda-hide">

# Owntrack-rs

</div>

## Overview

Owntrack-rs is a self-hosted GPS tracking solution that allows you to record and manage your location data from mobile phones or IoT devices.
It provides [OwnTracks](https://owntracks.org/booklet/) compatible HTTP and MQTT endpoints, [Meshtastic](https://meshtastic.org/) MQTT integration
and a built-in viewer application.

![Screenshot](static/screenshot.jpg)

<pre>
Features:
- [x] Owntracks compatible HTTP endpoint
- [x] Owntracks compatible MQTT interface
- [x] Meshtastic compatible MQTT interface
- [x] SQLite local file storage
- [x] PostgreSQL database storage
- [x] GeoJSON and GPX track exports
- [x] Built-In Viewer
- [ ] Password protected and public views
- [x] Mobile friendly vector tile maps
</pre>

## Installation

### Pre-built binaries

We provide several options to access pre-built binaries for a variety of platforms. If you would like to manually download a pre-built binary, checkout [the latest release on GitHub](https://github.com/pka/owntrack-rs/releases/latest).

### Installer scripts

#### macOS and Linux:

```sh
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/pka/owntrack-rs/releases/latest/download/owntrack-rs-installer.sh | sh
```

### Docker

There is a Docker image available on [Docker Hub](https://hub.docker.com/r/sourcepole/owntrack-rs).

Usage:
```
docker run -d --name owntrack-rs --rm -p 8083:8083 --user $(id -u):$(id -g) -v $PWD:/db -e DB_CONNECTION=sqlite://db/owntracks.sqlite sourcepole/owntrack-rs
```

### Docker Compose

There is a sample `docker-compose.yml` file that can be used to run owntrack-rs with a SQLite database and a Caddy reverse proxy.

Copy [docker-compose.yml](https://github.com/pka/owntrack-rs/raw/refs/heads/main/docker-compose.yml) and [Caddyfile](https://github.com/pka/owntrack-rs/raw/refs/heads/main/Caddyfile)
into an empty directory.

Setup your DNS pointing to the public IP address of your server.

Create an `.env` file with the following content:
```
#HTTP_ADDRESS=owntracks.example.org
OTRS_USERNAME=owntracks
OTRS_PASSWORD='private'
# docker compose exec caddy caddy hash-password --plaintext 'private'
OTRS_PASSWORD_HASH='$2a$14$CkbCQwO7/duJ/kCObNwEPeiZlmgXrWZ0PXah1DlfkCF70.BIwzZVC'
OTRS_DEVICE_ID=mobile
OTRS_TID=me

#MQTT_URL=mqtts://owntracks.example.org:8883
#MQTT_USER=otrecorder
#MQTT_PASSWORD='mypassword'
```

Make it readable for the current user only:
```
chmod 600 .env
```

Start the containers:
```
docker-compose up -d
```

Using the default values, owntrack-rs will be available at `https://localhost` with login user `owntracks` and password `private`.

### Build From Source

For users who need to install owntrack-rs on platforms that we do not yet provide pre-built binaries for, you will need to build from source.
`owntrack-rs` is written in [Rust](https://rust-lang.org) and uses [cargo](https://doc.rust-lang.org/cargo/index.html) to build. Once you've [installed the Rust toolchain (`rustup`)](https://rustup.rs/), run:

```sh
cargo install --path .
```

## Setup & run

Run with default configuration:

```
owntrack-rs
```

Configuration options are passed as environment variables or read from a `.env` configuration file.

Configuration options:
* `HTTP_LISTEN`: IP address and port to listen on. Default: `0.0.0.0:8083`

### MQTT

For getting location data via MQTT, an MQTT broker like Mosquitto is required.

Configuration options:
* `MQTT_URL`: MQTT broker URL. Example: `mqtts://owntracks.example:8883`
* `MQTT_USER`: MQTT user name.
* `MQTT_PASSWORD`: MQTT password.

### SQLite database

Configuration options:
* `DB_CONNECTION`: Database connection URL. Default: `sqlite://owntracks.sqlite`

### PostgreSQL database

Create database:
```
psql postgres -c "CREATE DATABASE owntracks"
```

Configuration options:
* `DB_CONNECTION`: libpq connection string or PostgreSQL URI. Example: `postgres://user:pass@localhost:5432/owntracks`

## Setup tracking devices

### OwnTracks apps

The [OwnTracks](https://owntracks.org/booklet/) apps can be used in MQTT or HTTP mode.

|  iOS   | Android |
| :----: | :-----: |
| [![AppStore](static/appstore.png)](https://apps.apple.com/us/app/owntracks/id692424691) | [![PlayStore](static/playstore.png)](https://play.google.com/store/apps/details?id=org.owntracks.android) |

For configuring the app, open the setup page of your hosted domain e.g. at `https://owntracks.example.org/setup`.

- [Configure the Android app](https://owntracks.org/booklet/guide/app/android/)

### Meshtastic devices

[Meshtastic](https://meshtastic.org/) devices can be integrated via [MQTT](https://meshtastic.org/docs/software/integrations/mqtt/).

Setup [channel](https://meshtastic.org/docs/configuration/radio/channels/) on nodes:
* Setup a **primary** channel with a private `PSK`
* Activate `uplink_enabled` and `downlink_enabled` (optional)
* Set `position_precision` to `32` (full precision)

Configure [position](https://meshtastic.org/docs/configuration/radio/position/) on nodes:
* Example values for a tracking node:
  * `position.gps_update_interval`: 10 seconds
  * `position.position_broadcast_smart_enabled`: `true`
  * `position.broadcast_smart_minimum_distance`: 10 metres
  * `position.broadcast_smart_minimum_interval_secs`: 10 seconds
  * `position.position_broadcast_secs`: 120 seconds

Configure [LoRa](https://meshtastic.org/docs/configuration/radio/lora/) on nodes:
* `lora.ignore_mqtt`: `false`
* `lora.config_ok_to_mqtt`: `true`

Setup an MQTT gateway node:
* Connect your gateway node to wifi, by setting the `network.wifi_ssid`, `network.wifi_psk` and `network.wifi_enabled` preferences.
* Configure your (MQTT settings)[https://meshtastic.org/docs/configuration/module/mqtt/]: `mqtt.address`, `mqtt.username`, and `mqtt.password`.
  * `mqtt.encryption_enabled`: `false`
  * `mqtt.json_enabled`: `false`
  * `mqtt.tls_enabled`: according to your MQTT server setup
  * `mqtt.root`: according to your MQTT server setup. For an OnwTracks compatible setup use e.g. `owntracks/<user>/msh`.

Tested with Firmware 2.6.4.

### Use your own devices

Send a POST request to the `owntracks` endpoint:
```
curl --data '{"_type":"location","lat":48.856826,"lon":2.292713,"tid":"me","tst":'$(date +%s)'}' -H "Content-Type: application/json" "http://127.0.0.1:8083/owntracks?u=me&d=mydevice"
```

<div class="oranda-hide">

## Development

### Prerequisites

* Just: https://just.systems/man/en/

### Build and run application

```
cargo run
```

### Frontend development

```
cd frontend
```

Preparation:
```
npm install
```

```
npm run dev
```

</div>
