#![allow(clippy::module_inception)]
use std::fmt;
use std::fmt::Display;

use crate::constants::DAY_IN_SECONDS;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{
    to_json_binary, Addr, Binary, CosmosMsg, Deps, StdError, StdResult, Timestamp, Uint64, WasmMsg,
};
use cw_controllers::HooksResponse;

#[cw_serde]
pub struct InstantiateMsg {
    /// The initial epoch to start the contract with.
    pub start_epoch: Epoch,
    /// The configuration for the epochs.
    pub epoch_config: EpochConfig,
}

#[cw_serde]
pub enum ExecuteMsg {
    /// Creates a new epoch. It's permissionless. A new epoch can only be created after the current
    /// one has ended.
    CreateEpoch,
    /// Adds a new hook to the hook registry, i.e. adds a contract to be notified when a new epoch
    /// is created.
    AddHook {
        /// The address of the contract to be added to the hook registry.
        contract_addr: String,
    },
    /// Removes a hook from the hook registry.
    RemoveHook {
        /// The address of the contract to be removed from the hook registry.
        contract_addr: String,
    },
    /// Updates the contract configuration.
    UpdateConfig {
        /// The new owner of the contract.
        owner: Option<String>,
        /// The new epoch configuration.
        epoch_config: Option<EpochConfig>,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Returns the current epoch, which is the last on the EPOCHS map.
    #[returns(ConfigResponse)]
    Config,
    /// Returns the current epoch, which is the last on the EPOCHS map.
    #[returns(EpochResponse)]
    CurrentEpoch,
    /// Returns the epoch with the given id.
    #[returns(EpochResponse)]
    Epoch {
        /// The id of the epoch to be queried.
        id: u64,
    },
    /// Returns the hooks in the registry.
    #[returns(HooksResponse)]
    Hooks,
    /// Returns whether or not a hook has been registered.
    #[returns(bool)]
    Hook {
        /// The address of the contract to be checked.
        hook: String,
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
    /// The duration of an epoch in nanoseconds.
    pub duration: Uint64,
    /// Timestamp for the first epoch, in nanoseconds.
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

/// The contract configuration.
#[cw_serde]
pub struct Config {
    /// The epoch configuration
    pub epoch_config: EpochConfig,
}

impl Config {
    pub fn to_config_response(self, owner: Addr) -> ConfigResponse {
        ConfigResponse {
            owner,
            epoch_config: self.epoch_config,
        }
    }
}

/// The response for the config query.
#[cw_serde]
pub struct ConfigResponse {
    /// The owner of the contract.
    pub owner: Addr,
    /// The epoch configuration.
    pub epoch_config: EpochConfig,
}

/// The response for the current epoch query.
#[cw_serde]
pub struct EpochResponse {
    /// The epoch queried.
    pub epoch: Epoch,
}

#[cw_serde]
pub struct EpochChangedHookMsg {
    // The current epoch
    pub current_epoch: Epoch,
}

impl EpochChangedHookMsg {
    /// serializes the message
    pub fn into_json_binary(self) -> StdResult<Binary> {
        let msg = EpochChangedExecuteMsg::EpochChangedHook(self);
        to_json_binary(&msg)
    }

    /// creates a cosmos_msg sending this struct to the named contract
    pub fn into_cosmos_msg<T: Into<String>>(self, contract_addr: T) -> StdResult<CosmosMsg> {
        let msg = self.into_json_binary()?;
        let execute = WasmMsg::Execute {
            contract_addr: contract_addr.into(),
            msg,
            funds: vec![],
        };
        Ok(execute.into())
    }
}

#[cw_serde]
enum EpochChangedExecuteMsg {
    EpochChangedHook(EpochChangedHookMsg),
}

/// Queries the current epoch from the epoch manager contract
pub fn get_current_epoch(deps: Deps, epoch_manager_addr: String) -> StdResult<Epoch> {
    let epoch_response: EpochResponse = deps
        .querier
        .query_wasm_smart(epoch_manager_addr, &QueryMsg::CurrentEpoch {})?;

    Ok(epoch_response.epoch)
}

/// Validates that the given epoch has not expired, i.e. not more than 24 hours have passed since the start of the epoch.
pub fn validate_epoch(epoch: &Epoch, current_time: Timestamp) -> StdResult<()> {
    if current_time
        .minus_seconds(epoch.start_time.seconds())
        .seconds()
        < DAY_IN_SECONDS
    {
        return Err(StdError::generic_err(
            "Current epoch has expired, please wait for the next epoch to start.",
        ));
    }

    Ok(())
}
