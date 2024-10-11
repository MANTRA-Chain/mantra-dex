use std::str::FromStr;

use cosmwasm_std::{Deps, StdResult, Uint128};
use osmosis_std::types::osmosis::tokenfactory::v1beta1::TokenfactoryQuerier;

/// Gets the factory denom creation fee
pub fn get_factory_denom_creation_fee(deps: Deps) -> StdResult<Vec<cosmwasm_std::Coin>> {
    let token_factory_params = TokenfactoryQuerier::new(&deps.querier).params()?;
    let denom_creation_params = token_factory_params.params;

    if let Some(denom_creation_fee) = denom_creation_params {
        // convert osmosis_std::types::cosmos::base::v1beta1::Coin to cosmwasm_std::Coin
        let denom_creation_fee: Vec<cosmwasm_std::Coin> = denom_creation_fee
            .denom_creation_fee
            .iter()
            .map(|coin| {
                let amount = Uint128::from_str(&coin.amount);
                match amount {
                    Ok(amount) => cosmwasm_std::Coin {
                        denom: coin.denom.clone(),
                        amount,
                    },
                    Err(err) => panic!("Invalid amount: {}", err),
                }
            })
            .collect();

        Ok(denom_creation_fee)
    } else {
        Ok(vec![])
    }
}
