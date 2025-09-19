use actix_web::{web, web::BytesMut, HttpResponse, HttpRequest, Scope};
use actix_multipart::Multipart;
use sqlx::PgPool;
use crate::api::dto::*;
use crate::services::jwt_service;
use crate::services::user_service;
use futures_util::{StreamExt, TryStreamExt};


pub fn user_scope() -> Scope {
    web::scope("/user")
        .route("/update_username", web::post().to(update_username))
        .route("/update_avatar", web::post().to(update_avatar))
}

async fn update_username(
    pool: web::Data<PgPool>,
    req: HttpRequest,
    payload: web::Json<AddUserNameRequestDto>
) -> HttpResponse {
    let user_info = match extract_user_info_from_request(&req) {
        Ok(user_info) => user_info,
        Err(response) => return response,
    };

    if let Err(err) = user_service::set_username(pool, user_info.user_id, &payload.username).await {
        return HttpResponse::Unauthorized().body(format!("err in creation: {}", err));
    }
    let token = jwt_service::create_jwt_with_user(
        user_info.user_id,
        user_info
            .current_wallet
            .as_deref()
            .unwrap_or_default(),
        Some(payload.username.clone()),
        user_info.avatar_url.clone(),
        &user_info.nonce,
    );


    let response = AddUserNameResponseDto {
        username: payload.username.to_string(),
        jwt: token,
    };
    HttpResponse::Ok().json(response)
}

async fn update_avatar(
    pool: web::Data<PgPool>,
    req: HttpRequest,
    mut payload: Multipart,
) -> HttpResponse {
    let user_info = match extract_user_info_from_request(&req) {
        Ok(user_info) => user_info,
        Err(response) => return response,
    };

    let bytes = match extract_single_png_from_multipart(&mut payload).await {
        Ok(bytes) => bytes,
        Err(resp) => return resp,
    };

    let avatar_url = match user_service::set_avatar(pool.clone(), user_info.user_id, &bytes).await {
        Ok(url) => url,
        Err(err) => return HttpResponse::InternalServerError()
            .body(format!("Avatar saving failed: {}", err)),
    };

    let token = jwt_service::create_jwt_with_user(
        user_info.user_id,
        user_info
            .current_wallet
            .as_deref()
            .unwrap_or_default(),
        Some(user_info.username.clone()),
        Some(avatar_url.clone()),
        &user_info.nonce,
    );

    let response = AddAvatarResponseDto {
        avatar_url: avatar_url.to_string(),
        jwt: token,
    };
    HttpResponse::Ok().json(response)
}

pub fn extract_user_info_from_request(req: &HttpRequest) -> Result<jwt_service::TokenWithUserInfo, HttpResponse> {
    let token = req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .ok_or_else(|| HttpResponse::Unauthorized().body("Missing or invalid token"))?;

    jwt_service::decode_jwt_with_user_info(token).map_err(|err| {
        HttpResponse::Unauthorized().body(format!("JWT is broken: {}", err))
    })
}

pub async fn extract_single_png_from_multipart(
    payload: &mut Multipart,
) -> Result<BytesMut, HttpResponse> {
    // let mut file_found = false;

    while let Ok(Some(mut field)) = payload.try_next().await {
        // if file_found {
        //     return Err(HttpResponse::BadRequest().body("Only one file is allowed"));
        // }
        // file_found = true;

        let mut bytes = BytesMut::new();

        while let Some(chunk) = field.next().await {
            match chunk {
                Ok(data) => bytes.extend_from_slice(&data),
                Err(err) => {
                    eprintln!("Multipart read error: {:?}", err);
                    return Err(HttpResponse::BadRequest().body("Invalid file upload"));
                }
            }
        }

        if !bytes.starts_with(&[0x89, b'P', b'N', b'G']) {
            return Err(HttpResponse::UnsupportedMediaType().body("Only PNG supported"));
        }

        return Ok(bytes);
    }

    Err(HttpResponse::BadRequest().body("No file uploaded"))
}

