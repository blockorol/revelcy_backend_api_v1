use actix_web::{web, HttpResponse, Scope};
use actix_multipart::Multipart;
use futures_util::stream::TryStreamExt;
use futures_util::StreamExt;
use crate::services::file_service;

pub fn file_scope() -> Scope {
    web::scope("/files")
        .route("/upload/{name}", web::post().to(upload_png)) // todo- remove me
        .route("/image/{name}", web::get().to(get_png))
}

// Загрузка файла
async fn upload_png(
    path: web::Path<String>,
    mut payload: Multipart
) -> HttpResponse {
    let name = path.into_inner();

    while let Ok(Some(mut field)) = payload.try_next().await {
        let mut bytes = web::BytesMut::new();

        while let Some(chunk) = field.next().await {
            match chunk {
                Ok(data) => bytes.extend_from_slice(&data),
                Err(_) => return HttpResponse::BadRequest().body("Invalid file upload"),
            }
        }

        if bytes.starts_with(&[0x89, b'P', b'N', b'G']) {
            if file_service::save_png(&name, &bytes).is_ok() {
                return HttpResponse::Ok().body("Saved");
            }
        }

        return HttpResponse::UnsupportedMediaType().body("Only PNG supported");
    }

    HttpResponse::BadRequest().body("No file uploaded")
}

// Получение файла
async fn get_png(path: web::Path<String>) -> HttpResponse {
    let name = path.into_inner();
    if let Some(png) = file_service::load_png(&name) {
        HttpResponse::Ok()
            .content_type("image/png")
            .body(png)
    } else {
        HttpResponse::NotFound().body("Not found")
    }
}
