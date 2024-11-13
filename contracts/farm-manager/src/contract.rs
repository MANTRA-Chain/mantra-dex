use cosmwasm_std::{
    ensure, entry_point, to_json_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response,
};
use cw2::set_contract_version;

use amm::farm_manager::{
    Config, ExecuteMsg, FarmAction, InstantiateMsg, MigrateMsg, PositionAction, QueryMsg,
};
use mantra_utils::validate_contract;

use crate::error::ContractError;
use crate::helpers::{
    validate_emergency_unlock_penalty, validate_farm_expiration_time, validate_unlocking_duration,
};
use crate::state::{CONFIG, FARM_COUNTER};
use crate::{farm, manager, position, queries};

const CONTRACT_NAME: &str = "mantra:farm-manager";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    // ensure that max_concurrent_farms is non-zero
    ensure!(
        msg.max_concurrent_farms > 0,
        ContractError::UnspecifiedConcurrentFarms
    );

    // ensure the unlocking duration range is valid
    validate_unlocking_duration(msg.min_unlocking_duration, msg.max_unlocking_duration)?;

    // ensure the farm expiration time is at least [MONTH_IN_SECONDS]
    validate_farm_expiration_time(msg.farm_expiration_time)?;

    // due to the circular dependency between the pool manager and the farm manager,
    // do not validate the pool manager address here, it has to be updated via the UpdateConfig msg
    // once the pool manager is instantiated
    let config = Config {
        epoch_manager_addr: deps.api.addr_validate(&msg.epoch_manager_addr)?,
        fee_collector_addr: deps.api.addr_validate(&msg.fee_collector_addr)?,
        pool_manager_addr: Addr::unchecked(msg.pool_manager_addr),
        create_farm_fee: msg.create_farm_fee,
        max_concurrent_farms: msg.max_concurrent_farms,
        max_farm_epoch_buffer: msg.max_farm_epoch_buffer,
        min_unlocking_duration: msg.min_unlocking_duration,
        max_unlocking_duration: msg.max_unlocking_duration,
        farm_expiration_time: msg.farm_expiration_time,
        emergency_unlock_penalty: validate_emergency_unlock_penalty(msg.emergency_unlock_penalty)?,
    };

    CONFIG.save(deps.storage, &config)?;
    FARM_COUNTER.save(deps.storage, &0)?;
    cw_ownable::initialize_owner(deps.storage, deps.api, Some(msg.owner.as_str()))?;

    Ok(Response::default().add_attributes(vec![
        ("action", "instantiate".to_string()),
        ("owner", msg.owner),
        ("epoch_manager_addr", config.epoch_manager_addr.to_string()),
        ("fee_collector_addr", config.fee_collector_addr.to_string()),
        ("create_farm_fee", config.create_farm_fee.to_string()),
        (
            "max_concurrent_farms",
            config.max_concurrent_farms.to_string(),
        ),
        (
            "max_farm_epoch_buffer",
            config.max_farm_epoch_buffer.to_string(),
        ),
        (
            "min_unlocking_duration",
            config.min_unlocking_duration.to_string(),
        ),
        (
            "max_unlocking_duration",
            config.max_unlocking_duration.to_string(),
        ),
        (
            "emergency_unlock_penalty",
            config.emergency_unlock_penalty.to_string(),
        ),
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
        ExecuteMsg::ManageFarm { action } => match action {
            FarmAction::Fill { params } => manager::commands::fill_farm(deps, env, info, params),
            FarmAction::Close { farm_identifier } => {
                manager::commands::close_farm(deps, info, farm_identifier)
            }
        },
        ExecuteMsg::UpdateOwnership(action) => {
            cw_utils::nonpayable(&info)?;
            mantra_utils::ownership::update_ownership(deps, env, info, action).map_err(Into::into)
        }
        ExecuteMsg::Claim {} => farm::commands::claim(deps, env, info),
        ExecuteMsg::ManagePosition { action } => match action {
            PositionAction::Create {
                identifier,
                unlocking_duration,
                receiver,
            } => position::commands::create_position(
                deps,
                &env,
                info,
                identifier,
                unlocking_duration,
                receiver,
            ),
            PositionAction::Expand { identifier } => {
                position::commands::expand_position(deps, &env, info, identifier)
            }
            PositionAction::Close {
                identifier,
                lp_asset,
            } => position::commands::close_position(deps, env, info, identifier, lp_asset),
            PositionAction::Withdraw {
                identifier,
                emergency_unlock,
            } => {
                position::commands::withdraw_position(deps, env, info, identifier, emergency_unlock)
            }
        },
        ExecuteMsg::UpdateConfig {
            fee_collector_addr,
            epoch_manager_addr,
            pool_manager_addr,
            create_farm_fee,
            max_concurrent_farms,
            max_farm_epoch_buffer,
            min_unlocking_duration,
            max_unlocking_duration,
            farm_expiration_time,
            emergency_unlock_penalty,
        } => {
            cw_utils::nonpayable(&info)?;
            manager::commands::update_config(
                deps,
                info,
                fee_collector_addr,
                epoch_manager_addr,
                pool_manager_addr,
                create_farm_fee,
                max_concurrent_farms,
                max_farm_epoch_buffer,
                min_unlocking_duration,
                max_unlocking_duration,
                farm_expiration_time,
                emergency_unlock_penalty,
            )
        }
    }
}

#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> Result<Binary, ContractError> {
    match msg {
        QueryMsg::Config {} => Ok(to_json_binary(&queries::query_manager_config(deps)?)?),
        QueryMsg::Ownership {} => Ok(to_json_binary(&cw_ownable::get_ownership(deps.storage)?)?),
        QueryMsg::Farms {
            filter_by,
            start_after,
            limit,
        } => Ok(to_json_binary(&queries::query_farms(
            deps,
            filter_by,
            start_after,
            limit,
        )?)?),
        QueryMsg::Positions {
            filter_by,
            open_state,
            start_after,
            limit,
        } => Ok(to_json_binary(&queries::query_positions(
            deps,
            filter_by,
            open_state,
            start_after,
            limit,
        )?)?),
        QueryMsg::Rewards { address } => Ok(to_json_binary(&queries::query_rewards(
            deps, &env, address,
        )?)?),
        QueryMsg::LpWeight {
            address,
            denom,
            epoch_id,
        } => Ok(to_json_binary(&queries::query_lp_weight(
            deps, address, denom, epoch_id,
        )?)?),
    }
}

#[entry_point]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    validate_contract!(deps, CONTRACT_NAME, CONTRACT_VERSION);
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    Ok(Response::default())
}
