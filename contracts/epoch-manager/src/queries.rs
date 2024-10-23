use cosmwasm_std::{ensure, Deps, Env, StdError, Timestamp, Uint64};

use crate::ContractError;
use amm::epoch_manager::{ConfigResponse, Epoch, EpochResponse};

use crate::state::CONFIG;

/// Queries the config. Returns a [ConfigResponse].
pub(crate) fn query_config(deps: Deps) -> Result<ConfigResponse, ContractError> {
    Ok(CONFIG.load(deps.storage)?)
}

/// Derives the current epoch. Returns an [EpochResponse].
pub(crate) fn query_current_epoch(deps: Deps, env: Env) -> Result<EpochResponse, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    ensure!(
        env.block.time.seconds() >= config.epoch_config.genesis_epoch.u64(),
        ContractError::GenesisEpochHasNotStarted
    );

    let current_epoch = Uint64::new(
        env.block
            .time
            .minus_seconds(config.epoch_config.genesis_epoch.u64())
            .seconds(),
    )
    .checked_div_floor((config.epoch_config.duration.u64(), 1u64))
    .map_err(|e| StdError::generic_err(format!("Error: {:?}", e)))?;

    query_epoch(deps, current_epoch.u64())
}

/// Queries the epoch with the given id. Returns an [EpochResponse].
pub(crate) fn query_epoch(deps: Deps, id: u64) -> Result<EpochResponse, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    let start_time = config
        .epoch_config
        .genesis_epoch
        .checked_add(
            Uint64::new(id)
                .checked_mul(config.epoch_config.duration)
                .map_err(|e| StdError::generic_err(format!("Error: {:?}", e)))?,
        )
        .map_err(|e| StdError::generic_err(format!("Error: {:?}", e)))?;

    let epoch = Epoch {
        id,
        start_time: Timestamp::from_seconds(start_time.u64()),
    };

    Ok(epoch.to_epoch_response())
}
