use std::cmp::Ordering;

use cosmwasm_std::{
    ensure, BankMsg, Coin, CosmosMsg, Decimal, MessageInfo, OverflowError, OverflowOperation,
    Uint128,
};

use amm::coin::{get_factory_token_creator, is_factory_token};
use amm::farm_manager::{Config, FarmParams, DEFAULT_FARM_DURATION};

use crate::ContractError;

/// Processes the farm creation fee and returns the appropriate messages to be sent
pub(crate) fn process_farm_creation_fee(
    config: &Config,
    info: &MessageInfo,
    farm_creation_fee: &Coin,
    params: &FarmParams,
) -> Result<Vec<CosmosMsg>, ContractError> {
    let mut messages: Vec<CosmosMsg> = vec![];

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

                if refund_amount > Uint128::zero() {
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
    }

    // send farm creation fee to fee collector
    messages.push(
        BankMsg::Send {
            to_address: config.fee_collector_addr.to_string(),
            amount: vec![farm_creation_fee.to_owned()],
        }
        .into(),
    );

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
    } else {
        ensure!(
            params
                .farm_asset
                .amount
                .checked_add(farm_creation_fee.amount)?
                == coin_sent.amount,
            ContractError::AssetMismatch
        );
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

    let preliminary_end_epoch = params.preliminary_end_epoch.unwrap_or(
        start_epoch
            .checked_add(DEFAULT_FARM_DURATION)
            .ok_or(ContractError::InvalidEndEpoch)?,
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
