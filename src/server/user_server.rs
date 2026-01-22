use actix_web::{web, web::BytesMut, HttpResponse, HttpRequest, HttpMessage, Scope};
use actix_multipart::Multipart;
use sqlx::PgPool;
use crate::api::user::{
    AddAvatarResponseDto,
    AddUserNameRequestDto, AddUserNameResponseDto,
    SearchUsersRequestDto, SearchUsersResponseDto,
    SetInviteCodeRequestDto, SetInviteCodeResponseDto,
    UserDto, UserSetInfoRequestDTO, UserSetInfoResponseDTO
};
use crate::api::errors::{ApiError, ApiErrorCode, FieldError, ApiResult};
use uuid::Uuid;

use crate::models::user::{ApplyInviteCodeResult, ScreenInfo, UserFingerprintEventFrontendData, UserFingerprintEventBackendData};
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
        .route("/set_invite_code", web::post().to(set_invite_code))
        .route("/update_username", web::post().to(update_username))
        .route("/update_avatar", web::post().to(update_avatar))
        .route("/search", web::post().to(search_users))
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
        .ok()
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .unwrap_or_else(|| "unknown".to_string());

    let fe_data: UserFingerprintEventFrontendData = UserFingerprintEventFrontendData {
        user_id: dto.client.user_id,
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

async fn set_invite_code(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    payload: web::Json<SetInviteCodeRequestDto>,
) -> ApiResult<HttpResponse> {
    let dto = payload.into_inner();

    let user_info = extract_user_info_from_request(&req)
        .map_err(|_| ApiError::auth_invalid_token())?;

    let invite_code = dto.invite_code.trim();
    if invite_code.is_empty() {
        return Err(ApiError::from_field_errors(vec![FieldError {
            field: "invite_code",
            code: ApiErrorCode::ValidationError,
            message: "invite_code is empty",
        }]));
    }

    let result = user_service::set_invite_code_once(pool.get_ref(), user_info.user_id, invite_code)
        .await
        .map_err(|e| {
            eprintln!("set_invite_code db error: {e:?}");
            ApiError::internal_update_db_error()
        })?;

    match result {
        ApplyInviteCodeResult::Applied => {
            let token = jwt_service::create_jwt_with_user(
                user_info.user_id,
                user_info.current_wallet.as_deref().unwrap_or_default(),
                Some(user_info.username.clone()),
                user_info.avatar_url.clone(),
                &user_info.nonce,
            );
            Ok(HttpResponse::Ok().json(SetInviteCodeResponseDto { jwt: token }))
        }
        ApplyInviteCodeResult::InviteCodeNotFound => Err(ApiError::invite_code_not_found()),
        ApplyInviteCodeResult::AlreadyApplied => Err(ApiError::invite_code_already_applied()),
    }
}


pub async fn update_username(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    payload: web::Json<AddUserNameRequestDto>,
) -> ApiResult<HttpResponse> {
    let dto = payload.into_inner();

    let user_info = extract_user_info_from_request(&req)
        .map_err(|_| ApiError::auth_invalid_token())?;

    let username = dto.username.trim();
    if username.is_empty() {
        return Err(ApiError::from_field_errors(vec![FieldError {
            field: "username",
            code: ApiErrorCode::ValidationError,
            message: "username is empty",
        }]));
    }

    user_service::set_username(pool.get_ref(), user_info.user_id, username)
        .await
        .map_err(|e| {
            eprintln!("update_username db error: {e:?}");
            ApiError::internal_update_db_error()
        })?;

    let token = jwt_service::create_jwt_with_user(
        user_info.user_id,
        user_info.current_wallet.as_deref().unwrap_or_default(),
        Some(username.to_string()),
        user_info.avatar_url.clone(),
        &user_info.nonce,
    );

    Ok(HttpResponse::Ok().json(AddUserNameResponseDto {
        username: username.to_string(),
        jwt: token,
    }))
}

pub async fn update_avatar(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    mut payload: Multipart,
) -> ApiResult<HttpResponse> {
    let user_info = extract_user_info_from_request(&req)
        .map_err(|_| ApiError::auth_invalid_token())?;

    let bytes = extract_single_png_from_multipart(&mut payload).await
        .map_err(|_| ApiError::from_field_errors(vec![FieldError {
            field: "avatar",
            code: ApiErrorCode::ValidationError,
            message: "invalid image upload".into(),
        }]))?;

    let avatar_url = user_service::set_avatar(pool.get_ref(), user_info.user_id, &bytes)
        .await
        .map_err(|e| {
            eprintln!("update_avatar save error: {e:?}");
            ApiError::internal_update_db_error()
        })?;

    let token = jwt_service::create_jwt_with_user(
        user_info.user_id,
        user_info.current_wallet.as_deref().unwrap_or_default(),
        Some(user_info.username.clone()),
        Some(avatar_url.clone()),
        &user_info.nonce,
    );

    Ok(HttpResponse::Ok().json(AddAvatarResponseDto {
        avatar_url,
        jwt: token,
    }))
}

pub async fn search_users(
    _req: HttpRequest,
    pool: web::Data<PgPool>,
    payload: web::Json<SearchUsersRequestDto>,
) -> ApiResult<HttpResponse> {
    let dto = payload.into_inner();
    let input = dto.input.trim();
    let limit = dto.limit.clamp(1, 50);

    if input.is_empty() {
        return Ok(HttpResponse::Ok().json(SearchUsersResponseDto { items: vec![] }));
    }

    let users = user_service::search_by_username(pool.get_ref(), input, limit)
        .await
        .map_err(|e| {
            eprintln!("search_username db error: {e:?}");
            ApiError::internal_get_db_error()
        })?;

    let items = users
        .into_iter()
        .map(|u| UserDto {
            id: u.id.to_string(),
            username: u.username,
            avatar_url: u.avatar_url,
            wallets: u.wallets,
        })
        .collect();

    Ok(HttpResponse::Ok().json(SearchUsersResponseDto { items }))
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

