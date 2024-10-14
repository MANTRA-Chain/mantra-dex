use cosmwasm_std::{
    CheckedFromRatioError, CheckedMultiplyFractionError, ConversionOverflowError,
    DivideByZeroError, OverflowError, StdError, Uint128,
};
use cw_migrate_error_derive::cw_migrate_invalid_version_error;
use cw_ownable::OwnershipError;
use cw_utils::PaymentError;
use thiserror::Error;

use amm::farm_manager::EpochId;

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

    #[error("A farm with the given identifier already exists")]
    FarmAlreadyExists,

    #[error("max_concurrent_farms cannot be set to zero")]
    UnspecifiedConcurrentFarms,

    #[error("Farm doesn't exist")]
    NonExistentFarm,

    #[error("Attempt to create a new farm with a small farm_asset amount, which is less than the minimum of {min}")]
    InvalidFarmAmount {
        /// The minimum amount of an asset to create a farm with
        min: u128,
    },

    #[error("Farm creation fee was not included")]
    FarmFeeMissing,

    #[error("Farm end timestamp was set to a time in the past")]
    FarmEndsInPast,

    #[error("The farm you are intending to create doesn't meet the minimum required of {min} after taking the fee")]
    EmptyFarmAfterFee { min: u128 },

    #[error("Farm creation fee was not fulfilled, only {paid_amount} / {required_amount} present")]
    FarmFeeNotPaid {
        /// The amount that was paid
        paid_amount: Uint128,
        /// The amount that needed to be paid
        required_amount: Uint128,
    },

    #[error("Farm start timestamp is after the end timestamp")]
    FarmStartTimeAfterEndTime,

    #[error("Farm start timestamp is too far into the future")]
    FarmStartTooFar,

    #[error("The farm has already expired, can't be expanded")]
    FarmAlreadyExpired,

    #[error("The expiration time for the farm is invalid, must be at least {min} seconds")]
    FarmExpirationTimeInvalid { min: u64 },

    #[error("The farm doesn't have enough funds to pay out the reward")]
    FarmExhausted,

    #[error("The asset sent doesn't match the asset expected")]
    AssetMismatch,

    #[error("Attempt to create a new farm, which exceeds the maximum of {max} farms allowed per LP at a time")]
    TooManyFarms {
        /// The maximum amount of farms that can exist
        max: u32,
    },

    #[error("Attempt to decrease the max concurrent farms to a value that is less than the current amount of concurrent farms")]
    MaximumConcurrentFarmsDecreased,

    #[error("The {which} epoch for this farm is invalid")]
    InvalidEpoch { which: String },

    #[error("The sender doesn't have open positions")]
    NoOpenPositions,

    #[error("No position found with the given identifier: {identifier}")]
    NoPositionFound { identifier: String },

    #[error("The position has not expired yet")]
    PositionNotExpired,

    #[error("The position with the identifier {identifier} is already closed")]
    PositionAlreadyClosed { identifier: String },

    #[error("The amount of LP specified when closing the position is invalid. Expected at most {expected}, actual {actual}.")]
    InvalidLpAmount { expected: Uint128, actual: Uint128 },

    #[error("Maximum number of open/close positions per user exceeded, max is {max}. If you are trying to open a position, close some and try again. If you are trying to close a position, withdraw some and try again.")]
    MaxPositionsPerUserExceeded { max: u32 },

    #[error("The position with the identifier {identifier} already exists")]
    PositionAlreadyExists { identifier: String },

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

    #[error("The farm expansion amount must be a multiple of the emission rate, which is {emission_rate}")]
    InvalidExpansionAmount {
        /// The emission rate of the farm
        emission_rate: Uint128,
    },

    #[error("There's no snapshot of the LP weight in the contract for the epoch {epoch_id}")]
    LpWeightNotFound { epoch_id: EpochId },

    #[error("Invalid identifier provided: {identifier}.")]
    InvalidIdentifier { identifier: String },
}

impl From<semver::Error> for ContractError {
    fn from(err: semver::Error) -> Self {
        Self::SemVer(err.to_string())
    }
}
