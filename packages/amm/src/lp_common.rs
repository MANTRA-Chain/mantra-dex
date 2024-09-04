use crate::coin::is_factory_token;
use crate::tokenfactory;
use cosmwasm_std::{ensure, Addr, Coin, CosmosMsg, StdError, StdResult, Uint128};

pub const MINIMUM_LIQUIDITY_AMOUNT: Uint128 = Uint128::new(1_000u128);

/// Creates the Mint LP message
#[allow(unused_variables)]
pub fn mint_lp_token_msg(
    liquidity_asset: String,
    recipient: &Addr,
    sender: &Addr,
    amount: Uint128,
) -> StdResult<CosmosMsg> {
    ensure!(
        is_factory_token(liquidity_asset.as_str()),
        StdError::generic_err("Invalid LP token")
    );

    Ok(tokenfactory::mint::mint(
        sender.clone(),
        Coin {
            denom: liquidity_asset,
            amount,
        },
        recipient.clone().into_string(),
    ))
}

/// Creates the Burn LP message
#[allow(unused_variables)]
pub fn burn_lp_asset_msg(
    liquidity_asset: String,
    sender: Addr,
    amount: Uint128,
) -> StdResult<CosmosMsg> {
    ensure!(
        is_factory_token(liquidity_asset.as_str()),
        StdError::generic_err("Invalid LP token")
    );

    Ok(tokenfactory::burn::burn(
        sender.clone(),
        Coin {
            denom: liquidity_asset,
            amount,
        },
        sender.into_string(),
    ))
}
