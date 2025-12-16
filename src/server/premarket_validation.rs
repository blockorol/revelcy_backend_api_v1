use chrono::Utc;

use crate::api::errors::{ApiErrorCode, FieldError};
use crate::models::premarket::BuildPremarketTxParams;

pub fn validate_create_premarket(
    params: &BuildPremarketTxParams,
) -> Result<(), Vec<FieldError>> {
    let mut errors = Vec::new();
    let now = Utc::now().timestamp();

    // deadline
    if params.deadline < now + 60 * 60 - 1 {
        errors.push(FieldError {
            field: "deadline",
            code: ApiErrorCode::PremarketDeadlineTooEarly,
            message: "deadline must be at least +1h from now",
        });
    }

    if params.deadline > now + 60 * 60 * 24 * 31 {
        errors.push(FieldError {
            field: "deadline",
            code: ApiErrorCode::PremarketDeadlineTooLate,
            message: "deadline must be less than 1 month",
        });
    }

    // goal / max
    if params.goal == 0 || params.max == 0 {
        errors.push(FieldError {
            field: "goal_sol_lamp,max_sol_lamp",
            code: ApiErrorCode::PremarketGoalOrMaxZero,
            message: "goal_sol_lamp and max_sol_lamp must be > 0",
        });
    }

    if params.max < params.goal {
        errors.push(FieldError {
            field: "max_sol_lamp",
            code: ApiErrorCode::PremarketMaxLessThanGoal,
            message: "max_sol_lamp must be >= goal_sol_lamp",
        });
    }

    // creator allocate
    if params.creator_allocate > params.goal {
        errors.push(FieldError {
            field: "creator_allocate_lamp",
            code: ApiErrorCode::PremarketCreatorAllocateGreaterThanGoal,
            message: "creator_allocate_lamp must be <= goal_sol_lamp",
        });
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}
