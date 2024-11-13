use cosmwasm_std::{ensure, entry_point, to_json_binary};
use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response};
use cw2::set_contract_version;

use amm::epoch_manager::{Config, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use mantra_utils::validate_contract;

use crate::error::ContractError;
use crate::helpers::validate_epoch_duration;
use crate::state::CONFIG;
use crate::{commands, queries};

// version info for migration info
const CONTRACT_NAME: &str = "mantra:epoch-manager";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    // validate start_time for the initial epoch
    ensure!(
        msg.epoch_config.genesis_epoch.u64() >= env.block.time.seconds(),
        ContractError::InvalidStartTime
    );

    validate_epoch_duration(msg.epoch_config.duration)?;

    cw_ownable::initialize_owner(deps.storage, deps.api, Some(msg.owner.as_str()))?;

    CONFIG.save(
        deps.storage,
        &Config {
            epoch_config: msg.epoch_config.clone(),
        },
    )?;

    Ok(Response::default().add_attributes(vec![
        ("action", "instantiate".to_string()),
        ("owner", msg.owner),
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
    match msg {
        ExecuteMsg::UpdateConfig { epoch_config } => {
            cw_utils::nonpayable(&info)?;
            commands::update_config(deps, &info, epoch_config)
        }
        ExecuteMsg::UpdateOwnership(action) => {
            cw_utils::nonpayable(&info)?;
            mantra_utils::ownership::update_ownership(deps, env, info, action).map_err(Into::into)
        }
    }
}

#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> Result<Binary, ContractError> {
    match msg {
        QueryMsg::Config {} => Ok(to_json_binary(&queries::query_config(deps)?)?),
        QueryMsg::CurrentEpoch {} => Ok(to_json_binary(&queries::query_current_epoch(deps, env)?)?),
        QueryMsg::Epoch { id } => Ok(to_json_binary(&queries::query_epoch(deps, id)?)?),
        QueryMsg::Ownership {} => Ok(to_json_binary(&cw_ownable::get_ownership(deps.storage)?)?),
    }
}

#[entry_point]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    validate_contract!(deps, CONTRACT_NAME, CONTRACT_VERSION);
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    Ok(Response::default())
}
