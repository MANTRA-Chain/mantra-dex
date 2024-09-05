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

    // Handle ownership errors from cw-ownable
    #[error("{0}")]
    OwnershipError(#[from] OwnershipError),

    // Handle Upgrade/Migrate related semver errors
    #[error("Semver parsing error: {0}")]
    SemVer(String),

    // Handle errors specific to payments from cw-util
    #[error("{0}")]
    PaymentError(#[from] PaymentError),
}

impl From<semver::Error> for ContractError {
    fn from(err: semver::Error) -> Self {
        Self::SemVer(err.to_string())
    }
}
