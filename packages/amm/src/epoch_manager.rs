#![allow(clippy::module_inception)]
use std::fmt;
use std::fmt::Display;

use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Deps, StdResult, Timestamp, Uint64};
use cw_ownable::{cw_ownable_execute, cw_ownable_query};

#[cw_serde]
pub struct InstantiateMsg {
    /// The owner of the contract.
    pub owner: String,
    /// The configuration for the epochs.
    pub epoch_config: EpochConfig,
}

#[cw_ownable_execute]
#[cw_serde]
pub enum ExecuteMsg {
    /// Updates the contract configuration.
    UpdateConfig {
        /// The new epoch configuration.
        epoch_config: Option<EpochConfig>,
    },
}

#[cw_ownable_query]
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Returns the configuration of the contract.
    #[returns(ConfigResponse)]
    Config {},
    /// Returns the current epoch, which is the last on the EPOCHS map.
    #[returns(EpochResponse)]
    CurrentEpoch {},
    /// Returns the epoch with the given id.
    #[returns(EpochResponse)]
    Epoch {
        /// The id of the epoch to be queried.
        id: u64,
    },
}

#[cw_serde]
pub struct MigrateMsg {}

/// The epoch definition.
#[cw_serde]
#[derive(Default)]
pub struct Epoch {
    // Epoch identifier
    pub id: u64,
    // Epoch start time
    pub start_time: Timestamp,
}

impl Epoch {
    pub fn to_epoch_response(self) -> EpochResponse {
        EpochResponse { epoch: self }
    }
}

impl Display for Epoch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Epoch {{ id: {}, start_time: {} }}",
            self.id, self.start_time,
        )
    }
}

/// The epoch configuration.
#[cw_serde]
pub struct EpochConfig {
    /// The duration of an epoch in seconds.
    pub duration: Uint64,
    /// Timestamp for the first epoch, in seconds.
    pub genesis_epoch: Uint64,
}

impl Display for EpochConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "EpochConfig {{ epoch_duration: {}, genesis_epoch: {}, }}",
            self.duration, self.genesis_epoch
        )
    }
}

pub type ConfigResponse = Config;

/// The contract configuration.
#[cw_serde]
pub struct Config {
    /// The epoch configuration
    pub epoch_config: EpochConfig,
}

/// The response for the current epoch query.
#[cw_serde]
pub struct EpochResponse {
    /// The epoch queried.
    pub epoch: Epoch,
}

/// Queries the current epoch from the epoch manager contract
pub fn get_current_epoch(deps: Deps, epoch_manager_addr: String) -> StdResult<Epoch> {
    let epoch_response: EpochResponse = deps
        .querier
        .query_wasm_smart(epoch_manager_addr, &QueryMsg::CurrentEpoch {})?;

    Ok(epoch_response.epoch)
}
