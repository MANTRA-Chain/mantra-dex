use cosmwasm_std::StdError;
use cw_migrate_error_derive::cw_migrate_invalid_version_error;
use cw_ownable::OwnershipError;
use cw_utils::PaymentError;
use thiserror::Error;

#[cw_migrate_invalid_version_error]
#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    OwnershipError(#[from] OwnershipError),

    #[error("{0}")]
    PaymentError(#[from] PaymentError),

    #[error("Semver parsing error: {0}")]
    SemVer(String),

    #[error("The genesis epoch has not started yet.")]
    GenesisEpochHasNotStarted,

    #[error("start_time must be in the future.")]
    InvalidStartTime,

    #[error("Invalid epoch duration, must be at least {min}.")]
    InvalidEpochDuration { min: u64 },
}

impl From<semver::Error> for ContractError {
    fn from(err: semver::Error) -> Self {
        Self::SemVer(err.to_string())
    }
}
