# Changelog

## 0.7.0 - 2025-05-25

- Initial support for Meshtastic devices

## 0.6.2 - 2025-03-22

- Setup page with OwnTracks configuration

## 0.6.1 - 2025-03-16

- Listen on 0.0.0.0:8083 by default
- Fix statistics without measurements
- Fix parsing of SQLite timestamp
- Include maplibre-gl.css
- Add docker-compose file and instructions

## 0.6.0 - 2025-03-15

- Zoom to positions and tracks
- Calculate track distance
- Add track point layer with statistics
- Select track by clicking on position marker
- Select track by click on row
- Show last device positions
- Add basic frontend layout
- DB schema V2
- Make DB queries SQLite compatible
- Embed DB migrations
- Rename to owntrack-rs

## 0.5.1 - 2025-03-03

- Query single tracks
- Fix user display

## 0.5.0 - 2025-03-01

- Migrate from duckdb-rs to sqlx
- Handle accuracy NULL values
- Embed and serve frontend
- Handle missing speed and elevation
- Share date betwwen components
- Load track list
- Add CORS headers
- Enable HTTP compression
- Add MapLibre GL viewer skeleton
- Ignore missing .env file
- Add GeoJSON endpoint

## 0.4.0 - 2025-02-22

- Store user+device
- Include available GPS measurements in track
- Add Dockerfile

## 0.3.7 - 2025-02-20

- Sleep after MQTT error

## 0.3.6 - 2025-02-20

- Add a GPX tracks endpoint

## 0.3.5 - 2025-02-19

- Add Logger to HTTP endpoint
- Keep session after MQTT reconnect

## 0.3.4 - 2025-02-17

- Fix cog type and continue on decoding error

## 0.3.3 - 2025-02-16

- Revert cargo update

## 0.3.2 - 2025-02-16

- Rollback to duckdb-rs 1.1.1

## 0.3.1 - 2025-02-16

- Add HTTP_LISTEN env var

## 0.3.0 - 2025-02-16

- Add HTTP endpoint

## 0.2.0 - 2025-01-12

- Add OwnTracks fields

## 0.1.1 - 2025-01-05

- Fix longitude

## 0.1.0 - 2025-01-03

- Insert locations into DB
- Decode location payload
- Add refinery migrations
- Load PostgreSQL extension
- Read MQTT configuration from .env
- Add MQTT client
- duckdb-rs usage example
