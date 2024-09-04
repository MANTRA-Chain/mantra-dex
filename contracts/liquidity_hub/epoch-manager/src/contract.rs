use cosmwasm_std::{ensure, entry_point, to_json_binary};
use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;

use amm::epoch_manager::{Config, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use mantra_utils::validate_contract;

use crate::error::ContractError;
use crate::state::{ADMIN, CONFIG, EPOCHS};
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
        msg.start_epoch.start_time >= env.block.time,
        ContractError::InvalidStartTime
    );

    ensure!(
        msg.epoch_config.genesis_epoch.u64() == msg.start_epoch.start_time.nanos(),
        ContractError::EpochConfigMismatch
    );

    ADMIN.set(deps.branch(), Some(info.sender))?;
    EPOCHS.save(deps.storage, msg.start_epoch.id, &msg.start_epoch)?;

    CONFIG.save(
        deps.storage,
        &Config {
            epoch_config: msg.epoch_config.clone(),
        },
    )?;
    Ok(Response::default().add_attributes(vec![
        ("action", "instantiate".to_string()),
        ("start_epoch", msg.start_epoch.to_string()),
        ("epoch_config", msg.epoch_config.to_string()),
    ]))
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    let api = deps.api;
    match msg {
        ExecuteMsg::AddHook { contract_addr } => {
            commands::add_hook(deps, info, api, &contract_addr)
        }
        ExecuteMsg::RemoveHook { contract_addr } => {
            commands::remove_hook(deps, info, api, &contract_addr)
        }
        ExecuteMsg::CreateEpoch => commands::create_epoch(deps, env, info),
        ExecuteMsg::UpdateConfig {
            owner,
            epoch_config,
        } => commands::update_config(deps, &info, owner, epoch_config),
    }
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config => Ok(to_json_binary(&queries::query_config(deps)?)?),
        QueryMsg::CurrentEpoch => Ok(to_json_binary(&queries::query_current_epoch(deps)?)?),
        QueryMsg::Epoch { id } => Ok(to_json_binary(&queries::query_epoch(deps, id)?)?),
        QueryMsg::Hooks => Ok(to_json_binary(&queries::query_hooks(deps)?)?),
        QueryMsg::Hook { hook } => Ok(to_json_binary(&queries::query_hook(deps, hook)?)?),
    }
}

#[entry_point]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    validate_contract!(deps, CONTRACT_NAME, CONTRACT_VERSION);
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    Ok(Response::default())
}
