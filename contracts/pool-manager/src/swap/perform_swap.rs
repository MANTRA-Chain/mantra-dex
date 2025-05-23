use std::str::FromStr;

use cosmwasm_std::{
    Coin, Decimal, Decimal256, DepsMut, Fraction, StdError, StdResult, Uint128, Uint256,
};

use mantra_dex_std::pool_manager::PoolInfo;

use crate::helpers::{aggregate_outgoing_fees, get_asset_indexes_in_pool};
use crate::{
    helpers,
    state::{get_pool_by_identifier, POOLS},
    ContractError,
};

#[derive(Debug)]
pub struct SwapResult {
    /// The asset that should be returned to the user from the swap.
    pub return_asset: Coin,
    /// The burn fee of `return_asset` associated with this swap transaction.
    pub burn_fee_asset: Coin,
    /// The protocol fee of `return_asset` associated with this swap transaction.
    pub protocol_fee_asset: Coin,
    /// The swap fee of `return_asset` associated with this swap transaction.
    pub swap_fee_asset: Coin,
    /// The extra fees of `return_asset` associated with this swap transaction.
    pub extra_fees_asset: Coin,
    /// The pool that was traded.
    pub pool_info: PoolInfo,
    /// The amount of slippage that occurred during the swap from the original exchange rate.
    pub slippage_amount: Uint128,
}

/// Attempts to perform a swap from `offer_asset` to the relevant opposing
/// asset in the pool identified by `pool_identifier`.
///
/// Assumes that `offer_asset` is a **native token**.
///
/// The resulting [`SwapResult`] has actions that should be taken, as the swap has been performed.
/// In other words, the caller of the `perform_swap` function _should_ make use
/// of each field in [`SwapResult`] (besides fields like `slippage_amount`).
pub fn perform_swap(
    deps: DepsMut,
    offer_asset: Coin,
    ask_asset_denom: String,
    pool_identifier: &str,
    belief_price: Option<Decimal>,
    max_slippage: Option<Decimal>,
) -> Result<SwapResult, ContractError> {
    let mut pool_info = get_pool_by_identifier(&deps.as_ref(), pool_identifier)?;

    let (_, _, offer_index, ask_index, _, _) =
        get_asset_indexes_in_pool(&pool_info, &offer_asset.denom, &ask_asset_denom)?;

    let swap_computation = helpers::compute_swap(&pool_info, &offer_asset, &ask_asset_denom)?;

    let return_asset = Coin {
        denom: ask_asset_denom.clone(),
        amount: swap_computation.return_amount,
    };

    // Assert slippage and other operations
    // check max slippage limit if exist
    assert_max_slippage(
        belief_price,
        max_slippage,
        offer_asset.amount,
        return_asset.amount,
        swap_computation.slippage_amount,
    )?;

    // State changes to the pools balances
    {
        // add the offer amount to the pool
        pool_info.assets[offer_index].amount = pool_info.assets[offer_index]
            .amount
            .checked_add(offer_asset.amount)?;

        // Deduct the return amount and fees from the pool
        let outgoing_fees = aggregate_outgoing_fees(&swap_computation.to_simulation_response())?;

        pool_info.assets[ask_index].amount = pool_info.assets[ask_index]
            .amount
            .checked_sub(return_asset.amount)?
            .checked_sub(outgoing_fees)?;

        POOLS.save(deps.storage, pool_identifier, &pool_info)?;
    }

    let burn_fee_asset = Coin {
        denom: ask_asset_denom.clone(),
        amount: swap_computation.burn_fee_amount,
    };
    let protocol_fee_asset = Coin {
        denom: ask_asset_denom.clone(),
        amount: swap_computation.protocol_fee_amount,
    };
    let extra_fees_asset = Coin {
        denom: ask_asset_denom.clone(),
        amount: swap_computation.extra_fees_amount,
    };

    #[allow(clippy::redundant_clone)]
    let swap_fee_asset = Coin {
        denom: ask_asset_denom.clone(),
        amount: swap_computation.swap_fee_amount,
    };

    Ok(SwapResult {
        return_asset,
        swap_fee_asset,
        burn_fee_asset,
        protocol_fee_asset,
        pool_info,
        extra_fees_asset,
        slippage_amount: swap_computation.slippage_amount,
    })
}

/// Default swap slippage in case max_slippage is not specified
pub const DEFAULT_SLIPPAGE: &str = "0.01";
/// Cap on the maximum swap slippage that is allowed. If max_slippage goes over this limit, it will
/// be capped to this value.
pub const MAX_ALLOWED_SLIPPAGE: &str = "0.5";

/// If `belief_price` and `max_slippage` both are given,
/// we compute new slippage else we just use pool network
/// slippage to check `max_slippage`
pub fn assert_max_slippage(
    belief_price: Option<Decimal>,
    max_slippage: Option<Decimal>,
    offer_amount: Uint128,
    return_amount: Uint128,
    slippage_amount: Uint128,
) -> StdResult<()> {
    let max_slippage: Decimal256 = max_slippage
        .unwrap_or(Decimal::from_str(DEFAULT_SLIPPAGE)?)
        .min(Decimal::from_str(MAX_ALLOWED_SLIPPAGE)?)
        .into();

    if let Some(belief_price) = belief_price {
        let expected_return =
            Decimal256::from_ratio(Uint256::from_uint128(offer_amount), Uint256::one())
                .checked_mul(
                    Decimal256::from(belief_price)
                        .inv()
                        .ok_or_else(|| StdError::generic_err("Belief price can't be zero"))?,
                )?
                .to_uint_floor();
        let slippage_amount = expected_return.saturating_sub(Uint256::from_uint128(return_amount));

        if Uint256::from_uint128(return_amount) < expected_return
            && Decimal256::from_ratio(slippage_amount, expected_return) > max_slippage
        {
            return Err(StdError::generic_err("Slippage limit exceeded"));
        }
    } else if Decimal256::from_ratio(slippage_amount, return_amount + slippage_amount)
        > max_slippage
    {
        return Err(StdError::generic_err("Slippage limit exceeded"));
    }

    Ok(())
}
