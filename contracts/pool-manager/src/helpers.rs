use std::ops::Mul;

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{
    ensure, Addr, Coin, Decimal, Decimal256, Deps, DepsMut, Env, Isqrt, MessageInfo, StdError,
    StdResult, Uint128, Uint256, Uint512,
};
use mantra_dex_std::coin::{add_coins, aggregate_coins, FACTORY_MAX_SUBDENOM_SIZE};
use mantra_dex_std::constants::LP_SYMBOL;
use mantra_dex_std::fee::PoolFee;
use mantra_dex_std::pool_manager::{PoolInfo, PoolType, SimulationResponse};

use crate::error::ContractError;
use crate::math::Decimal256Helper;

/// The amount of iterations to perform when calculating the Newton-Raphson approximation.
const NEWTON_ITERATIONS: u64 = 255;

/// Generic helper function for Newton-Raphson iteration pattern.
///
/// Takes a value type that can be compared and a closure to calculate the next value.
/// Returns the converged value or a ConvergeError.
fn newton_raphson_iterate<T, F>(
    initial_value: T,
    max_iterations: u64,
    convergence_threshold: T,
    next_value_fn: F,
) -> Result<T, ContractError>
where
    T: std::cmp::PartialOrd + std::ops::Sub<Output = T> + Clone,
    F: Fn(T) -> Result<T, ContractError>,
{
    let mut current = initial_value;

    for _ in 0..max_iterations {
        let previous = current.clone();
        current = next_value_fn(previous.clone())?;

        if current.clone() >= previous.clone() {
            if current.clone().sub(previous) <= convergence_threshold {
                return Ok(current);
            }
        } else if previous.sub(current.clone()) <= convergence_threshold {
            return Ok(current);
        }
    }

    // completed iterations but never approximated correctly
    Err(ContractError::ConvergeError)
}

/// Encodes all results of swapping from a source token to a destination token.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SwapResult {
    /// New amount of source token
    pub new_source_amount: Uint128,
    /// New amount of destination token
    pub new_destination_amount: Uint128,
    /// Amount of destination token swapped
    pub amount_swapped: Uint128,
}

/// Calculates the sum of all pool assets with proper precision handling
fn calculate_pool_assets_sum(pool_info: &PoolInfo) -> Result<Decimal256, ContractError> {
    pool_info
        .assets
        .iter()
        .enumerate()
        .try_fold::<_, _, Result<_, ContractError>>(Decimal256::zero(), |acc, (index, asset)| {
            let pool_amount =
                Decimal256::decimal_with_precision(asset.amount, pool_info.asset_decimals[index])?;
            acc.checked_add(pool_amount)
                .map_err(|err| ContractError::Std(StdError::overflow(err)))
        })
}

/// Calculates the amplification factor * number of coins (ann)
fn calculate_ann(amp: &u64, n_coins: Uint256) -> Result<Decimal256, ContractError> {
    let amp_uint = Uint256::from_u128((*amp).into());
    let product = amp_uint
        .checked_mul(n_coins)
        .map_err(|err| ContractError::Std(StdError::overflow(err)))?;

    Ok(Decimal256::from_ratio(product, 1u8))
}

/// Finds the indices of the offer and ask assets in the pool
fn find_asset_indices(
    pool_info: &PoolInfo,
    offer_ask_denoms: &OfferAskDenoms,
) -> Result<(usize, usize), ContractError> {
    // Find the index of the offer asset
    let offer_index = pool_info
        .asset_denoms
        .iter()
        .position(|d| d == &offer_ask_denoms.0)
        .ok_or_else(|| StdError::generic_err("Offer denom not found".to_string()))?;

    // Find the index of the ask asset
    let ask_index = pool_info
        .asset_denoms
        .iter()
        .position(|d| d == &offer_ask_denoms.1)
        .ok_or_else(|| StdError::generic_err("Ask denom not found".to_string()))?;

    Ok((offer_index, ask_index))
}

/// Calculates pool sum and adjusts it based on swap direction
fn calculate_pool_sum(
    pool_info: &PoolInfo,
    offer_ask_denoms: &OfferAskDenoms,
    ask_pool_amount: Decimal256,
    offer_amount: Decimal256,
    direction: &StableSwapDirection,
    max_precision: u8,
) -> Result<Uint512, ContractError> {
    // Get the indices of the offer and ask assets
    let (offer_index, ask_index) = find_asset_indices(pool_info, offer_ask_denoms)?;

    // Calculate the sum of all pools
    let mut pool_sum = Uint512::zero();

    for (i, asset) in pool_info.assets.iter().enumerate() {
        let pool_amount =
            Decimal256::decimal_with_precision(asset.amount, pool_info.asset_decimals[i])?;

        let x = match direction {
            StableSwapDirection::Simulate => {
                if i == offer_index {
                    offer_amount.checked_add(pool_amount)?
                } else if i != ask_index {
                    pool_amount
                } else {
                    continue;
                }
            }
            StableSwapDirection::ReverseSimulate => {
                if i == offer_index {
                    ask_pool_amount.checked_sub(offer_amount)?
                } else if i != ask_index {
                    pool_amount
                } else {
                    continue;
                }
            }
        };

        let x = Uint512::from(x.to_uint256_with_precision(u32::from(max_precision))?);
        pool_sum = pool_sum.checked_add(x)?;
    }

    Ok(pool_sum)
}

/// Calculates the coefficient c for stableswap_y calculation
#[allow(clippy::too_many_arguments)]
fn calculate_stableswap_coefficient_c(
    pool_info: &PoolInfo,
    offer_ask_denoms: &OfferAskDenoms,
    ask_pool_amount: Decimal256,
    offer_amount: Decimal256,
    d_512: Uint512,
    n_coins_512: Uint512,
    direction: &StableSwapDirection,
    max_precision: u8,
) -> Result<Uint512, ContractError> {
    // Get the indices of the offer and ask assets
    let (offer_index, ask_index) = find_asset_indices(pool_info, offer_ask_denoms)?;

    // Initialize c value
    let mut c_512 = d_512;

    // Calculate the product of all pools divided by each pool times n_coins
    for (i, asset) in pool_info.assets.iter().enumerate() {
        let pool_amount =
            Decimal256::decimal_with_precision(asset.amount, pool_info.asset_decimals[i])?;

        let x = match direction {
            StableSwapDirection::Simulate => {
                if i == offer_index {
                    offer_amount.checked_add(pool_amount)?
                } else if i != ask_index {
                    pool_amount
                } else {
                    continue;
                }
            }
            StableSwapDirection::ReverseSimulate => {
                if i == offer_index {
                    ask_pool_amount.checked_sub(offer_amount)?
                } else if i != ask_index {
                    pool_amount
                } else {
                    continue;
                }
            }
        };

        let x = Uint512::from(x.to_uint256_with_precision(u32::from(max_precision))?);
        c_512 = c_512
            .checked_mul(d_512)?
            .checked_div(x.checked_mul(n_coins_512)?)?;
    }

    Ok(c_512)
}

/// Calculates the coefficient b for stableswap_y calculation
fn calculate_stableswap_coefficient_b(
    pool_sum: Uint512,
    d_512: Uint512,
    ann: Uint512,
) -> Result<Uint512, ContractError> {
    Ok(pool_sum.checked_add(d_512.checked_div(ann)?)?)
}

/// Core D calculation logic shared between compute_d and calculate_stableswap_d
fn calculate_d_core(amp_factor: &u64, deposits: &[Uint128], n_coins: Uint128) -> Option<Uint512> {
    // sum(x_i), a.k.a S
    let sum_x = deposits
        .iter()
        .fold(Uint128::zero(), |acc, x| acc.checked_add(*x).unwrap());

    if sum_x == Uint128::zero() {
        Some(Uint512::zero())
    } else {
        // do as below but for a generic number of assets
        let amount_times_coins: Vec<Uint128> = deposits
            .iter()
            .map(|amount| amount.checked_mul(n_coins).unwrap())
            .collect();

        // Newton's method to approximate D
        let mut d_prev: Uint512;
        let mut d: Uint512 = sum_x.into();
        for _ in 0..NEWTON_ITERATIONS {
            let mut d_prod = d;
            for amount in amount_times_coins.clone().into_iter() {
                d_prod = d_prod
                    .checked_mul(d)
                    .unwrap()
                    .checked_div(amount.into())
                    .unwrap();
            }
            d_prev = d;
            d = compute_next_d(amp_factor, d, d_prod, sum_x, n_coins).unwrap();
            // Equality with the precision of 1
            if d > d_prev {
                if d.checked_sub(d_prev).unwrap() <= Uint512::one() {
                    break;
                }
            } else if d_prev.checked_sub(d).unwrap() <= Uint512::one() {
                break;
            }
        }

        Some(d)
    }
}

#[allow(clippy::unwrap_used)]
pub fn compute_d(amp_factor: &u64, deposits: &[Coin]) -> Option<Uint512> {
    let n_coins = Uint128::from(deposits.len() as u128);
    let deposits: Vec<Uint128> = deposits.iter().map(|coin| coin.amount).collect();
    calculate_d_core(amp_factor, &deposits, n_coins)
}

/// Determines the direction of `offer_pool` -> `ask_pool`.
///
/// In a `ReverseSimulate`, we subtract the `offer_pool` from `offer_amount` to get the pool sum.
///
/// In a `Simulate`, we add the two.
pub enum StableSwapDirection {
    Simulate,
    ReverseSimulate,
}

pub(crate) type OfferAskDenoms = (String, String);

/// Calculates the new pool amount given the current pools and swap size.
pub fn calculate_stableswap_y(
    pool_info: &PoolInfo,
    offer_ask_denoms: OfferAskDenoms,
    ask_pool_amount: Decimal256,
    offer_amount: Decimal256,
    amp: &u64,
    direction: StableSwapDirection,
) -> Result<Uint256, ContractError> {
    let amp_512 = Uint512::from_uint256(Uint256::from_u128((*amp).into()));
    let n_coins_256 = Uint256::from(pool_info.assets.len() as u128);
    let n_coins_512 = Uint512::from_uint256(n_coins_256);
    let ann = amp_512.checked_mul(n_coins_512)?;

    // Determine max precision from asset_decimals
    let max_precision = *pool_info.asset_decimals.iter().max().unwrap();

    // Calculate D invariant
    let d_512 = calculate_stableswap_d(pool_info, n_coins_256, amp)?
        .to_uint512_with_precision(u32::from(max_precision))?;

    // Calculate pool sum based on swap direction
    let pool_sum = calculate_pool_sum(
        pool_info,
        &offer_ask_denoms,
        ask_pool_amount,
        offer_amount,
        &direction,
        max_precision,
    )?;

    // Calculate coefficient c
    let mut c_512 = calculate_stableswap_coefficient_c(
        pool_info,
        &offer_ask_denoms,
        ask_pool_amount,
        offer_amount,
        d_512,
        n_coins_512,
        &direction,
        max_precision,
    )?;

    // Finalize coefficient c calculation
    let ann_times_n_coins = ann.checked_mul(n_coins_512)?;
    c_512 = c_512.checked_mul(d_512)?.checked_div(ann_times_n_coins)?;

    // Calculate coefficient b
    let b = calculate_stableswap_coefficient_b(pool_sum, d_512, ann)?;

    // Use newton_raphson_iterate for the approximation
    newton_raphson_iterate(d_512, NEWTON_ITERATIONS, Uint512::one(), |y| {
        // y = (y^2 + c) / (2y + b - d)
        let next_y = y
            .checked_mul(y)?
            .checked_add(c_512)?
            .checked_div(y.checked_add(y)?.checked_add(b)?.checked_sub(d_512)?)?;

        Ok(next_y)
    })
    .and_then(|y| y.try_into().map_err(|_| ContractError::SwapOverflowError))
}

mod test {
    #[cfg(test)]
    mod tests {
        use crate::helpers::calculate_stableswap_y;
        use crate::helpers::StableSwapDirection;

        use cosmwasm_std::assert_approx_eq;
        use cosmwasm_std::{coin, Decimal, Decimal256, Uint128};

        use mantra_dex_std::fee::Fee;
        use mantra_dex_std::fee::PoolFee;
        use mantra_dex_std::pool_manager::{PoolInfo, PoolStatus, PoolType};

        #[test]
        fn test_calculate_stableswap_y() {
            let pool_info = PoolInfo {
                assets: vec![
                    coin(100000000u128, "denom1"),
                    coin(200000000u128, "denom2"),
                    coin(300000000u128, "denom3"),
                ],
                asset_decimals: vec![6, 6, 6],
                asset_denoms: vec![
                    "denom1".to_string(),
                    "denom2".to_string(),
                    "denom3".to_string(),
                ],
                pool_type: PoolType::StableSwap { amp: 100 },
                pool_identifier: "asdasd".to_string(),
                lp_denom: "asdasd".to_string(),
                pool_fees: PoolFee {
                    swap_fee: Fee {
                        share: Decimal::percent(0),
                    },
                    protocol_fee: Fee {
                        share: Decimal::percent(0),
                    },
                    burn_fee: Fee {
                        share: Decimal::percent(0),
                    },
                    extra_fees: vec![],
                },
                status: PoolStatus::default(),
            };

            let offer_ask_denoms = ("denom1".to_string(), "denom2".to_string());
            let ask_pool_amount = Decimal256::from_ratio(200000000u128, 1u128);
            let offer_amount = Decimal256::from_ratio(10u128, 1u128);
            let amp = 100u64;
            let direction = StableSwapDirection::Simulate;

            let result = calculate_stableswap_y(
                &pool_info,
                offer_ask_denoms,
                ask_pool_amount,
                offer_amount,
                &amp,
                direction,
            )
            .unwrap();

            assert_approx_eq!(
                result.try_into().unwrap(),
                Uint128::from(189000000u128),
                "0.01"
            );
        }
    }
}

#[allow(clippy::too_many_arguments)]
/// computes a swap
#[allow(clippy::too_many_arguments)]
pub fn compute_swap(
    pool_info: &PoolInfo,
    offer_asset: &Coin,
    ask_asset_denom: &str,
) -> Result<SwapComputation, ContractError> {
    let (offer_pool, ask_pool, _, _, offer_precision, ask_precision) =
        get_asset_indexes_in_pool(pool_info, &offer_asset.denom, ask_asset_denom)?;

    let offer_pool_amount: Uint256 = offer_pool.amount.into();
    let ask_pool_amount: Uint256 = ask_pool.amount.into();
    let offer_amount: Uint256 = offer_asset.amount.into();

    match &pool_info.pool_type {
        PoolType::ConstantProduct => {
            // offer => ask
            // ask_amount = (ask_pool * offer_amount / (offer_pool + offer_amount)) - swap_fee - protocol_fee - burn_fee
            let return_amount: Uint256 = Decimal256::from_ratio(
                ask_pool_amount.mul(offer_amount),
                offer_pool_amount + offer_amount,
            )
            .to_uint_floor();

            // calculate spread, swap and protocol fees
            let exchange_rate = Decimal256::checked_from_ratio(ask_pool_amount, offer_pool_amount)
                .map_err(|_| ContractError::PoolHasNoAssets)?;
            let spread_amount: Uint256 = (Decimal256::from_ratio(offer_amount, Uint256::one())
                .checked_mul(exchange_rate)?
                .to_uint_floor())
            .checked_sub(return_amount)?;

            let fees_computation = compute_fees(&pool_info.pool_fees, return_amount)?;

            Ok(get_swap_computation(
                return_amount,
                spread_amount,
                fees_computation,
            )?)
        }
        PoolType::StableSwap { amp } => {
            let ask_pool_amount =
                Decimal256::decimal_with_precision(ask_pool_amount, ask_precision)?;
            let offer_amount = Decimal256::decimal_with_precision(offer_amount, offer_precision)?;

            let max_precision = pool_info.asset_decimals.iter().max().unwrap().to_owned();

            let mut new_pool = calculate_stableswap_y(
                pool_info,
                (offer_pool.denom, ask_pool.denom),
                ask_pool_amount,
                offer_amount,
                amp,
                StableSwapDirection::Simulate,
            )?;

            //new_pool is returned with the max_precision. If ask_precision is lower, we need to convert it
            if ask_precision < max_precision {
                new_pool =
                    Decimal256::decimal_with_precision(new_pool, max_precision - ask_precision)?
                        .to_uint_floor();
            }

            let return_amount = ask_pool_amount
                .to_uint256_with_precision(u32::from(ask_precision))?
                .checked_sub(new_pool)?;

            // Return amount is previously returned with the max_precision.
            // We need to convert it to the ask_precision to calculate the spread.
            let adjusted_return_amount = Decimal256::from_ratio(return_amount, 1u128)
                .to_uint256_with_precision(u32::from(max_precision - ask_precision))?;
            let adjusted_offer_amount =
                offer_amount.to_uint256_with_precision(u32::from(max_precision))?;

            let mut spread_amount = adjusted_offer_amount.saturating_sub(adjusted_return_amount);

            // If offer_precision < max_precision, we need to convert the spread_amount to the offer_precision
            if offer_precision < max_precision {
                spread_amount = Decimal256::decimal_with_precision(
                    spread_amount,
                    max_precision - offer_precision,
                )?
                .to_uint_floor();
            }

            let fees_computation = compute_fees(&pool_info.pool_fees, return_amount)?;

            Ok(get_swap_computation(
                return_amount,
                spread_amount,
                fees_computation,
            )?)
        }
    }
}

/// Computes the pool fees for a given (return) amount
fn compute_fees(pool_fees: &PoolFee, amount: Uint256) -> Result<FeesComputation, ContractError> {
    let swap_fee_amount: Uint256 = pool_fees.swap_fee.compute(amount)?;
    let protocol_fee_amount: Uint256 = pool_fees.protocol_fee.compute(amount)?;
    let burn_fee_amount: Uint256 = pool_fees.burn_fee.compute(amount)?;

    let extra_fees_amount: Uint256 = if !pool_fees.extra_fees.is_empty() {
        let mut extra_fees_amount: Uint256 = Uint256::zero();

        for extra_fee in &pool_fees.extra_fees {
            extra_fees_amount = extra_fees_amount.checked_add(extra_fee.compute(amount)?)?;
        }

        extra_fees_amount
    } else {
        Uint256::zero()
    };

    Ok(FeesComputation {
        swap_fee_amount,
        protocol_fee_amount,
        burn_fee_amount,
        extra_fees_amount,
    })
}

/// Builds the swap computation struct, subtracting the fees from the return amount.
fn get_swap_computation(
    return_amount: Uint256,
    spread_amount: Uint256,
    fees_computation: FeesComputation,
) -> Result<SwapComputation, ContractError> {
    let return_amount = return_amount
        .checked_sub(fees_computation.swap_fee_amount)?
        .checked_sub(fees_computation.protocol_fee_amount)?
        .checked_sub(fees_computation.burn_fee_amount)?
        .checked_sub(fees_computation.extra_fees_amount)?;

    let spread_amount = spread_amount
        .checked_add(fees_computation.swap_fee_amount)?
        .checked_add(fees_computation.protocol_fee_amount)?
        .checked_add(fees_computation.burn_fee_amount)?
        .checked_add(fees_computation.extra_fees_amount)?;

    Ok(SwapComputation {
        return_amount: return_amount
            .try_into()
            .map_err(|_| ContractError::SwapOverflowError)?,
        spread_amount: spread_amount
            .try_into()
            .map_err(|_| ContractError::SwapOverflowError)?,
        swap_fee_amount: fees_computation
            .swap_fee_amount
            .try_into()
            .map_err(|_| ContractError::SwapOverflowError)?,
        protocol_fee_amount: fees_computation
            .protocol_fee_amount
            .try_into()
            .map_err(|_| ContractError::SwapOverflowError)?,
        burn_fee_amount: fees_computation
            .burn_fee_amount
            .try_into()
            .map_err(|_| ContractError::SwapOverflowError)?,
        extra_fees_amount: fees_computation
            .extra_fees_amount
            .try_into()
            .map_err(|_| ContractError::SwapOverflowError)?,
    })
}

/// Represents the swap computation values
#[cw_serde]
pub struct FeesComputation {
    pub swap_fee_amount: Uint256,
    pub protocol_fee_amount: Uint256,
    pub burn_fee_amount: Uint256,
    pub extra_fees_amount: Uint256,
}

/// Represents the swap computation values
#[cw_serde]
pub struct SwapComputation {
    pub return_amount: Uint128,
    pub spread_amount: Uint128,
    pub swap_fee_amount: Uint128,
    pub protocol_fee_amount: Uint128,
    pub burn_fee_amount: Uint128,
    pub extra_fees_amount: Uint128,
}

impl SwapComputation {
    /// Converts the SwapComputation struct to a SimulationResponse struct
    pub fn to_simulation_response(&self) -> SimulationResponse {
        SimulationResponse {
            return_amount: self.return_amount,
            spread_amount: self.spread_amount,
            swap_fee_amount: self.swap_fee_amount,
            protocol_fee_amount: self.protocol_fee_amount,
            burn_fee_amount: self.burn_fee_amount,
            extra_fees_amount: self.extra_fees_amount,
        }
    }
}

pub fn compute_offer_amount(
    offer_asset_in_pool: Uint128,
    ask_asset_in_pool: Uint128,
    ask_amount: Uint128,
    pool_fees: PoolFee,
) -> StdResult<OfferAmountComputation> {
    let offer_asset_in_pool: Uint256 = offer_asset_in_pool.into();
    let ask_asset_in_pool: Uint256 = ask_asset_in_pool.into();
    let ask_amount: Uint256 = ask_amount.into();

    // ask => offer
    // offer_amount = cp / (ask_pool - ask_amount / (1 - fees)) - offer_pool
    let mut fees = pool_fees
        .swap_fee
        .to_decimal_256()
        .checked_add(pool_fees.protocol_fee.to_decimal_256())?
        .checked_add(pool_fees.burn_fee.to_decimal_256())?;

    for extra_fee in pool_fees.extra_fees.iter() {
        fees = fees.checked_add(extra_fee.to_decimal_256())?;
    }

    let one_minus_commission = Decimal256::one() - fees;
    let inv_one_minus_commission = Decimal256::one() / one_minus_commission;

    let cp: Uint256 = offer_asset_in_pool * ask_asset_in_pool;
    let offer_amount: Uint256 = Uint256::one()
        .multiply_ratio(
            cp,
            ask_asset_in_pool
                .checked_sub(
                    Decimal256::from_ratio(ask_amount, Uint256::one())
                        .checked_mul(inv_one_minus_commission)?
                        .to_uint_floor(),
                )?
                .checked_sub(Uint256::one())?,
        )
        .checked_sub(offer_asset_in_pool)?;

    let before_commission_deduction: Uint256 = Decimal256::from_ratio(ask_amount, Uint256::one())
        .checked_mul(inv_one_minus_commission)?
        .to_uint_floor();
    let before_spread_deduction: Uint256 = Decimal256::from_ratio(offer_amount, Uint256::one())
        .checked_mul(Decimal256::from_ratio(
            ask_asset_in_pool,
            offer_asset_in_pool,
        ))?
        .to_uint_floor();

    let spread_amount = before_spread_deduction.saturating_sub(before_commission_deduction);

    let swap_fee_amount: Uint256 = pool_fees.swap_fee.compute(before_commission_deduction)?;
    let protocol_fee_amount: Uint256 = pool_fees
        .protocol_fee
        .compute(before_commission_deduction)?;
    let burn_fee_amount: Uint256 = pool_fees.burn_fee.compute(before_commission_deduction)?;

    let mut extra_fees_amount: Uint256 = Uint256::zero();
    for extra_fee in pool_fees.extra_fees.iter() {
        extra_fees_amount =
            extra_fees_amount.checked_add(extra_fee.compute(before_commission_deduction)?)?;
    }

    Ok(OfferAmountComputation {
        offer_amount: offer_amount.try_into()?,
        spread_amount: spread_amount.try_into()?,
        swap_fee_amount: swap_fee_amount.try_into()?,
        protocol_fee_amount: protocol_fee_amount.try_into()?,
        burn_fee_amount: burn_fee_amount.try_into()?,
        extra_fees_amount: extra_fees_amount.try_into()?,
    })
}

/// Represents the offer amount computation values
#[cw_serde]
pub struct OfferAmountComputation {
    pub offer_amount: Uint128,
    pub spread_amount: Uint128,
    pub swap_fee_amount: Uint128,
    pub protocol_fee_amount: Uint128,
    pub burn_fee_amount: Uint128,
    pub extra_fees_amount: Uint128,
}

pub fn assert_slippage_tolerance(
    slippage_tolerance: &Option<Decimal>,
    deposits: &[Coin],
    pool_assets: &mut [Coin],
    pool_type: PoolType,
) -> Result<(), ContractError> {
    if let Some(slippage_tolerance) = *slippage_tolerance {
        let slippage_tolerance: Decimal256 = slippage_tolerance.into();
        if slippage_tolerance > Decimal256::one() {
            return Err(StdError::generic_err("slippage_tolerance cannot bigger than 1").into());
        }

        let one_minus_slippage_tolerance = Decimal256::one() - slippage_tolerance;
        let deposit_amounts: Vec<Uint256> =
            deposits.iter().map(|coin| coin.amount.into()).collect();

        // Sort assets by denom to ensure the order of the assets in the pool is the same as the
        // deposits, which are sorted previously
        pool_assets.sort_by(|a, b| a.denom.cmp(&b.denom));

        let pools: Vec<Uint256> = pool_assets.iter().map(|coin| coin.amount.into()).collect();

        // Ensure each prices are not dropped as much as slippage tolerance rate
        match pool_type {
            PoolType::StableSwap { amp: amp_factor } => {
                let d_initial = compute_d(&amp_factor, pool_assets).unwrap();
                let final_pool_assets = add_coins(pool_assets.to_vec(), deposits.to_vec())?;
                let d_final = compute_d(&amp_factor, &final_pool_assets).unwrap();

                // Safe conversion to Uint256, since a Sqrt of a Uint512 will always fit into a Uint256
                let d_initial_sqrt: Uint256 = d_initial.isqrt().try_into().unwrap();
                let d_final_sqrt: Uint256 = d_final.isqrt().try_into().unwrap();
                // Determine the ratio of the final and initial D values by squaring the ratio of the square roots
                let d_ratio_delta = Decimal256::from_ratio(d_final_sqrt, d_initial_sqrt).pow(2);

                if d_ratio_delta > slippage_tolerance {
                    return Err(ContractError::MaxSlippageAssertion);
                }
            }
            PoolType::ConstantProduct => {
                if deposit_amounts.len() != 2 || pools.len() != 2 {
                    return Err(ContractError::InvalidPoolAssetsLength {
                        expected: 2,
                        actual: deposit_amounts.len(),
                    });
                }

                // both deposits and pools are sorted by denom so the indexes match
                if Decimal256::from_ratio(deposit_amounts[0], deposit_amounts[1])
                    * one_minus_slippage_tolerance
                    > Decimal256::from_ratio(pools[0], pools[1])
                    || Decimal256::from_ratio(deposit_amounts[1], deposit_amounts[0])
                        * one_minus_slippage_tolerance
                        > Decimal256::from_ratio(pools[1], pools[0])
                {
                    return Err(ContractError::MaxSlippageAssertion);
                }
            }
        }
    }

    Ok(())
}

/// This function compares the address of the message sender with the contract admin
/// address. This provides a convenient way to verify if the sender
/// is the admin in a single line.
pub fn assert_admin(deps: Deps, env: &Env, sender: &Addr) -> Result<(), ContractError> {
    let contract_info = deps
        .querier
        .query_wasm_contract_info(env.contract.address.clone())?;
    if let Some(admin) = contract_info.admin {
        if sender != deps.api.addr_validate(admin.as_str())? {
            return Err(ContractError::Unauthorized);
        }
    }
    Ok(())
}

/// Validates the amounts after a single side liquidity provision swap are correct.
pub fn validate_asset_balance(
    deps: &DepsMut,
    env: &Env,
    expected_balance: &Coin,
) -> Result<(), ContractError> {
    let new_asset_balance = deps
        .querier
        .query_balance(&env.contract.address, expected_balance.denom.to_owned())?;

    ensure!(
        expected_balance == &new_asset_balance,
        ContractError::InvalidSingleSideLiquidityProvisionSwap {
            expected: expected_balance.amount,
            actual: new_asset_balance.amount
        }
    );

    Ok(())
}

/// Validates pool identifier is correct, ensuring the identifier doesn't exceed 41 characters,
/// as the LP token symbol will be created as identifier.LP_SYMBOL. Also, that it contains
pub fn validate_pool_identifier(identifier: &str) -> Result<(), ContractError> {
    ensure!(
        identifier.len() < FACTORY_MAX_SUBDENOM_SIZE - LP_SYMBOL.len()
            && identifier
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '/' || c == '.'),
        ContractError::InvalidPoolIdentifier {
            identifier: identifier.to_string()
        }
    );

    Ok(())
}

/// Aggregates the fees from a simulation response that go out of the contract, i.e. protocol fee and burn fee.
/// Doesn't know about the denom, just the amount.
pub fn aggregate_outgoing_fees(
    simulation_response: &SimulationResponse,
) -> Result<Uint128, ContractError> {
    let fees = simulation_response
        .protocol_fee_amount
        .checked_add(simulation_response.burn_fee_amount)?;

    Ok(fees)
}

/// Validates that the pool creation and token factory fees are paid with the transaction.
/// Returns the total amount of fees paid.
pub fn validate_fees_are_paid(
    pool_creation_fee: &Coin,
    denom_creation_fee: Vec<Coin>,
    info: &MessageInfo,
) -> Result<Vec<Coin>, ContractError> {
    let info = &MessageInfo {
        sender: info.sender.clone(),
        funds: aggregate_coins(info.funds.clone())?,
    };

    // Check if pool_creation_fee is in denom_creation_fee
    let pool_fee_included = denom_creation_fee
        .iter()
        .any(|fee| fee.denom == pool_creation_fee.denom);

    // Calculate total pool creation fee amount
    let total_pool_fee_amount = if pool_fee_included {
        denom_creation_fee
            .iter()
            .find(|fee| fee.denom == pool_creation_fee.denom)
            .map(|fee| {
                fee.amount
                    .checked_add(pool_creation_fee.amount)
                    .unwrap_or_default()
            })
            .ok_or(ContractError::PoolCreationFeeMissing)?
    } else {
        pool_creation_fee.amount
    };

    // Ensure pool creation fee is paid
    let paid_pool_fee_amount = get_paid_fee_amount(info, &pool_creation_fee.denom)?;
    ensure!(
        paid_pool_fee_amount == total_pool_fee_amount,
        ContractError::InvalidPoolCreationFee {
            amount: paid_pool_fee_amount,
            expected: total_pool_fee_amount,
        }
    );

    // Accumulate all fees paid
    let mut total_fees = vec![Coin {
        denom: pool_creation_fee.denom.clone(),
        amount: paid_pool_fee_amount,
    }];

    // Ensure all denom_creation_fee are paid, which is required by the token factory
    // Filter out the pool_creation_fee from denom_creation_fee as it was accounted for above,
    // and check the remaining fees
    for fee in denom_creation_fee
        .iter()
        .filter(|fee| fee.denom != pool_creation_fee.denom)
    {
        let paid_fee_amount = get_paid_fee_amount(info, &fee.denom)?;
        ensure!(
            paid_fee_amount == fee.amount,
            ContractError::InvalidTokenFactoryFee {
                denom: fee.denom.clone(),
                amount: paid_fee_amount,
                expected: fee.amount,
            }
        );

        total_fees.push(Coin {
            denom: fee.denom.clone(),
            amount: paid_fee_amount,
        });
    }

    Ok(total_fees)
}

/// Gets the amount of a specific coin denom paid by the user
fn get_paid_fee_amount(info: &MessageInfo, denom: &str) -> Result<Uint128, ContractError> {
    Ok(info
        .funds
        .iter()
        .filter(|fund| fund.denom == denom)
        .map(|fund| fund.amount)
        .try_fold(Uint128::zero(), |acc, amount| acc.checked_add(amount))
        .unwrap_or(Uint128::zero()))
}

/// Validates that no additional funds besides the fees for the pool creation and token factory
/// were sent with the transaction.
pub(crate) fn validate_no_additional_funds_sent_with_pool_creation(
    info: &MessageInfo,
    total_fees: Vec<Coin>,
) -> Result<(), ContractError> {
    let aggregated_funds = aggregate_coins(info.funds.clone())?;

    // Check that the user didn't send more tokens in info.funds than the ones in total_fees
    let extra_funds_exist = aggregated_funds.iter().any(|fund| {
        !total_fees
            .iter()
            .any(|fee| fee.denom == fund.denom && fee.amount == fund.amount)
    });

    ensure!(!extra_funds_exist, ContractError::ExtraFundsSent);

    Ok(())
}

/// Gets the offer and ask asset indexes in a pool, together with their decimals.
pub fn get_asset_indexes_in_pool(
    pool_info: &PoolInfo,
    offer_asset_denom: &str,
    ask_asset_denom: &str,
) -> Result<(Coin, Coin, usize, usize, u8, u8), ContractError> {
    // Find the index of the offer and ask asset in the pools
    let offer_index = pool_info
        .assets
        .iter()
        .position(|pool| offer_asset_denom == pool.denom)
        .ok_or(ContractError::AssetMismatch)?;
    let ask_index = pool_info
        .assets
        .iter()
        .position(|pool| ask_asset_denom == pool.denom)
        .ok_or(ContractError::AssetMismatch)?;

    // make sure it's not the same asset
    ensure!(offer_index != ask_index, ContractError::AssetMismatch);

    let decimals = &pool_info.asset_decimals;

    let offer_asset_in_pool = pool_info.assets[offer_index].clone();
    let ask_asset_in_pool = pool_info.assets[ask_index].clone();
    let offer_decimal = decimals[offer_index];
    let ask_decimal = decimals[ask_index];

    Ok((
        offer_asset_in_pool,
        ask_asset_in_pool,
        offer_index,
        ask_index,
        offer_decimal,
        ask_decimal,
    ))
}

#[allow(clippy::unwrap_used)]
fn compute_next_d(
    amp_factor: &u64,
    d_init: Uint512,
    d_prod: Uint512,
    sum_x: Uint128,
    n_coins: Uint128,
) -> Option<Uint512> {
    let ann = amp_factor.checked_mul(n_coins.u128() as u64)?;
    let leverage = Uint512::from(sum_x).checked_mul(ann.into()).unwrap();
    // d = (ann * sum_x + d_prod * n_coins) * d / ((ann - 1) * d + (n_coins + 1) * d_prod)
    let numerator = d_init
        .checked_mul(
            d_prod
                .checked_mul(n_coins.into())
                .unwrap()
                .checked_add(leverage)
                .unwrap(),
        )
        .unwrap();
    let denominator = d_init
        .checked_mul(ann.checked_sub(1)?.into())
        .unwrap()
        .checked_add(
            d_prod
                .checked_mul((n_coins.checked_add(1u128.into()).unwrap()).into())
                .unwrap(),
        )
        .unwrap();
    Some(numerator.checked_div(denominator).unwrap())
}

/// Computes the amount of lp tokens to mint after a deposit for a stableswap pool.
/// Assumes the deposits have already been credited to the pool_assets.
#[allow(clippy::unwrap_used, clippy::too_many_arguments)]
pub fn compute_lp_mint_amount_for_stableswap_deposit(
    amp_factor: &u64,
    old_pool_assets: &[Coin],
    new_pool_assets: &[Coin],
    pool_lp_token_total_supply: Uint128,
) -> Result<Option<Uint128>, ContractError> {
    // Initial invariant
    let d_0 = compute_d(amp_factor, old_pool_assets).ok_or(ContractError::StableInvariantError)?;

    // Invariant after change, i.e. after deposit
    // notice that new_pool_assets already added the new deposits to the pool
    let d_1 = compute_d(amp_factor, new_pool_assets).ok_or(ContractError::StableInvariantError)?;

    // If the invariant didn't change, return None
    if d_1 <= d_0 {
        Ok(None)
    } else {
        let amount = Uint512::from(pool_lp_token_total_supply)
            .checked_mul(d_1.checked_sub(d_0)?)?
            .checked_div(d_0)?;

        Ok(Some(Uint128::try_from(amount)?))
    }
}

/// Compute the swap amount `y` in proportion to `x`.
///
/// Solve for `y`:
///
/// ```text
/// y**2 + y * (sum' - (A*n**n - 1) * D / (A * n**n)) = D ** (n + 1) / (n ** (2 * n) * prod' * A)
/// y**2 + b*y = c
/// ```
///
#[allow(clippy::many_single_char_names, clippy::unwrap_used)]
pub fn compute_y_raw(
    n_coins: u8,
    amp_factor: &u64,
    swap_in: Uint128,
    //swap_out: Uint128,
    no_swap: Uint128,
    d: Uint512,
) -> Option<Uint512> {
    let ann = amp_factor.checked_mul(n_coins.into())?; // A * n ** n

    // sum' = prod' = x
    // c =  D ** (n + 1) / (n ** (2 * n) * prod' * A)
    let mut c = d;

    c = c
        .checked_mul(d)
        .unwrap()
        .checked_div(swap_in.checked_mul(n_coins.into()).unwrap().into())
        .unwrap();

    c = c
        .checked_mul(d)
        .unwrap()
        .checked_div(no_swap.checked_mul(n_coins.into()).unwrap().into())
        .unwrap();
    c = c
        .checked_mul(d)
        .unwrap()
        .checked_div(ann.checked_mul(n_coins.into()).unwrap().into())
        .unwrap();
    // b = sum(swap_in, no_swap) + D // Ann - D
    // not subtracting D here because that could result in a negative.
    let b = d
        .checked_div(ann.into())
        .unwrap()
        .checked_add(swap_in.into())
        .unwrap()
        .checked_add(no_swap.into())
        .unwrap();

    // Solve for y by approximating: y**2 + b*y = c
    let mut y_prev: Uint512;
    let mut y = d;
    for _ in 0..1000 {
        y_prev = y;
        // y = (y * y + c) / (2 * y + b - d);
        let y_numerator = y.checked_mul(y).unwrap().checked_add(c).unwrap();
        let y_denominator = y
            .checked_mul(Uint512::from(2u8))
            .unwrap()
            .checked_add(b)
            .unwrap()
            .checked_sub(d)
            .unwrap();
        y = y_numerator.checked_div(y_denominator).unwrap();
        if y > y_prev {
            if y.checked_sub(y_prev).unwrap() <= Uint512::one() {
                break;
            }
        } else if y_prev.checked_sub(y).unwrap() <= Uint512::one() {
            break;
        }
    }
    Some(y)
}

/// Computes the swap amount `y` in proportion to `x`.
#[allow(clippy::unwrap_used)]
#[cfg(test)]
pub fn compute_y(
    n_coins: u8,
    amp_factor: &u64,
    x: Uint128,
    no_swap: Uint128,
    d: Uint512,
) -> Option<Uint128> {
    let amount = compute_y_raw(n_coins, amp_factor, x, no_swap, d)?;
    Some(Uint128::try_from(amount).unwrap())
}

/// Compute SwapResult after an exchange
#[allow(clippy::unwrap_used)]
#[cfg(test)]
pub fn swap_to(
    n_coins: u8,
    amp_factor: &u64,
    source_amount: Uint128,
    swap_source_amount: Uint128,
    swap_destination_amount: Uint128,
    unswaped_amount: Uint128,
) -> Option<SwapResult> {
    use cosmwasm_std::coin;

    let deposits = vec![
        coin(swap_source_amount.u128(), "denom1"),
        coin(swap_destination_amount.u128(), "denom2"),
        coin(unswaped_amount.u128(), "denom3"),
    ];
    let y = compute_y(
        n_coins,
        amp_factor,
        swap_source_amount.checked_add(source_amount).unwrap(),
        unswaped_amount,
        compute_d(amp_factor, &deposits).unwrap(),
    )?;
    // https://github.com/curvefi/curve-contract/blob/b0bbf77f8f93c9c5f4e415bce9cd71f0cdee960e/contracts/pool-templates/base/SwapTemplateBase.vy#L466
    let dy = swap_destination_amount
        .checked_sub(y)
        .unwrap()
        .checked_sub(Uint128::one())
        .unwrap();

    let amount_swapped = dy;
    let new_destination_amount = swap_destination_amount.checked_sub(amount_swapped).unwrap();
    let new_source_amount = swap_source_amount.checked_add(source_amount).unwrap();

    Some(SwapResult {
        new_source_amount,
        new_destination_amount,
        amount_swapped,
    })
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::arithmetic_side_effects,
    clippy::too_many_arguments
)]
mod tests {
    use cosmwasm_std::coin;
    use mantra_dex_std::fee::Fee;
    use mantra_dex_std::pool_manager::PoolStatus;
    use proptest::prelude::*;
    use rand::Rng;

    use sim::Model;

    use super::*;

    /// Minimum amplification coefficient.
    pub const MIN_AMP: u64 = 1;

    /// Maximum amplification coefficient.
    pub const MAX_AMP: u64 = 1_000_000;

    /// Maximum number of tokens to swap at once.
    pub const MAX_TOKENS_IN: Uint128 = Uint128::new(2u128 << 110);

    /// Number of coins in a swap. Hardcoded to 3 to reuse previous tests
    pub const N_COINS: u8 = 3;

    fn check_d(model: &Model, amount_a: u128, amount_b: u128, amount_c: u128) -> Uint512 {
        let deposits = vec![
            coin(amount_a, "denom1"),
            coin(amount_b, "denom2"),
            coin(amount_c, "denom4"),
        ];

        compute_d(&model.amp_factor, &deposits).unwrap()
    }

    fn check_y(model: &Model, swap_in: u128, no_swap: u128, d: Uint512) {
        let y = compute_y_raw(
            N_COINS,
            &model.amp_factor,
            Uint128::new(swap_in),
            Uint128::new(no_swap),
            d,
        )
        .unwrap();
        assert_eq!(
            Uint128::try_from(y).unwrap().u128(),
            model.sim_y(0, 1, swap_in)
        )
    }

    #[test]
    fn test_curve_math_specific() {
        // Specific cases
        let model_no_balance = Model::new(1, vec![0, 0, 0], N_COINS);
        check_d(&model_no_balance, 0, 0, 0);

        let amount_a = 1046129065254161082u128;
        let amount_b = 1250710035549196829u128;
        let amount_c = 1111111111111111111u128;
        let model = Model::new(1188, vec![amount_a, amount_b, amount_c], N_COINS);
        let d = check_d(&model, amount_a, amount_b, amount_c);
        let amount_x = 2045250484898639148u128;
        check_y(&model, amount_x, amount_c, d);

        let amount_a = 862538457714585493u128;
        let amount_b = 492548187909826733u128;
        let amount_c = 777777777777777777u128;
        let model = Model::new(9, vec![amount_a, amount_b, amount_c], N_COINS);
        let d = check_d(&model, amount_a, amount_b, amount_c);
        let amount_x = 815577754938955939u128;

        check_y(&model, amount_x, amount_c, d);
    }

    #[test]
    fn test_compute_mint_amount_for_deposit() {
        let deposits = vec![
            coin(MAX_TOKENS_IN.u128(), "denom1"),
            coin(MAX_TOKENS_IN.u128(), "denom2"),
            coin(MAX_TOKENS_IN.u128(), "denom4"),
        ];

        let pool_assets = vec![
            coin(MAX_TOKENS_IN.u128() + MAX_TOKENS_IN.u128(), "denom1"),
            coin(MAX_TOKENS_IN.u128() + MAX_TOKENS_IN.u128(), "denom2"),
            coin(MAX_TOKENS_IN.u128() + MAX_TOKENS_IN.u128(), "denom4"),
        ];

        let pool_token_supply = MAX_TOKENS_IN;

        let actual_mint_amount = compute_lp_mint_amount_for_stableswap_deposit(
            &MIN_AMP,
            &deposits,
            &pool_assets,
            pool_token_supply,
        )
        .unwrap();
        let expected_mint_amount = Some(MAX_TOKENS_IN);

        assert_eq!(actual_mint_amount, expected_mint_amount);
    }

    #[test]
    fn test_curve_math_with_random_inputs() {
        for _ in 0..100 {
            let mut rng = rand::thread_rng();

            let amp_factor: u64 = rng.gen_range(MIN_AMP..=MAX_AMP);
            let amount_a = rng.gen_range(1..=MAX_TOKENS_IN.u128());
            let amount_b = rng.gen_range(1..=MAX_TOKENS_IN.u128());
            let amount_c = rng.gen_range(1..=MAX_TOKENS_IN.u128());

            let model = Model::new(amp_factor, vec![amount_a, amount_b, amount_c], N_COINS);
            let d = check_d(&model, amount_a, amount_b, amount_c);
            let amount_x = rng.gen_range(0..=amount_a);

            check_y(&model, amount_x, amount_c, d);
        }
    }

    #[derive(Debug)]
    struct SwapTest {
        pub amp_factor: u64,
        pub swap_reserve_balance_a: Uint128,
        pub swap_reserve_balance_b: Uint128,
        pub swap_reserve_balance_c: Uint128,
        pub user_token_balance_a: Uint128,
        pub user_token_balance_b: Uint128,
    }

    impl SwapTest {
        pub fn swap_a_to_b(&mut self, swap_amount: Uint128) {
            self.do_swap(true, swap_amount)
        }

        pub fn swap_b_to_a(&mut self, swap_amount: Uint128) {
            self.do_swap(false, swap_amount)
        }

        fn do_swap(&mut self, swap_a_to_b: bool, source_amount: Uint128) {
            let (swap_source_amount, swap_dest_amount) = match swap_a_to_b {
                true => (self.swap_reserve_balance_a, self.swap_reserve_balance_b),
                false => (self.swap_reserve_balance_b, self.swap_reserve_balance_a),
            };

            let SwapResult {
                new_source_amount,
                new_destination_amount,
                amount_swapped,
                ..
            } = swap_to(
                N_COINS,
                &self.amp_factor,
                source_amount,
                swap_source_amount,
                swap_dest_amount,
                self.swap_reserve_balance_c,
            )
            .unwrap();

            match swap_a_to_b {
                true => {
                    self.swap_reserve_balance_a = new_source_amount;
                    self.swap_reserve_balance_b = new_destination_amount;
                    self.user_token_balance_a -= source_amount;
                    self.user_token_balance_b += amount_swapped;
                }
                false => {
                    self.swap_reserve_balance_a = new_destination_amount;
                    self.swap_reserve_balance_b = new_source_amount;
                    self.user_token_balance_a += amount_swapped;
                    self.user_token_balance_b -= source_amount;
                }
            }
        }
    }

    proptest! {
        #[test]
        fn test_swaps_does_not_result_in_more_tokens(
            amp_factor in MIN_AMP..=MAX_AMP,
            initial_user_token_a_amount in 10_000_000..MAX_TOKENS_IN.u128() >> 16,
            initial_user_token_b_amount in 10_000_000..MAX_TOKENS_IN.u128() >> 16,
        ) {

            let mut t = SwapTest { amp_factor, swap_reserve_balance_a: MAX_TOKENS_IN, swap_reserve_balance_b: MAX_TOKENS_IN,
                swap_reserve_balance_c: MAX_TOKENS_IN,
                user_token_balance_a: Uint128::new(initial_user_token_a_amount),
                user_token_balance_b:Uint128::new(initial_user_token_b_amount),
                };

            const ITERATIONS: u64 = 100;
            const SHRINK_MULTIPLIER: u64= 10;

            for i in 0..ITERATIONS {
                let before_balance_a = t.user_token_balance_a;
                let before_balance_b = t.user_token_balance_b;
                let swap_amount = before_balance_a / Uint128::from((i + 1) * SHRINK_MULTIPLIER);
                t.swap_a_to_b(swap_amount);
                let after_balance = t.user_token_balance_a + t.user_token_balance_b;

                assert!(before_balance_a + before_balance_b >= after_balance, "before_a: {}, before_b: {}, after_a: {}, after_b: {}, amp_factor: {:?}", before_balance_a, before_balance_b, t.user_token_balance_a, t.user_token_balance_b, amp_factor);
            }

            for i in 0..ITERATIONS {
                let before_balance_a = t.user_token_balance_a;
                let before_balance_b = t.user_token_balance_b;
                let swap_amount = before_balance_a / Uint128::from((i + 1) * SHRINK_MULTIPLIER);
                t.swap_a_to_b(swap_amount);
                let after_balance = t.user_token_balance_a + t.user_token_balance_b;

                assert!(before_balance_a + before_balance_b >= after_balance, "before_a: {}, before_b: {}, after_a: {}, after_b: {}, amp_factor: {:?}", before_balance_a, before_balance_b, t.user_token_balance_a, t.user_token_balance_b, amp_factor);
            }
        }
    }

    #[test]
    fn test_swaps_does_not_result_in_more_tokens_specific_one() {
        const AMP_FACTOR: u64 = 324449;
        const INITIAL_SWAP_RESERVE_AMOUNT: Uint128 = Uint128::new(100_000_000_000u128);
        const INITIAL_USER_TOKEN_AMOUNT: Uint128 = Uint128::new(10_000_000_000u128);

        let mut t = SwapTest {
            amp_factor: AMP_FACTOR,
            swap_reserve_balance_a: INITIAL_SWAP_RESERVE_AMOUNT,
            swap_reserve_balance_b: INITIAL_SWAP_RESERVE_AMOUNT,
            swap_reserve_balance_c: INITIAL_SWAP_RESERVE_AMOUNT,
            user_token_balance_a: INITIAL_USER_TOKEN_AMOUNT,
            user_token_balance_b: INITIAL_USER_TOKEN_AMOUNT,
        };

        t.swap_a_to_b(Uint128::new(2097152u128));
        t.swap_a_to_b(Uint128::new(8053063680u128));
        t.swap_a_to_b(Uint128::new(48u128));
        assert!(
            t.user_token_balance_a + t.user_token_balance_b
                <= INITIAL_USER_TOKEN_AMOUNT * Uint128::from(2u8)
        );
    }

    #[test]
    fn test_swaps_does_not_result_in_more_tokens_specific_two() {
        const AMP_FACTOR: u64 = 186512;
        const INITIAL_SWAP_RESERVE_AMOUNT: Uint128 = Uint128::new(100_000_000_000u128);
        const INITIAL_USER_TOKEN_AMOUNT: Uint128 = Uint128::new(1_000_000_000u128);

        let mut t = SwapTest {
            amp_factor: AMP_FACTOR,
            swap_reserve_balance_a: INITIAL_SWAP_RESERVE_AMOUNT,
            swap_reserve_balance_b: INITIAL_SWAP_RESERVE_AMOUNT,
            swap_reserve_balance_c: INITIAL_SWAP_RESERVE_AMOUNT,
            user_token_balance_a: INITIAL_USER_TOKEN_AMOUNT,
            user_token_balance_b: INITIAL_USER_TOKEN_AMOUNT,
        };

        t.swap_b_to_a(Uint128::new(33579101u128));
        t.swap_a_to_b(Uint128::new(2097152u128));
        assert!(
            t.user_token_balance_a + t.user_token_balance_b
                <= INITIAL_USER_TOKEN_AMOUNT * Uint128::from(2u8)
        );
    }

    #[test]
    fn test_swaps_does_not_result_in_more_tokens_specific_three() {
        const AMP_FACTOR: u64 = 1220;
        const INITIAL_SWAP_RESERVE_AMOUNT: Uint128 = Uint128::new(100_000_000_000u128);
        const INITIAL_USER_TOKEN_AMOUNT: Uint128 = Uint128::new(1_000_000_000u128);

        let mut t = SwapTest {
            amp_factor: AMP_FACTOR,
            swap_reserve_balance_a: INITIAL_SWAP_RESERVE_AMOUNT,
            swap_reserve_balance_b: INITIAL_SWAP_RESERVE_AMOUNT,
            swap_reserve_balance_c: INITIAL_SWAP_RESERVE_AMOUNT,
            user_token_balance_a: INITIAL_USER_TOKEN_AMOUNT,
            user_token_balance_b: INITIAL_USER_TOKEN_AMOUNT,
        };

        t.swap_b_to_a(Uint128::from(65535u128));
        t.swap_b_to_a(Uint128::from(6133503u128));
        t.swap_a_to_b(Uint128::from(65535u128));
        assert!(
            t.user_token_balance_a + t.user_token_balance_b
                <= INITIAL_USER_TOKEN_AMOUNT * Uint128::from(2u8)
        );
    }

    proptest! {
        #[test]
        fn test_virtual_price_does_not_decrease_from_deposit(
            amp_factor in MIN_AMP..=MAX_AMP,
            deposit_amount_a in 0..MAX_TOKENS_IN.u128() >> 2,
            deposit_amount_b in 0..MAX_TOKENS_IN.u128() >> 2,
            deposit_amount_c in 0..MAX_TOKENS_IN.u128() >> 2,
            pool_token_a_amount in 0..MAX_TOKENS_IN.u128(),
            pool_token_b_amount in 0..MAX_TOKENS_IN.u128(),
            pool_token_c_amount in 0..MAX_TOKENS_IN.u128(),
            pool_token_supply in 0..MAX_TOKENS_IN.u128(),
        ) {
            let pool_assets = vec![
                coin(pool_token_a_amount, "denom1"),
                coin(pool_token_b_amount, "denom2"),
                coin(pool_token_c_amount, "denom3"),
            ];

            let d0 = compute_d(&amp_factor, &pool_assets).unwrap();
            let deposits = vec![
                coin(deposit_amount_a, "denom1"),
                coin(deposit_amount_b, "denom2"),
                coin(deposit_amount_c, "denom3"),
            ];

            // by the time compute_mint_amount_for_stableswap_deposit is called within the contract
            // to compute the lp shares for the stableswap, pool assets include the new deposits already
            let new_pool_assets = vec![
                coin(pool_token_a_amount + deposit_amount_a, "denom1"),
                coin(pool_token_b_amount + deposit_amount_b, "denom2"),
                coin(pool_token_c_amount + deposit_amount_c, "denom3"),
            ];

            let mint_amount = compute_lp_mint_amount_for_stableswap_deposit(
                &amp_factor,
                &deposits,
                &new_pool_assets,
                Uint128::new(pool_token_supply),
                ).unwrap();

            prop_assume!(mint_amount.is_some());

            let d1 = compute_d(&amp_factor, &new_pool_assets).unwrap();

            assert!(d0 < d1);
        }
    }

    proptest! {
        #[test]
        fn test_virtual_price_does_not_decrease_from_swap(
            amp_factor in MIN_AMP..=MAX_AMP,
            source_token_amount in 0..MAX_TOKENS_IN.u128(),
            swap_source_amount in 0..MAX_TOKENS_IN.u128(),
            swap_destination_amount in 0..MAX_TOKENS_IN.u128(),
            unswapped_amount in 0..MAX_TOKENS_IN.u128(),
        ) {
            let source_token_amount = source_token_amount;
            let swap_source_amount = swap_source_amount;
            let swap_destination_amount = swap_destination_amount;
            let unswapped_amount = unswapped_amount;

            let deposits = vec![
                coin(swap_source_amount, "denom1"),
                coin(swap_destination_amount, "denom2"),
                coin(unswapped_amount, "denom3"),
            ];

            let d0 = compute_d(&amp_factor, &deposits).unwrap();

            let swap_result = swap_to(N_COINS, &amp_factor, source_token_amount.into(), swap_source_amount.into(), swap_destination_amount.into(), unswapped_amount.into());
            prop_assume!(swap_result.is_some());

            let swap_result = swap_result.unwrap();

            let swaps = vec![
                coin(swap_result.new_source_amount.u128(), "denom1"),
                coin(swap_result.new_destination_amount.u128(), "denom2"),
                coin(unswapped_amount, "denom3"),
            ];

            let d1 = compute_d(&amp_factor, &swaps).unwrap();

            assert!(d0 <= d1);  // Pool token supply not changed on swaps
        }
    }

    #[test]
    fn test_stableswap_large_amounts_overflow() {
        let min_amp = 1;
        // The test demonstrates that calculate_stableswap_y function works well with large token amounts
        // especially for tokens with high decimal places (18)

        // Define large pool amounts - 1M tokens with 18 decimals (1e24)
        let large_pool = Uint128::new(1_000_000_000_000_000_000_000_000u128);

        // Define large swap amount - 100k tokens with 18 decimals (1e23)
        let large_amount = Uint128::new(100_000_000_000_000_000_000_000u128);

        let amp = min_amp;
        let ask_pool = large_pool;
        let offer_amount = large_amount;

        // Convert to Decimal256 for precision
        let ask_pool_dec = Decimal256::from_ratio(ask_pool, Uint128::new(1));
        let offer_amount_dec = Decimal256::from_ratio(offer_amount, Uint128::new(1));

        let pool_info = PoolInfo {
            pool_identifier: "x".to_string(),
            asset_denoms: vec!["uusdc".to_string(), "uusdt".to_string()],
            lp_denom: "lp".to_string(),
            asset_decimals: vec![18, 18],
            assets: vec![
                coin(large_pool.u128(), "uusd"),
                coin(large_pool.u128(), "uusdt"),
            ],
            pool_type: PoolType::StableSwap { amp },
            pool_fees: PoolFee {
                protocol_fee: Fee {
                    share: Decimal::zero(),
                },
                swap_fee: Fee {
                    share: Decimal::zero(),
                },
                burn_fee: Fee {
                    share: Decimal::zero(),
                },
                extra_fees: vec![],
            },
            status: PoolStatus::default(),
        };

        // This will panic with a CheckedMultiplyRatioError(Overflow) because intermediate
        // calculations in calculate_stableswap_y overflow Uint256 when dealing with large values
        let result = calculate_stableswap_y(
            &pool_info,
            ("uusdc".to_string(), "uusdt".to_string()),
            ask_pool_dec,
            offer_amount_dec,
            &amp,
            StableSwapDirection::Simulate,
        );

        match result {
            Ok(_) => {
                // If we get here, the test is successful
            }
            Err(e) => {
                // We expect a CheckedMultiplyRatioError due to overflow
                match e {
                    crate::error::ContractError::CheckedMultiplyRatioError(_) => {
                        panic!("Error received: {:?}", e);
                    }
                    _ => {
                        // Any other error is unexpected
                        panic!("Unexpected error: {:?}", e);
                    }
                }
            }
        }
    }
}

fn calculate_stableswap_d(
    pool_info: &PoolInfo,
    n_coins: Uint256,
    amp: &u64,
) -> Result<Decimal256, ContractError> {
    let n_coins_decimal = Decimal256::from_ratio(n_coins, Uint256::one());

    // Calculate sum of pools with proper precision
    let sum_pools = calculate_pool_assets_sum(pool_info)?;

    if sum_pools.is_zero() {
        // there was nothing to swap, return `0`.
        return Ok(Decimal256::zero());
    }

    // Calculate ann = amp * n_coins
    let ann = calculate_ann(amp, n_coins)?;

    // Use newton_raphson_iterate for the approximation
    let precision_threshold = Decimal256::one();

    newton_raphson_iterate(
        sum_pools,
        NEWTON_ITERATIONS,
        precision_threshold,
        |current_d| {
            let new_d = pool_info
                .assets
                .iter()
                .enumerate()
                .try_fold::<_, _, Result<_, ContractError>>(current_d, |acc, (index, asset)| {
                    let pool_amount = Decimal256::decimal_with_precision(
                        asset.amount,
                        pool_info.asset_decimals[index],
                    )?;
                    let mul_pools = pool_amount.checked_mul(n_coins_decimal)?;
                    acc.checked_multiply_ratio(current_d, mul_pools)
                })?;

            // current_d = ((ann * sum_pools + new_d * n_coins) * current_d) / ((ann - 1) * current_d + (n_coins + 1) * new_d)
            let next_d = (ann
                .checked_mul(sum_pools)?
                .checked_add(new_d.checked_mul(n_coins_decimal)?)?
                .checked_mul(current_d)?)
            .checked_div(
                (ann.checked_sub(Decimal256::one())?
                    .checked_mul(current_d)?
                    .checked_add(
                        n_coins_decimal
                            .checked_add(Decimal256::one())?
                            .checked_mul(new_d)?,
                    ))?,
            )?;

            Ok(next_d)
        },
    )
}
