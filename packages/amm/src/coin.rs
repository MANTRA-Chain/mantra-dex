use std::collections::HashMap;

use cosmwasm_std::{BankMsg, Coin, CosmosMsg, StdError, StdResult, Uint128};

pub const FACTORY_PREFIX: &str = "factory";
pub const FACTORY_MAX_SUBDENOM_SIZE: usize = 44usize;

/// Verifies if the given denom is a factory token or not.
/// A factory token has the following structure: factory/{creating contract address}/{subdenom}
/// Subdenom can be of length at most 44 characters, in [0-9a-zA-Z./].
/// For more details about what's expected from a factory token, please refer to
/// https://docs.osmosis.zone/osmosis-core/modules/tokenfactory
pub fn is_factory_token(denom: &str) -> bool {
    let split: Vec<&str> = denom.splitn(3, '/').collect();

    if split.len() != 3 || split[0] != FACTORY_PREFIX {
        return false;
    }

    let subdenom = split[2];

    let valid_subdenom = subdenom
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '/' || c == '.');

    if !valid_subdenom {
        return false;
    }

    if subdenom.len() > FACTORY_MAX_SUBDENOM_SIZE {
        return false;
    }

    let creator_address = split[1];
    let total_len = FACTORY_PREFIX.len() + 2 + creator_address.len() + subdenom.len();

    if total_len > 128 {
        return false;
    }

    true
}

/// Gets the subdenom of a factory token. To be called after [is_factory_token] has been successful.
pub fn get_factory_token_subdenom(denom: &str) -> StdResult<&str> {
    let subdenom = denom.splitn(3, '/').nth(2);

    subdenom.map_or_else(
        || {
            Err(StdError::generic_err(
                "Splitting factory token subdenom failed",
            ))
        },
        Ok,
    )
}

/// Gets the creator of a factory token. To be called after [is_factory_token] has been successful.
#[allow(clippy::needless_splitn)]
pub fn get_factory_token_creator(denom: &str) -> StdResult<&str> {
    let creator = denom.splitn(3, '/').nth(1);

    creator.map_or_else(
        || {
            Err(StdError::generic_err(
                "Splitting factory token creator failed",
            ))
        },
        Ok,
    )
}

/// Add the coins in `to_add` to `coins` if they exist.
pub fn add_coins(coins: Vec<Coin>, to_add: Vec<Coin>) -> StdResult<Vec<Coin>> {
    let mut updated_coins = coins.to_vec();

    for coin in to_add {
        if let Some(existing_coin) = updated_coins.iter_mut().find(|c| c.denom == coin.denom) {
            existing_coin.amount = existing_coin.amount.checked_add(coin.amount)?;
        } else {
            return Err(StdError::generic_err(format!(
                "Error: Cannot add {} {}. Coin not found.",
                coin.amount, coin.denom
            )));
        }
    }

    updated_coins.retain(|coin| coin.amount > Uint128::zero());

    Ok(updated_coins)
}

/// Aggregates coins from two vectors, summing up the amounts of coins that are the same.
pub fn aggregate_coins(coins: Vec<Coin>) -> StdResult<Vec<Coin>> {
    let mut aggregation_map: HashMap<String, Uint128> = HashMap::new();

    // aggregate coins by denom
    for coin in coins {
        if let Some(existing_amount) = aggregation_map.get_mut(&coin.denom) {
            *existing_amount = existing_amount.checked_add(coin.amount)?;
        } else {
            aggregation_map.insert(coin.denom.clone(), coin.amount);
        }
    }

    // create a new vector from the aggregation map
    let mut aggregated_coins: Vec<Coin> = Vec::new();
    for (denom, amount) in aggregation_map {
        aggregated_coins.push(Coin { denom, amount });
    }

    // sort coins by denom
    aggregated_coins.sort_by(|a, b| a.denom.cmp(&b.denom));

    Ok(aggregated_coins)
}

/// Creates a CosmosMsg::Bank::BankMsg::Burn message with the given coin.
pub fn burn_coin_msg(coin: Coin) -> CosmosMsg {
    CosmosMsg::Bank(BankMsg::Burn { amount: vec![coin] })
}

#[cfg(test)]
mod coin_tests {
    use crate::coin::{get_factory_token_creator, is_factory_token};

    #[test]
    fn is_factory_token_test() {
        let coin_0 = "ibc/3A6F4C8D5B2E7A1F0C4D5B6E7A8F9C3D4E5B6A7F8E9C4D5B6E7A8F9C3D4E5B6A";
        let coin_1 = "ibc/A1B2C3D4E5F6G7H8I9J0K1L2M3N4O5P6Q7R8S9T0U1V2W3X4Y5Z6A7B8C9D0E1F2";
        let coin_2 = "factory/mantra158xlpsqqkqpkmcrgnlcrc5fjyhy7j7x2vpa79r/subdenom";
        // malformed factory tokens
        let coin_3 =  "factory/mantra1zwv6feuzhy6a9wekh96cd57lsarmqlwxdypdsplw6zhfncqw6ftqlydlr9/ibc/3A6F4C8D5B2E7A1F0C4D5B6E7A8F9C3D4E5B6A7F8E9C4D5B6E7A8F9C3D4E5B6A-ibc/A1B2C3D4E5F6G7H8I9J0K1L2M3N4O5P6Q7R8S9T0U1V2W3X4Y5Z6A7B8C9D0E1F2.pool.0.LP";
        let coin_4 =  "factory/mantra1zwv6feuzhy6a9wekh96cd57lsarmqlwxdypdsplw6zhfncqw6ftqlydlr9/invalid-denom";
        let coin_5 = "uom";

        assert!(!is_factory_token(coin_0));
        assert!(!is_factory_token(coin_1));
        assert!(is_factory_token(coin_2));
        assert!(!is_factory_token(coin_3));
        assert!(!is_factory_token(coin_4));
        assert!(!is_factory_token(coin_5));
    }

    #[test]
    fn test_factory_token_creator() {
        let denom = "factory/creator/subdenom";

        assert_eq!(get_factory_token_creator(denom).unwrap(), "creator");
    }
}
