use crate::services::file_service;
use actix_multipart::Multipart;
use actix_web::{web, HttpResponse, Scope};
use futures_util::stream::TryStreamExt;
use futures_util::StreamExt;

pub fn file_scope() -> Scope {
    web::scope("/files")
        .route("/upload/{name}", web::post().to(upload_png))
        .route("/image/{name}", web::get().to(get_png))
        .route("/avatar/{user_id}", web::get().to(get_avatar))
        .route(
            "/community_image/{token_id}",
            web::get().to(get_community_image),
        )
}

async fn upload_png(path: web::Path<String>, mut payload: Multipart) -> HttpResponse {
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

async fn get_png(path: web::Path<String>) -> HttpResponse {
    let name = path.into_inner();
    if let Some(png) = file_service::load_png(&name) {
        HttpResponse::Ok().content_type("image/png").body(png)
    } else {
        HttpResponse::NotFound().body("Not found")
    }
}

async fn get_avatar(path: web::Path<String>) -> HttpResponse {
    // todo: add id checket to remove symbols
    let user_id = path.into_inner();
    if let Some(png) = file_service::load_avatar(&user_id) {
        HttpResponse::Ok().content_type("image/png").body(png)
    } else {
        HttpResponse::NotFound().body("Not found")
    }
}

async fn get_community_image(path: web::Path<String>) -> HttpResponse {
    let token_id = path.into_inner();
    // todo: add id checket to remove symbols
    // todo: add token_id by other key
    if let Some(png) = file_service::load_png(&token_id) {
        HttpResponse::Ok().content_type("image/png").body(png)
    } else {
        HttpResponse::NotFound().body("Not found")
    }
}
