use cosmwasm_std::{
    ensure, Addr, BankMsg, Coin, CosmosMsg, Decimal, Decimal256, Deps, DepsMut, Env, MessageInfo,
    Order, StdError, Storage, Uint128,
};

use amm::farm_manager::{Config, EpochId, Position, RewardsResponse};

use crate::farm::commands::sync_address_lp_weight_history;
use crate::queries::query_rewards;
use crate::state::{
    get_positions_by_receiver, has_any_lp_weight, CONFIG, LAST_CLAIMED_EPOCH, LP_WEIGHT_HISTORY,
    MAX_ITEMS_LIMIT,
};
use crate::ContractError;

const SECONDS_IN_DAY: u64 = 86400;
const SECONDS_IN_YEAR: u64 = 31556926;

/// The prefix used when creation a position with an auto-generated ID
pub const AUTO_POSITION_ID_PREFIX: &str = "p-";

/// The prefix used when creation a position with an explicitly provided ID
pub const EXPLICIT_POSITION_ID_PREFIX: &str = "u-";

/// The penalty fee share that is given to the owner of the farm if someone does an emergency withdraw
pub const PENALTY_FEE_SHARE: Decimal = Decimal::percent(50);

/// Calculates the weight size for a user filling a position
pub fn calculate_weight(
    lp_asset: &Coin,
    unlocking_duration: u64,
) -> Result<Uint128, ContractError> {
    if !(SECONDS_IN_DAY..=SECONDS_IN_YEAR).contains(&unlocking_duration) {
        return Err(ContractError::InvalidWeight { unlocking_duration });
    }

    // store in Uint128 form for later
    let amount_uint = lp_asset.amount;

    // interpolate between [(86400, 1), (15778463, 5), (31556926, 16)]
    // note that 31556926 is not exactly one 365-day year, but rather one Earth rotation year
    // similarly, 15778463 is not 1/2 a 365-day year, but rather 1/2 a one Earth rotation year

    // first we need to convert into decimals
    let unlocking_duration = Decimal256::from_atomics(unlocking_duration, 0).unwrap();
    let amount = Decimal256::from_atomics(lp_asset.amount, 0).unwrap();

    let unlocking_duration_squared = unlocking_duration.checked_pow(2)?;
    let unlocking_duration_mul =
        unlocking_duration_squared.checked_mul(Decimal256::raw(109498841))?;
    let unlocking_duration_part =
        unlocking_duration_mul.checked_div(Decimal256::raw(7791996353100889432894))?;

    let next_part = unlocking_duration
        .checked_mul(Decimal256::raw(249042009202369))?
        .checked_div(Decimal256::raw(7791996353100889432894))?;

    let final_part = Decimal256::from_ratio(246210981355969u64, 246918738317569u64);

    let weight: Uint128 = amount
        .checked_mul(
            unlocking_duration_part
                .checked_add(next_part)?
                .checked_add(final_part)?,
        )?
        .atomics()
        .checked_div(10u128.pow(Decimal::DECIMAL_PLACES).into())?
        .try_into()?;

    // we must clamp it to max(computed_value, amount) as
    // otherwise we might get a multiplier of 0.999999999999999998 when
    // computing the final_part decimal value, which is over 200 digits.
    Ok(weight.max(amount_uint))
}

/// Gets the latest available weight snapshot recorded for the given address.
pub fn get_latest_address_weight(
    storage: &dyn Storage,
    address: &Addr,
    lp_denom: &str,
) -> Result<(EpochId, Uint128), ContractError> {
    let result = LP_WEIGHT_HISTORY
        .prefix((address, lp_denom))
        .range(storage, None, None, Order::Descending)
        .take(1usize)
        // take only one item, the last item. Since it's being sorted in descending order, it's the latest one.
        .next()
        .transpose();

    return_latest_weight(result)
}

/// Helper function to return the weight from the result. If the result is None, i.e. the weight
/// was not found in the map, it returns (0, 0).
fn return_latest_weight(
    weight_result: Result<Option<(EpochId, Uint128)>, StdError>,
) -> Result<(EpochId, Uint128), ContractError> {
    match weight_result {
        Ok(Some(item)) => Ok(item),
        Ok(None) => Ok((0u64, Uint128::zero())),
        Err(std_err) => Err(std_err.into()),
    }
}

/// Validates the `unlocking_duration` specified in the position params is within the range specified
/// in the config.
pub(crate) fn validate_unlocking_duration_for_position(
    config: &Config,
    unlocking_duration: u64,
) -> Result<(), ContractError> {
    if unlocking_duration < config.min_unlocking_duration
        || unlocking_duration > config.max_unlocking_duration
    {
        return Err(ContractError::InvalidUnlockingDuration {
            min: config.min_unlocking_duration,
            max: config.max_unlocking_duration,
            specified: unlocking_duration,
        });
    }

    Ok(())
}

/// Validates the amount of positions a user can have either open or closed at a given time.
pub(crate) fn validate_positions_limit(
    deps: Deps,
    receiver: &Addr,
    open_state: bool,
) -> Result<(), ContractError> {
    let existing_user_positions = get_positions_by_receiver(
        deps.storage,
        receiver.as_str(),
        Some(open_state),
        None,
        Some(MAX_ITEMS_LIMIT),
    )?;

    ensure!(
        existing_user_positions.len() < MAX_ITEMS_LIMIT as usize,
        ContractError::MaxPositionsPerUserExceeded {
            max: MAX_ITEMS_LIMIT
        }
    );

    Ok(())
}

/// Validates the amount of positions a user can have either open or closed at a given time.
pub(crate) fn create_penalty_share_msg(
    lp_asset_denom: String,
    commission: Uint128,
    receiver: &Addr,
) -> CosmosMsg {
    let penalty = Coin {
        denom: lp_asset_denom,
        amount: commission,
    };

    BankMsg::Send {
        to_address: receiver.to_string(),
        amount: vec![penalty],
    }
    .into()
}

/// Validates that the user has no pending rewards before performing an operation.
pub fn validate_no_pending_rewards(
    deps: Deps,
    env: &Env,
    info: &MessageInfo,
) -> Result<(), ContractError> {
    let rewards_response = query_rewards(deps, env, info.sender.clone().into_string())?;

    match rewards_response {
        RewardsResponse::RewardsResponse { total_rewards, .. } => {
            ensure!(total_rewards.is_empty(), ContractError::PendingRewards)
        }
        _ => return Err(ContractError::Unauthorized),
    }

    Ok(())
}

/// Reconciles a user's state by updating or removing stale data based on their current open positions.
///
/// This function checks for two primary conditions:
/// 1. If the user has no more open positions, it clears the LAST_CLAIMED_EPOCH state item.
/// 2. If the user has no more open positions for a specific LP denom, it wipes the LP weight history for that denom.
///
/// Why do we need to do this?
/// If the lp history and the LAST_CLAIMED_EPOCH for the user is not cleared when fully existing the farm,
/// if the user would create a new position in the future for the same denom, the contract would try to
/// claim rewards for old epochs that would be irrelevant, as the LAST_CLAIMED_EPOCH is recorded when
/// the user claims rewards. At that point, the user weight would be zero for the given LP, which renders
/// the computation for those epochs useless. Additionally, if the user were be the only user in the farm,
/// exiting the farms would record the lp weight for both the user and contract as zero. If the LAST_CLAIMED_EPOCH
/// and lp weight history were not cleared, if the user opens another position for the same LP denom in the future,
/// as the contract would try to claim previous epoch rewards there would be a DivideByZero error as the
/// total_lp_weight would be zero when calculating user's share of the rewards.
pub fn reconcile_user_state(
    deps: DepsMut,
    receiver: &Addr,
    position: &Position,
) -> Result<(), ContractError> {
    let receiver_open_positions = get_positions_by_receiver(
        deps.storage,
        receiver.as_ref(),
        Some(true),
        None,
        Some(MAX_ITEMS_LIMIT),
    )?;

    // if the user has no more open positions, clear the last claimed epoch
    if receiver_open_positions.is_empty() {
        LAST_CLAIMED_EPOCH.remove(deps.storage, receiver);
    }

    // if the user has no more open positions for the position's LP denom, wipe the LP weight
    // history for that denom
    if receiver_open_positions
        .iter()
        .filter(|p| p.lp_asset.denom == position.lp_asset.denom)
        .collect::<Vec<_>>()
        .is_empty()
    {
        // if it doesn't have any it means it was already cleared up when closing the position,
        // but it is different if the user emergency exits an open position.
        // if withdrawing a position after closing it, this won't be triggered as it was already
        // called when closing the position.
        if has_any_lp_weight(deps.storage, receiver, &position.lp_asset.denom)? {
            clear_lp_weight_history(deps, receiver, &position.lp_asset.denom)?;
        }
    }

    Ok(())
}

/// Clears the lp weight history.
fn clear_lp_weight_history(
    deps: DepsMut,
    address: &Addr,
    lp_denom: &str,
) -> Result<(), ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let current_epoch = amm::epoch_manager::get_current_epoch(
        deps.as_ref(),
        config.epoch_manager_addr.to_string(),
    )?;

    // by passing the false flag the lp weight for the current epoch won't be saved, which we want
    // as we want to clear the whole lp weight history for this lp denom.
    sync_address_lp_weight_history(deps.storage, address, lp_denom, &current_epoch.id, false)?;

    Ok(())
}
