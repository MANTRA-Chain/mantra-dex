use amm::farm_manager::{Position, RewardsResponse};
use cosmwasm_std::{
    coin, ensure, Addr, BankMsg, Coin, CosmosMsg, Decimal, DepsMut, Env, MessageInfo, Response,
    StdError, Uint128,
};

use crate::helpers::validate_lp_denom;
use crate::position::helpers::{
    calculate_weight, get_latest_address_weight, AUTO_POSITION_ID_PREFIX,
    EXPLICIT_POSITION_ID_PREFIX,
};
use crate::position::helpers::{
    validate_positions_limit, validate_unlocking_duration_for_position,
};
use crate::queries::query_rewards;
use crate::state::{get_position, CONFIG, LP_WEIGHT_HISTORY, POSITIONS, POSITION_ID_COUNTER};
use crate::ContractError;

/// Creates a position
pub(crate) fn create_position(
    deps: DepsMut,
    env: &Env,
    info: MessageInfo,
    identifier: Option<String>,
    unlocking_duration: u64,
    receiver: Option<String>,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let lp_asset = cw_utils::one_coin(&info)?;

    // ensure the lp denom is valid and was created by the pool manager
    validate_lp_denom(&lp_asset.denom, config.pool_manager_addr.as_str())?;

    // validate unlocking duration
    validate_unlocking_duration_for_position(&config, unlocking_duration)?;

    // if a receiver was specified, check that it was the pool manager who
    // is sending the message, as it has the possibility to lock LP tokens on
    // behalf of the user
    let receiver = if let Some(ref receiver) = receiver {
        let receiver = deps.api.addr_validate(receiver)?;
        ensure!(
            info.sender == config.pool_manager_addr || info.sender == receiver,
            ContractError::Unauthorized
        );

        receiver
    } else {
        info.sender.clone()
    };

    // computes the position identifier
    let position_id_counter = POSITION_ID_COUNTER
        .may_load(deps.storage)?
        .unwrap_or_default()
        + 1u64;

    // compute the identifier for this position
    let identifier = if let Some(identifier) = identifier {
        // prepend EXPLICIT_POSITION_ID_PREFIX to identifier
        format!("{EXPLICIT_POSITION_ID_PREFIX}{identifier}")
    } else {
        // prepend AUTO_POSITION_ID_PREFIX to the position_id_counter
        format!("{AUTO_POSITION_ID_PREFIX}{position_id_counter}")
    };

    // check if there's an existing position with the computed identifier
    let position = get_position(deps.storage, Some(identifier.clone()))?;

    ensure!(
        position.is_none(),
        ContractError::PositionAlreadyExists {
            identifier: identifier.clone(),
        }
    );

    // No position found, create a new one

    // ensure the user doesn't have more than the maximum allowed close positions
    validate_positions_limit(deps.as_ref(), &receiver, true)?;

    POSITION_ID_COUNTER.save(deps.storage, &position_id_counter)?;

    let position = Position {
        identifier: identifier.clone(),
        lp_asset: lp_asset.clone(),
        unlocking_duration,
        open: true,
        expiring_at: None,
        receiver: receiver.clone(),
    };

    POSITIONS.save(deps.storage, &identifier, &position)?;

    // Update weights for the LP and the user
    update_weights(deps, env, &receiver, &lp_asset, unlocking_duration, true)?;

    Ok(Response::default().add_attributes(vec![
        ("action", "open_position".to_string()),
        ("position", position.to_string()),
    ]))
}

/// Expands an existing position
pub(crate) fn expand_position(
    deps: DepsMut,
    env: &Env,
    info: MessageInfo,
    identifier: String,
) -> Result<Response, ContractError> {
    let mut position = get_position(deps.storage, Some(identifier.clone()))?.ok_or(
        ContractError::NoPositionFound {
            identifier: identifier.clone(),
        },
    )?;

    let lp_asset = cw_utils::one_coin(&info)?;

    // ensure the lp denom is valid and was created by the pool manager
    let config = CONFIG.load(deps.storage)?;
    validate_lp_denom(&lp_asset.denom, config.pool_manager_addr.as_str())?;

    // make sure the lp asset sent matches the lp asset of the position
    ensure!(
        position.lp_asset.denom == lp_asset.denom,
        ContractError::AssetMismatch
    );

    ensure!(
        position.open,
        ContractError::PositionAlreadyClosed {
            identifier: position.identifier.clone(),
        }
    );

    // ensure only the receiver itself or the pool manager can refill the position
    ensure!(
        position.receiver == info.sender || info.sender == config.pool_manager_addr,
        ContractError::Unauthorized
    );

    position.lp_asset.amount = position.lp_asset.amount.checked_add(lp_asset.amount)?;
    POSITIONS.save(deps.storage, &position.identifier, &position)?;

    // Update weights for the LP and the user
    update_weights(
        deps,
        env,
        &position.receiver,
        &lp_asset,
        position.unlocking_duration,
        true,
    )?;

    Ok(Response::default().add_attributes(vec![
        ("action", "expand_position".to_string()),
        ("receiver", position.receiver.to_string()),
        ("lp_asset", lp_asset.to_string()),
        (
            "unlocking_duration",
            position.unlocking_duration.to_string(),
        ),
    ]))
}

/// Closes an existing position
pub(crate) fn close_position(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    identifier: String,
    lp_asset: Option<Coin>,
) -> Result<Response, ContractError> {
    cw_utils::nonpayable(&info)?;

    // check if the user has pending rewards. Can't close a position without claiming pending rewards first
    let rewards_response = query_rewards(deps.as_ref(), &env, info.sender.clone().into_string())?;
    match rewards_response {
        RewardsResponse::RewardsResponse { rewards } => {
            ensure!(rewards.is_empty(), ContractError::PendingRewards)
        }
        _ => return Err(ContractError::Unauthorized),
    }

    let mut position = get_position(deps.storage, Some(identifier.clone()))?.ok_or(
        ContractError::NoPositionFound {
            identifier: identifier.clone(),
        },
    )?;

    ensure!(
        position.receiver == info.sender,
        ContractError::Unauthorized
    );

    ensure!(
        position.open,
        ContractError::PositionAlreadyClosed { identifier }
    );

    let mut attributes = vec![
        ("action", "close_position".to_string()),
        ("receiver", info.sender.to_string()),
        ("identifier", identifier.to_string()),
    ];

    let expires_at = env
        .block
        .time
        .plus_seconds(position.unlocking_duration)
        .seconds();

    // ensure the user doesn't have more than the maximum allowed close positions
    validate_positions_limit(deps.as_ref(), &info.sender, false)?;

    // check if it's going to be closed in full or partially
    let lp_amount_to_close = if let Some(lp_asset) = lp_asset {
        // close position partially

        // make sure the lp_asset requested to close matches the lp_asset of the position, and since
        // this is a partial close, the amount requested to close should be less than the amount in the position
        ensure!(
            lp_asset.denom == position.lp_asset.denom && lp_asset.amount < position.lp_asset.amount,
            ContractError::AssetMismatch
        );

        position.lp_asset.amount = position.lp_asset.amount.saturating_sub(lp_asset.amount);

        // add the partial closing position to the storage
        let position_id_counter = POSITION_ID_COUNTER
            .may_load(deps.storage)?
            .unwrap_or_default()
            + 1u64;
        POSITION_ID_COUNTER.save(deps.storage, &position_id_counter)?;

        let identifier = format!("{AUTO_POSITION_ID_PREFIX}{position_id_counter}");

        let partial_position = Position {
            identifier: identifier.to_string(),
            lp_asset: lp_asset.clone(),
            unlocking_duration: position.unlocking_duration,
            open: false,
            expiring_at: Some(expires_at),
            receiver: position.receiver.clone(),
        };

        POSITIONS.save(deps.storage, &identifier.to_string(), &partial_position)?;
        // partial amount
        lp_asset.amount
    } else {
        // close position in full
        position.open = false;
        position.expiring_at = Some(expires_at);
        // full amount
        position.lp_asset.amount
    };

    let close_in_full = !position.open;
    attributes.push(("close_in_full", close_in_full.to_string()));

    update_weights(
        deps.branch(),
        &env,
        &info.sender,
        &coin(lp_amount_to_close.u128(), &position.lp_asset.denom),
        position.unlocking_duration,
        false,
    )?;

    POSITIONS.save(deps.storage, &identifier, &position)?;

    Ok(Response::default().add_attributes(attributes))
}

/// Withdraws the given position. The position needs to have expired.
pub(crate) fn withdraw_position(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    identifier: String,
    emergency_unlock: Option<bool>,
) -> Result<Response, ContractError> {
    cw_utils::nonpayable(&info)?;

    let mut position = get_position(deps.storage, Some(identifier.clone()))?.ok_or(
        ContractError::NoPositionFound {
            identifier: identifier.clone(),
        },
    )?;

    ensure!(
        position.receiver == info.sender,
        ContractError::Unauthorized
    );

    // check if the user has pending rewards. Can't withdraw a position without claiming pending rewards first
    let rewards_response = query_rewards(deps.as_ref(), &env, info.sender.clone().into_string())?;
    match rewards_response {
        RewardsResponse::RewardsResponse { rewards } => {
            ensure!(rewards.is_empty(), ContractError::PendingRewards)
        }
        _ => return Err(ContractError::Unauthorized),
    }

    let mut messages: Vec<CosmosMsg> = vec![];

    // check if the emergency unlock is requested, will pull the whole position out whether it's open, closed or expired, paying the penalty
    if emergency_unlock.is_some() && emergency_unlock.unwrap() {
        let emergency_unlock_penalty = CONFIG.load(deps.storage)?.emergency_unlock_penalty;

        let penalty_fee = Decimal::from_ratio(position.lp_asset.amount, Uint128::one())
            .checked_mul(emergency_unlock_penalty)?
            .to_uint_floor();

        // sanity check
        ensure!(
            penalty_fee < position.lp_asset.amount,
            ContractError::InvalidEmergencyUnlockPenalty
        );

        let penalty = Coin {
            denom: position.lp_asset.denom.to_string(),
            amount: penalty_fee,
        };

        let fee_collector_addr = CONFIG.load(deps.storage)?.fee_collector_addr;

        // send penalty to the fee collector
        if penalty.amount > Uint128::zero() {
            messages.push(
                BankMsg::Send {
                    to_address: fee_collector_addr.to_string(),
                    amount: vec![penalty],
                }
                .into(),
            );
        }

        // if the position is open, update the weights when doing the emergency withdrawal
        // otherwise not, as the weights have already being updated when the position was closed
        if position.open {
            update_weights(
                deps.branch(),
                &env,
                &info.sender,
                &position.lp_asset,
                position.unlocking_duration,
                false,
            )?;
        }

        // subtract the penalty from the original position
        position.lp_asset.amount = position.lp_asset.amount.saturating_sub(penalty_fee);
    } else {
        // check if this position is eligible for withdrawal
        if position.open || position.expiring_at.is_none() {
            return Err(ContractError::Unauthorized);
        }

        if position.expiring_at.unwrap() > env.block.time.seconds() {
            return Err(ContractError::PositionNotExpired);
        }
    }

    // sanity check
    if !position.lp_asset.amount.is_zero() {
        // withdraw the remaining LP tokens
        messages.push(
            BankMsg::Send {
                to_address: position.receiver.to_string(),
                amount: vec![position.lp_asset],
            }
            .into(),
        );
    }

    POSITIONS.remove(deps.storage, &identifier)?;

    Ok(Response::default()
        .add_attributes(vec![
            ("action", "withdraw_position".to_string()),
            ("receiver", info.sender.to_string()),
            ("identifier", identifier),
        ])
        .add_messages(messages))
}

/// Updates the weights when managing a position. Computes what the weight is gonna be in the next epoch.
fn update_weights(
    deps: DepsMut,
    env: &Env,
    receiver: &Addr,
    lp_asset: &Coin,
    unlocking_duration: u64,
    fill: bool,
) -> Result<(), ContractError> {
    let config = CONFIG.load(deps.storage)?;

    let current_epoch = amm::epoch_manager::get_current_epoch(
        deps.as_ref(),
        config.epoch_manager_addr.to_string(),
    )?;

    let weight = calculate_weight(lp_asset, unlocking_duration)?;

    let (_, mut lp_weight) =
        get_latest_address_weight(deps.storage, &env.contract.address, &lp_asset.denom)?;

    if fill {
        // filling position
        lp_weight = lp_weight.checked_add(weight)?;
    } else {
        // closing position
        lp_weight = lp_weight.saturating_sub(weight);
    }

    // update the LP weight for the contract
    LP_WEIGHT_HISTORY.update::<_, StdError>(
        deps.storage,
        (
            &env.contract.address,
            &lp_asset.denom,
            current_epoch.id + 1u64,
        ),
        |_| Ok(lp_weight),
    )?;

    // update the user's weight for this LP
    let (_, mut address_lp_weight) =
        get_latest_address_weight(deps.storage, receiver, &lp_asset.denom)?;

    if fill {
        // filling position
        address_lp_weight = address_lp_weight.checked_add(weight)?;
    } else {
        // closing position
        address_lp_weight = address_lp_weight.saturating_sub(weight);
    }

    //todo if the address weight is zero, remove it from the storage?
    LP_WEIGHT_HISTORY.update::<_, StdError>(
        deps.storage,
        (receiver, &lp_asset.denom, current_epoch.id + 1u64),
        |_| Ok(address_lp_weight),
    )?;

    Ok(())
}
