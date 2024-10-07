use cosmwasm_std::{ensure, entry_point, to_json_binary};
use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;

use amm::epoch_manager::{Config, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use mantra_utils::validate_contract;

use crate::error::ContractError;
use crate::state::{ADMIN, CONFIG};
use crate::{commands, queries};

// version info for migration info
const CONTRACT_NAME: &str = "mantra_epoch-manager";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[entry_point]
pub fn instantiate(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    // validate start_time for the initial epoch
    ensure!(
        msg.epoch_config.genesis_epoch.u64() >= env.block.time.nanos(), //todo change to seconds
        ContractError::InvalidStartTime
    );

    ADMIN.set(deps.branch(), Some(info.sender))?;

    CONFIG.save(
        deps.storage,
        &Config {
            epoch_config: msg.epoch_config.clone(),
        },
    )?;
    Ok(Response::default().add_attributes(vec![
        ("action", "instantiate".to_string()),
        ("epoch_config", msg.epoch_config.to_string()),
    ]))
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::UpdateConfig {
            owner,
            epoch_config,
        } => commands::update_config(deps, &info, owner, epoch_config),
    }
}

#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => Ok(to_json_binary(&queries::query_config(deps)?)?),
        QueryMsg::CurrentEpoch {} => Ok(to_json_binary(&queries::query_current_epoch(deps, env)?)?),
        QueryMsg::Epoch { id } => Ok(to_json_binary(&queries::query_epoch(deps, id)?)?),
    }
}

#[entry_point]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    validate_contract!(deps, CONTRACT_NAME, CONTRACT_VERSION);
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    Ok(Response::default())
}
