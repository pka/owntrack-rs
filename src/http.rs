use crate::db::{Db, TrackRef};
use crate::geojson;
use crate::gpx;
use crate::owntracks::{otrc_json, AppConfig, Message};
use actix_cors::Cors;
use actix_web::{
    error, get, middleware, middleware::Logger, post, route, web, App, HttpRequest, HttpResponse,
    HttpServer, Responder,
};
use actix_web_rust_embed_responder::{EmbedResponse, EmbedableFileResponse, IntoResponse};
use rust_embed_for_web::RustEmbed;
use serde::Deserialize;

#[derive(Deserialize)]
struct OtParams {
    u: Option<String>,
    d: Option<String>,
}

/// OwnTracks endpoint for storing locations
#[post("/owntracks")]
async fn owntracks(
    db: web::Data<Db>,
    msg: web::Json<Message>,
    params: web::Query<OtParams>,
) -> actix_web::Result<impl Responder> {
    log::debug!("{msg:?}");
    if let Message::Location(loc) = msg.into_inner() {
        let user = params.u.clone().unwrap_or("".to_string());
        let device = params.d.clone().unwrap_or("".to_string());
        // TODO: read user/device from msg.topic and/or from X-Limit-U + X-Limit-D headers
        if let Err(e) = db.insert_location(&user, &device, &loc).await {
            log::error!("{e}");
        }
    }
    Ok(web::Json::<Vec<Message>>(Vec::new()))
}

/// Generic JSON endpoint
#[post("/rawjson")]
async fn rawjson(body: String) -> actix_web::Result<impl Responder> {
    // https://github.com/tszheichoi/awesome-sensor-logger
    // {
    //     "messageId": 30,
    //     "sessionId": "7123c623-110c-4cb7-ac56-f7a2402118ab",
    //     "deviceId": "dcfb8d93-d144-4d19-9752-e296a96d7136",
    //     "payload": [{
    //         "values": {
    //             "bearingAccuracy": 45,
    //             "speedAccuracy": 1.5,
    //             "verticalAccuracy": 0.6529032588005066,
    //             "horizontalAccuracy": 15.60099983215332,
    //             "speed": 0.16229715943336487,
    //             "bearing": 201.24215698242188,
    //             "altitude": 549.7999877929688,
    //             "longitude": 9.4370853,
    //             "latitude": 47.0496454
    //         },
    //         "name": "location",
    //         "time": 1745600807648164600
    //     }]
    // }
    log::info!("{body}");
    Ok("ok")
}

#[derive(Deserialize)]
struct TracksParams {
    date: String,
}

#[get("/trackinfos")]
async fn trackinfos(
    db: web::Data<Db>,
    params: web::Query<TracksParams>,
) -> actix_web::Result<impl Responder> {
    match db.query_tracks_info(&params.date).await {
        Ok(track_infos) => Ok(web::Json(track_infos)),
        Err(e) => {
            log::error!("{e}");
            Err(error::ErrorInternalServerError(
                "Failed to fetch track infos",
            ))
        }
    }
}

/// Get GeoJSON track
#[get("/track")]
async fn track(db: web::Data<Db>, track_ref: web::Query<TrackRef>) -> HttpResponse {
    let track = match db.query_track(&track_ref).await {
        Ok(data) => data,
        Err(e) => {
            log::error!("Failed to fetch track: {e}");
            return HttpResponse::InternalServerError()
                .reason("Failed to fetch track")
                .finish();
        }
    };
    let geojson = if track_ref.segmented.unwrap_or(false) {
        geojson::track_with_segments(&[track])
    } else {
        geojson::track(&[track])
    };
    let json = match geojson {
        Ok(json) => json,
        Err(e) => {
            log::error!("Failed to fetch track: {e}");
            return HttpResponse::InternalServerError()
                .reason("Failed to fetch track")
                .finish();
        }
    };
    HttpResponse::Ok()
        .content_type("application/geo+json")
        .body(json)
}

/// Get GPX track
#[get("/gpxtrack")]
async fn gpxtrack(db: web::Data<Db>, track_ref: web::Query<TrackRef>) -> HttpResponse {
    let track_ = match db.query_track(&track_ref).await {
        Ok(data) => data,
        Err(e) => {
            log::error!("Failed to fetch track: {e}");
            return HttpResponse::InternalServerError()
                .reason("Failed to fetch track")
                .finish();
        }
    };
    let gpx = match gpx::tracks(&[track_]) {
        Ok(gpx) => gpx,
        Err(e) => {
            log::error!("Failed to fetch tracks: {e}");
            return HttpResponse::InternalServerError()
                .reason("Failed to fetch track")
                .finish();
        }
    };
    HttpResponse::Ok()
        .content_type("application/gpx+xml")
        .body(gpx)
}

/// Get GeoJSON track points
#[get("/trackpoints")]
async fn trackpoints(db: web::Data<Db>, track_ref: web::Query<TrackRef>) -> HttpResponse {
    let track_ = match db.query_track(&track_ref).await {
        Ok(data) => data,
        Err(e) => {
            log::error!("Failed to fetch track: {e}");
            return HttpResponse::InternalServerError()
                .reason("Failed to fetch track")
                .finish();
        }
    };
    let geojson = geojson::track_points(&[track_]);
    let json = match geojson {
        Ok(json) => json,
        Err(e) => {
            log::error!("Failed to fetch track: {e}");
            return HttpResponse::InternalServerError()
                .reason("Failed to fetch track")
                .finish();
        }
    };
    HttpResponse::Ok()
        .content_type("application/geo+json")
        .body(json)
}

/// Get GeoJSON with current device positions
#[get("/positions")]
async fn positions(db: web::Data<Db>, params: web::Query<TracksParams>) -> HttpResponse {
    let positions = match db.query_positions(&params.date).await {
        Ok(data) => data,
        Err(e) => {
            log::error!("Failed to fetch positions: {e}");
            return HttpResponse::InternalServerError()
                .reason("Failed to fetch positions")
                .finish();
        }
    };
    let json = match geojson::positions(&positions) {
        Ok(json) => json,
        Err(e) => {
            log::error!("Failed to fetch positions: {e}");
            return HttpResponse::InternalServerError()
                .reason("Failed to fetch positions")
                .finish();
        }
    };
    HttpResponse::Ok()
        .content_type("application/geo+json")
        .body(json)
}

#[get("/otrc")]
async fn otrc(db: web::Data<Db>, req: HttpRequest) -> actix_web::Result<impl Responder> {
    match db.is_valid_invite().await {
        Ok(false) | Err(_) => {
            return Err(actix_web::error::ErrorForbidden(""));
        }
        _ => {}
    }
    let conn = req.connection_info();
    let url = format!("{}://{}", conn.scheme(), conn.host());
    let otrc = otrc_json(&AppConfig::from_env(Some(url)));
    Ok(web::Json(otrc))
}

#[derive(RustEmbed)]
#[folder = "./static/"]
struct Embed;

// This responder implements both GET and HEAD
#[route("/{path:.*}", method = "GET", method = "HEAD")]
async fn serve_assets(path: web::Path<String>) -> EmbedResponse<EmbedableFileResponse> {
    let path = match path.as_str() {
        "" => "index.html",
        "setup" => "setup.html",
        p => p,
    };
    Embed::get(path).into_response()
}

pub async fn webserver(db: Db) -> std::io::Result<()> {
    let bind_addr = dotenvy::var("HTTP_LISTEN").unwrap_or("0.0.0.0:8083".to_string());
    log::info!("Listening on http://{bind_addr}/");
    HttpServer::new(move || {
        let cors = if cfg!(debug_assertions) {
            Cors::permissive()
        } else {
            Cors::default()
        };

        // custom `Query` extractor configuration
        let query_cfg = web::QueryConfig::default().error_handler(|err, _req| {
            log::info!("{err}");
            error::InternalError::from_response(err, HttpResponse::Conflict().finish()).into()
        });

        // custom `Json` extractor configuration
        let json_cfg = web::JsonConfig::default()
            // limit request payload size
            .limit(4096)
            .error_handler(|err, _req| {
                log::info!("{err}");
                error::InternalError::from_response(err, HttpResponse::Conflict().into()).into()
            });

        App::new()
            .app_data(query_cfg)
            .app_data(json_cfg)
            .wrap(Logger::default().log_target("owntrack_rs::http"))
            .wrap(middleware::Compress::default())
            .wrap(cors)
            .app_data(web::Data::new(db.clone()))
            .service(owntracks)
            .service(rawjson)
            .service(trackinfos)
            .service(gpxtrack)
            .service(track)
            .service(trackpoints)
            .service(positions)
            .service(otrc)
            .service(serve_assets)
    })
    .bind(bind_addr)?
    .run()
    .await
}
