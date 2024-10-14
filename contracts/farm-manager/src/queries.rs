use cosmwasm_std::{Coin, Deps, Env};

use amm::coin::aggregate_coins;
use amm::farm_manager::{
    Config, EpochId, FarmsBy, FarmsResponse, LpWeightResponse, PositionsBy, PositionsResponse,
    RewardsResponse,
};

use crate::farm::commands::calculate_rewards;
use crate::helpers::get_unique_lp_asset_denoms_from_positions;
use crate::state::{
    get_farm_by_identifier, get_farms, get_farms_by_farm_asset, get_farms_by_lp_denom,
    get_position, get_positions, get_positions_by_receiver, CONFIG, LP_WEIGHT_HISTORY,
    MAX_ITEMS_LIMIT,
};
use crate::ContractError;

/// Queries the manager config
pub(crate) fn query_manager_config(deps: Deps) -> Result<Config, ContractError> {
    Ok(CONFIG.load(deps.storage)?)
}

/// Queries all farms. If `lp_asset` is provided, it will return all farms for that
/// particular lp.
pub(crate) fn query_farms(
    deps: Deps,
    filter_by: Option<FarmsBy>,
    start_after: Option<String>,
    limit: Option<u32>,
) -> Result<FarmsResponse, ContractError> {
    let farms = if let Some(filter_by) = filter_by {
        match filter_by {
            FarmsBy::Identifier(identifier) => {
                vec![get_farm_by_identifier(deps.storage, &identifier)?]
            }
            FarmsBy::LpDenom(lp_denom) => {
                get_farms_by_lp_denom(deps.storage, lp_denom.as_str(), start_after, limit)?
            }
            FarmsBy::FarmAsset(farm_asset) => {
                get_farms_by_farm_asset(deps.storage, farm_asset.as_str(), start_after, limit)?
            }
        }
    } else {
        get_farms(deps.storage, start_after, limit)?
    };

    Ok(FarmsResponse { farms })
}

/// Queries all positions. If `open_state` is provided, it will return all positions that match that
/// open state, i.e. open positions if true, closed positions if false.
pub(crate) fn query_positions(
    deps: Deps,
    filter_by: Option<PositionsBy>,
    open_state: Option<bool>,
    start_after: Option<String>,
    limit: Option<u32>,
) -> Result<PositionsResponse, ContractError> {
    let positions = if let Some(filter_by) = filter_by {
        match filter_by {
            PositionsBy::Identifier(identifier) => {
                vec![get_position(deps.storage, Some(identifier.clone()))?
                    .ok_or(ContractError::NoPositionFound { identifier })?]
            }
            PositionsBy::Receiver(receiver) => {
                get_positions_by_receiver(deps.storage, &receiver, open_state, start_after, limit)?
            }
        }
    } else {
        get_positions(deps.storage, start_after, limit)?
    };

    Ok(PositionsResponse { positions })
}

/// Queries the rewards for a given address.
pub(crate) fn query_rewards(
    deps: Deps,
    env: &Env,
    address: String,
) -> Result<RewardsResponse, ContractError> {
    let receiver = deps.api.addr_validate(&address)?;
    // check if the user has any open LP positions
    let open_positions = get_positions_by_receiver(
        deps.storage,
        receiver.as_str(),
        Some(true),
        None,
        Some(MAX_ITEMS_LIMIT),
    )?;

    if open_positions.is_empty() {
        // if the user has no open LP positions, return an empty rewards list
        return Ok(RewardsResponse::RewardsResponse {
            total_rewards: vec![],
            rewards_per_lp_denom: vec![],
        });
    }

    let config = CONFIG.load(deps.storage)?;
    let current_epoch =
        amm::epoch_manager::get_current_epoch(deps, config.epoch_manager_addr.into_string())?;

    let mut total_rewards = vec![];
    let mut rewards_per_lp: Vec<(String, Vec<Coin>)> = vec![];

    let lp_denoms = get_unique_lp_asset_denoms_from_positions(open_positions);

    for lp_denom in &lp_denoms {
        // calculate the rewards for the lp denom
        let rewards_response =
            calculate_rewards(deps, env, lp_denom, &receiver, current_epoch.id, false)?;
        match rewards_response {
            RewardsResponse::QueryRewardsResponse { rewards } => {
                total_rewards.append(&mut rewards.clone());
                //append to rewards_per_lp
                rewards_per_lp.push((lp_denom.clone(), rewards));
            }
            _ => return Err(ContractError::Unauthorized),
        }
    }

    //sort rewards_per_lp by lp denom
    rewards_per_lp.sort_by(|a, b| a.0.cmp(&b.0));

    Ok(RewardsResponse::RewardsResponse {
        total_rewards: aggregate_coins(total_rewards)?,
        rewards_per_lp_denom: rewards_per_lp,
    })
}

/// Queries the total lp weight for the given denom on the given epoch, i.e. the lp weight snapshot.
pub(crate) fn query_lp_weight(
    deps: Deps,
    address: String,
    denom: String,
    epoch_id: EpochId,
) -> Result<LpWeightResponse, ContractError> {
    let lp_weight = LP_WEIGHT_HISTORY
        .may_load(
            deps.storage,
            (&deps.api.addr_validate(&address)?, denom.as_str(), epoch_id),
        )?
        .ok_or(ContractError::LpWeightNotFound { epoch_id })?;

    Ok(LpWeightResponse {
        lp_weight,
        epoch_id,
    })
}
