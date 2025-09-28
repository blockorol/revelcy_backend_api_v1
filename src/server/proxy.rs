use actix_web::{web, HttpResponse, Scope, HttpRequest};
use actix_web::http::header;
use futures_util::TryStreamExt as _;
use log::info;
use awc::Client;

pub fn proxy_scope() -> Scope {
    web::scope("/proxy")
        .route("/pump_ipfs", web::post().to(pump_ipfs))
        .route("/pump_ipfs", web::options().to(pump_ipfs_options))
}

async fn pump_ipfs_options() -> HttpResponse {
    HttpResponse::NoContent()
        .insert_header(("Access-Control-Allow-Origin", "*"))
        .insert_header(("Access-Control-Allow-Methods", "POST, OPTIONS"))
        .insert_header(("Access-Control-Allow-Headers", "Content-Type, Authorization"))
        .finish()
}

async fn pump_ipfs(req: HttpRequest, mut payload: web::Payload) -> actix_web::Result<HttpResponse> {
    info!(">>> /proxy/pump_ipfs {} {}", req.method(), req.uri());

    let content_type = req
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("application/octet-stream")
        .to_string();

    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.try_next().await? {
        body.extend_from_slice(&chunk);
    }
    info!("  body size: {} bytes", body.len());

    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(90))
        .finish();

    let mut upstream = client
        .post("https://pump.fun/api/ipfs")
        .insert_header((header::CONTENT_TYPE, content_type))
        .send_body(body.freeze())
        .await
        .map_err(|e| {
            info!("!!! upstream send error: {:?}", e);
            actix_web::error::ErrorBadGateway(e)
        })?;

    let status = upstream.status();
    let bytes = upstream.body().await.map_err(|e| {
        info!("!!! upstream read error: {:?}", e);
        actix_web::error::ErrorBadGateway(e)
    })?;
    info!("<<< upstream status: {}, bytes: {}", status, bytes.len());

    Ok(HttpResponse::build(status)
        .insert_header(("Access-Control-Allow-Origin", "*"))
        .insert_header(("Access-Control-Allow-Methods", "POST, OPTIONS"))
        .insert_header(("Access-Control-Allow-Headers", "Content-Type, Authorization"))
        .body(bytes))
}
