use cosmwasm_std::{
    coin, ensure, Addr, BankMsg, Coin, CosmosMsg, Decimal, DepsMut, Env, MessageInfo, Response,
    Uint128,
};
use std::collections::HashSet;

use amm::farm_manager::Position;

use crate::helpers::{validate_identifier, validate_lp_denom};
use crate::position::helpers::{
    calculate_weight, create_penalty_share_msg, get_latest_address_weight, reconcile_user_state,
    validate_no_pending_rewards, AUTO_POSITION_ID_PREFIX, EXPLICIT_POSITION_ID_PREFIX,
    PENALTY_FEE_SHARE,
};
use crate::position::helpers::{
    validate_positions_limit, validate_unlocking_duration_for_position,
};
use crate::state::{
    get_farms_by_lp_denom, get_position, CONFIG, LP_WEIGHT_HISTORY, MAX_ITEMS_LIMIT, POSITIONS,
    POSITION_ID_COUNTER,
};
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
        POSITION_ID_COUNTER.save(deps.storage, &position_id_counter)?;
        // prepend AUTO_POSITION_ID_PREFIX to the position_id_counter
        format!("{AUTO_POSITION_ID_PREFIX}{position_id_counter}")
    };

    validate_identifier(&identifier)?;

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
    validate_no_pending_rewards(deps.as_ref(), &env, &info)?;

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
        // make sure the lp_asset requested to close matches the lp_asset of the position
        ensure!(
            lp_asset.denom == position.lp_asset.denom,
            ContractError::AssetMismatch
        );

        match lp_asset.amount.cmp(&position.lp_asset.amount) {
            std::cmp::Ordering::Equal => close_position_in_full(&mut position, expires_at),
            std::cmp::Ordering::Less => {
                // close position partially
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
            }
            std::cmp::Ordering::Greater => {
                return Err(ContractError::InvalidLpAmount {
                    expected: position.lp_asset.amount,
                    actual: lp_asset.amount,
                });
            }
        }
    } else {
        close_position_in_full(&mut position, expires_at)
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

    reconcile_user_state(deps, &info.sender, &position)?;

    Ok(Response::default().add_attributes(attributes))
}

/// Modifies the position to be closed in full, returning the total amount of lp for this position.
fn close_position_in_full(position: &mut Position, expires_at: u64) -> Uint128 {
    // close position in full
    position.open = false;
    position.expiring_at = Some(expires_at);
    // returns full amount to be closed
    position.lp_asset.amount
}

/// Withdraws the given position. If the position has not expired, i.e. the unlocking period has not
/// passed, the position can be withdrawn with a penalty fee using the`emergency_unlock` param.
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

    let current_time = env.block.time.seconds();
    let mut messages: Vec<CosmosMsg> = vec![];

    // check if the emergency unlock is requested, will pull the whole position out whether it's
    // open, closed or expired.
    // If the position already expired, the position can be withdrawn without penalty, even if the
    // emergency_unlock is requested
    if emergency_unlock.is_some() && emergency_unlock.unwrap() && !position.is_expired(current_time)
    {
        let emergency_unlock_penalty = CONFIG.load(deps.storage)?.emergency_unlock_penalty;

        let total_penalty_fee = Decimal::from_ratio(position.lp_asset.amount, Uint128::one())
            .checked_mul(emergency_unlock_penalty)?
            .to_uint_floor();

        // sanity check
        ensure!(
            total_penalty_fee < position.lp_asset.amount,
            ContractError::InvalidEmergencyUnlockPenalty
        );

        // calculate the penalty fee that goes to the owner of the farm
        let owner_penalty_fee_comission = Decimal::from_ratio(total_penalty_fee, Uint128::one())
            .checked_mul(PENALTY_FEE_SHARE)?
            .to_uint_floor();

        let mut penalty_fee_fee_collector =
            total_penalty_fee.saturating_sub(owner_penalty_fee_comission);

        let fee_collector_addr = CONFIG.load(deps.storage)?.fee_collector_addr;

        let farms = get_farms_by_lp_denom(
            deps.storage,
            &position.lp_asset.denom,
            None,
            Some(MAX_ITEMS_LIMIT),
        )?;

        // get unique farm owners for this lp denom
        let unique_farm_owners: Vec<Addr> = farms
            .iter()
            .map(|farm| farm.owner.clone())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();

        // if there are no farms for this lp denom there's no need to send any penalty to the farm
        // owners, as there are none. Send it all to the fee collector
        if unique_farm_owners.is_empty() {
            // send the whole penalty fee to the fee collector
            penalty_fee_fee_collector = total_penalty_fee;
        } else {
            // send penalty to farm owners
            let penalty_fee_share_per_farm_owner = Decimal::from_ratio(
                owner_penalty_fee_comission,
                unique_farm_owners.len() as u128,
            )
            .to_uint_floor();

            // if the farm owner penalty fee is greater than zero, send it to the farm owners,
            // otherwise send the whole penalty fee to the fee collector
            if penalty_fee_share_per_farm_owner > Uint128::zero() {
                for farm_owner in unique_farm_owners {
                    messages.push(create_penalty_share_msg(
                        position.lp_asset.denom.to_string(),
                        penalty_fee_share_per_farm_owner,
                        &farm_owner,
                    ));
                }
            } else {
                // if the penalty fee share per farm owner is zero, then the whole penalty fee goes
                // to the fee collector, if any
                penalty_fee_fee_collector = total_penalty_fee;
            }
        }

        // send penalty to the fee collector
        if penalty_fee_fee_collector > Uint128::zero() {
            messages.push(create_penalty_share_msg(
                position.lp_asset.denom.to_string(),
                penalty_fee_fee_collector,
                &fee_collector_addr,
            ));
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
        position.lp_asset.amount = position.lp_asset.amount.saturating_sub(total_penalty_fee);
    } else {
        // check if this position is eligible for withdrawal
        ensure!(position.expiring_at.is_some(), ContractError::Unauthorized);

        ensure!(
            position.is_expired(current_time),
            ContractError::PositionNotExpired
        );
    }

    // sanity check
    if !position.lp_asset.amount.is_zero() {
        // withdraw the remaining LP tokens
        messages.push(
            BankMsg::Send {
                to_address: position.receiver.to_string(),
                amount: vec![position.lp_asset.clone()],
            }
            .into(),
        );
    }

    POSITIONS.remove(deps.storage, &identifier)?;

    // if the position to remove was open, i.e. withdrawn via the emergency unlock feature, then
    // we need to reconcile the user state
    if position.open {
        reconcile_user_state(deps, &info.sender, &position)?;
    }

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
    LP_WEIGHT_HISTORY.save(
        deps.storage,
        (
            &env.contract.address,
            &lp_asset.denom,
            current_epoch.id + 1u64,
        ),
        &lp_weight,
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

    LP_WEIGHT_HISTORY.save(
        deps.storage,
        (receiver, &lp_asset.denom, current_epoch.id + 1u64),
        &address_lp_weight,
    )?;

    Ok(())
}
