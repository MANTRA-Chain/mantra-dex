use cosmwasm_std::{
    entry_point, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response,
};
use cosmwasm_std::{wasm_execute, Reply, StdError};
use cw2::set_contract_version;

use amm::pool_manager::{ExecuteMsg, FeatureToggle, InstantiateMsg, MigrateMsg, QueryMsg};
use mantra_utils::validate_contract;

use crate::error::ContractError;
use crate::helpers::validate_asset_balance;
use crate::state::{
    Config, SingleSideLiquidityProvisionBuffer, CONFIG, POOL_COUNTER,
    SINGLE_SIDE_LIQUIDITY_PROVISION_BUFFER,
};
use crate::{liquidity, manager, queries, router, swap};

// version info for migration info
const CONTRACT_NAME: &str = "mantra:pool-manager";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const SINGLE_SIDE_LIQUIDITY_PROVISION_REPLY_ID: u64 = 1;

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    let config: Config = Config {
        fee_collector_addr: deps.api.addr_validate(&msg.fee_collector_addr)?,
        farm_manager_addr: deps.api.addr_validate(&msg.farm_manager_addr)?,
        pool_creation_fee: msg.pool_creation_fee.clone(),
        feature_toggle: FeatureToggle {
            withdrawals_enabled: true,
            deposits_enabled: true,
            swaps_enabled: true,
        },
    };
    CONFIG.save(deps.storage, &config)?;
    // initialize pool counter
    POOL_COUNTER.save(deps.storage, &0u64)?;
    cw_ownable::initialize_owner(deps.storage, deps.api, Some(info.sender.as_str()))?;

    Ok(Response::default().add_attributes(vec![
        ("action", "instantiate".to_string()),
        ("owner", info.sender.to_string()),
        ("fee_collector_addr", msg.fee_collector_addr),
        ("farm_manager_addr", msg.farm_manager_addr),
        ("pool_creation_fee", msg.pool_creation_fee.to_string()),
    ]))
}

#[entry_point]
pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> Result<Response, ContractError> {
    match msg.id {
        SINGLE_SIDE_LIQUIDITY_PROVISION_REPLY_ID => {
            let SingleSideLiquidityProvisionBuffer {
                receiver,
                expected_offer_asset_balance_in_contract,
                expected_ask_asset_balance_in_contract,
                offer_asset_half,
                expected_ask_asset,
                liquidity_provision_data,
            } = SINGLE_SIDE_LIQUIDITY_PROVISION_BUFFER.load(deps.storage)?;

            validate_asset_balance(&deps, &env, &expected_offer_asset_balance_in_contract)?;
            validate_asset_balance(&deps, &env, &expected_ask_asset_balance_in_contract)?;

            SINGLE_SIDE_LIQUIDITY_PROVISION_BUFFER.remove(deps.storage);

            Ok(Response::default().add_message(wasm_execute(
                env.contract.address.into_string(),
                &ExecuteMsg::ProvideLiquidity {
                    slippage_tolerance: liquidity_provision_data.slippage_tolerance,
                    max_spread: liquidity_provision_data.max_spread,
                    receiver: Some(receiver),
                    pool_identifier: liquidity_provision_data.pool_identifier,
                    unlocking_duration: liquidity_provision_data.unlocking_duration,
                    lock_position_identifier: liquidity_provision_data.lock_position_identifier,
                },
                vec![offer_asset_half, expected_ask_asset],
            )?))
        }
        _ => Err(StdError::generic_err("reply id not found").into()),
    }
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::CreatePool {
            asset_denoms,
            asset_decimals,
            pool_fees,
            pool_type,
            pool_identifier,
        } => manager::commands::create_pool(
            deps,
            env,
            info,
            asset_denoms,
            asset_decimals,
            pool_fees,
            pool_type,
            pool_identifier,
        ),
        ExecuteMsg::ProvideLiquidity {
            max_spread,
            slippage_tolerance,
            receiver,
            pool_identifier,
            unlocking_duration,
            lock_position_identifier,
        } => liquidity::commands::provide_liquidity(
            deps,
            env,
            info,
            slippage_tolerance,
            max_spread,
            receiver,
            pool_identifier,
            unlocking_duration,
            lock_position_identifier,
        ),
        ExecuteMsg::Swap {
            ask_asset_denom,
            belief_price,
            max_spread,
            receiver,
            pool_identifier,
        } => swap::commands::swap(
            deps,
            info.clone(),
            info.sender,
            ask_asset_denom,
            belief_price,
            max_spread,
            receiver,
            pool_identifier,
        ),
        ExecuteMsg::WithdrawLiquidity { pool_identifier } => {
            liquidity::commands::withdraw_liquidity(deps, env, info, pool_identifier)
        }
        ExecuteMsg::UpdateOwnership(action) => {
            mantra_utils::ownership::update_ownership(deps, env, info, action).map_err(Into::into)
        }
        ExecuteMsg::ExecuteSwapOperations {
            operations,
            minimum_receive,
            receiver,
            max_spread,
        } => router::commands::execute_swap_operations(
            deps,
            info,
            operations,
            minimum_receive,
            receiver,
            max_spread,
        ),
        ExecuteMsg::UpdateConfig {
            fee_collector_addr,
            farm_manager_addr,
            pool_creation_fee,
            feature_toggle,
        } => manager::update_config(
            deps,
            info,
            fee_collector_addr,
            farm_manager_addr,
            pool_creation_fee,
            feature_toggle,
        ),
    }
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> Result<Binary, ContractError> {
    match msg {
        QueryMsg::Config {} => Ok(to_json_binary(&queries::query_config(deps)?)?),
        QueryMsg::AssetDecimals {
            pool_identifier,
            denom,
        } => Ok(to_json_binary(&queries::query_asset_decimals(
            deps,
            pool_identifier,
            denom,
        )?)?),
        QueryMsg::Simulation {
            offer_asset,
            ask_asset_denom,
            pool_identifier,
        } => Ok(to_json_binary(&queries::query_simulation(
            deps,
            offer_asset,
            ask_asset_denom,
            pool_identifier,
        )?)?),
        QueryMsg::ReverseSimulation {
            ask_asset,
            offer_asset_denom,
            pool_identifier,
        } => Ok(to_json_binary(&queries::query_reverse_simulation(
            deps,
            ask_asset,
            offer_asset_denom,
            pool_identifier,
        )?)?),
        QueryMsg::SimulateSwapOperations {
            offer_amount,
            operations,
        } => Ok(to_json_binary(&queries::simulate_swap_operations(
            deps,
            offer_amount,
            operations,
        )?)?),
        QueryMsg::ReverseSimulateSwapOperations {
            ask_amount,
            operations,
        } => Ok(to_json_binary(&queries::reverse_simulate_swap_operations(
            deps, ask_amount, operations,
        )?)?),
        QueryMsg::Ownership {} => Ok(to_json_binary(&cw_ownable::get_ownership(deps.storage)?)?),
        QueryMsg::Pools {
            pool_identifier,
            start_after,
            limit,
        } => Ok(to_json_binary(&queries::get_pools(
            deps,
            pool_identifier,
            start_after,
            limit,
        )?)?),
    }
}

#[entry_point]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    validate_contract!(deps, CONTRACT_NAME, CONTRACT_VERSION);
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    Ok(Response::default())
}
