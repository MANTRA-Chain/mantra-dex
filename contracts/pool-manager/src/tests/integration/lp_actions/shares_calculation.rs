use cosmwasm_std::{coin, Coin, Decimal, Uint128};
use mantra_common_testing::multi_test::stargate_mock::StargateMock;
use mantra_dex_std::{
    fee::{Fee, PoolFee},
    lp_common::MINIMUM_LIQUIDITY_AMOUNT,
    pool_manager::PoolType,
};
use test_utils::common_constants::{
    DECIMALS_6, DENOM_ULUNA, DENOM_UOM, DENOM_UOSMO, DENOM_UUSD, DENOM_UUSDC, DENOM_UWHALE,
    ONE_THOUSAND, STARGATE_MOCK_UOM_AMOUNT,
};

use crate::tests::suite::TestingSuite;

// Constants using common_constants where available
const INITIAL_BALANCE: u128 = 10_000_000u128;
const SMALL_INITIAL_BALANCE: u128 = 10_000u128;
const TF_FEE_UOM_AMOUNT: u128 = STARGATE_MOCK_UOM_AMOUNT;
const TF_FEE_UUSD_AMOUNT: u128 = ONE_THOUSAND;

const WHALE_ULUNA_POOL_LABEL: &str = "whale.uluna";
const O_WHALE_ULUNA_LP_DENOM_RAW: &str = "o.whale.uluna"; // Raw, because suite.get_lp_denom() is used

const POOL_FEE_PERCENT: u64 = 1;
const ASSET_PRECISION: u8 = DECIMALS_6;

const LIQUIDITY_10K: u128 = 10_000u128;
const LIQUIDITY_9K: u128 = 9_000u128;
const LIQUIDITY_5K: u128 = 5_000u128;

// For provide_liquidity_emits_right_lp_shares test
const VERY_LARGE_INITIAL_BALANCE: u128 = 1_000_000_000_000u128;
const UOM_USDC_POOL_ID_RAW: &str = "p.1"; // Raw, because suite.get_lp_denom() is used for pool id p.1
const SWAP_FEE_PERMILLE: u64 = 30;
const UOM_LIQUIDITY_AMOUNT: u128 = 1_500_000u128;
const USDC_LIQUIDITY_AMOUNT: u128 = 1_000_000u128;

// For withdraw_liquidity_burns_proportional_lp_shares test
const ULUNA_LIQUIDITY_AMOUNT: u128 = 2_000_000u128;
const UWHALE_LIQUIDITY_AMOUNT: u128 = 1_000_000u128;
// Calculated as isqrt(UWHALE_LIQUIDITY_AMOUNT * ULUNA_LIQUIDITY_AMOUNT) - MINIMUM_LIQUIDITY_AMOUNT
// isqrt(1_000_000 * 2_000_000) - 1000 = 1_414_213 - 1000 = 1_413_213
const EXPECTED_LP_SHARES_PROPORTIONAL_BURN: u128 = 1_413_213u128;
const WITHDRAW_LP_AMOUNT: u128 = 500_000u128;
const EXPECTED_ULUNA_AFTER_WITHDRAW: u128 = 707_107u128; // Updated based on test log and recalculation
const EXPECTED_UWHALE_AFTER_WITHDRAW: u128 = 353_553u128; // Updated based on recalculation

#[test]
fn provide_liquidity_emit_proportional_lp_shares() {
    let mut suite = TestingSuite::default_with_balances(
        vec![
            coin(INITIAL_BALANCE, DENOM_UWHALE.to_string()),
            coin(INITIAL_BALANCE, DENOM_ULUNA.to_string()),
            coin(INITIAL_BALANCE, DENOM_UOSMO.to_string()),
            coin(SMALL_INITIAL_BALANCE, DENOM_UUSD.to_string()),
            coin(SMALL_INITIAL_BALANCE, DENOM_UOM.to_string()),
        ],
        StargateMock::new(vec![coin(TF_FEE_UOM_AMOUNT, DENOM_UOM.to_string())]),
    );
    let creator = suite.creator();
    let other = suite.senders[1].clone();

    // Asset denoms with uwhale and uluna
    let asset_denoms = vec![DENOM_UWHALE.to_string(), DENOM_ULUNA.to_string()];

    let pool_fees = PoolFee {
        protocol_fee: Fee {
            share: Decimal::percent(POOL_FEE_PERCENT),
        },
        swap_fee: Fee {
            share: Decimal::percent(POOL_FEE_PERCENT),
        },
        burn_fee: Fee {
            share: Decimal::zero(),
        },
        extra_fees: vec![],
    };

    // Create a pool
    suite.instantiate_default().add_one_epoch().create_pool(
        &creator,
        asset_denoms,
        vec![ASSET_PRECISION, ASSET_PRECISION],
        pool_fees,
        PoolType::ConstantProduct,
        Some(WHALE_ULUNA_POOL_LABEL.to_string()),
        vec![
            coin(TF_FEE_UUSD_AMOUNT, DENOM_UUSD.to_string()),
            coin(TF_FEE_UOM_AMOUNT, DENOM_UOM.to_string()),
        ],
        |result| {
            result.unwrap();
        },
    );

    let contract_addr = suite.pool_manager_addr.clone();
    let lp_denom = suite.get_lp_denom(O_WHALE_ULUNA_LP_DENOM_RAW.to_string());

    // let's provide liquidity with two assets
    suite
        .provide_liquidity(
            &creator,
            O_WHALE_ULUNA_LP_DENOM_RAW.to_string(),
            None,
            None,
            None,
            None,
            None,
            vec![
                Coin {
                    denom: DENOM_UWHALE.to_string(),
                    amount: Uint128::from(LIQUIDITY_10K),
                },
                Coin {
                    denom: DENOM_ULUNA.to_string(),
                    amount: Uint128::from(LIQUIDITY_10K),
                },
            ],
            |result| {
                result.unwrap();
            },
        )
        .query_all_balances(&creator.to_string(), |result| {
            let balances = result.unwrap();

            // user should have 10_000u128 LP shares - MINIMUM_LIQUIDITY_AMOUNT
            assert!(balances.iter().any(|coin| {
                coin.denom == lp_denom && coin.amount == Uint128::from(LIQUIDITY_9K)
            }));
        })
        // contract should have 1_000 LP shares (MINIMUM_LIQUIDITY_AMOUNT)
        .query_all_balances(&contract_addr.to_string(), |result| {
            let balances = result.unwrap();
            // check that balances has 999_000 factory/mantra1zwv6feuzhy6a9wekh96cd57lsarmqlwxdypdsplw6zhfncqw6ftqlydlr9/o.whale.uluna.LP
            assert!(balances.iter().any(|coin| {
                coin.denom == lp_denom.clone() && coin.amount == MINIMUM_LIQUIDITY_AMOUNT
            }));
        });

    println!(
        ">>>> provide liquidity: {} {}, {} {}",
        LIQUIDITY_5K, DENOM_UWHALE, LIQUIDITY_5K, DENOM_ULUNA
    );
    // other provides liquidity as well, half of the tokens the creator provided
    // this should result in ~half LP tokens given to other
    suite
        .provide_liquidity(
            &other,
            O_WHALE_ULUNA_LP_DENOM_RAW.to_string(),
            None,
            None,
            None,
            None,
            None,
            vec![
                Coin {
                    denom: DENOM_UWHALE.to_string(),
                    amount: Uint128::from(LIQUIDITY_5K),
                },
                Coin {
                    denom: DENOM_ULUNA.to_string(),
                    amount: Uint128::from(LIQUIDITY_5K),
                },
            ],
            |result| {
                result.unwrap();
            },
        )
        .query_all_balances(&other.to_string(), |result| {
            let balances = result.unwrap();
            // user should have 5_000 * 10_000 / 10_000 = 5_000 LP shares
            assert!(balances.iter().any(|coin| {
                coin.denom == lp_denom && coin.amount == Uint128::new(LIQUIDITY_5K)
            }));
        });
}

#[test]
fn provide_liquidity_emits_right_lp_shares() {
    let mut suite = TestingSuite::default_with_balances(
        vec![
            coin(VERY_LARGE_INITIAL_BALANCE, DENOM_UWHALE.to_string()),
            coin(VERY_LARGE_INITIAL_BALANCE, DENOM_ULUNA.to_string()),
            coin(VERY_LARGE_INITIAL_BALANCE, DENOM_UOSMO.to_string()),
            coin(VERY_LARGE_INITIAL_BALANCE, DENOM_UUSD.to_string()),
            coin(VERY_LARGE_INITIAL_BALANCE, DENOM_UUSDC.to_string()),
            coin(VERY_LARGE_INITIAL_BALANCE, DENOM_UOM.to_string()),
        ],
        StargateMock::new(vec![coin(TF_FEE_UOM_AMOUNT, DENOM_UOM.to_string())]),
    );
    let creator = suite.creator();

    let asset_denoms = vec![DENOM_UOM.to_string(), DENOM_UUSDC.to_string()];

    let pool_fees = PoolFee {
        protocol_fee: Fee {
            share: Decimal::zero(),
        },
        swap_fee: Fee {
            share: Decimal::permille(SWAP_FEE_PERMILLE),
        },
        burn_fee: Fee {
            share: Decimal::zero(),
        },
        extra_fees: vec![],
    };

    // Create a pool
    suite.instantiate_default().add_one_epoch().create_pool(
        &creator,
        asset_denoms,
        vec![ASSET_PRECISION, ASSET_PRECISION],
        pool_fees,
        PoolType::ConstantProduct,
        None,
        vec![
            coin(TF_FEE_UUSD_AMOUNT, DENOM_UUSD.to_string()),
            coin(TF_FEE_UOM_AMOUNT, DENOM_UOM.to_string()),
        ],
        |result| {
            result.unwrap();
        },
    );

    let contract_addr = suite.pool_manager_addr.clone();
    let lp_denom = suite.get_lp_denom(UOM_USDC_POOL_ID_RAW.to_string());

    // let's provide liquidity 1.5 om, 1 usdc
    suite
        .provide_liquidity(
            &creator,
            UOM_USDC_POOL_ID_RAW.to_string(),
            None,
            None,
            None,
            None,
            None,
            vec![
                Coin {
                    denom: DENOM_UOM.to_string(),
                    amount: Uint128::new(UOM_LIQUIDITY_AMOUNT),
                },
                Coin {
                    denom: DENOM_UUSDC.to_string(),
                    amount: Uint128::new(USDC_LIQUIDITY_AMOUNT),
                },
            ],
            |result| {
                result.unwrap();
            },
        )
        .query_all_balances(&creator.to_string(), |result| {
            let balances = result.unwrap();

            // user should have (sqrt(1.5 * 1_000_000 * 1 * 1_000_000) - 1_000) LP shares = 1_224_744 - 1000 = 1_223_744
            assert!(balances.iter().any(|coin| {
                coin.denom == lp_denom && coin.amount == Uint128::new(1_223_744u128)
            }));
        })
        // contract should have 1_000 LP shares (MINIMUM_LIQUIDITY_AMOUNT)
        .query_all_balances(&contract_addr.to_string(), |result| {
            let balances = result.unwrap();
            assert!(balances.iter().any(|coin| {
                coin.denom == lp_denom.clone() && coin.amount == MINIMUM_LIQUIDITY_AMOUNT
            }));
        });
}

#[test]
fn withdraw_liquidity_burns_proportional_lp_shares() {
    let mut suite = TestingSuite::default_with_balances(
        vec![
            coin(INITIAL_BALANCE, DENOM_UWHALE.to_string()),
            coin(INITIAL_BALANCE, DENOM_ULUNA.to_string()),
            coin(INITIAL_BALANCE, DENOM_UOSMO.to_string()),
            coin(SMALL_INITIAL_BALANCE, DENOM_UUSD.to_string()),
            coin(SMALL_INITIAL_BALANCE, DENOM_UOM.to_string()),
        ],
        StargateMock::new(vec![coin(TF_FEE_UOM_AMOUNT, DENOM_UOM.to_string())]),
    );
    let creator = suite.creator();

    // Asset denoms with uwhale and uluna
    let asset_denoms = vec![DENOM_UWHALE.to_string(), DENOM_ULUNA.to_string()];

    let pool_fees = PoolFee {
        protocol_fee: Fee {
            share: Decimal::percent(POOL_FEE_PERCENT),
        },
        swap_fee: Fee {
            share: Decimal::percent(POOL_FEE_PERCENT),
        },
        burn_fee: Fee {
            share: Decimal::zero(),
        },
        extra_fees: vec![],
    };

    // Create a pool
    suite.instantiate_default().add_one_epoch().create_pool(
        &creator,
        asset_denoms.clone(),
        vec![ASSET_PRECISION, ASSET_PRECISION],
        pool_fees.clone(),
        PoolType::ConstantProduct,
        Some(WHALE_ULUNA_POOL_LABEL.to_string()),
        vec![
            coin(TF_FEE_UUSD_AMOUNT, DENOM_UUSD.to_string()),
            coin(TF_FEE_UOM_AMOUNT, DENOM_UOM.to_string()),
        ],
        |result| {
            result.unwrap();
        },
    );

    let lp_denom = suite.get_lp_denom(O_WHALE_ULUNA_LP_DENOM_RAW.to_string());

    // Provide initial liquidity: 1 uwhale, 2 uluna
    suite
        .provide_liquidity(
            &creator,
            O_WHALE_ULUNA_LP_DENOM_RAW.to_string(),
            None,
            None,
            None,
            None,
            None,
            vec![
                Coin {
                    denom: DENOM_UWHALE.to_string(),
                    amount: Uint128::from(UWHALE_LIQUIDITY_AMOUNT),
                },
                Coin {
                    denom: DENOM_ULUNA.to_string(),
                    amount: Uint128::from(ULUNA_LIQUIDITY_AMOUNT),
                },
            ],
            |result| {
                result.unwrap();
            },
        )
        .query_all_balances(&creator.to_string(), |result| {
            let balances = result.unwrap();
            assert!(balances.iter().any(|coin| coin.denom == lp_denom
                && coin.amount == Uint128::from(EXPECTED_LP_SHARES_PROPORTIONAL_BURN)));
        });

    // Withdraw 500_000 LP tokens
    suite
        .withdraw_liquidity(
            &creator,
            O_WHALE_ULUNA_LP_DENOM_RAW.to_string(),
            vec![Coin {
                denom: lp_denom.clone(),
                amount: Uint128::from(WITHDRAW_LP_AMOUNT),
            }],
            |result| {
                result.unwrap();
            },
        )
        .query_all_balances(&creator.to_string(), |result| {
            let balances = result.unwrap();
            let uluna_balance = balances
                .iter()
                .find(|c| c.denom == DENOM_ULUNA.to_string())
                .unwrap()
                .amount;
            let uwhale_balance = balances
                .iter()
                .find(|c| c.denom == DENOM_UWHALE.to_string())
                .unwrap()
                .amount;

            // Expected amounts after withdrawal. Consider initial balances.
            // Initial uluna: 10_000_000. Provided: 2_000_000. Remaining: 8_000_000. Withdrawn: ~706_612. Total: ~8_706_612
            // Initial uwhale: 10_000_000. Provided: 1_000_000. Remaining: 9_000_000. Withdrawn: ~353_306. Total: ~9_353_306

            // Check if the withdrawn amount is close to the expected.
            // There might be slight differences due to rounding or fees not accounted for in manual calculation.
            // We check if the actual withdrawn amount is within a small delta of the expected.
            // Actual withdrawn = current balance - (initial_balance - provided_liquidity)
            let withdrawn_uluna = uluna_balance.u128() - (INITIAL_BALANCE - ULUNA_LIQUIDITY_AMOUNT);
            let withdrawn_uwhale =
                uwhale_balance.u128() - (INITIAL_BALANCE - UWHALE_LIQUIDITY_AMOUNT);

            // Allow a small delta for precision issues (e.g., 10 units)
            let delta = 10u128;

            assert!(
                withdrawn_uluna >= EXPECTED_ULUNA_AFTER_WITHDRAW.saturating_sub(delta)
                    && withdrawn_uluna <= EXPECTED_ULUNA_AFTER_WITHDRAW.saturating_add(delta),
                "ULUNA balance after withdraw mismatch. Expected around {}, got {}. Withdrawn: {}",
                EXPECTED_ULUNA_AFTER_WITHDRAW,
                uluna_balance,
                withdrawn_uluna
            );
            assert!(
                withdrawn_uwhale >= EXPECTED_UWHALE_AFTER_WITHDRAW.saturating_sub(delta)
                    && withdrawn_uwhale <= EXPECTED_UWHALE_AFTER_WITHDRAW.saturating_add(delta),
                "UWHALE balance after withdraw mismatch. Expected around {}, got {}. Withdrawn: {}",
                EXPECTED_UWHALE_AFTER_WITHDRAW,
                uwhale_balance,
                withdrawn_uwhale
            );

            // Check remaining LP tokens
            let remaining_lp_tokens = EXPECTED_LP_SHARES_PROPORTIONAL_BURN - WITHDRAW_LP_AMOUNT;
            assert!(balances
                .iter()
                .any(|coin| coin.denom == lp_denom
                    && coin.amount == Uint128::from(remaining_lp_tokens)));
        });
}
