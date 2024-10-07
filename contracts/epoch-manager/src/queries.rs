use cosmwasm_std::{ensure, Addr, Deps, Env, StdError, StdResult, Timestamp, Uint64};

use amm::epoch_manager::{ConfigResponse, Epoch, EpochResponse};

use crate::state::{ADMIN, CONFIG};

/// Queries the config. Returns a [ConfigResponse].
pub(crate) fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let admin = ADMIN.get(deps)?.unwrap_or(Addr::unchecked(""));
    let config = CONFIG.load(deps.storage)?;

    Ok(ConfigResponse {
        owner: admin,
        epoch_config: config.epoch_config,
    })
}

/// Derives the current epoch. Returns an [EpochResponse].
pub(crate) fn query_current_epoch(deps: Deps, env: Env) -> StdResult<EpochResponse> {
    let config = CONFIG.load(deps.storage)?;

    ensure!(
        env.block.time.nanos() >= config.epoch_config.genesis_epoch.u64(),
        StdError::generic_err("Genesis epoch has not started")
    );

    let current_epoch = Uint64::new(
        env.block
            .time
            .minus_nanos(config.epoch_config.genesis_epoch.u64())
            .nanos(),
    )
    .checked_div_floor((config.epoch_config.duration.u64(), 1u64))
    .map_err(|e| StdError::generic_err(format!("Error: {:?}", e)))?;

    let start_time = config
        .epoch_config
        .genesis_epoch
        .checked_add(
            current_epoch
                .checked_mul(config.epoch_config.duration)
                .map_err(|e| StdError::generic_err(format!("Error: {:?}", e)))?,
        )
        .map_err(|e| StdError::generic_err(format!("Error: {:?}", e)))?;

    let epoch = Epoch {
        id: current_epoch.u64(),
        start_time: Timestamp::from_nanos(start_time.u64()),
    };

    Ok(EpochResponse { epoch })
}

/// Queries the current epoch. Returns an [EpochResponse].
pub(crate) fn query_epoch(deps: Deps, id: u64) -> StdResult<EpochResponse> {
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
        start_time: Timestamp::from_nanos(start_time.u64()),
    };

    Ok(epoch.to_epoch_response())
}
