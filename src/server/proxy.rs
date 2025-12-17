use actix_web::{web, HttpResponse, Scope, HttpRequest};
use actix_web::http::header;
use futures_util::TryStreamExt as _;
use awc::Client;

pub fn proxy_scope() -> Scope {
    web::scope("/proxy")
        .route("/pump_ipfs", web::post().to(pump_ipfs))
}

async fn pump_ipfs(req: HttpRequest, mut payload: web::Payload) -> actix_web::Result<HttpResponse> {
    println!(">>> /proxy/pump_ipfs {} {}", req.method(), req.uri());

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
    println!("  body size: {} bytes", body.len());

    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(90))
        .finish();

    let mut upstream = match client
        .post("https://pump.fun/api/ipfs")
        .insert_header((header::CONTENT_TYPE, content_type))
        .send_body(body.freeze())
        .await
    {
        Ok(resp) => resp,
        Err(e) => {
            eprintln!("!!! upstream send error: {:?}", e);
            // ВАЖНО: возвращаем Ok(HttpResponse), а не Err(...)
            return Ok(
                HttpResponse::BadGateway()
                    .body("upstream (pump.fun) error while sending request")
            );
        }
    };

    let status = upstream.status();
    let bytes = match upstream.body().await {
        Ok(b) => b,
        Err(e) => {
            eprintln!("!!! upstream read error: {:?}", e);
            return Ok(
                HttpResponse::BadGateway()
                    .body("upstream (pump.fun) error while reading response")
            );
        }
    };

    println!("<<< upstream status: {}, bytes: {}", status, bytes.len());

    Ok(HttpResponse::build(status).body(bytes))
}
