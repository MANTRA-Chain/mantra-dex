use cosmwasm_std::{
    ensure, BankMsg, Coin, CosmosMsg, Decimal, DepsMut, Env, MessageInfo, Response, StdError,
    Storage, Uint128, Uint64,
};

use amm::farm_manager::MIN_FARM_AMOUNT;
use amm::farm_manager::{Curve, Farm, FarmParams};

use crate::farm::{AUTO_FARM_ID_PREFIX, EXPLICIT_FARM_ID_PREFIX};
use crate::helpers::{
    assert_farm_asset, is_farm_expired, process_farm_creation_fee,
    validate_emergency_unlock_penalty, validate_farm_epochs, validate_farm_expiration_time,
    validate_identifier, validate_lp_denom, validate_unlocking_duration,
};
use crate::state::{get_farm_by_identifier, get_farms_by_lp_denom, CONFIG, FARMS, FARM_COUNTER};
use crate::ContractError;

pub(crate) fn fill_farm(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    params: FarmParams,
) -> Result<Response, ContractError> {
    // if a farm_identifier was passed in the params, check if a farm with such identifier
    // exists and if the sender is allowed to refill it, otherwise create a new farm
    if let Some(farm_identifier) = params.clone().farm_identifier {
        let farm_result = get_farm_by_identifier(deps.storage, &farm_identifier);

        if let Ok(farm) = farm_result {
            // the farm exists, try to expand it
            return expand_farm(deps, env, info, farm, params);
        }
        // the farm does not exist, try to create it
    }

    // if no identifier was passed in the params or if the farm does not exist, try to create the farm
    create_farm(deps, env, info, params)
}

/// Creates a farm with the given params
fn create_farm(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    params: FarmParams,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    // ensure the lp denom is valid and was created by the pool manager
    validate_lp_denom(&params.lp_denom, config.pool_manager_addr.as_str())?;

    // check if there are any expired farms for this LP asset
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
        .partition(|farm| is_farm_expired(farm, deps.as_ref(), &env, &config).unwrap_or(false));

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
        // verify the fee to create a farm is being paid
        messages.append(&mut process_farm_creation_fee(&config, &info, &params)?);
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
    let farm_identifier = if let Some(id) = params.farm_identifier {
        // prepend EXPLICIT_FARM_ID_PREFIX to identifier
        format!("{EXPLICIT_FARM_ID_PREFIX}{id}")
    } else {
        let farm_id =
            FARM_COUNTER.update::<_, StdError>(deps.storage, |current_id| Ok(current_id + 1u64))?;
        // prepend AUTO_FARM_ID_PREFIX to the position_id_counter
        format!("{AUTO_FARM_ID_PREFIX}{farm_id}")
    };

    validate_identifier(&farm_identifier)?;

    // sanity check. Make sure another farm with the same identifier doesn't exist. Theoretically this should
    // never happen, since the fill_farm function would try to expand the farm if a user tries
    // filling a farm with an identifier that already exists
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

/// Closes a farm. Only the farm creator or the owner of the contract can close a farm, except if
/// the farm has expired, in which case anyone can close it while creating a new farm.
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

        if farm.farm_asset.amount > Uint128::zero() {
            messages.push(
                BankMsg::Send {
                    to_address: farm.owner.into_string(),
                    amount: vec![farm.farm_asset],
                }
                .into(),
            );
        }
    }

    Ok(messages)
}

/// Expands a farm with the given params
fn expand_farm(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    mut farm: Farm,
    params: FarmParams,
) -> Result<Response, ContractError> {
    // only the farm owner can expand it
    ensure!(farm.owner == info.sender, ContractError::Unauthorized);

    let config = CONFIG.load(deps.storage)?;

    // check if the farm has already expired, can't be expanded
    ensure!(
        !is_farm_expired(&farm, deps.as_ref(), &env, &config)?,
        ContractError::FarmAlreadyExpired
    );

    // ensure the lp denom is valid and was created by the pool manager
    validate_lp_denom(&params.lp_denom, config.pool_manager_addr.as_str())?;

    // ensure the farm asset, i.e. the additional reward, was sent
    let reward = cw_utils::one_coin(&info)?;

    ensure!(reward == params.farm_asset, ContractError::AssetMismatch);

    // check that the asset sent matches the asset expected
    ensure!(
        farm.farm_asset.denom == params.farm_asset.denom,
        ContractError::AssetMismatch
    );

    // make sure the expansion is a multiple of the emission rate
    ensure!(
        reward.amount % farm.emission_rate == Uint128::zero(),
        ContractError::InvalidExpansionAmount {
            emission_rate: farm.emission_rate
        }
    );

    // increase the total amount of the farm
    farm.farm_asset.amount = farm.farm_asset.amount.checked_add(reward.amount)?;

    let additional_epochs = params.farm_asset.amount.checked_div(farm.emission_rate)?;

    // adjust the preliminary end_epoch
    farm.preliminary_end_epoch = farm
        .preliminary_end_epoch
        .checked_add(Uint64::try_from(additional_epochs)?.u64())
        .ok_or(ContractError::InvalidEpoch {
            which: "end".to_string(),
        })?;

    FARMS.save(deps.storage, &farm.identifier, &farm)?;

    Ok(Response::default().add_attributes(vec![
        ("action", "expand_farm".to_string()),
        ("farm_identifier", farm.identifier),
        ("expanded_by", params.farm_asset.to_string()),
        ("total_farm", farm.farm_asset.to_string()),
    ]))
}

#[allow(clippy::too_many_arguments)]
/// Updates the configuration of the contract
pub(crate) fn update_config(
    deps: DepsMut,
    info: MessageInfo,
    fee_collector_addr: Option<String>,
    epoch_manager_addr: Option<String>,
    pool_manager_addr: Option<String>,
    create_farm_fee: Option<Coin>,
    max_concurrent_farms: Option<u32>,
    max_farm_epoch_buffer: Option<u32>,
    min_unlocking_duration: Option<u64>,
    max_unlocking_duration: Option<u64>,
    farm_expiration_time: Option<u64>,
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

    if let Some(pool_manager_addr) = pool_manager_addr {
        config.pool_manager_addr = deps.api.addr_validate(&pool_manager_addr)?;
    }

    if let Some(create_farm_fee) = create_farm_fee {
        config.create_farm_fee = create_farm_fee;
    }

    if let Some(max_concurrent_farms) = max_concurrent_farms {
        ensure!(
            max_concurrent_farms >= config.max_concurrent_farms,
            ContractError::MaximumConcurrentFarmsDecreased
        );

        config.max_concurrent_farms = max_concurrent_farms;
    }

    if let Some(max_farm_epoch_buffer) = max_farm_epoch_buffer {
        config.max_farm_epoch_buffer = max_farm_epoch_buffer;
    }

    if let Some(max_unlocking_duration) = max_unlocking_duration {
        validate_unlocking_duration(config.min_unlocking_duration, max_unlocking_duration)?;
        config.max_unlocking_duration = max_unlocking_duration;
    }

    if let Some(min_unlocking_duration) = min_unlocking_duration {
        validate_unlocking_duration(min_unlocking_duration, config.max_unlocking_duration)?;
        config.min_unlocking_duration = min_unlocking_duration;
    }

    if let Some(farm_expiration_time) = farm_expiration_time {
        validate_farm_expiration_time(farm_expiration_time)?;
        config.farm_expiration_time = farm_expiration_time;
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
        ("pool_manager_addr", config.pool_manager_addr.to_string()),
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
            "farm_expiration_time",
            config.farm_expiration_time.to_string(),
        ),
        (
            "emergency_unlock_penalty",
            config.emergency_unlock_penalty.to_string(),
        ),
    ]))
}
