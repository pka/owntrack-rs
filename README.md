# Owntrack-rs

## Installation

```
cargo install --path .
```

## Setup & run

Run with default configuration:

```
owntrack-rs
```

### MQTT

For getting location data via MQTT, an MQTT broker like Mosquitto is required.

Add MQTT access configuration to `.env`:
```
cat >.env <<EOF
MQTT_URL="mqtts://owntracks.example:8883"
MQTT_USER=$(id -u)
MQTT_PASSWORD="xxx"
EOF
chmod 600 .env
edit .env
```

### SQLite database

Add connection information to `.env`:
```
cat >>.env <<EOF
DB_CONNECTION="sqlite://owntracks.sqlite"
EOF
```

### PostgreSQL database

Create database:
```
psql postgres -c "CREATE DATABASE owntracks"
```

Add connection information to `.env`:
```
cat >>.env <<EOF
# libpq connection string or PostgreSQL URI
DB_CONNECTION="postgres://user:pass@localhost:5432/owntracks"
EOF
```

## Development

### Prerequisites

* Just: https://just.systems/man/en/

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
