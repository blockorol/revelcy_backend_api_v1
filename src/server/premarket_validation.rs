use chrono::Utc;

use crate::api::errors::{ApiErrorCode, FieldError};
use crate::models::premarket::{BuildJoinTxParams, BuildPremarketTxParams, FullPremarketInfo, PremarketInfoServiceModel, PremarketState};

const MAX_JOIN_SOL_LAMPORTS: u64 = 2 * solana_sdk::native_token::LAMPORTS_PER_SOL;


pub fn validate_create_premarket(
    current_pubkey: &str,
    params: &BuildPremarketTxParams,
    premarket: &PremarketInfoServiceModel
) -> Result<(), Vec<FieldError>> {
    let mut errors = Vec::new();
    let now = Utc::now().timestamp();

    if current_pubkey != premarket.creator.blockchain_address {
        errors.push(FieldError {
            field: "creator|current_pubkey",
            code: ApiErrorCode::PremarketDeadlineTooEarly,
            message: "creator and current_pubkey should be the same",
        });
    }

    if premarket.state != PremarketState::Concept {
        errors.push(FieldError {
            field: "state",
            code: ApiErrorCode::PremarketDeadlineTooEarly,
            message: "state should be Concept",
        });
    }

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


pub fn validate_create_premarket_base(
    deadline: i64,
    goal: u64,
    creator_allocate: u64,
) -> Result<(), Vec<FieldError>> {
    let mut errors = Vec::new();
    let now = Utc::now().timestamp();

    // deadline
    if deadline < now + 60 * 60 - 1 {
        errors.push(FieldError {
            field: "deadline",
            code: ApiErrorCode::PremarketDeadlineTooEarly,
            message: "deadline must be at least +1h from now",
        });
    }

    if deadline > now + 60 * 60 * 24 * 31 {
        errors.push(FieldError {
            field: "deadline",
            code: ApiErrorCode::PremarketDeadlineTooLate,
            message: "deadline must be less than 1 month",
        });
    }

    // goal / max
    if goal == 0 {
        errors.push(FieldError {
            field: "goal",
            code: ApiErrorCode::PremarketGoalOrMaxZero,
            message: "goal must be > 0",
        });
    }

    // creator allocate
    if creator_allocate > goal {
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

pub fn validate_join_premarket(params: &BuildJoinTxParams) -> Result<(), Vec<FieldError>> {
    let mut errors = Vec::new();

    if params.amount == 0 {
        errors.push(FieldError {
            field: "amount_sol_lamp",
            code: ApiErrorCode::PremarketAmountZero,
            message: "amount_sol_lamp must be > 0",
        });
    }
    
    if params.amount > MAX_JOIN_SOL_LAMPORTS {
        errors.push(FieldError {
            field: "amount_sol_lamp",
            code: ApiErrorCode::PremarketJoinAmountTooLarge,
            message: "amount_sol_lamp must be less 2",
        });
    }

    if errors.is_empty() { Ok(()) } else { Err(errors) }
}

// todo: check me and add other fields
pub fn validate_update_uri_premarket(
    state: PremarketState,
    // old_token_info: &TokenInfo,
    // new_token_info: &TokenInfo
) -> Result<(), Vec<FieldError>> {
    let mut errors = Vec::new();

    if state != PremarketState::Premarket {
        errors.push(FieldError {
            field: "premarket",
            code: ApiErrorCode::PremarketWrongStateForExtension,
            message: "premarket is wrong state for extension",
        });
    }
    // if old_token_info.description != new_token_info.description {
    //     errors.push(FieldError {
    //         field: "description",
    //         code: ApiErrorCode::PremarketWrongStateForExtension,
    //         message: "description was changed",
    //     });
    // }
    // if old_token_info.name != old_token_info.name {
    //     errors.push(FieldError {
    //         field: "name",
    //         code: ApiErrorCode::PremarketWrongStateForExtension,
    //         message: "name was changed",
    //     });
    // }
    // if old_token_info.symbol != old_token_info.symbol {
    //     errors.push(FieldError {
    //         field: "symbol",
    //         code: ApiErrorCode::PremarketWrongStateForExtension,
    //         message: "symbol was changed",
    //     });
    // }

    if errors.is_empty() { Ok(()) } else { Err(errors) }
}

pub fn validate_extend_premarket(
    is_extended: bool,
    state: PremarketState,
    current_deadline: i64,
    new_deadline: i64,
) -> Result<(), Vec<FieldError>> {
    let mut errors = Vec::new();
    let now = Utc::now().timestamp();

    if is_extended {
        errors.push(FieldError {
            field: "premarket",
            code: ApiErrorCode::PremarketAlreadyExtended,
            message: "premarket is already extended",
        });
    }

    if state != PremarketState::Premarket {
        errors.push(FieldError {
            field: "premarket",
            code: ApiErrorCode::PremarketWrongStateForExtension,
            message: "premarket is wrong state for extension",
        });
    }

    if current_deadline > now {
        errors.push(FieldError {
            field: "premarket",
            code: ApiErrorCode::PremarketDeadlineNotPassed,
            message: "premarket deadline has not yet passed",
        });
    }

    let max_deadline = now + 7 * 24 * 60 * 60;
    if new_deadline > max_deadline {
        errors.push(FieldError {
            field: "new_deadline",
            code: ApiErrorCode::PremarketExtendTooLate,
            message: "new_deadline cannot be more than 1 week from now",
        });
    }

    if errors.is_empty() { Ok(()) } else { Err(errors) }
}

pub fn validate_finish_premarket(
    premarket: &FullPremarketInfo,
) -> Result<(), Vec<FieldError>> {
    let mut errors = Vec::new();
    // let now = Utc::now().timestamp();

    // 1) state
    if premarket.main_info.state != PremarketState::Premarket {
        errors.push(FieldError {
            field: "premarket",
            code: ApiErrorCode::PremarketFinishWrongState,
            message: "premarket is wrong state for finish",
        });
    }

    // 2) already finished flag (optional but полезно)
    if premarket.main_info.finished_timestamp.is_some() {
        errors.push(FieldError {
            field: "premarket",
            code: ApiErrorCode::PremarketAlreadyFinished,
            message: "premarket is already finished",
        });
    }

    // 3) deadline passed
    // we skip it for the puppy PM
    // if premarket.main_info.deadline_timestamp > now {
    //     errors.push(FieldError {
    //         field: "premarket",
    //         code: ApiErrorCode::PremarketFinishTooEarly,
    //         message: "premarket deadline has not yet passed",
    //     });
    // }

    // 4) goal reached (по твоей динамике это reserved_sol_lamp)
    // Важно: проверь, что это реально та метрика, которая должна сравниваться с goal.
    // Если goal лежит в premarket.main_info.goal.goal_sol_lamp — подставь корректное поле.
    // let goal = premarket.main_info.goal.goal_sol_lamp;
    // if dynamic.reserved_sol_lamp < goal {
    //     errors.push(FieldError {
    //         field: "premarket",
    //         code: ApiErrorCode::PremarketFinishGoalNotReached,
    //         message: "premarket goal not reached",
    //     });
    // }

    if errors.is_empty() { Ok(()) } else { Err(errors) }
}

pub fn validate_refund_premarket(
    main_info: &PremarketInfoServiceModel
)  -> Result<(), Vec<FieldError>> {
    let mut errors = Vec::new();
    let now = Utc::now().timestamp();

    // 1) state
    if main_info.state != PremarketState::Premarket {
        errors.push(FieldError {
            field: "premarket",
            code: ApiErrorCode::PremarketFinishWrongState,
            message: "premarket is wrong state for finish",
        });
    }

    // 2) already finished flag (optional but полезно)
    if main_info.finished_timestamp.is_some() {
        errors.push(FieldError {
            field: "premarket",
            code: ApiErrorCode::PremarketAlreadyFinished,
            message: "premarket is already finished",
        });
    }

    // 3) deadline passed
    if main_info.deadline_timestamp > now {
        errors.push(FieldError {
            field: "premarket",
            code: ApiErrorCode::PremarketFinishTooEarly,
            message: "premarket deadline has not yet passed",
        });
    }

    // 4) goal reached
    // Важно: проверь, что это реально та метрика, которая должна сравниваться с goal.
    // Если goal лежит в premarket.main_info.goal.goal_sol_lamp — подставь корректное поле.
    // let goal = premarket.main_info.goal.goal_sol_lamp;
    // if dynamic.reserved_sol_lamp >= goal {
    //     errors.push(FieldError {
    //         field: "premarket",
    //         code: ApiErrorCode::PremarketFinishGoalNotReached,
    //         message: "premarket goal is reached",
    //     });
    // }

    if errors.is_empty() { Ok(()) } else { Err(errors) }
}

pub fn validate_withdraw_vesting() -> Result<(), Vec<FieldError>> {
    let errors = Vec::new();

    // TODO: Add validation logic for withdraw vesting
    // For now, this is a placeholder that accepts all requests
    
    if errors.is_empty() { Ok(()) } else { Err(errors) }
}