use std::cmp::Ordering;
use std::collections::HashSet;

use cosmwasm_std::{
    ensure, BankMsg, Coin, CosmosMsg, Decimal, Deps, Env, MessageInfo, OverflowError,
    OverflowOperation, Uint128,
};

use amm::coin::{get_factory_token_creator, is_factory_token};
use amm::constants::MONTH_IN_SECONDS;
use amm::epoch_manager::{EpochResponse, QueryMsg};
use amm::farm_manager::{Config, Farm, FarmParams, Position, DEFAULT_FARM_DURATION};

use crate::ContractError;

/// Processes the farm creation fee and returns the appropriate messages to be sent
pub(crate) fn process_farm_creation_fee(
    config: &Config,
    info: &MessageInfo,
    params: &FarmParams,
) -> Result<Vec<CosmosMsg>, ContractError> {
    let mut messages: Vec<CosmosMsg> = vec![];

    let farm_creation_fee = &config.create_farm_fee;

    // verify the fee to create a farm is being paid
    let paid_fee_amount = info
        .funds
        .iter()
        .find(|coin| coin.denom == farm_creation_fee.denom)
        .ok_or(ContractError::FarmFeeMissing)?
        .amount;

    match paid_fee_amount.cmp(&farm_creation_fee.amount) {
        Ordering::Equal => (), // do nothing if user paid correct amount,
        Ordering::Less => {
            // user underpaid
            return Err(ContractError::FarmFeeNotPaid {
                paid_amount: paid_fee_amount,
                required_amount: farm_creation_fee.amount,
            });
        }
        Ordering::Greater => {
            // if the user is paying more than the farm_creation_fee, check if it's trying to create
            // a farm with the same asset as the farm_creation_fee.
            // otherwise, refund the difference
            if farm_creation_fee.denom == params.farm_asset.denom {
                // check if the amounts add up, i.e. the fee + farm asset = paid amount. That is because the farm asset
                // and the creation fee asset are the same, all go in the info.funds of the transaction

                ensure!(
                    params
                        .farm_asset
                        .amount
                        .checked_add(farm_creation_fee.amount)?
                        == paid_fee_amount,
                    ContractError::AssetMismatch
                );
            } else {
                let refund_amount = paid_fee_amount.saturating_sub(farm_creation_fee.amount);

                messages.push(
                    BankMsg::Send {
                        to_address: info.sender.clone().into_string(),
                        amount: vec![Coin {
                            amount: refund_amount,
                            denom: farm_creation_fee.denom.clone(),
                        }],
                    }
                    .into(),
                );
            }
        }
    }

    // send farm creation fee to fee collector
    if farm_creation_fee.amount > Uint128::zero() {
        messages.push(
            BankMsg::Send {
                to_address: config.fee_collector_addr.to_string(),
                amount: vec![farm_creation_fee.to_owned()],
            }
            .into(),
        );
    }

    Ok(messages)
}

/// Asserts the farm asset was sent correctly, considering the farm creation fee if applicable.
pub(crate) fn assert_farm_asset(
    info: &MessageInfo,
    farm_creation_fee: &Coin,
    params: &FarmParams,
) -> Result<(), ContractError> {
    let coin_sent = info
        .funds
        .iter()
        .find(|sent| sent.denom == params.farm_asset.denom)
        .ok_or(ContractError::AssetMismatch)?;

    if farm_creation_fee.denom != params.farm_asset.denom {
        ensure!(
            coin_sent.amount == params.farm_asset.amount,
            ContractError::AssetMismatch
        );
        // if the farm creation denom and the farm asset denom are different,
        // ensure only those two assets were sent
        ensure!(info.funds.len() == 2usize, ContractError::AssetMismatch);
    } else {
        ensure!(
            params
                .farm_asset
                .amount
                .checked_add(farm_creation_fee.amount)?
                == coin_sent.amount,
            ContractError::AssetMismatch
        );
        // if the farm creation denom and the farm asset denom are the same,
        // then make sure only that asset was sent in the transaction
        ensure!(info.funds.len() == 1usize, ContractError::AssetMismatch);
    }

    Ok(())
}

/// Validates the farm epochs. Returns a tuple of (start_epoch, end_epoch) for the farm.
pub(crate) fn validate_farm_epochs(
    params: &FarmParams,
    current_epoch: u64,
    max_farm_epoch_buffer: u64,
) -> Result<(u64, u64), ContractError> {
    // assert epoch params are correctly set
    let start_epoch = params.start_epoch.unwrap_or(current_epoch + 1u64);

    ensure!(
        start_epoch > 0u64,
        ContractError::InvalidEpoch {
            which: "start".to_string()
        }
    );

    let preliminary_end_epoch = params.preliminary_end_epoch.unwrap_or(
        start_epoch
            .checked_add(DEFAULT_FARM_DURATION)
            .ok_or(ContractError::InvalidEpoch {
                which: "end".to_string(),
            })?,
    );

    // ensure that start date is before end date
    ensure!(
        start_epoch < preliminary_end_epoch,
        ContractError::FarmStartTimeAfterEndTime
    );

    // ensure the farm is set to end in a future epoch
    ensure!(
        preliminary_end_epoch > current_epoch,
        ContractError::FarmEndsInPast
    );

    // ensure that start date is set within buffer
    ensure!(
        start_epoch
            <= current_epoch.checked_add(max_farm_epoch_buffer).ok_or(
                ContractError::OverflowError(OverflowError {
                    operation: OverflowOperation::Add
                })
            )?,
        ContractError::FarmStartTooFar
    );

    Ok((start_epoch, preliminary_end_epoch))
}

/// Validates the emergency unlock penalty is within the allowed range (0-100%). Returns value it's validating, i.e. the penalty.
pub(crate) fn validate_emergency_unlock_penalty(
    emergency_unlock_penalty: Decimal,
) -> Result<Decimal, ContractError> {
    ensure!(
        emergency_unlock_penalty <= Decimal::percent(100),
        ContractError::InvalidEmergencyUnlockPenalty
    );

    Ok(emergency_unlock_penalty)
}

/// Validates that the denom was created by the pool manager, i.e. it belongs to a valid pool.
pub(crate) fn validate_lp_denom(
    lp_denom: &str,
    pool_manager_addr: &str,
) -> Result<(), ContractError> {
    ensure!(
        is_factory_token(lp_denom) && get_factory_token_creator(lp_denom)? == pool_manager_addr,
        ContractError::AssetMismatch
    );

    Ok(())
}

/// Validates the unlocking duration range
pub(crate) fn validate_unlocking_duration(
    min_unlocking_duration: u64,
    max_unlocking_duration: u64,
) -> Result<(), ContractError> {
    ensure!(
        max_unlocking_duration >= min_unlocking_duration,
        ContractError::InvalidUnlockingRange {
            min: min_unlocking_duration,
            max: max_unlocking_duration,
        }
    );

    Ok(())
}

/// Validates the farm expiration time
pub(crate) fn validate_farm_expiration_time(
    farm_expiration_time: u64,
) -> Result<(), ContractError> {
    ensure!(
        farm_expiration_time >= MONTH_IN_SECONDS,
        ContractError::FarmExpirationTimeInvalid {
            min: MONTH_IN_SECONDS
        }
    );

    Ok(())
}

/// Gets the unique LP asset denoms from a list of positions
pub(crate) fn get_unique_lp_asset_denoms_from_positions(positions: Vec<Position>) -> Vec<String> {
    positions
        .iter()
        .map(|position| position.lp_asset.denom.clone())
        .collect::<HashSet<_>>()
        .into_iter()
        .collect()
}

/// Checks if the farm is expired. A farm is considered to be expired if there's no more assets to claim
/// or if there has passed the config.farm_expiration_time since the farm ended.
pub(crate) fn is_farm_expired(
    farm: &Farm,
    deps: Deps,
    env: &Env,
    config: &Config,
) -> Result<bool, ContractError> {
    let epoch_response: EpochResponse = deps
        .querier
        // query preliminary_end_epoch + 1 because the farm is preliminary ending at that epoch, including it.
        .query_wasm_smart(
            config.epoch_manager_addr.to_string(),
            &QueryMsg::Epoch {
                id: farm.preliminary_end_epoch + 1u64,
            },
        )?;

    let farm_ending_at = epoch_response.epoch.start_time;

    Ok(
        farm.farm_asset.amount.saturating_sub(farm.claimed_amount) == Uint128::zero()
            || farm_ending_at.plus_seconds(config.farm_expiration_time) < env.block.time,
    )
}

// 64 char hash  + 2 char prefix allowed
const MAX_IDENTIFIER_LENGTH: usize = 66usize;

/// Validates that farms and positions identifiers are correct, ensuring the identifier doesn't
/// exceed 64 characters, it's alphanumeric, and can contain '.', '-' and '_'.
pub fn validate_identifier(identifier: &str) -> Result<(), ContractError> {
    ensure!(
        identifier.len() <= MAX_IDENTIFIER_LENGTH
            && identifier
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '_'),
        ContractError::InvalidIdentifier {
            identifier: identifier.to_string()
        }
    );

    Ok(())
}
