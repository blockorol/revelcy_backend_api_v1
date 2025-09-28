use actix_web::{web, HttpResponse, Scope};
use awc::Client;
use futures_util::TryStreamExt as _; // для чтения Payload
use actix_web::http::header;

pub fn public_scope() -> Scope {
    web::scope("/proxy")
        .route("/pump_ipfs", web::post().to(pump_ipfs))
}

async fn pump_ipfs(req: actix_web::HttpRequest, mut payload: web::Payload) -> actix_web::Result<HttpResponse> {
    // Логируем входящий запрос
    println!(">>> pump_ipfs: входящий запрос {} {}", req.method(), req.uri());
    for (h, v) in req.headers().iter() {
        println!("  header: {} = {:?}", h, v);
    }

    let content_type = req
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("application/octet-stream")
        .to_string();
    println!("  content-type: {}", content_type);

    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.try_next().await? {
        println!("  получен chunk размером {}", chunk.len());
        body.extend_from_slice(&chunk);
    }
    println!("  общий размер тела: {} байт", body.len());

    // Отправляем в pump.fun
    let client = Client::default();
    println!(">>> pump_ipfs: отправка в https://pump.fun/api/ipfs");
    let mut upstream = client
        .post("https://pump.fun/api/ipfs")
        .insert_header((header::CONTENT_TYPE, content_type))
        .send_body(body.freeze())
        .await
        .map_err(|e| {
            println!("!!! ошибка при отправке в pump.fun: {:?}", e);
            actix_web::error::ErrorBadGateway(e)
        })?;

    let status = upstream.status();
    let bytes = upstream.body().await.map_err(|e| {
        println!("!!! ошибка при чтении ответа pump.fun: {:?}", e);
        actix_web::error::ErrorBadGateway(e)
    })?;

    println!("<<< pump_ipfs: ответ от pump.fun: status = {}, размер {} байт",
        status, bytes.len());

    // Вернём клиенту
    Ok(HttpResponse::build(status)
        .insert_header(("Access-Control-Allow-Origin", "*"))
        .insert_header(("Access-Control-Allow-Methods", "POST, OPTIONS"))
        .insert_header(("Access-Control-Allow-Headers", "Content-Type, Authorization"))
        .body(bytes))
}
