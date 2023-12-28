use cosmwasm_std::{OverflowError, StdError, Uint128};
use cw_ownable::OwnershipError;
use semver::Version;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Semver parsing error: {0}")]
    SemVer(String),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("{0}")]
    OwnershipError(#[from] OwnershipError),

    #[error("{0}")]
    OverflowError(#[from] OverflowError),

    #[error("An incentive with the given identifier already exists")]
    IncentiveAlreadyExists,

    #[error("max_concurrent_flows cannot be set to zero")]
    UnspecifiedConcurrentIncentives,

    #[error("Invalid unbonding range, specified min as {min} and max as {max}")]
    InvalidUnbondingRange {
        /// The minimum unbonding time
        min: u64,
        /// The maximum unbonding time
        max: u64,
    },

    #[error("Incentive doesn't exist")]
    NonExistentIncentive {},

    #[error("Attempt to create a new incentive, which exceeds the maximum of {max} incentives allowed per LP at a time")]
    TooManyIncentives {
        /// The maximum amount of incentives that can exist
        max: u32,
    },

    #[error("Attempt to create a new incentive with a small incentive_asset amount, which is less than the minimum of {min}")]
    InvalidIncentiveAmount {
        /// The minimum amount of an asset to create an incentive with
        min: u128,
    },

    #[error("The asset sent is not supported for fee payments")]
    FeeAssetNotSupported,

    #[error("Incentive creation fee was not included")]
    IncentiveFeeMissing,

    #[error("The incentive you are intending to create doesn't meet the minimum required of {min} after taking the fee")]
    EmptyIncentiveAfterFee { min: u128 },

    #[error("The asset sent doesn't match the asset expected")]
    AssetMismatch,

    #[error(
        "Incentive creation fee was not fulfilled, only {paid_amount} / {required_amount} present"
    )]
    IncentiveFeeNotPaid {
        /// The amount that was paid
        paid_amount: Uint128,
        /// The amount that needed to be paid
        required_amount: Uint128,
    },

    #[error("Specified incentive asset was not transferred")]
    IncentiveAssetNotSent,

    #[error("The end epoch for this incentive is invalid")]
    InvalidEndEpoch {},

    #[error("Incentive end timestamp was set to a time in the past")]
    IncentiveEndsInPast,

    #[error("Incentive start timestamp is after the end timestamp")]
    IncentiveStartTimeAfterEndTime,

    #[error("Incentive start timestamp is too far into the future")]
    IncentiveStartTooFar,

    #[error("Attempt to migrate to version {new_version}, but contract is on a higher version {current_version}")]
    MigrateInvalidVersion {
        new_version: Version,
        current_version: Version,
    },
}

impl From<semver::Error> for ContractError {
    fn from(err: semver::Error) -> Self {
        Self::SemVer(err.to_string())
    }
}
