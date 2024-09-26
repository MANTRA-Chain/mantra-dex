use cosmwasm_std::{
    ensure, BankMsg, Coin, CosmosMsg, Decimal, DepsMut, Env, MessageInfo, Response, StdError,
    Storage, Uint128, Uint64,
};

use amm::coin::{get_factory_token_subdenom, is_factory_token};
use amm::constants::LP_SYMBOL;
use amm::epoch_manager::EpochChangedHookMsg;
use amm::farm_manager::MIN_FARM_AMOUNT;
use amm::farm_manager::{Curve, Farm, FarmParams};

use crate::helpers::{
    assert_farm_asset, process_farm_creation_fee, validate_emergency_unlock_penalty,
    validate_farm_epochs,
};
use crate::state::{
    get_farm_by_identifier, get_farms_by_lp_denom, get_latest_address_lp_weight, CONFIG, FARMS,
    FARM_COUNTER, LP_WEIGHT_HISTORY,
};
use crate::ContractError;

pub(crate) fn fill_farm(
    deps: DepsMut,
    info: MessageInfo,
    params: FarmParams,
) -> Result<Response, ContractError> {
    // if a farm_identifier was passed in the params, check if a farm with such identifier
    // exists and if the sender is allow to refill it, otherwise create a new farm
    if let Some(farm_indentifier) = params.clone().farm_identifier {
        let farm_result = get_farm_by_identifier(deps.storage, &farm_indentifier);

        if let Ok(farm) = farm_result {
            // the farm exists, try to expand it
            return expand_farm(deps, info, farm, params);
        }
        // the farm does not exist, try to create it
    }

    // if no identifier was passed in the params or if the farm does not exist, try to create the farm
    create_farm(deps, info, params)
}

/// Creates a farm with the given params
fn create_farm(
    deps: DepsMut,
    info: MessageInfo,
    params: FarmParams,
) -> Result<Response, ContractError> {
    // check if there are any expired farms for this LP asset
    let config = CONFIG.load(deps.storage)?;
    let farms = get_farms_by_lp_denom(
        deps.storage,
        &params.lp_denom,
        None,
        Some(config.max_concurrent_farms),
    )?;

    let current_epoch = amm::epoch_manager::get_current_epoch(
        deps.as_ref(),
        config.epoch_manager_addr.clone().into_string(),
    )?;

    let (expired_farms, farms): (Vec<_>, Vec<_>) = farms
        .into_iter()
        .partition(|farm| farm.is_expired(current_epoch.id));

    let mut messages: Vec<CosmosMsg> = vec![];

    // close expired farms if there are any
    if !expired_farms.is_empty() {
        messages.append(&mut close_farms(deps.storage, expired_farms)?);
    }

    // check if more farms can be created for this particular LP asset
    ensure!(
        farms.len() < config.max_concurrent_farms as usize,
        ContractError::TooManyFarms {
            max: config.max_concurrent_farms,
        }
    );

    // check the farm is being created with a valid amount
    ensure!(
        params.farm_asset.amount >= MIN_FARM_AMOUNT,
        ContractError::InvalidFarmAmount {
            min: MIN_FARM_AMOUNT.u128()
        }
    );

    let farm_creation_fee = config.clone().create_farm_fee;

    if farm_creation_fee.amount != Uint128::zero() {
        // verify the fee to create an farm is being paid
        messages.append(&mut process_farm_creation_fee(
            &config,
            &info,
            &farm_creation_fee,
            &params,
        )?);
    }

    // verify the farm asset was sent
    assert_farm_asset(&info, &farm_creation_fee, &params)?;

    // assert epoch params are correctly set
    let (start_epoch, preliminary_end_epoch) = validate_farm_epochs(
        &params,
        current_epoch.id,
        u64::from(config.max_farm_epoch_buffer),
    )?;

    // create farm identifier
    let farm_id =
        FARM_COUNTER.update::<_, StdError>(deps.storage, |current_id| Ok(current_id + 1u64))?;
    let farm_identifier = params.farm_identifier.unwrap_or(farm_id.to_string());

    // sanity check. Make sure another farm with the same identifier doesn't exist. Theoretically this should
    // never happen, since the fill_farm function would try to expand the farm if a user tries
    // filling an farm with an identifier that already exists
    ensure!(
        get_farm_by_identifier(deps.storage, &farm_identifier).is_err(),
        ContractError::FarmAlreadyExists
    );
    // the farm does not exist, all good, continue

    // calculates the emission rate. The way it's calculated, it makes the last epoch to be
    // non-inclusive, i.e. the last epoch is not counted in the emission
    let emission_rate = params
        .farm_asset
        .amount
        .checked_div_floor((preliminary_end_epoch.saturating_sub(start_epoch), 1u64))?;

    // create the farm
    let farm = Farm {
        identifier: farm_identifier,
        start_epoch,
        preliminary_end_epoch,
        curve: params.curve.unwrap_or(Curve::Linear),
        farm_asset: params.farm_asset,
        lp_denom: params.lp_denom,
        owner: info.sender,
        claimed_amount: Uint128::zero(),
        emission_rate,
        last_epoch_claimed: start_epoch - 1,
    };

    FARMS.save(deps.storage, &farm.identifier, &farm)?;

    Ok(Response::default()
        .add_messages(messages)
        .add_attributes(vec![
            ("action", "create_farm".to_string()),
            ("farm_creator", farm.owner.to_string()),
            ("farm_identifier", farm.identifier),
            ("start_epoch", farm.start_epoch.to_string()),
            (
                "preliminary_end_epoch",
                farm.preliminary_end_epoch.to_string(),
            ),
            ("emission_rate", emission_rate.to_string()),
            ("curve", farm.curve.to_string()),
            ("farm_asset", farm.farm_asset.to_string()),
            ("lp_denom", farm.lp_denom),
        ]))
}

/// Closes a farm. If the farm has expired, anyone can close it. Otherwise, only the
/// farm creator or the owner of the contract can close a farm.
pub(crate) fn close_farm(
    deps: DepsMut,
    info: MessageInfo,
    farm_identifier: String,
) -> Result<Response, ContractError> {
    cw_utils::nonpayable(&info)?;

    // validate that user is allowed to close the farm. Only the farm creator or the owner
    // of the contract can close a farm
    let farm = get_farm_by_identifier(deps.storage, &farm_identifier)?;

    ensure!(
        farm.owner == info.sender || cw_ownable::is_owner(deps.storage, &info.sender)?,
        ContractError::Unauthorized
    );

    Ok(Response::default()
        .add_messages(close_farms(deps.storage, vec![farm])?)
        .add_attributes(vec![
            ("action", "close_farm".to_string()),
            ("farm_identifier", farm_identifier),
        ]))
}

/// Closes a list of farms. Does not validate the sender, do so before calling this function.
fn close_farms(
    storage: &mut dyn Storage,
    farms: Vec<Farm>,
) -> Result<Vec<CosmosMsg>, ContractError> {
    let mut messages: Vec<CosmosMsg> = vec![];

    for mut farm in farms {
        // remove the farm from the storage
        FARMS.remove(storage, &farm.identifier)?;

        // return the available asset, i.e. the amount that hasn't been claimed
        farm.farm_asset.amount = farm.farm_asset.amount.saturating_sub(farm.claimed_amount);

        messages.push(
            BankMsg::Send {
                to_address: farm.owner.into_string(),
                amount: vec![farm.farm_asset],
            }
            .into(),
        );
    }

    Ok(messages)
}

/// Expands a farm with the given params
fn expand_farm(
    deps: DepsMut,
    info: MessageInfo,
    mut farm: Farm,
    params: FarmParams,
) -> Result<Response, ContractError> {
    // only the farm owner can expand it
    ensure!(farm.owner == info.sender, ContractError::Unauthorized);

    let config = CONFIG.load(deps.storage)?;
    let current_epoch = amm::epoch_manager::get_current_epoch(
        deps.as_ref(),
        config.epoch_manager_addr.into_string(),
    )?;

    // check if the farm has already expired, can't be expanded
    ensure!(
        !farm.is_expired(current_epoch.id),
        ContractError::FarmAlreadyExpired
    );

    // check that the asset sent matches the asset expected
    ensure!(
        farm.farm_asset.denom == params.farm_asset.denom,
        ContractError::AssetMismatch
    );

    // make sure the expansion is a multiple of the emission rate
    ensure!(
        params.farm_asset.amount % farm.emission_rate == Uint128::zero(),
        ContractError::InvalidExpansionAmount {
            emission_rate: farm.emission_rate
        }
    );

    // increase the total amount of the farm
    farm.farm_asset.amount = farm
        .farm_asset
        .amount
        .checked_add(params.farm_asset.amount)?;

    let additional_epochs = params.farm_asset.amount.checked_div(farm.emission_rate)?;

    // adjust the preliminary end_epoch
    farm.preliminary_end_epoch = farm
        .preliminary_end_epoch
        .checked_add(Uint64::try_from(additional_epochs)?.u64())
        .ok_or(ContractError::InvalidEndEpoch)?;

    FARMS.save(deps.storage, &farm.identifier, &farm)?;

    Ok(Response::default().add_attributes(vec![
        ("action", "expand_farm".to_string()),
        ("farm_identifier", farm.identifier),
        ("expanded_by", params.farm_asset.to_string()),
        ("total_farm", farm.farm_asset.to_string()),
    ]))
}

/// EpochChanged hook implementation. Updates the LP_WEIGHTS.
pub(crate) fn on_epoch_changed(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: EpochChangedHookMsg,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    // only the epoch manager can trigger this
    ensure!(
        info.sender == config.epoch_manager_addr,
        ContractError::Unauthorized
    );

    // get all LP tokens and update the LP_WEIGHTS_HISTORY
    let lp_denoms = deps
        .querier
        .query_all_balances(env.contract.address.clone())?
        .into_iter()
        .filter(|asset| {
            if is_factory_token(asset.denom.as_str()) {
                match get_factory_token_subdenom(asset.denom.as_str()) {
                    Ok(subdenom) => subdenom.ends_with(LP_SYMBOL),
                    Err(_) => false,
                }
            } else {
                false
            }
        })
        .map(|asset| asset.denom)
        .collect::<Vec<String>>();

    for lp_denom in &lp_denoms {
        let lp_weight_option = LP_WEIGHT_HISTORY.may_load(
            deps.storage,
            (&env.contract.address, lp_denom, msg.current_epoch.id),
        )?;

        // if the weight for this LP token at this epoch has already been recorded, i.e. someone
        // opened or closed positions in the previous epoch, skip it
        if lp_weight_option.is_some() {
            continue;
        } else {
            // if the weight for this LP token at this epoch has not been recorded, i.e. no one
            // opened or closed positions in the previous epoch, get the last recorded weight
            let (_, latest_lp_weight_record) = get_latest_address_lp_weight(
                deps.storage,
                &env.contract.address,
                lp_denom,
                &msg.current_epoch.id,
            )?;

            LP_WEIGHT_HISTORY.save(
                deps.storage,
                (&env.contract.address, lp_denom, msg.current_epoch.id),
                &latest_lp_weight_record,
            )?;
        }
    }

    Ok(Response::default().add_attributes(vec![
        ("action", "on_epoch_changed".to_string()),
        ("epoch", msg.current_epoch.to_string()),
    ]))
}

#[allow(clippy::too_many_arguments)]
/// Updates the configuration of the contract
pub(crate) fn update_config(
    deps: DepsMut,
    info: MessageInfo,
    fee_collector_addr: Option<String>,
    epoch_manager_addr: Option<String>,
    create_farm_fee: Option<Coin>,
    max_concurrent_farms: Option<u32>,
    max_farm_epoch_buffer: Option<u32>,
    min_unlocking_duration: Option<u64>,
    max_unlocking_duration: Option<u64>,
    emergency_unlock_penalty: Option<Decimal>,
) -> Result<Response, ContractError> {
    cw_ownable::assert_owner(deps.storage, &info.sender)?;

    let mut config = CONFIG.load(deps.storage)?;

    if let Some(new_fee_collector_addr) = fee_collector_addr {
        config.fee_collector_addr = deps.api.addr_validate(&new_fee_collector_addr)?;
    }

    if let Some(epoch_manager_addr) = epoch_manager_addr {
        config.epoch_manager_addr = deps.api.addr_validate(&epoch_manager_addr)?;
    }

    if let Some(create_farm_fee) = create_farm_fee {
        config.create_farm_fee = create_farm_fee;
    }

    if let Some(max_concurrent_farms) = max_concurrent_farms {
        if max_concurrent_farms == 0u32 {
            return Err(ContractError::UnspecifiedConcurrentFarms);
        }

        config.max_concurrent_farms = max_concurrent_farms;
    }

    if let Some(max_farm_epoch_buffer) = max_farm_epoch_buffer {
        config.max_farm_epoch_buffer = max_farm_epoch_buffer;
    }

    if let Some(max_unlocking_duration) = max_unlocking_duration {
        if max_unlocking_duration < config.min_unlocking_duration {
            return Err(ContractError::InvalidUnlockingRange {
                min: config.min_unlocking_duration,
                max: max_unlocking_duration,
            });
        }

        config.max_unlocking_duration = max_unlocking_duration;
    }

    if let Some(min_unlocking_duration) = min_unlocking_duration {
        if config.max_unlocking_duration < min_unlocking_duration {
            return Err(ContractError::InvalidUnlockingRange {
                min: min_unlocking_duration,
                max: config.max_unlocking_duration,
            });
        }

        config.min_unlocking_duration = min_unlocking_duration;
    }

    if let Some(emergency_unlock_penalty) = emergency_unlock_penalty {
        config.emergency_unlock_penalty =
            validate_emergency_unlock_penalty(emergency_unlock_penalty)?;
    }

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::default().add_attributes(vec![
        ("action", "update_config".to_string()),
        ("fee_collector_addr", config.fee_collector_addr.to_string()),
        ("epoch_manager_addr", config.epoch_manager_addr.to_string()),
        ("create_flow_fee", config.create_farm_fee.to_string()),
        (
            "max_concurrent_flows",
            config.max_concurrent_farms.to_string(),
        ),
        (
            "max_flow_epoch_buffer",
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
