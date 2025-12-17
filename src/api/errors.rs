// src/api/errors.rs
use actix_web::{HttpResponse, ResponseError};
use actix_web::http::StatusCode;
use serde::Serialize;
use std::fmt;

pub type ApiResult<T> = Result<T, ApiError>;

/// Коды ошибок для фронта. Можно расширять по мере роста API.
#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ApiErrorCode {
    // Basic
    InvalidNetwork,
    InvalidUserPubkey,
    WrongUserPubkeyForUser,
    AuthMissingWallet,

    // Premarket creation-validation
    PremarketDeadlineTooEarly,
    PremarketDeadlineTooLate,
    PremarketGoalOrMaxZero,
    PremarketMaxLessThanGoal,
    PremarketCreatorAllocateGreaterThanGoal,

    // Premarket Join-validation
    PremarketAmountZero,
    PremarketJoinAmountTooLarge,

    // Premarket Mint-validation
    InvalidTokenMintPubkey,

    // Premarket extend-validation
    PremarketAlreadyExtended,
    PremarketWrongStateForExtension,
    PremarketDeadlineNotPassed,
    PremarketExtendTooLate,

    // Premarket-finish validation
    PremarketNotFound,
    PremarketFinishWrongState,
    PremarketFinishTooEarly,
    PremarketFinishGoalNotReached,
    PremarketAlreadyFinished,

    // Internal Error
    InternalBuildTxFailed,
    InternalSignTxFailed,

    // basic error - about validation
    ValidationError,
    InvalidTxType,
    InvalidPremarketPubkey,
    MissingPremarket,
}

/// Описание ошибки конкретного поля.
#[derive(Debug, Serialize)]
pub struct FieldError {
    pub field: &'static str,
    pub code: ApiErrorCode,
    pub message: &'static str,
}

/// Тело ответа при ошибке.
/// Может описывать как одну ошибку, так и набор полевых ошибок.
#[derive(Debug, Serialize)]
pub struct ApiErrorResponse {
    /// Общий тип ошибки: "validation_error", "unauthorized", "forbidden" и т.п.
    pub error: &'static str,

    /// Код ошибки (machine-readable).
    pub code: ApiErrorCode,

    /// Для одиночной ошибки — поле, к которому она относится.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub field: Option<&'static str>,

    /// Для одиночной ошибки — текст сообщения.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,

    /// Для валидационных ошибок формы — список ошибок по полям.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<Vec<FieldError>>,
}

#[derive(Debug)]
pub struct ApiError {
    pub response: ApiErrorResponse,
}

impl ApiError {
    pub fn invalid_token_mint() -> Self {
        Self {
            response: ApiErrorResponse {
                error: "validation_error",
                code: ApiErrorCode::InvalidTokenMintPubkey,
                field: Some("token_mint"),
                message: Some("invalid token_mint pubkey format".into()),
                errors: None,
            },
        }
    }

    pub fn premarket_amount_zero() -> Self {
        Self {
            response: ApiErrorResponse {
                error: "validation_error",
                code: ApiErrorCode::PremarketAmountZero,
                field: Some("amount_sol_lamp"),
                message: Some("amount_sol_lamp must be less than 2".into()),
                errors: None,
            },
        }
    }

    pub fn premarket_amount_too_large() -> Self {
        Self {
            response: ApiErrorResponse {
                error: "validation_error",
                code: ApiErrorCode::PremarketJoinAmountTooLarge,
                field: Some("amount_sol_lamp"),
                message: Some("amount_sol_lamp must be > 0".into()),
                errors: None,
            },
        }
    }

    pub fn internal_sign_tx_failed() -> Self {
        Self {
            response: ApiErrorResponse {
                error: "internal_error",
                code: ApiErrorCode::InternalSignTxFailed,
                field: None,
                message: Some("failed to sign transaction".into()),
                errors: None,
            },
        }
    }
    pub fn invalid_premarket_pubkey() -> Self {
        Self {
            response: ApiErrorResponse {
                error: "validation_error",
                code: ApiErrorCode::InvalidPremarketPubkey,
                field: Some("premarket"),
                message: Some("invalid premarket pubkey format".into()),
                errors: None,
            },
        }
    }

    pub fn missing_premarket() -> Self {
        Self {
            response: ApiErrorResponse {
                error: "validation_error",
                code: ApiErrorCode::MissingPremarket,
                field: Some("premarket"),
                message: Some("missing premarket field".into()),
                errors: None,
            },
        }
    }

    pub fn internal_build_tx_failed() -> Self {
        Self {
            response: ApiErrorResponse {
                error: "internal_error",
                code: ApiErrorCode::InternalBuildTxFailed,
                field: None,
                message: Some("failed to build transaction".into()),
                errors: None,
            },
        }
    }

    /// 400 invalid network
    pub fn invalid_network() -> Self {
        Self {
            response: ApiErrorResponse {
                error: "validation_error",
                code: ApiErrorCode::InvalidNetwork,
                field: Some("network"),
                message: Some("invalid network".into()),
                errors: None,
            },
        }
    }

    /// 400 invalid format user_pubkey
    pub fn invalid_user_pubkey() -> Self {
        Self {
            response: ApiErrorResponse {
                error: "validation_error",
                code: ApiErrorCode::InvalidUserPubkey,
                field: Some("user_pubkey"),
                message: Some("invalid user_pubkey".into()),
                errors: None,
            },
        }
    }

    /// 403
    pub fn wrong_user_pubkey_for_user() -> Self {
        Self {
            response: ApiErrorResponse {
                error: "forbidden",
                code: ApiErrorCode::WrongUserPubkeyForUser,
                field: Some("user_pubkey"),
                message: Some("user_pubkey not linked with current user".into()),
                errors: None,
            },
        }
    }

    /// 401
    pub fn auth_missing_wallet() -> Self {
        Self {
            response: ApiErrorResponse {
                error: "unauthorized",
                code: ApiErrorCode::AuthMissingWallet,
                field: Some("user_pubkey"),
                message: Some("user has no linked wallet".into()),
                errors: None,
            },
        }
    }

    /// validation error with 1 format (Vec<FieldError> → один ответ).
    pub fn from_field_errors(errors: Vec<FieldError>) -> Self {
        Self {
            response: ApiErrorResponse {
                error: "validation_error",
                code: ApiErrorCode::ValidationError,
                field: None,
                message: None,
                errors: Some(errors),
            },
        }
    }
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // for log just error + code
        write!(f, "{} ({:?})", self.response.error, self.response.code)
    }
}

impl ResponseError for ApiError {
    fn status_code(&self) -> StatusCode {
        use ApiErrorCode::*;
        match self.response.code {
            InvalidNetwork
            | InvalidUserPubkey
            | PremarketDeadlineTooEarly
            | PremarketDeadlineTooLate
            | PremarketGoalOrMaxZero
            | PremarketMaxLessThanGoal
            | PremarketCreatorAllocateGreaterThanGoal
            | ValidationError
            | InvalidTxType
            | InvalidPremarketPubkey 
            | PremarketAmountZero
            | PremarketJoinAmountTooLarge
            | InvalidTokenMintPubkey
            | PremarketAlreadyExtended
            | PremarketWrongStateForExtension
            | PremarketDeadlineNotPassed
            | PremarketExtendTooLate
            | PremarketNotFound
            | PremarketFinishWrongState
            | PremarketFinishTooEarly
            | PremarketFinishGoalNotReached
            | PremarketAlreadyFinished
            | MissingPremarket
            => StatusCode::BAD_REQUEST,
            
            WrongUserPubkeyForUser => StatusCode::FORBIDDEN,
            AuthMissingWallet => StatusCode::UNAUTHORIZED,

            InternalBuildTxFailed
            | InternalSignTxFailed => StatusCode::INTERNAL_SERVER_ERROR,

        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).json(&self.response)
    }
}
