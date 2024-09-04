use cosmwasm_std::{
    CheckedFromRatioError, CheckedMultiplyFractionError, ConversionOverflowError,
    DivideByZeroError, OverflowError, StdError, Uint128,
};
use cw_migrate_error_derive::cw_migrate_invalid_version_error;
use cw_ownable::OwnershipError;
use cw_utils::PaymentError;
use thiserror::Error;

use amm::incentive_manager::EpochId;

#[cw_migrate_invalid_version_error]
#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Semver parsing error: {0}")]
    SemVer(String),

    #[error("Unauthorized")]
    Unauthorized,

    #[error("{0}")]
    PaymentError(#[from] PaymentError),

    #[error("{0}")]
    OwnershipError(#[from] OwnershipError),

    #[error("{0}")]
    OverflowError(#[from] OverflowError),

    #[error("{0}")]
    CheckedFromRatioError(#[from] CheckedFromRatioError),

    #[error("{0}")]
    CheckedMultiplyFractionError(#[from] CheckedMultiplyFractionError),

    #[error("{0}")]
    ConversionOverflowError(#[from] ConversionOverflowError),

    #[error("{0}")]
    DivideByZeroError(#[from] DivideByZeroError),

    #[error("An incentive with the given identifier already exists")]
    IncentiveAlreadyExists,

    #[error("max_concurrent_flows cannot be set to zero")]
    UnspecifiedConcurrentIncentives,

    #[error("Incentive doesn't exist")]
    NonExistentIncentive {},

    #[error("Attempt to create a new incentive with a small incentive_asset amount, which is less than the minimum of {min}")]
    InvalidIncentiveAmount {
        /// The minimum amount of an asset to create an incentive with
        min: u128,
    },

    #[error("Incentive creation fee was not included")]
    IncentiveFeeMissing,

    #[error("Incentive end timestamp was set to a time in the past")]
    IncentiveEndsInPast,

    #[error("The incentive you are intending to create doesn't meet the minimum required of {min} after taking the fee")]
    EmptyIncentiveAfterFee { min: u128 },

    #[error(
        "Incentive creation fee was not fulfilled, only {paid_amount} / {required_amount} present"
    )]
    IncentiveFeeNotPaid {
        /// The amount that was paid
        paid_amount: Uint128,
        /// The amount that needed to be paid
        required_amount: Uint128,
    },

    #[error("Incentive start timestamp is after the end timestamp")]
    IncentiveStartTimeAfterEndTime,

    #[error("Incentive start timestamp is too far into the future")]
    IncentiveStartTooFar,

    #[error("The incentive has already expired, can't be expanded")]
    IncentiveAlreadyExpired,

    #[error("The incentive doesn't have enough funds to pay out the reward")]
    IncentiveExhausted,

    #[error("The asset sent doesn't match the asset expected")]
    AssetMismatch,

    #[error("Attempt to create a new incentive, which exceeds the maximum of {max} incentives allowed per LP at a time")]
    TooManyIncentives {
        /// The maximum amount of incentives that can exist
        max: u32,
    },

    #[error("The end epoch for this incentive is invalid")]
    InvalidEndEpoch,

    #[error("The sender doesn't have open positions")]
    NoOpenPositions,

    #[error("No position found with the given identifier: {identifier}")]
    NoPositionFound { identifier: String },

    #[error("The position has not expired yet")]
    PositionNotExpired,

    #[error("The position with the identifier {identifier} is already closed")]
    PositionAlreadyClosed { identifier: String },

    #[error(
        "Invalid unlocking duration of {specified} specified, must be between {min} and {max}"
    )]
    InvalidUnlockingDuration {
        /// The minimum amount of seconds that a user must lock for.
        min: u64,
        /// The maximum amount of seconds that a user can lock for.
        max: u64,
        /// The amount of seconds the user attempted to lock for.
        specified: u64,
    },

    #[error("Invalid unlocking range, specified min as {min} and max as {max}")]
    InvalidUnlockingRange {
        /// The minimum unlocking time
        min: u64,
        /// The maximum unlocking time
        max: u64,
    },

    #[error("Attempt to compute the weight of a duration of {unlocking_duration} which is outside the allowed bounds")]
    InvalidWeight { unlocking_duration: u64 },

    #[error("The emergency unlock penalty provided is invalid")]
    InvalidEmergencyUnlockPenalty,

    #[error("There are pending rewards to be claimed before this action can be executed")]
    PendingRewards,

    #[error("The incentive expansion amount must be a multiple of the emission rate, which is {emission_rate}")]
    InvalidExpansionAmount {
        /// The emission rate of the incentive
        emission_rate: Uint128,
    },

    #[error("There's no snapshot of the LP weight in the contract for the epoch {epoch_id}")]
    LpWeightNotFound { epoch_id: EpochId },
}

impl From<semver::Error> for ContractError {
    fn from(err: semver::Error) -> Self {
        Self::SemVer(err.to_string())
    }
}
