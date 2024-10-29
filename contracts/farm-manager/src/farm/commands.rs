use std::collections::HashMap;

use cosmwasm_std::{
    ensure, Addr, BankMsg, Coin, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response, Storage,
    Uint128,
};

use amm::coin::aggregate_coins;
use amm::farm_manager::{EpochId, Farm, RewardsResponse};

use crate::helpers::get_unique_lp_asset_denoms_from_positions;
use crate::state::{
    get_earliest_address_lp_weight, get_farms_by_lp_denom, get_latest_address_lp_weight,
    get_positions_by_receiver, CONFIG, FARMS, LAST_CLAIMED_EPOCH, LP_WEIGHT_HISTORY,
    MAX_ITEMS_LIMIT,
};
use crate::ContractError;

/// Claims pending rewards for farms where the user has LP
pub(crate) fn claim(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    cw_utils::nonpayable(&info)?;

    // check if the user has any open LP positions
    let open_positions = get_positions_by_receiver(
        deps.storage,
        info.sender.as_str(),
        Some(true),
        None,
        Some(MAX_ITEMS_LIMIT),
    )?;
    ensure!(!open_positions.is_empty(), ContractError::NoOpenPositions);

    let config = CONFIG.load(deps.storage)?;
    let current_epoch = amm::epoch_manager::get_current_epoch(
        deps.as_ref(),
        config.epoch_manager_addr.into_string(),
    )?;

    let mut total_rewards = vec![];

    let lp_denoms = get_unique_lp_asset_denoms_from_positions(open_positions);

    for lp_denom in &lp_denoms {
        // calculate the rewards for the lp denom
        let rewards_response = calculate_rewards(
            deps.as_ref(),
            &env,
            lp_denom,
            &info.sender,
            current_epoch.id,
            true,
        )?;

        match rewards_response {
            RewardsResponse::ClaimRewards {
                rewards,
                modified_farms,
            } => {
                total_rewards.append(&mut rewards.clone());

                // update the farms with the claimed rewards
                for (farm_identifier, claimed_reward) in modified_farms {
                    FARMS.update(
                        deps.storage,
                        &farm_identifier,
                        |farm| -> Result<_, ContractError> {
                            let mut farm = farm.unwrap();
                            farm.last_epoch_claimed = current_epoch.id;
                            farm.claimed_amount =
                                farm.claimed_amount.checked_add(claimed_reward)?;

                            // sanity check to make sure a farm doesn't get drained
                            ensure!(
                                farm.claimed_amount <= farm.farm_asset.amount,
                                ContractError::FarmExhausted
                            );

                            Ok(farm)
                        },
                    )?;
                }

                // sync the address lp weight history for the user
                sync_address_lp_weight_history(
                    deps.storage,
                    &info.sender,
                    lp_denom,
                    &current_epoch.id,
                    true,
                )?;
            }
            _ => return Err(ContractError::Unauthorized),
        }
    }

    // update the last claimed epoch for the user
    LAST_CLAIMED_EPOCH.save(deps.storage, &info.sender, &current_epoch.id)?;

    let mut messages = vec![];

    // don't send any bank message if there's nothing to send
    if !total_rewards.is_empty() {
        messages.push(CosmosMsg::Bank(BankMsg::Send {
            to_address: info.sender.to_string(),
            amount: aggregate_coins(total_rewards)?,
        }));
    }

    Ok(Response::default()
        .add_messages(messages)
        .add_attributes(vec![("action", "claim".to_string())]))
}

/// Calculates the rewards for a position
/// ### Returns
/// A [RewardsResponse] with the rewards for the position. If is_claim is true, the RewardsResponse type is
/// ClaimRewards, which contains the rewards and the modified farms (this is to modify the
/// farms in the claim function afterward). If is_claim is false, the RewardsResponse only returns
/// the rewards.
pub(crate) fn calculate_rewards(
    deps: Deps,
    env: &Env,
    lp_denom: &str,
    receiver: &Addr,
    current_epoch_id: EpochId,
    is_claim: bool,
) -> Result<RewardsResponse, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    let farms = get_farms_by_lp_denom(
        deps.storage,
        lp_denom,
        None,
        Some(config.max_concurrent_farms),
    )?;

    let last_claimed_epoch_for_user = LAST_CLAIMED_EPOCH.may_load(deps.storage, receiver)?;

    // Check if the user ever claimed before
    if let Some(last_claimed_epoch) = last_claimed_epoch_for_user {
        // if the last claimed epoch is the same as the current epoch, then there is nothing to claim
        if current_epoch_id == last_claimed_epoch {
            return if is_claim {
                Ok(RewardsResponse::ClaimRewards {
                    rewards: vec![],
                    modified_farms: HashMap::new(),
                })
            } else {
                Ok(RewardsResponse::QueryRewardsResponse { rewards: vec![] })
            };
        }
    }

    let mut rewards: Vec<Coin> = vec![];
    // what farms are going to mutate when claiming rewards. Not used/returned when querying rewards.
    let mut modified_farms: HashMap<String, Uint128> = HashMap::new();

    for farm in farms {
        // skip farms that have not started
        if farm.start_epoch > current_epoch_id {
            continue;
        }

        // compute where the user can start claiming rewards for the farm
        let start_from_epoch = compute_start_from_epoch_for_address(
            deps.storage,
            &farm.lp_denom,
            last_claimed_epoch_for_user,
            receiver,
        )?;

        // compute the weights of the user for the epochs between start_from_epoch and current_epoch_id
        let user_weights = compute_address_weights(
            deps.storage,
            receiver,
            lp_denom,
            &start_from_epoch,
            &current_epoch_id,
        )?;

        // compute the weights of the contract for the epochs between start_from_epoch and current_epoch_id
        let contract_weights = compute_contract_weights(
            deps.storage,
            &env.contract.address,
            lp_denom,
            &start_from_epoch,
            &current_epoch_id,
        )?;

        // compute the farm emissions for the epochs between start_from_epoch and current_epoch_id
        let (farm_emissions, until_epoch) =
            compute_farm_emissions(&farm, &start_from_epoch, &current_epoch_id)?;

        for epoch_id in start_from_epoch..=until_epoch {
            if farm.start_epoch > epoch_id {
                continue;
            }

            let user_weight = user_weights[&epoch_id];
            let total_lp_weight = contract_weights
                .get(&epoch_id)
                .unwrap_or(&Uint128::zero())
                .to_owned();

            let user_share = (user_weight, total_lp_weight);

            let reward = farm_emissions
                .get(&epoch_id)
                .unwrap_or(&Uint128::zero())
                .to_owned()
                .checked_mul_floor(user_share)?;

            // sanity check
            ensure!(
                reward.checked_add(farm.claimed_amount)? <= farm.farm_asset.amount,
                ContractError::FarmExhausted
            );

            if reward > Uint128::zero() {
                rewards.push(Coin {
                    denom: farm.farm_asset.denom.clone(),
                    amount: reward,
                });
            }

            if is_claim {
                // compound the rewards for the farm
                let maybe_reward = modified_farms
                    .get(&farm.identifier)
                    .unwrap_or(&Uint128::zero())
                    .to_owned();

                modified_farms.insert(farm.identifier.clone(), reward.checked_add(maybe_reward)?);
            }
        }
    }

    rewards = aggregate_coins(rewards)?;

    if is_claim {
        Ok(RewardsResponse::ClaimRewards {
            rewards,
            modified_farms,
        })
    } else {
        Ok(RewardsResponse::QueryRewardsResponse { rewards })
    }
}

/// Computes the epoch from which the user can start claiming rewards for a given farm
pub(crate) fn compute_start_from_epoch_for_address(
    storage: &dyn Storage,
    lp_denom: &str,
    last_claimed_epoch: Option<EpochId>,
    receiver: &Addr,
) -> Result<u64, ContractError> {
    let first_claimable_epoch_for_user = if let Some(last_claimed_epoch) = last_claimed_epoch {
        // if the user has claimed before, then the next epoch is the one after the last claimed epoch
        last_claimed_epoch + 1u64
    } else {
        // if the user has never claimed before but has a weight, get the epoch at which the user
        // first had a weight in the system
        get_earliest_address_lp_weight(storage, receiver, lp_denom)?.0
    };

    Ok(first_claimable_epoch_for_user)
}

/// Computes the user weights for a given LP asset. This assumes that [compute_start_from_epoch_for_address]
/// was called before this function, computing the start_from_epoch for the user with either the last_claimed_epoch
/// or the first epoch the user had a weight in the system.
pub(crate) fn compute_address_weights(
    storage: &dyn Storage,
    address: &Addr,
    lp_asset_denom: &str,
    start_from_epoch: &EpochId,
    current_epoch_id: &EpochId,
) -> Result<HashMap<EpochId, Uint128>, ContractError> {
    let mut address_weights = HashMap::new();
    let mut last_weight_seen = Uint128::zero();

    // starts from start_from_epoch - 1 in case the user has a last_claimed_epoch, which means the user
    // has a weight for the last_claimed_epoch. [compute_start_from_epoch_for_farm] would return
    // last_claimed_epoch + 1 in that case, which is correct, and if the user has not modified its
    // position, the weight will be the same for start_from_epoch as it is for last_claimed_epoch.
    for epoch_id in *start_from_epoch - 1..=*current_epoch_id {
        let weight = LP_WEIGHT_HISTORY.may_load(storage, (address, lp_asset_denom, epoch_id))?;

        if let Some(weight) = weight {
            last_weight_seen = weight;
            address_weights.insert(epoch_id, weight);
        } else {
            address_weights.insert(epoch_id, last_weight_seen);
        }
    }
    Ok(address_weights)
}

/// Computes the contract weights for a given LP denom for the epochs between start_from_epoch and current_epoch_id.
pub(crate) fn compute_contract_weights(
    storage: &dyn Storage,
    contract: &Addr,
    lp_asset_denom: &str,
    start_from_epoch: &EpochId,
    current_epoch_id: &EpochId,
) -> Result<HashMap<EpochId, Uint128>, ContractError> {
    let mut contract_weights = HashMap::new();

    let contract_lp_weight =
        LP_WEIGHT_HISTORY.may_load(storage, (contract, lp_asset_denom, *start_from_epoch))?;

    // get which epoch to start filling the hashmap from
    let (start_epoch_id_with_contract_lp_weight, mut last_weight_seen) =
        if let Some(weight) = contract_lp_weight {
            // there's a weight recorded for start_from_epoch for the contract, start from there
            // insert it in the hashmap as we need a weight for the start_from_epoch, which is the epoch
            // the user can start claiming rewards from
            contract_weights.insert(*start_from_epoch, weight);
            (*start_from_epoch, weight)
        } else {
            // there's no weight recorded for start_from_epoch for the contract, which means nobody has
            // opened or closed a position during the last epoch. Go fetch the last recoded
            // lp weight and derive weights from there
            let earliest_contract_lp_weight_result =
                get_earliest_address_lp_weight(storage, contract, lp_asset_denom);

            match earliest_contract_lp_weight_result {
                Err(_) => {
                    // it means the contract has not recorded a lp weight ever for this denom, which should
                    // not happen if someone has ever created a position, and this wouldn't even reach this
                    // point as the function to calculate farm rewards only loops on opened positions.
                    return Err(ContractError::Unauthorized);
                }
                Ok((earliest_epoch_id, weight)) => {
                    // some weight was recorded for the contract in the past, start from there
                    (earliest_epoch_id, weight)
                }
            }
        };

    // start from the epoch after the last recorded weight for the contract. If the start_epoch_id_with_contract_lp_weight
    // is the same as the start_from_epoch, the weight was already put into the hashmap above. Otherwise,
    // it means the epoch is in the past, before the start_epoch_from, so we can safely move on and start
    // from the next epoch after start_epoch_id_with_contract_lp_weight.
    for epoch_id in start_epoch_id_with_contract_lp_weight + 1u64..=*current_epoch_id {
        let weight = LP_WEIGHT_HISTORY.may_load(storage, (contract, lp_asset_denom, epoch_id))?;

        if let Some(weight) = weight {
            last_weight_seen = weight;
        }

        // store the weight in the hashmap only if it's relevant, i.e. if the epoch id is greater or equal
        // than the start_from_epoch.
        if epoch_id >= *start_from_epoch {
            contract_weights.insert(epoch_id, last_weight_seen);
        }
    }

    Ok(contract_weights)
}

/// Computes the rewards emissions for a given farm. Let's assume for now that the farm
/// is expanded by a multiple of the original emission rate.
/// ### Returns
/// A pair with the reward emissions for each epoch between start_from_epoch and the current_epoch_id in a hashmap
/// and the last epoch for which the farm emissions were computed
fn compute_farm_emissions(
    farm: &Farm,
    start_from_epoch: &EpochId,
    current_epoch_id: &EpochId,
) -> Result<(HashMap<EpochId, Uint128>, EpochId), ContractError> {
    let mut farm_emissions = HashMap::new();

    let until_epoch = if farm.preliminary_end_epoch <= *current_epoch_id {
        // the preliminary_end_epoch is not inclusive, so we subtract 1
        farm.preliminary_end_epoch - 1u64
    } else {
        *current_epoch_id
    };

    for epoch in *start_from_epoch..=until_epoch {
        farm_emissions.insert(epoch, farm.emission_rate);
    }

    Ok((farm_emissions, until_epoch))
}

/// Syncs the address lp weight history for the given address and epoch_id, removing all the previous
/// entries as the user has already claimed those epochs, and setting the weight for the current epoch.
pub fn sync_address_lp_weight_history(
    storage: &mut dyn Storage,
    address: &Addr,
    lp_denom: &str,
    current_epoch_id: &u64,
    save_last_lp_weight: bool,
) -> Result<(), ContractError> {
    let (earliest_epoch_id, _) = get_earliest_address_lp_weight(storage, address, lp_denom)?;
    let (latest_epoch_id, latest_address_lp_weight) =
        get_latest_address_lp_weight(storage, address, lp_denom, current_epoch_id)?;

    // remove previous entries
    for epoch_id in earliest_epoch_id..=latest_epoch_id {
        LP_WEIGHT_HISTORY.remove(storage, (address, lp_denom, epoch_id));
    }

    if save_last_lp_weight {
        // save the latest weight for the current epoch
        LP_WEIGHT_HISTORY.save(
            storage,
            (address, lp_denom, *current_epoch_id),
            &latest_address_lp_weight,
        )?;
    }

    Ok(())
}
