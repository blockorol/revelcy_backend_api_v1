use actix_web::{web, web::BytesMut, HttpResponse, HttpRequest, HttpMessage, Scope};
use actix_multipart::Multipart;
use sqlx::PgPool;
use crate::api::premarket;
use crate::api::user::{
    AddAvatarResponseDto,
    AddUserNameRequestDto,
    AddUserNameResponseDto,
    UserSetInfoRequestDTO,
    UserSetInfoResponseDTO
};
use crate::api::errors::{ApiError, ApiResult};
use uuid::Uuid;

use crate::models::user::{ScreenInfo, UserFingerprintEventFrontendData, UserFingerprintEventBackendData};
use crate::services::jwt_service;
use crate::services::user_service;
use futures_util::{StreamExt, TryStreamExt};
use crate::server::user_server_extractor::{
    extract_client_ip,
    extract_header,
};
use crate::services::user_info_service;



pub fn user_scope() -> Scope {
    web::scope("/user")
        .route("/set_additional_info", web::post().to(user_set_info))
        .route("/update_username", web::post().to(update_username))
        .route("/update_avatar", web::post().to(update_avatar))
}
pub async fn user_set_info(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    payload: web::Json<UserSetInfoRequestDTO>,
) -> ApiResult<HttpResponse> {
    let dto = payload.into_inner();

    // ─────────────────────────────────────────────────────────────
    // JwtMiddleware already put token into extensions
    // ─────────────────────────────────────────────────────────────
    let token = req
        .extensions()
        .get::<String>()
        .cloned()
        .unwrap_or_default();

    let token_data = jwt_service::decode_jwt_with_user_info(&token)
        .ok();

    let user_id_opt: Option<Uuid> = token_data
        .and_then(|t| Some(t.user_id));

    // ─────────────────────────────────────────────────────────────
    // Backend-collected request context (source of truth)
    // ─────────────────────────────────────────────────────────────
    let ip = extract_client_ip(&req).unwrap_or_else(|| "default".into());
    let user_agent = extract_header(&req, "user-agent").unwrap_or_else(|| "default".into());
    let accept_language = extract_header(&req, "accept-language").unwrap_or_else(|| "default".into());

    let sec_ch_ua = extract_header(&req, "sec-ch-ua").unwrap_or_else(|| "default".into());
    let sec_ch_ua_platform = extract_header(&req, "sec-ch-ua-platform").unwrap_or_else(|| "default".into());
    let sec_ch_ua_mobile = extract_header(&req, "sec-ch-ua-mobile").unwrap_or_else(|| "default".into());
    let event_type_str = serde_json::to_value(&dto.event_type)
        .map_err(|e| {
            eprintln!("user_set_info: serialize event_type error: {e:?}");
            ApiError::internal_server_error()
        })?
        .as_str()
        .unwrap_or("other")
        .to_string();

    let fe_data: UserFingerprintEventFrontendData = UserFingerprintEventFrontendData {
        event_type: event_type_str,

        install_id: dto.client.install_id.unwrap_or_else(|| "default".into()),
        install_id_source: dto.client.install_id_source.unwrap_or_else(|| "default".into()),

        client_ts_ms: dto.client_timestamp_ms,

        timezone: dto.client.timezone,
        locale: dto.client.locale,
        languages: dto.client.languages,
        language: dto.client.language,
        screen: ScreenInfo {
            height: dto.client.screen_height,
            width: dto.client.screen_width,
        },
        
        pixel_ratio: dto.client.pixel_ratio,
        user_agent: dto.client.user_agent,
        phantom_version: dto.client.phantom_version,
    };
    
    let be_data: UserFingerprintEventBackendData= UserFingerprintEventBackendData{
        ip,
        user_agent,
        accept_language,
        sec_ch_ua,
        sec_ch_ua_platform,
        sec_ch_ua_mobile,
    };
    user_info_service::write_user_fingerprint_event(
        pool.get_ref(),
        user_id_opt,
        dto.premarket,
        fe_data,
        be_data,
    )
    .await
    .map_err(|e| {
        eprintln!("user_set_info: db write error: {e:?}");
        ApiError::internal_server_error()
    })?;

    Ok(HttpResponse::Ok().json(UserSetInfoResponseDTO { ok: true }))
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
    println!("🔄 [UPDATE_AVATAR] Starting avatar update request");
    
    // Extract user info from JWT token
    let user_info = match extract_user_info_from_request(&req) {
        Ok(user_info) => {
            println!("✅ [UPDATE_AVATAR] Successfully extracted user info - User ID: {}, Username: {:?}, Wallet: {:?}", 
                user_info.user_id, 
                user_info.username, 
                user_info.current_wallet
            );
            user_info
        },
        Err(response) => {
            println!("❌ [UPDATE_AVATAR] Failed to extract user info from request");
            return response;
        },
    };

    // Extract image file from multipart payload
    println!("📁 [UPDATE_AVATAR] Extracting image file from multipart payload");
    let bytes = match extract_single_png_from_multipart(&mut payload).await {
        Ok(bytes) => {
            println!("✅ [UPDATE_AVATAR] Successfully extracted image file - Size: {} bytes", bytes.len());
            bytes
        },
        Err(resp) => {
            println!("❌ [UPDATE_AVATAR] Failed to extract image file from multipart payload - Response: {:?}", resp);
            return resp;
        },
    };

    // Save avatar to storage
    println!("💾 [UPDATE_AVATAR] Saving avatar to storage for user ID: {}", user_info.user_id);
    let avatar_url = match user_service::set_avatar(pool.clone(), user_info.user_id, &bytes).await {
        Ok(url) => {
            println!("✅ [UPDATE_AVATAR] Successfully saved avatar - URL: {}", url);
            url
        },
        Err(err) => {
            println!("❌ [UPDATE_AVATAR] Failed to save avatar - Error: {}", err);
            return HttpResponse::InternalServerError()
                .body(format!("Avatar saving failed: {}", err));
        },
    };

    // Create new JWT token with updated avatar
    println!("🔐 [UPDATE_AVATAR] Creating new JWT token with updated avatar");
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
    
    println!("✅ [UPDATE_AVATAR] Successfully completed avatar update - Final avatar URL: {}", response.avatar_url);
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
    let mut field_count = 0;

    loop {
        match payload.try_next().await {
            Ok(Some(mut field)) => {
                field_count += 1;
                println!("📄 [EXTRACT_PNG] Processing field #{} - Name: {:?}, Content-Type: {:?}", 
                    field_count, 
                    field.name(), 
                    field.content_type()
                );

                let mut bytes = BytesMut::new();
                let mut chunk_count = 0;

                while let Some(chunk) = field.next().await {
                    chunk_count += 1;
                    match chunk {
                        Ok(data) => {
                            println!("📦 [EXTRACT_PNG] Received chunk #{} - Size: {} bytes", chunk_count, data.len());
                            bytes.extend_from_slice(&data);
                        },
                        Err(err) => {
                            println!("❌ [EXTRACT_PNG] Multipart read error on chunk #{}: {:?}", chunk_count, err);
                            return Err(HttpResponse::BadRequest().body("Invalid file upload"));
                        }
                    }
                }

                println!("📊 [EXTRACT_PNG] Field processing complete - Total bytes: {}, Chunks received: {}", bytes.len(), chunk_count);

                // Check image signature (PNG or JPEG)
                if bytes.len() < 4 {
                    println!("❌ [EXTRACT_PNG] File too small to be a valid image ({} bytes)", bytes.len());
                    return Err(HttpResponse::BadRequest().body("File too small"));
                }

                let image_signature = &bytes[0..4];
                println!("🔍 [EXTRACT_PNG] Checking image signature - First 4 bytes: {:?}", image_signature);
                
                let is_png = bytes.starts_with(&[0x89, b'P', b'N', b'G']);
                let is_jpeg = bytes.starts_with(&[0xFF, 0xD8, 0xFF]);
                
                if !is_png && !is_jpeg {
                    println!("❌ [EXTRACT_PNG] Invalid image signature - Expected PNG [137, 80, 78, 71] or JPEG [255, 216, 255], Got: {:?}", image_signature);
                    return Err(HttpResponse::UnsupportedMediaType().body("Only PNG and JPEG supported"));
                }

                let image_type = if is_png { "PNG" } else { "JPEG" };
                println!("✅ [EXTRACT_PNG] Valid {} file extracted - Size: {} bytes", image_type, bytes.len());
                return Ok(bytes);
            },
            Ok(None) => {
                println!("❌ [EXTRACT_PNG] No more fields in multipart payload (processed {} fields)", field_count);
                break;
            },
            Err(err) => {
                println!("❌ [EXTRACT_PNG] Error parsing multipart payload: {:?}", err);
                return Err(HttpResponse::BadRequest().body("Invalid multipart data"));
            }
        }
    }

    println!("❌ [EXTRACT_PNG] No fields found in multipart payload (processed {} fields)", field_count);
    Err(HttpResponse::BadRequest().body("No file uploaded"))
}

