use actix_web::http::header;
use actix_web::{web, HttpRequest, HttpResponse, Scope};
use futures_util::TryStreamExt as _;

use crate::services::http_client;

pub fn proxy_scope() -> Scope {
    web::scope("/proxy").route("/pump_ipfs", web::post().to(pump_ipfs))
}

async fn pump_ipfs(req: HttpRequest, mut payload: web::Payload) -> actix_web::Result<HttpResponse> {
    tracing::info!(">>> /proxy/pump_ipfs {} {}", req.method(), req.uri());

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
    tracing::info!("  body size: {} bytes", body.len());

    let client = match http_client::long_timeout_client() {
        Ok(client) => client,
        Err(e) => {
            tracing::error!("!!! http client build error: {:?}", e);
            return Ok(HttpResponse::BadGateway().body("failed to build upstream http client"));
        }
    };

    let upstream = match client
        .post("https://pump.fun/api/ipfs")
        .header(reqwest::header::CONTENT_TYPE, content_type)
        .body(body.freeze())
        .send()
        .await
    {
        Ok(resp) => resp,
        Err(e) => {
            tracing::error!("!!! upstream send error: {:?}", e);
            // ВАЖНО: возвращаем Ok(HttpResponse), а не Err(...)
            return Ok(
                HttpResponse::BadGateway().body("upstream (pump.fun) error while sending request")
            );
        }
    };

    let status = actix_web::http::StatusCode::from_u16(upstream.status().as_u16())
        .unwrap_or(actix_web::http::StatusCode::BAD_GATEWAY);
    let bytes = match upstream.bytes().await {
        Ok(b) => b,
        Err(e) => {
            tracing::error!("!!! upstream read error: {:?}", e);
            return Ok(
                HttpResponse::BadGateway().body("upstream (pump.fun) error while reading response")
            );
        }
    };

    tracing::info!("<<< upstream status: {}, bytes: {}", status, bytes.len());

    Ok(HttpResponse::build(status).body(bytes))
}
