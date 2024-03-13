use cosmwasm_std::{
    ensure, BankMsg, Coin, CosmosMsg, DepsMut, Env, MessageInfo, Response, StdError,
};

use white_whale_std::incentive_manager::Position;

use crate::helpers::validate_unlocking_duration;
use crate::position::helpers::{calculate_weight, get_latest_address_weight, get_latest_lp_weight};
use crate::state::{
    get_position, ADDRESS_LP_WEIGHT_HISTORY, CONFIG, LP_WEIGHTS_HISTORY, POSITIONS,
    POSITION_ID_COUNTER,
};
use crate::ContractError;

/// Fills a position. If the position already exists, it will be expanded. Otherwise, a new position is created.
pub(crate) fn fill_position(
    deps: DepsMut,
    info: MessageInfo,
    identifier: Option<String>,
    unlocking_duration: u64,
    receiver: Option<String>,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    let lp_asset = cw_utils::one_coin(&info)?;

    // validate unlocking duration
    validate_unlocking_duration(&config, unlocking_duration)?;

    // if receiver was not specified, default to the sender of the message.
    let receiver = receiver
        .map(|r| deps.api.addr_validate(&r))
        .transpose()?
        .map(|receiver| MessageInfo {
            funds: info.funds.clone(),
            sender: receiver,
        })
        .unwrap_or_else(|| info.clone());

    // check if there's an existing open position with the given `identifier`
    let mut position = get_position(deps.storage, identifier)?;

    if let Some(ref mut position) = position {
        // there is a position, fill it
        ensure!(
            position.lp_asset.denom == lp_asset.denom,
            ContractError::AssetMismatch
        );

        position.lp_asset.amount = position.lp_asset.amount.checked_add(lp_asset.amount)?;
        POSITIONS.save(deps.storage, &position.identifier, position)?;
    } else {
        // No position found, create a new one
        let identifier = POSITION_ID_COUNTER
            .may_load(deps.storage)?
            .unwrap_or_default()
            + 1u64;

        POSITION_ID_COUNTER.save(deps.storage, &identifier)?;

        POSITIONS.save(
            deps.storage,
            &identifier.to_string(),
            &Position {
                identifier: identifier.to_string(),
                lp_asset: lp_asset.clone(),
                unlocking_duration,
                open: true,
                expiring_at: None,
                receiver: receiver.sender.clone(),
            },
        )?;
    }

    // Update weights for the LP and the user
    update_weights(deps, &receiver, &lp_asset, unlocking_duration, true)?;

    let action = match position {
        Some(_) => "expand_position",
        None => "open_position",
    };

    Ok(Response::default().add_attributes(vec![
        ("action", action.to_string()),
        ("receiver", receiver.sender.to_string()),
        ("lp_asset", lp_asset.to_string()),
        ("unlocking_duration", unlocking_duration.to_string()),
    ]))
}

/// Closes an existing position
pub(crate) fn close_position(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    identifier: String,
    lp_asset: Option<Coin>,
) -> Result<Response, ContractError> {
    cw_utils::nonpayable(&info)?;

    //todo do this validation to see if there are pending rewards
    //query and check if the user has pending rewards
    // let rewards_query_result = get_rewards(deps.as_ref(), info.sender.clone().into_string());
    // if let Ok(rewards_response) = rewards_query_result {
    //     // can't close a position if there are pending rewards
    //     if !rewards_response.rewards.is_empty() {
    //         return Err(ContractError::PendingRewards);
    //     }
    // }

    let mut position = get_position(deps.storage, Some(identifier.clone()))?.ok_or(
        ContractError::NoPositionFound {
            identifier: identifier.clone(),
        },
    )?;

    if position.receiver != info.sender {
        return Err(ContractError::Unauthorized);
    }

    let mut attributes = vec![
        ("action", "close_position".to_string()),
        ("receiver", info.sender.to_string()),
        ("identifier", identifier.to_string()),
    ];

    // check if it's gonna be closed in full or partially
    if let Some(lp_asset) = lp_asset {
        // close position partially

        // check if the lp_asset requested to close matches the lp_asset of the position
        ensure!(
            lp_asset.denom == position.lp_asset.denom,
            ContractError::AssetMismatch
        );

        position.lp_asset.amount = position.lp_asset.amount.saturating_sub(lp_asset.amount);

        // add the partial closing position to the storage
        let expires_at = env
            .block
            .time
            .plus_seconds(position.unlocking_duration)
            .seconds();

        let identifier = POSITION_ID_COUNTER
            .may_load(deps.storage)?
            .unwrap_or_default()
            + 1u64;
        POSITION_ID_COUNTER.save(deps.storage, &identifier)?;

        let partial_position = Position {
            identifier: identifier.to_string(),
            lp_asset,
            unlocking_duration: position.unlocking_duration,
            open: false,
            expiring_at: Some(expires_at),
            receiver: position.receiver.clone(),
        };
        POSITIONS.save(deps.storage, &identifier.to_string(), &partial_position)?;

        attributes.push(("close_in_full", false.to_string()));
    } else {
        // close position in full
        position.open = false;
        attributes.push(("close_in_full", true.to_string()));
    }

    POSITIONS.save(deps.storage, &identifier, &position)?;

    Ok(Response::default().add_attributes(attributes))
}

/// Withdraws the given position. The position needs to have expired.
pub(crate) fn withdraw_position(
    deps: DepsMut,
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

    if position.receiver != info.sender {
        return Err(ContractError::Unauthorized);
    }

    let mut messages: Vec<CosmosMsg> = vec![];

    // check if the emergency unlock is requested, will pull the whole position out whether it's open, closed or expired, paying the penalty
    if emergency_unlock.is_some() && emergency_unlock.unwrap() {
        let emergency_unlock_penalty = CONFIG.load(deps.storage)?.emergency_unlock_penalty;

        let penalty_fee = position.lp_asset.amount * emergency_unlock_penalty;

        let penalty = Coin {
            denom: position.lp_asset.denom.to_string(),
            amount: penalty_fee,
        };

        let whale_lair_addr = CONFIG.load(deps.storage)?.whale_lair_addr;

        // send penalty to whale lair for distribution
        //todo the whale lair needs to withdraw the LP tokens from the corresponding pool when this happens
        messages.push(white_whale_std::whale_lair::fill_rewards_msg_coin(
            whale_lair_addr.into_string(),
            vec![penalty],
        )?);

        // subtract the penalty from the original position
        position.lp_asset.amount = position.lp_asset.amount.saturating_sub(penalty_fee);
    } else {
        // check if this position is eligible for withdrawal
        if position.open || position.expiring_at.is_none() {
            return Err(ContractError::Unauthorized);
        }

        if position.expiring_at.unwrap() < env.block.time.seconds() {
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
    receiver: &MessageInfo,
    lp_asset: &Coin,
    unlocking_duration: u64,
    fill: bool,
) -> Result<(), ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let current_epoch = white_whale_std::epoch_manager::common::get_current_epoch(
        deps.as_ref(),
        config.epoch_manager_addr.into_string(),
    )?;

    let weight = calculate_weight(lp_asset, unlocking_duration)?;

    let (_, mut lp_weight) = get_latest_lp_weight(deps.storage, lp_asset.denom.as_bytes())?;

    if fill {
        // filling position
        lp_weight = lp_weight.checked_add(weight)?;
    } else {
        // closing position
        lp_weight = lp_weight.saturating_sub(weight);
    }

    LP_WEIGHTS_HISTORY.update::<_, StdError>(
        deps.storage,
        (lp_asset.denom.as_bytes(), current_epoch.id + 1u64),
        |_| Ok(lp_weight),
    )?;

    // update the user's weight for this LP
    let (_, mut address_lp_weight) = get_latest_address_weight(deps.storage, &receiver.sender)?;

    if fill {
        // filling position
        address_lp_weight = address_lp_weight.checked_add(weight)?;
    } else {
        // closing position
        address_lp_weight = address_lp_weight.saturating_sub(weight);
    }

    ADDRESS_LP_WEIGHT_HISTORY.update::<_, StdError>(
        deps.storage,
        (&receiver.sender, current_epoch.id + 1u64),
        |_| Ok(address_lp_weight),
    )?;

    Ok(())
}
