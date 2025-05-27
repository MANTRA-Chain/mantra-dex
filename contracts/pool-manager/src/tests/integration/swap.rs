use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::LazyLock;

use crate::tests::integration::helpers::extract_pool_reserves;
use crate::tests::suite::TestingSuite;
use cosmwasm_std::{assert_approx_eq, coin, Coin, Decimal, Uint128};
use mantra_common_testing::multi_test::stargate_mock::StargateMock;
use mantra_dex_std::fee::{Fee, PoolFee};
use mantra_dex_std::lp_common::MINIMUM_LIQUIDITY_AMOUNT;
use mantra_dex_std::pool_manager::{PoolType, SimulationResponse};
use test_utils::common_constants::{
    DECIMALS_12, DECIMALS_18, DECIMALS_6, DENOM_ULUNA, DENOM_UOM, DENOM_UOSMO, DENOM_UUSD,
    DENOM_UUSDC, DENOM_UUSDT, DENOM_UWHALE, ONE_HUNDRED_TRILLION, ONE_MILLION, ONE_THOUSAND,
    STABLESWAP_AMP_FACTOR, STARGATE_MOCK_UOM_AMOUNT,
};

// Constants for denoms (using common constants where available)
const AUSDY_DENOM: &str = "ausdy"; // Different from DENOM_UUSDY, keep separate

// Constants for pool identifiers
const WHALE_ULUNA_POOL_RAW: &str = "whale.uluna";
const WHALE_ULUNA_POOL_ID: &str = "o.whale.uluna";

const UOM_UUSD_POOL_ID: &str = "o.uom.uusd";
const ULUNA_UUSD_POOL_RAW: &str = "uluna.uusd";
const ULUNA_UUSD_POOL_ID: &str = "o.uluna.uusd";
const P1_POOL_ID: &str = "p.1";

// Common Amounts (using common constants)
const ONE_MILLION_U128: Uint128 = Uint128::new(ONE_MILLION);
const ONE_THOUSAND_U128: Uint128 = Uint128::new(ONE_THOUSAND);
const EIGHT_EIGHT_EIGHT_EIGHT_U128: Uint128 = Uint128::new(STARGATE_MOCK_UOM_AMOUNT);

const ULUNA_LIQUIDITY_RESERVES: Uint128 = Uint128::new(3_000_000u128);
const UUSD_LIQUIDITY_RESERVES_ULUNA_POOL: Uint128 = Uint128::new(1_000_000u128);

#[allow(clippy::inconsistent_digit_grouping)]
const BALANCE_1T_U128: Uint128 = Uint128::new(1_000_000_000_000u128);

// Default Fee Shares
static DEFAULT_PROTOCOL_FEE_SHARE: LazyLock<Decimal> =
    LazyLock::new(|| Decimal::from_ratio(1u128, 100_000u128));
static DEFAULT_SWAP_FEE_SHARE: LazyLock<Decimal> =
    LazyLock::new(|| Decimal::from_ratio(1u128, 100_000u128));
static DECIMAL_ZERO: LazyLock<Decimal> = LazyLock::new(Decimal::zero);

// Fee Shares for specific tests

static SWAP_FEE_PERMILLE_5: LazyLock<Decimal> = LazyLock::new(|| Decimal::permille(5));

// Other specific constants (using common constants where available)
const STABLE_SWAP_AMP: u64 = STABLESWAP_AMP_FACTOR;
const LIQUIDITY_1B_U128: Uint128 = Uint128::new(1000_000000u128);
const OFFER_10M_UWHALE_U128: Uint128 = Uint128::new(10000000u128);

// -- Constants for swap_large_digits_xyk --
#[allow(clippy::inconsistent_digit_grouping)]
const BALANCE_UOM_150T_XYK: Uint128 = Uint128::new(150_000_000_000_000_000000u128);
#[allow(clippy::inconsistent_digit_grouping)]
const BALANCE_AUSDY_100Q_XYK: Uint128 = Uint128::new(100_000_000_000_000_000000000000000000u128);
#[allow(clippy::inconsistent_digit_grouping)]
#[allow(clippy::inconsistent_digit_grouping)]
const SWAP_2Q_AUSDY_XYK: Uint128 = Uint128::new(2_000_000_000_000_000000000000000000u128);
#[allow(clippy::inconsistent_digit_grouping)]
const SWAP_10T_UOM_XYK: Uint128 = Uint128::new(10_000_000_000_000_000000u128);

#[allow(clippy::inconsistent_digit_grouping)]
#[allow(clippy::inconsistent_digit_grouping)]
const SIMULATED_RETURN_2_852T_UOM_XYK: Uint128 = Uint128::new(2_852_941_176_470_588236u128);
#[allow(clippy::inconsistent_digit_grouping)]
const SIMULATED_RETURN_6_296Q_AUSDY_XYK: Uint128 =
    Uint128::new(6_296_013_475_575_519371168089897701u128);

// -- Constants for swap_large_digits_stable --
#[allow(clippy::inconsistent_digit_grouping)]
const BALANCE_UUSDC_100T_STABLE: Uint128 = Uint128::new(100_000_000_000_000_000000u128);
#[allow(clippy::inconsistent_digit_grouping)]
const BALANCE_AUSDY_100Q_STABLE: Uint128 = Uint128::new(100_000_000_000_000_000000000000000000u128);
#[allow(clippy::inconsistent_digit_grouping)]

const LIQUIDITY_UUSDC_100T_STABLE: Uint128 = BALANCE_UUSDC_100T_STABLE;
const LIQUIDITY_AUSDY_100Q_STABLE: Uint128 = BALANCE_AUSDY_100Q_STABLE;

#[allow(clippy::inconsistent_digit_grouping)]
const SWAP_10T_UUSDC_STABLE: Uint128 = Uint128::new(10_000_000_000_000_000000u128);
#[allow(clippy::inconsistent_digit_grouping)]
const SWAP_20Q_AUSDY_STABLE: Uint128 = Uint128::new(20_000_000_000_000_000000000000000000u128);

#[allow(clippy::inconsistent_digit_grouping)]
const SIMULATED_RETURN_9_476Q_AUSDY_STABLE: Uint128 =
    Uint128::new(9_476_190_476_190_476190476190476190u128);
#[allow(clippy::inconsistent_digit_grouping)]
const SIMULATED_RETURN_19_850T_UUSDC_STABLE: Uint128 = Uint128::new(19_850_486_315_313_277539u128);

// -- Constants for swap_large_digits_stable_18_digits --
const PUSDC_DENOM: &str = "pusdc";
#[allow(clippy::inconsistent_digit_grouping)]
const BALANCE_300T_HIGH_PREC_U128: Uint128 =
    Uint128::new(300_000_000_000_000_000000000000000000u128);
#[allow(clippy::inconsistent_digit_grouping)]
const SWAP_100T_HIGH_PREC_U128: Uint128 = Uint128::new(100_000_000_000_000_000000000000000000u128);
#[allow(clippy::inconsistent_digit_grouping)]
const SIMULATED_RETURN_74_625T_HIGH_PREC_U128: Uint128 =
    Uint128::new(74_625_000_000_000_000000000000000000u128);
#[allow(clippy::inconsistent_digit_grouping)]
const SWAP_50T_HIGH_PREC_U128: Uint128 = Uint128::new(50_000_000_000_000_000000000000000000u128);
#[allow(clippy::inconsistent_digit_grouping)]
const SIMULATED_RETURN_72_265T_HIGH_PREC_U128: Uint128 =
    Uint128::new(72_265_093_054_925_102133454380390377u128);

static DECIMAL_PERCENT_30: LazyLock<Decimal> = LazyLock::new(|| Decimal::percent(30));

// -- Constants for swap_3pool_same_decimals --
#[allow(clippy::inconsistent_digit_grouping)]
const BALANCE_300T_3POOL_U128: Uint128 = Uint128::new(300_000_000_000_000_000000000000000000u128);

const SWAP_OFFER_200M_UUSDC_U128: Uint128 = Uint128::new(200_000_000u128);
const SIMULATED_RETURN_199_517_195_UUSDT_U128: Uint128 = Uint128::new(199_517_195u128);

// -- Constants for swap_3pool_different_decimals --
#[allow(clippy::inconsistent_digit_grouping)]
const SWAP_10K_ATTO_UUSDT_3POOL_DIFFERENT_DECIMALS: Uint128 =
    Uint128::new(10_000_000000000000000000u128);
static MAX_SLIPPAGE_PERCENT_30_3POOL_DIFFERENT_DECIMALS: LazyLock<Decimal> =
    LazyLock::new(|| Decimal::percent(30));

// -- Constants for swap_4pool_different_decimals and its setup helper --
const UUSDX_DENOM: &str = "uusdx";

// -- Constants for simulation_vs_reverse_simulation_3pool --

static ONE_BPS_DECIMAL_TOLERANCE: LazyLock<Decimal> =
    LazyLock::new(|| Decimal::from_ratio(1u128, 10000u128));

// -- Constants for belief_price_works_decimals_independent --
const UWETH_DENOM: &str = "uweth";

const UUSD_UWETH_POOL_ID: &str = "o.uusd.uweth";
const SWAP_OFFER_UWETH_BELIEF_PRICE: Uint128 = Uint128::new(10_0000000000000000000u128); // 10 * 10^18
const SWAP_OFFER_UUSD_BELIEF_PRICE: Uint128 = Uint128::new(10_000_000u128); // 10 * 10^6
const UUSDC_UUSDT_POOL_ID: &str = "o.uusdc.uusdt";

const ALICE_INITIAL_LIQ_BASE_SKEWED_TEST: Uint128 = Uint128::new(1_000u128);
const BOB_INITIAL_LIQ_USDT_BASE_SKEWED_TEST: Uint128 = Uint128::new(110u128);
const BOB_INITIAL_LIQ_USDC_BASE_SKEWED_TEST: Uint128 = Uint128::new(90u128);

// -- Constants for basic_swapping_test --

const EVENT_KEY_RETURN_AMOUNT: &str = "return_amount";
const EVENT_KEY_OFFER_AMOUNT: &str = "offer_amount";
const EVENT_TYPE_WASM: &str = "wasm";

const BASIC_SWAP_ASSERT_APPROX_TOLERANCE_A: &str = "0.002";
const BASIC_SWAP_ASSERT_APPROX_TOLERANCE_B: &str = "0.003";

// -- Constants for basic_swapping_pool_reserves_event_test --

const POOL_RESERVES_KEY_POOL_RESERVES: &str = "pool_reserves";
const POOL_RESERVES_KEY_POOL_IDENTIFIER: &str = "pool_identifier";
static POOL_RESERVES_DECIMAL_PERCENT_10: LazyLock<Decimal> = LazyLock::new(|| Decimal::percent(10));

// -- Constants for basic_swapping_test_stable_swap_two_assets --

const STABLE_SWAP_TWO_ASSETS_LIQUIDITY_AMOUNT: Uint128 = Uint128::new(1_000_000u128);

const SWAP_WITH_FEES_LIQUIDITY_AMOUNT: Uint128 = LIQUIDITY_1B_U128; // Reuse existing constant

// -- Constants for swap_large_digits_xyk --
const SWAP_LARGE_DIGITS_XYK_DEFAULT_BALANCE: Uint128 = Uint128::new(1_000_000_000_000u128);

// -- Constants for swap_large_digits_stable --

#[test]
fn basic_swapping_test() {
    let mut suite = TestingSuite::default_with_balances(
        vec![
            coin(1_000_000_001u128, DENOM_UWHALE.to_string()),
            coin(1_000_000_000u128, DENOM_ULUNA.to_string()),
            coin(1_000_000_001u128, DENOM_UUSD.to_string()),
            coin(1_000_000_001u128, DENOM_UOM.to_string()),
        ],
        StargateMock::new(vec![coin(
            EIGHT_EIGHT_EIGHT_EIGHT_U128.u128(),
            DENOM_UOM.to_string(),
        )]),
    );
    let creator = suite.creator();
    let _other = suite.senders[1].clone();
    let _unauthorized = suite.senders[2].clone();
    // Asset infos with uwhale and uluna

    let asset_infos = vec![DENOM_UWHALE.to_string(), DENOM_ULUNA.to_string()];

    // Protocol fee is 0.01% and swap fee is 0.02% and burn fee is 0%
    let pool_fees = PoolFee {
        protocol_fee: Fee {
            share: *DEFAULT_PROTOCOL_FEE_SHARE,
        },
        swap_fee: Fee {
            share: *DEFAULT_SWAP_FEE_SHARE,
        },
        burn_fee: Fee {
            share: *DECIMAL_ZERO,
        },
        extra_fees: vec![],
    };

    // Create a pool
    suite.instantiate_default().add_one_epoch().create_pool(
        &creator,
        asset_infos,
        vec![DECIMALS_6, DECIMALS_6],
        pool_fees,
        PoolType::ConstantProduct,
        Some(WHALE_ULUNA_POOL_RAW.to_string()),
        vec![
            coin(ONE_THOUSAND_U128.u128(), DENOM_UUSD.to_string()),
            coin(EIGHT_EIGHT_EIGHT_EIGHT_U128.u128(), DENOM_UOM.to_string()),
        ],
        |result| {
            result.unwrap();
        },
    );

    // Query pool info to ensure the query is working fine
    suite.query_pools(
        Some(WHALE_ULUNA_POOL_ID.to_string()),
        None,
        None,
        |result| {
            assert_eq!(
                result.unwrap().pools[0].pool_info.asset_decimals,
                vec![DECIMALS_6, DECIMALS_6]
            );
        },
    );

    // Let's try to add liquidity
    suite
        .provide_liquidity(
            &creator,
            WHALE_ULUNA_POOL_ID.to_string(),
            None,
            None,
            None,
            None,
            None,
            vec![
                Coin {
                    denom: DENOM_UWHALE.to_string(),
                    amount: ONE_MILLION_U128,
                },
                Coin {
                    denom: DENOM_ULUNA.to_string(),
                    amount: ONE_MILLION_U128,
                },
            ],
            |result| {
                // Ensure we got 999_000 in the response which is 1mil less the initial liquidity amount
                assert!(result.unwrap().events.iter().any(|event| {
                    event.attributes.iter().any(|attr| {
                        attr.key == "added_shares"
                            && attr.value
                                == (ONE_MILLION_U128 - MINIMUM_LIQUIDITY_AMOUNT).to_string()
                    })
                }));
            },
        )
        .query_pools(
            Some(WHALE_ULUNA_POOL_ID.to_string()),
            None,
            None,
            |result| {
                let response = result.unwrap();
                assert_eq!(
                    response.pools[0].total_share,
                    Coin {
                        denom: response.pools[0].pool_info.lp_denom.clone(),
                        amount: ONE_MILLION_U128,
                    }
                );
            },
        );

    let simulated_return_amount = RefCell::new(Uint128::zero());
    suite.query_simulation(
        WHALE_ULUNA_POOL_ID.to_string(),
        Coin {
            denom: DENOM_UWHALE.to_string(),
            amount: ONE_THOUSAND_U128,
        },
        DENOM_ULUNA.to_string(),
        |result| {
            // Ensure that the return amount is 1_000 minus spread
            assert_eq!(
                result.as_ref().unwrap().return_amount + result.as_ref().unwrap().slippage_amount,
                ONE_THOUSAND_U128
            );
            *simulated_return_amount.borrow_mut() = result.unwrap().return_amount;
        },
    );

    // Now Let's try a swap
    suite.swap(
        &creator,
        DENOM_ULUNA.to_string(),
        None,
        None,
        None,
        WHALE_ULUNA_POOL_ID.to_string(),
        vec![coin(ONE_THOUSAND_U128.u128(), DENOM_UWHALE.to_string())],
        |result| {
            // Find the key with 'offer_amount' and the key with 'return_amount'
            // Ensure that the offer amount is 1000 and the return amount is greater than 0
            let mut return_amount = String::new();
            let mut offer_amount = String::new();

            for event in result.unwrap().events {
                if event.ty == EVENT_TYPE_WASM {
                    for attribute in event.attributes {
                        match attribute.key.as_str() {
                            s if s == EVENT_KEY_RETURN_AMOUNT => return_amount = attribute.value,
                            s if s == EVENT_KEY_OFFER_AMOUNT => offer_amount = attribute.value,
                            _ => {}
                        }
                    }
                }
            }
            // Because the Pool was created and 1_000_000 of each token has been provided as liquidity
            // Assuming no fees we should expect a small swap of 1000 to result in not too much slippage
            // Expect 1000 give or take 0.002 difference
            // Once fees are added and being deducted properly only the "0.002" should be changed.
            assert_approx_eq!(
                offer_amount.parse::<u128>().unwrap(),
                return_amount.parse::<u128>().unwrap(),
                BASIC_SWAP_ASSERT_APPROX_TOLERANCE_A
            );
            assert_approx_eq!(
                simulated_return_amount.borrow().u128(),
                return_amount.parse::<u128>().unwrap(),
                BASIC_SWAP_ASSERT_APPROX_TOLERANCE_A
            );
        },
    );

    let simulated_offer_amount = RefCell::new(Uint128::zero());
    suite.query_reverse_simulation(
        WHALE_ULUNA_POOL_ID.to_string(),
        Coin {
            denom: DENOM_UWHALE.to_string(),
            amount: ONE_THOUSAND_U128,
        },
        DENOM_ULUNA.to_string(),
        |result| {
            *simulated_offer_amount.borrow_mut() = result.unwrap().offer_amount;
        },
    );
    // Another swap but this time the other way around
    // Now Let's try a swap
    suite.swap(
        &creator,
        DENOM_UWHALE.to_string(),
        None,
        None,
        None,
        WHALE_ULUNA_POOL_ID.to_string(),
        vec![coin(
            simulated_offer_amount.borrow().u128(),
            DENOM_ULUNA.to_string(),
        )],
        |result| {
            // Find the key with 'offer_amount' and the key with 'return_amount'
            // Ensure that the offer amount is 1000 and the return amount is greater than 0
            let mut return_amount = String::new();
            let mut offer_amount = String::new();

            for event in result.unwrap().events {
                if event.ty == EVENT_TYPE_WASM {
                    for attribute in event.attributes {
                        match attribute.key.as_str() {
                            s if s == EVENT_KEY_RETURN_AMOUNT => return_amount = attribute.value,
                            s if s == EVENT_KEY_OFFER_AMOUNT => offer_amount = attribute.value,
                            _ => {}
                        }
                    }
                }
            }
            assert_approx_eq!(
                simulated_offer_amount.borrow().u128(),
                offer_amount.parse::<u128>().unwrap(),
                BASIC_SWAP_ASSERT_APPROX_TOLERANCE_A
            );

            assert_approx_eq!(
                ONE_THOUSAND_U128.u128(),
                return_amount.parse::<u128>().unwrap(),
                BASIC_SWAP_ASSERT_APPROX_TOLERANCE_B
            );
        },
    );
}

#[test]
fn basic_swapping_pool_reserves_event_test() {
    let mut suite = TestingSuite::default_with_balances(
        vec![
            coin(1_000_000_001u128, DENOM_UWHALE.to_string()),
            coin(1_000_000_000u128, DENOM_ULUNA.to_string()),
            coin(1_000_000_001u128, DENOM_UUSD.to_string()),
            coin(1_000_000_001u128, DENOM_UOM.to_string()),
        ],
        StargateMock::new(vec![coin(
            EIGHT_EIGHT_EIGHT_EIGHT_U128.u128(),
            DENOM_UOM.to_string(),
        )]),
    );
    let creator = suite.creator();
    // Asset infos with uwhale and uluna

    // Protocol fee is 0.01% and swap fee is 0.02% and burn fee is 0%
    let pool_fees = PoolFee {
        protocol_fee: Fee {
            share: *DEFAULT_PROTOCOL_FEE_SHARE,
        },
        swap_fee: Fee {
            share: *DEFAULT_SWAP_FEE_SHARE,
        },
        burn_fee: Fee {
            share: *DECIMAL_ZERO,
        },
        extra_fees: vec![],
    };

    // Create a pool
    suite
        .instantiate_default()
        .add_one_epoch()
        .create_pool(
            &creator,
            vec![DENOM_UOM.to_string(), DENOM_UUSD.to_string()],
            vec![DECIMALS_6, DECIMALS_6],
            pool_fees.clone(),
            PoolType::ConstantProduct,
            Some("uom.uusd".to_string()),
            vec![
                coin(ONE_THOUSAND_U128.u128(), DENOM_UUSD.to_string()),
                coin(EIGHT_EIGHT_EIGHT_EIGHT_U128.u128(), DENOM_UOM.to_string()),
            ],
            |result| {
                result.unwrap();
            },
        )
        .create_pool(
            &creator,
            vec![DENOM_ULUNA.to_string(), DENOM_UUSD.to_string()],
            vec![DECIMALS_6, DECIMALS_6],
            pool_fees,
            PoolType::ConstantProduct,
            Some(ULUNA_UUSD_POOL_RAW.to_string()),
            vec![
                coin(ONE_THOUSAND_U128.u128(), DENOM_UUSD.to_string()),
                coin(EIGHT_EIGHT_EIGHT_EIGHT_U128.u128(), DENOM_UOM.to_string()),
            ],
            |result| {
                result.unwrap();
            },
        )
        .provide_liquidity(
            &creator,
            UOM_UUSD_POOL_ID.to_string(),
            None,
            None,
            None,
            None,
            None,
            vec![
                Coin {
                    denom: DENOM_UOM.to_string(),
                    amount: Uint128::new(1_000_000u128),
                },
                Coin {
                    denom: DENOM_UUSD.to_string(),
                    amount: Uint128::new(6_000_000u128),
                },
            ],
            |result| {
                result.unwrap();
            },
        )
        .provide_liquidity(
            &creator,
            ULUNA_UUSD_POOL_ID.to_string(),
            None,
            None,
            None,
            None,
            None,
            vec![
                Coin {
                    denom: DENOM_ULUNA.to_string(),
                    amount: ULUNA_LIQUIDITY_RESERVES,
                },
                Coin {
                    denom: DENOM_UUSD.to_string(),
                    amount: UUSD_LIQUIDITY_RESERVES_ULUNA_POOL,
                },
            ],
            |result| {
                result.unwrap();
            },
        );

    let expected_pool_reserves = RefCell::<Vec<Vec<Coin>>>::new(vec![]);

    // Now Let's try a swap
    suite
        .swap(
            &creator,
            DENOM_UOM.to_string(),
            None,
            None,
            None,
            UOM_UUSD_POOL_ID.to_string(),
            vec![coin(1000u128, DENOM_UUSD.to_string())],
            |result| {
                for event in result.unwrap().events {
                    if event.ty == EVENT_TYPE_WASM {
                        for attribute in event.attributes {
                            match attribute.key.as_str() {
                                s if s == POOL_RESERVES_KEY_POOL_RESERVES => {
                                    expected_pool_reserves.borrow_mut().clear();
                                    extract_pool_reserves(&attribute, &expected_pool_reserves);
                                    println!(
                                        "pool_reserves: {:?}",
                                        expected_pool_reserves.borrow()
                                    );
                                }
                                s if s == POOL_RESERVES_KEY_POOL_IDENTIFIER => {
                                    assert_eq!(attribute.value, UOM_UUSD_POOL_ID);
                                }
                                _ => {}
                            }
                        }
                    }
                }
            },
        )
        .query_pools(Some(UOM_UUSD_POOL_ID.to_string()), None, None, |result| {
            let response = result.unwrap();
            let mut assets = response.pools[0].pool_info.assets.clone();
            assets.sort_by(|a, b| a.denom.cmp(&b.denom));
            assert_eq!(assets, expected_pool_reserves.borrow()[0]);
        })
        .query_pools(Some(ULUNA_UUSD_POOL_ID.to_string()), None, None, |result| {
            let response = result.unwrap();
            let mut assets = response.pools[0].pool_info.assets.clone();
            assets.sort_by(|a, b| a.denom.cmp(&b.denom));
            assert_eq!(
                assets,
                vec![
                    coin(ULUNA_LIQUIDITY_RESERVES.u128(), DENOM_ULUNA.to_string()),
                    coin(
                        UUSD_LIQUIDITY_RESERVES_ULUNA_POOL.u128(),
                        DENOM_UUSD.to_string()
                    )
                ]
            );
        });

    // now a swap via the router, single and multiswap
    let swap_operations_single = vec![mantra_dex_std::pool_manager::SwapOperation::MantraSwap {
        token_in_denom: DENOM_UOM.to_string(),
        token_out_denom: DENOM_UUSD.to_string(),
        pool_identifier: UOM_UUSD_POOL_ID.to_string(),
    }];

    suite
        .execute_swap_operations(
            &creator,
            swap_operations_single,
            None,
            None,
            Some(POOL_RESERVES_DECIMAL_PERCENT_10.clone()),
            vec![coin(1_000u128, DENOM_UOM.to_string())],
            |result| {
                for event in result.unwrap().events {
                    if event.ty == EVENT_TYPE_WASM {
                        for attribute in event.attributes {
                            match attribute.key.as_str() {
                                s if s == POOL_RESERVES_KEY_POOL_RESERVES => {
                                    expected_pool_reserves.borrow_mut().clear();
                                    extract_pool_reserves(&attribute, &expected_pool_reserves);
                                }
                                s if s == POOL_RESERVES_KEY_POOL_IDENTIFIER => {
                                    assert_eq!(attribute.value, UOM_UUSD_POOL_ID);
                                }
                                _ => {}
                            }
                        }
                    }
                }
            },
        )
        .query_pools(Some(UOM_UUSD_POOL_ID.to_string()), None, None, |result| {
            let response = result.unwrap();
            let mut assets = response.pools[0].pool_info.assets.clone();
            assets.sort_by(|a, b| a.denom.cmp(&b.denom));
            assert_eq!(assets, expected_pool_reserves.borrow()[0]);
        })
        .query_pools(Some(ULUNA_UUSD_POOL_ID.to_string()), None, None, |result| {
            let response = result.unwrap();
            let mut assets = response.pools[0].pool_info.assets.clone();
            assets.sort_by(|a, b| a.denom.cmp(&b.denom));
            assert_eq!(
                assets,
                vec![
                    coin(ULUNA_LIQUIDITY_RESERVES.u128(), DENOM_ULUNA.to_string()),
                    coin(
                        UUSD_LIQUIDITY_RESERVES_ULUNA_POOL.u128(),
                        DENOM_UUSD.to_string()
                    )
                ]
            );
        });

    let swap_operations_multi = vec![
        mantra_dex_std::pool_manager::SwapOperation::MantraSwap {
            token_in_denom: DENOM_UOM.to_string(),
            token_out_denom: DENOM_UUSD.to_string(),
            pool_identifier: UOM_UUSD_POOL_ID.to_string(),
        },
        mantra_dex_std::pool_manager::SwapOperation::MantraSwap {
            token_in_denom: DENOM_UUSD.to_string(),
            token_out_denom: DENOM_ULUNA.to_string(),
            pool_identifier: ULUNA_UUSD_POOL_ID.to_string(),
        },
    ];

    let expected_pool_reserves_multi = RefCell::<Vec<Vec<Coin>>>::new(vec![]); // Renamed to avoid conflict

    suite
        .execute_swap_operations(
            &creator,
            swap_operations_multi,
            None,
            None,
            Some(POOL_RESERVES_DECIMAL_PERCENT_10.clone()),
            vec![coin(1_000u128, DENOM_UOM.to_string())],
            |result| {
                let mut pool_identifiers = vec![];

                expected_pool_reserves_multi.borrow_mut().clear();
                for event in result.unwrap().events {
                    if event.ty == EVENT_TYPE_WASM {
                        for attribute in event.attributes {
                            match attribute.key.as_str() {
                                s if s == POOL_RESERVES_KEY_POOL_RESERVES => {
                                    extract_pool_reserves(
                                        &attribute,
                                        &expected_pool_reserves_multi,
                                    );
                                }
                                s if s == POOL_RESERVES_KEY_POOL_IDENTIFIER => {
                                    pool_identifiers.push(attribute.value.clone());
                                }
                                _ => {}
                            }
                        }
                    }
                }
                assert_eq!(pool_identifiers, vec![UOM_UUSD_POOL_ID, ULUNA_UUSD_POOL_ID]);
            },
        )
        .query_pools(Some(UOM_UUSD_POOL_ID.to_string()), None, None, |result| {
            let response = result.unwrap();
            let mut assets = response.pools[0].pool_info.assets.clone();
            assets.sort_by(|a, b| a.denom.cmp(&b.denom));
            assert_eq!(assets, expected_pool_reserves_multi.borrow()[0]);
        })
        .query_pools(Some(ULUNA_UUSD_POOL_ID.to_string()), None, None, |result| {
            let response = result.unwrap();
            let mut assets = response.pools[0].pool_info.assets.clone();
            assets.sort_by(|a, b| a.denom.cmp(&b.denom));
            assert_eq!(assets, expected_pool_reserves_multi.borrow()[1]);
        });
}

#[test]
fn basic_swapping_test_stable_swap_two_assets() {
    let mut suite = TestingSuite::default_with_balances(
        vec![
            coin(1_000_000_001u128, DENOM_UWHALE.to_string()),
            coin(1_000_000_000u128, DENOM_ULUNA.to_string()),
            coin(1_000_000_001u128, DENOM_UUSD.to_string()),
            coin(1_000_000_001u128, DENOM_UOM.to_string()),
        ],
        StargateMock::new(vec![coin(
            EIGHT_EIGHT_EIGHT_EIGHT_U128.u128(),
            DENOM_UOM.to_string(),
        )]),
    );
    let creator = suite.creator();
    let _other = suite.senders[1].clone();
    let _unauthorized = suite.senders[2].clone();
    // Asset infos with uwhale and uluna

    let asset_infos = vec![DENOM_UWHALE.to_string(), DENOM_ULUNA.to_string()];

    // Protocol fee is 0.01% and swap fee is 0.02% and burn fee is 0%
    let pool_fees = PoolFee {
        protocol_fee: Fee {
            share: Decimal::from_ratio(1u128, 1000u128),
        },
        swap_fee: Fee {
            share: Decimal::from_ratio(1u128, 10_000u128),
        },
        burn_fee: Fee {
            share: *DECIMAL_ZERO,
        },
        extra_fees: vec![],
    };

    // Create a stableswap pool with amp = 100
    suite.instantiate_default().add_one_epoch().create_pool(
        &creator,
        asset_infos,
        vec![DECIMALS_6, DECIMALS_6],
        pool_fees,
        PoolType::StableSwap {
            amp: STABLE_SWAP_AMP,
        },
        Some(WHALE_ULUNA_POOL_RAW.to_string()),
        vec![
            coin(ONE_THOUSAND_U128.u128(), DENOM_UUSD.to_string()),
            coin(EIGHT_EIGHT_EIGHT_EIGHT_U128.u128(), DENOM_UOM.to_string()),
        ],
        |result| {
            result.unwrap();
        },
    );

    // Let's try to add liquidity
    suite.provide_liquidity(
        &creator,
        WHALE_ULUNA_POOL_ID.to_string(),
        None,
        None,
        None,
        None,
        None,
        vec![
            Coin {
                denom: DENOM_UWHALE.to_string(),
                amount: STABLE_SWAP_TWO_ASSETS_LIQUIDITY_AMOUNT,
            },
            Coin {
                denom: DENOM_ULUNA.to_string(),
                amount: STABLE_SWAP_TWO_ASSETS_LIQUIDITY_AMOUNT,
            },
        ],
        |result| {
            // Ensure we got 999000 in the response which is 1mil less the initial liquidity amount
            for event in result.unwrap().events {
                println!("{:?}", event);
            }
        },
    );
    let simulated_return_amount = RefCell::new(Uint128::zero());
    suite.query_simulation(
        WHALE_ULUNA_POOL_ID.to_string(),
        Coin {
            denom: DENOM_UWHALE.to_string(),
            amount: ONE_THOUSAND_U128,
        },
        DENOM_ULUNA.to_string(),
        |result| {
            *simulated_return_amount.borrow_mut() = result.unwrap().return_amount;
        },
    );

    // Now Let's try a swap
    suite.swap(
        &creator,
        DENOM_ULUNA.to_string(),
        None,
        None,
        None,
        WHALE_ULUNA_POOL_ID.to_string(),
        vec![coin(ONE_THOUSAND_U128.u128(), DENOM_UWHALE.to_string())],
        |result| {
            // Find the key with 'offer_amount' and the key with 'return_amount'
            // Ensure that the offer amount is 1000 and the return amount is greater than 0
            let mut return_amount = String::new();
            let mut offer_amount = String::new();

            for event in result.unwrap().events {
                if event.ty == EVENT_TYPE_WASM {
                    for attribute in event.attributes {
                        match attribute.key.as_str() {
                            s if s == EVENT_KEY_RETURN_AMOUNT => return_amount = attribute.value,
                            s if s == EVENT_KEY_OFFER_AMOUNT => offer_amount = attribute.value,
                            _ => {}
                        }
                    }
                }
            }
            // Because the Pool was created and 1_000_000 of each token has been provided as liquidity
            // Assuming no fees we should expect a small swap of 1000 to result in not too much slippage
            // Expect 1000 give or take 0.002 difference
            // Once fees are added and being deducted properly only the "0.002" should be changed.
            assert_approx_eq!(
                offer_amount.parse::<u128>().unwrap(),
                return_amount.parse::<u128>().unwrap(),
                BASIC_SWAP_ASSERT_APPROX_TOLERANCE_A
            );
            assert_approx_eq!(
                simulated_return_amount.borrow().u128(),
                return_amount.parse::<u128>().unwrap(),
                BASIC_SWAP_ASSERT_APPROX_TOLERANCE_A
            );
        },
    );

    let simulated_offer_amount = RefCell::new(Uint128::zero());
    suite.query_reverse_simulation(
        WHALE_ULUNA_POOL_ID.to_string(),
        Coin {
            denom: DENOM_UWHALE.to_string(),
            amount: ONE_THOUSAND_U128,
        },
        DENOM_ULUNA.to_string(),
        |result| {
            *simulated_offer_amount.borrow_mut() = result.unwrap().offer_amount;
        },
    );
    // Another swap but this time the other way around
    // Now Let's try a swap
    suite.swap(
        &creator,
        DENOM_UWHALE.to_string(),
        None,
        None,
        None,
        WHALE_ULUNA_POOL_ID.to_string(),
        vec![coin(
            simulated_offer_amount.borrow().u128(),
            DENOM_ULUNA.to_string(),
        )],
        |result| {
            // Find the key with 'offer_amount' and the key with 'return_amount'
            // Ensure that the offer amount is 1000 and the return amount is greater than 0
            let mut return_amount = String::new();
            let mut offer_amount = String::new();

            for event in result.unwrap().events {
                if event.ty == EVENT_TYPE_WASM {
                    for attribute in event.attributes {
                        match attribute.key.as_str() {
                            s if s == EVENT_KEY_RETURN_AMOUNT => return_amount = attribute.value,
                            s if s == EVENT_KEY_OFFER_AMOUNT => offer_amount = attribute.value,
                            _ => {}
                        }
                    }
                }
            }
            assert_approx_eq!(
                simulated_offer_amount.borrow().u128(),
                offer_amount.parse::<u128>().unwrap(),
                BASIC_SWAP_ASSERT_APPROX_TOLERANCE_A
            );

            assert_approx_eq!(
                ONE_THOUSAND_U128.u128(),
                return_amount.parse::<u128>().unwrap(),
                BASIC_SWAP_ASSERT_APPROX_TOLERANCE_B
            );
        },
    );
}

#[test]
fn swap_with_fees() {
    let mut suite = TestingSuite::default_with_balances(
        vec![
            coin(1_000_000_000_001u128, DENOM_UWHALE.to_string()),
            coin(1_000_000_000_000u128, DENOM_ULUNA.to_string()),
            coin(1_000_000_000_001u128, DENOM_UUSD.to_string()),
            coin(1_000_000_000_001u128, DENOM_UOM.to_string()),
        ],
        StargateMock::new(vec![coin(
            EIGHT_EIGHT_EIGHT_EIGHT_U128.u128(),
            DENOM_UOM.to_string(),
        )]),
    );
    let creator = suite.creator();
    let _other = suite.senders[1].clone();
    let _unauthorized = suite.senders[2].clone();
    // Asset infos with uwhale and uluna

    let asset_infos = vec![DENOM_UWHALE.to_string(), DENOM_ULUNA.to_string()];

    // Protocol fee is 0.001% and swap fee is 0.002% and burn fee is 0%
    let pool_fees = PoolFee {
        protocol_fee: Fee {
            share: *DEFAULT_PROTOCOL_FEE_SHARE, // Reused as it's 1/100_000
        },
        swap_fee: Fee {
            share: Decimal::from_ratio(2u128, 100_000u128),
        },
        burn_fee: Fee {
            share: *DECIMAL_ZERO,
        },
        extra_fees: vec![],
    };

    // Create a pool
    suite.instantiate_default().add_one_epoch().create_pool(
        &creator,
        asset_infos,
        vec![DECIMALS_6, DECIMALS_6],
        pool_fees,
        PoolType::ConstantProduct,
        Some(WHALE_ULUNA_POOL_RAW.to_string()),
        vec![
            coin(ONE_THOUSAND_U128.u128(), DENOM_UUSD.to_string()),
            coin(EIGHT_EIGHT_EIGHT_EIGHT_U128.u128(), DENOM_UOM.to_string()),
        ],
        |result| {
            result.unwrap();
        },
    );

    // Let's try to add liquidity, 1000 of each token.
    suite.provide_liquidity(
        &creator,
        WHALE_ULUNA_POOL_ID.to_string(),
        None,
        None,
        None,
        None,
        None,
        vec![
            Coin {
                denom: DENOM_UWHALE.to_string(),
                amount: SWAP_WITH_FEES_LIQUIDITY_AMOUNT,
            },
            Coin {
                denom: DENOM_ULUNA.to_string(),
                amount: SWAP_WITH_FEES_LIQUIDITY_AMOUNT,
            },
        ],
        |result| {
            // Ensure we got 999000 in the response which is 1mil less the initial liquidity amount
            for event in result.unwrap().events {
                println!("{:?}", event);
            }
        },
    );

    // Now Let's try a swap, max spread is set to 1%
    // With 1000 of each token and a swap of 10 WHALE
    // We should expect a return of 9900792 of ULUNA
    // Applying Fees on the swap:
    //    - Protocol Fee: 0.001% on uLUNA -> 99.
    //    - Swap Fee: 0.002% on uLUNA -> 198.
    // Total Fees: 297 uLUNA

    // Spread Amount: 99,010 uLUNA.
    // Swap Fee Amount: 198 uLUNA.
    // Protocol Fee Amount: 99 uLUNA.
    // Burn Fee Amount: 0 uLUNA (as expected since burn fee is set to 0%).
    // Total -> 9,900,693 (Returned Amount) + 99,010 (Spread)(0.009x%) + 198 (Swap Fee) + 99 (Protocol Fee) = 10,000,000 uLUNA
    suite.swap(
        &creator,
        DENOM_ULUNA.to_string(),
        None,
        Some(Decimal::percent(1)),
        None,
        WHALE_ULUNA_POOL_ID.to_string(),
        vec![coin(OFFER_10M_UWHALE_U128.u128(), DENOM_UWHALE.to_string())],
        |result| {
            // Find the key with 'offer_amount' and the key with 'return_amount'
            // Ensure that the offer amount is 1000 and the return amount is greater than 0
            let mut return_amount = String::new();
            let mut offer_amount = String::new();

            for event in result.unwrap().events {
                if event.ty == EVENT_TYPE_WASM {
                    for attribute in event.attributes {
                        match attribute.key.as_str() {
                            s if s == EVENT_KEY_RETURN_AMOUNT => return_amount = attribute.value,
                            s if s == EVENT_KEY_OFFER_AMOUNT => offer_amount = attribute.value,
                            _ => {}
                        }
                    }
                }
            }
            // Because the Pool was created and 1_000_000 of each token has been provided as liquidity
            // Assuming no fees we should expect a small swap of 1000 to result in not too much slippage
            // Expect 1000 give or take 0.002 difference
            // Once fees are added and being deducted properly only the "0.002" should be changed.
            assert_approx_eq!(
                offer_amount.parse::<u128>().unwrap(),
                return_amount.parse::<u128>().unwrap(),
                "0.01"
            );
        },
    );

    // Verify fee collection by querying the address of the fee collector and checking its balance
    // Should be 99 uLUNA
    suite.query_balance(
        &suite.fee_collector_addr.to_string(),
        DENOM_ULUNA.to_string(),
        |result| {
            assert_eq!(result.unwrap().amount, Uint128::new(99u128));
        },
    );
}

#[allow(clippy::inconsistent_digit_grouping)]
#[test]
fn swap_large_digits_xyk() {
    let mut suite = TestingSuite::default_with_balances(
        vec![
            coin(
                SWAP_LARGE_DIGITS_XYK_DEFAULT_BALANCE.u128(),
                DENOM_UWHALE.to_string(),
            ),
            coin(
                SWAP_LARGE_DIGITS_XYK_DEFAULT_BALANCE.u128(),
                DENOM_ULUNA.to_string(),
            ),
            coin(
                SWAP_LARGE_DIGITS_XYK_DEFAULT_BALANCE.u128(),
                DENOM_UOSMO.to_string(),
            ),
            coin(
                SWAP_LARGE_DIGITS_XYK_DEFAULT_BALANCE.u128(),
                DENOM_UUSD.to_string(),
            ),
            coin(100_000_000_000_000_000000u128, DENOM_UUSDC.to_string()),
            coin(BALANCE_AUSDY_100Q_XYK.u128(), AUSDY_DENOM.to_string()),
            coin(BALANCE_UOM_150T_XYK.u128(), DENOM_UOM.to_string()),
        ],
        StargateMock::new(vec![coin(
            EIGHT_EIGHT_EIGHT_EIGHT_U128.u128(),
            DENOM_UOM.to_string(),
        )]),
    );
    let alice = suite.creator();
    let bob = suite.senders[1].clone();
    let carol = suite.senders[2].clone();
    let dan = suite.senders[3].clone();

    let asset_denoms = vec![DENOM_UOM.to_string(), AUSDY_DENOM.to_string()];

    let pool_fees = PoolFee {
        protocol_fee: Fee {
            share: Decimal::zero(),
        },
        swap_fee: Fee {
            share: Decimal::permille(30),
        },
        burn_fee: Fee {
            share: *DECIMAL_ZERO,
        },
        extra_fees: vec![],
    };

    // Create a pool
    suite.instantiate_default().add_one_epoch().create_pool(
        &alice,
        asset_denoms,
        vec![DECIMALS_6, DECIMALS_18],
        pool_fees,
        PoolType::ConstantProduct,
        None,
        vec![
            coin(ONE_THOUSAND_U128.u128(), DENOM_UUSD.to_string()),
            coin(EIGHT_EIGHT_EIGHT_EIGHT_U128.u128(), DENOM_UOM.to_string()),
        ],
        |result| {
            result.unwrap();
        },
    );

    let contract_addr = suite.pool_manager_addr.clone();
    let lp_denom = suite.get_lp_denom(P1_POOL_ID.to_string());

    // let's provide liquidity 150T om, 100T usdy
    suite
        .provide_liquidity(
            &bob,
            P1_POOL_ID.to_string(),
            None,
            None,
            None,
            None,
            None,
            vec![
                Coin {
                    denom: DENOM_UOM.to_string(),
                    amount: BALANCE_UOM_150T_XYK,
                },
                Coin {
                    denom: AUSDY_DENOM.to_string(),
                    amount: BALANCE_AUSDY_100Q_XYK,
                },
            ],
            |result| {
                result.unwrap();
            },
        )
        .query_all_balances(&contract_addr.to_string(), |result| {
            let balances = result.unwrap();
            assert!(balances.iter().any(|coin| {
                coin.denom == lp_denom.clone() && coin.amount == MINIMUM_LIQUIDITY_AMOUNT
            }));
        })
        .query_all_balances(&bob.to_string(), |result| {
            let balances = result.unwrap();
            assert!(balances.iter().any(|coin| {
                coin.denom == lp_denom.clone()
                    && coin.amount == Uint128::new(122_474_487_139_158_904_909_863_203u128)
            }));
        });

    // swap 2T usdy for om
    suite
        .query_balance(&carol.to_string(), AUSDY_DENOM.to_string(), |result| {
            assert_eq!(result.unwrap().amount, BALANCE_AUSDY_100Q_XYK);
        })
        .query_balance(&carol.to_string(), DENOM_UOM.to_string(), |result| {
            assert_eq!(result.unwrap().amount, BALANCE_UOM_150T_XYK);
        })
        .query_simulation(
            P1_POOL_ID.to_string(),
            Coin {
                denom: AUSDY_DENOM.to_string(),
                amount: SWAP_2Q_AUSDY_XYK,
            },
            DENOM_UOM.to_string(),
            |result| {
                assert_eq!(
                    result.unwrap().return_amount,
                    SIMULATED_RETURN_2_852T_UOM_XYK
                );
            },
        )
        .swap(
            &carol,
            DENOM_UOM.to_string(),
            None,
            Some(Decimal::percent(5)),
            None,
            P1_POOL_ID.to_string(),
            vec![coin(SWAP_2Q_AUSDY_XYK.u128(), AUSDY_DENOM.to_string())],
            |result| {
                result.unwrap();
            },
        )
        .query_balance(&carol.to_string(), AUSDY_DENOM.to_string(), |result| {
            assert_eq!(
                result.unwrap().amount,
                BALANCE_AUSDY_100Q_XYK - SWAP_2Q_AUSDY_XYK
            );
        })
        .query_balance(&carol.to_string(), DENOM_UOM.to_string(), |result| {
            assert_eq!(
                result.unwrap().amount,
                BALANCE_UOM_150T_XYK + SIMULATED_RETURN_2_852T_UOM_XYK
            );
        });

    // swap 10T om for usdy
    suite
        .query_balance(&dan.to_string(), AUSDY_DENOM.to_string(), |result| {
            assert_eq!(result.unwrap().amount, BALANCE_AUSDY_100Q_XYK);
        })
        .query_balance(&dan.to_string(), DENOM_UOM.to_string(), |result| {
            assert_eq!(result.unwrap().amount, BALANCE_UOM_150T_XYK);
        })
        .query_simulation(
            P1_POOL_ID.to_string(),
            Coin {
                denom: DENOM_UOM.to_string(),
                amount: SWAP_10T_UOM_XYK,
            },
            AUSDY_DENOM.to_string(),
            |result| {
                assert_eq!(
                    result.unwrap().return_amount,
                    SIMULATED_RETURN_6_296Q_AUSDY_XYK
                );
            },
        )
        .swap(
            &dan,
            AUSDY_DENOM.to_string(),
            None,
            Some(Decimal::percent(20)),
            None,
            P1_POOL_ID.to_string(),
            vec![coin(SWAP_10T_UOM_XYK.u128(), DENOM_UOM.to_string())],
            |result| {
                result.unwrap();
            },
        )
        .query_balance(&dan.to_string(), AUSDY_DENOM.to_string(), |result| {
            assert_eq!(
                result.unwrap().amount,
                BALANCE_AUSDY_100Q_XYK + SIMULATED_RETURN_6_296Q_AUSDY_XYK
            );
        })
        .query_balance(&dan.to_string(), DENOM_UOM.to_string(), |result| {
            assert_eq!(
                result.unwrap().amount,
                BALANCE_UOM_150T_XYK - SWAP_10T_UOM_XYK
            );
        });
}

#[allow(clippy::inconsistent_digit_grouping)]
#[test]
fn swap_large_digits_stable() {
    let mut suite = TestingSuite::default_with_balances(
        vec![
            coin(BALANCE_1T_U128.u128(), DENOM_UWHALE.to_string()),
            coin(BALANCE_1T_U128.u128(), DENOM_ULUNA.to_string()),
            coin(BALANCE_1T_U128.u128(), DENOM_UOSMO.to_string()),
            coin(BALANCE_1T_U128.u128(), DENOM_UUSD.to_string()),
            coin(BALANCE_UUSDC_100T_STABLE.u128(), DENOM_UUSDC.to_string()),
            coin(BALANCE_AUSDY_100Q_STABLE.u128(), AUSDY_DENOM.to_string()),
            coin(150_000_000_000_000_000000u128, DENOM_UOM.to_string()),
        ],
        StargateMock::new(vec![coin(
            EIGHT_EIGHT_EIGHT_EIGHT_U128.u128(),
            DENOM_UOM.to_string(),
        )]),
    );
    let alice = suite.creator();
    let bob = suite.senders[1].clone();
    let carol = suite.senders[2].clone();
    let dan = suite.senders[3].clone();

    let asset_denoms = vec![AUSDY_DENOM.to_string(), DENOM_UUSDC.to_string()];

    let pool_fees = PoolFee {
        protocol_fee: Fee {
            share: Decimal::zero(),
        },
        swap_fee: Fee {
            share: *SWAP_FEE_PERMILLE_5,
        },
        burn_fee: Fee {
            share: *DECIMAL_ZERO,
        },
        extra_fees: vec![],
    };

    // Create a pool
    suite.instantiate_default().add_one_epoch().create_pool(
        &alice,
        asset_denoms,
        vec![DECIMALS_18, DECIMALS_6],
        pool_fees,
        PoolType::ConstantProduct, // Note: Test name says stable, but uses ConstantProduct here.
        None,
        vec![
            coin(ONE_THOUSAND_U128.u128(), DENOM_UUSD.to_string()),
            coin(EIGHT_EIGHT_EIGHT_EIGHT_U128.u128(), DENOM_UOM.to_string()),
        ],
        |result| {
            result.unwrap();
        },
    );

    // let's provide liquidity 200T usdc, 200T usdy
    suite
        .provide_liquidity(
            &alice,
            P1_POOL_ID.to_string(),
            None,
            None,
            None,
            None,
            None,
            vec![
                Coin {
                    denom: DENOM_UUSDC.to_string(),
                    amount: LIQUIDITY_UUSDC_100T_STABLE,
                },
                Coin {
                    denom: AUSDY_DENOM.to_string(),
                    amount: LIQUIDITY_AUSDY_100Q_STABLE,
                },
            ],
            |result| {
                result.unwrap();
            },
        )
        .provide_liquidity(
            &bob,
            P1_POOL_ID.to_string(),
            None,
            None,
            None,
            None,
            None,
            vec![
                Coin {
                    denom: DENOM_UUSDC.to_string(),
                    amount: LIQUIDITY_UUSDC_100T_STABLE,
                },
                Coin {
                    denom: AUSDY_DENOM.to_string(),
                    amount: LIQUIDITY_AUSDY_100Q_STABLE,
                },
            ],
            |result| {
                result.unwrap();
            },
        );

    // swap 10T usdc for usdy
    suite
        .query_balance(&carol.to_string(), AUSDY_DENOM.to_string(), |result| {
            assert_eq!(result.unwrap().amount, BALANCE_AUSDY_100Q_STABLE);
        })
        .query_balance(&carol.to_string(), DENOM_UUSDC.to_string(), |result| {
            assert_eq!(result.unwrap().amount, BALANCE_UUSDC_100T_STABLE);
        })
        .query_simulation(
            P1_POOL_ID.to_string(),
            Coin {
                denom: DENOM_UUSDC.to_string(),
                amount: SWAP_10T_UUSDC_STABLE,
            },
            AUSDY_DENOM.to_string(),
            |result| {
                assert_eq!(
                    result.unwrap().return_amount,
                    SIMULATED_RETURN_9_476Q_AUSDY_STABLE
                );
            },
        )
        .swap(
            &carol,
            AUSDY_DENOM.to_string(),
            None,
            Some(Decimal::percent(6)),
            None,
            P1_POOL_ID.to_string(),
            vec![coin(SWAP_10T_UUSDC_STABLE.u128(), DENOM_UUSDC.to_string())],
            |result| {
                result.unwrap();
            },
        )
        .query_balance(&carol.to_string(), AUSDY_DENOM.to_string(), |result| {
            assert_eq!(
                result.unwrap().amount,
                BALANCE_AUSDY_100Q_STABLE + SIMULATED_RETURN_9_476Q_AUSDY_STABLE
            );
        })
        .query_balance(&carol.to_string(), DENOM_UUSDC.to_string(), |result| {
            assert_eq!(
                result.unwrap().amount,
                BALANCE_UUSDC_100T_STABLE - SWAP_10T_UUSDC_STABLE
            );
        });

    // swap 20T usdy for usdc
    suite
        .query_balance(&dan.to_string(), AUSDY_DENOM.to_string(), |result| {
            assert_eq!(result.unwrap().amount, BALANCE_AUSDY_100Q_STABLE);
        })
        .query_balance(&dan.to_string(), DENOM_UUSDC.to_string(), |result| {
            assert_eq!(result.unwrap().amount, BALANCE_UUSDC_100T_STABLE);
        })
        .query_simulation(
            P1_POOL_ID.to_string(),
            Coin {
                denom: AUSDY_DENOM.to_string(),
                amount: SWAP_20Q_AUSDY_STABLE,
            },
            DENOM_UUSDC.to_string(),
            |result| {
                assert_eq!(
                    result.unwrap().return_amount,
                    SIMULATED_RETURN_19_850T_UUSDC_STABLE
                );
            },
        )
        .swap(
            &dan,
            DENOM_UUSDC.to_string(),
            None,
            Some(Decimal::percent(10)),
            None,
            P1_POOL_ID.to_string(),
            vec![coin(SWAP_20Q_AUSDY_STABLE.u128(), AUSDY_DENOM.to_string())],
            |result| {
                result.unwrap();
            },
        )
        .query_balance(&dan.to_string(), AUSDY_DENOM.to_string(), |result| {
            assert_eq!(
                result.unwrap().amount,
                BALANCE_AUSDY_100Q_STABLE - SWAP_20Q_AUSDY_STABLE
            );
        })
        .query_balance(&dan.to_string(), DENOM_UUSDC.to_string(), |result| {
            assert_eq!(
                result.unwrap().amount,
                BALANCE_UUSDC_100T_STABLE + SIMULATED_RETURN_19_850T_UUSDC_STABLE
            );
        });
}

#[allow(clippy::inconsistent_digit_grouping)]
#[test]
fn swap_large_digits_stable_18_digits() {
    let mut suite = TestingSuite::default_with_balances(
        vec![
            coin(BALANCE_1T_U128.u128(), DENOM_UWHALE.to_string()),
            coin(BALANCE_1T_U128.u128(), DENOM_ULUNA.to_string()),
            coin(BALANCE_1T_U128.u128(), DENOM_UOSMO.to_string()),
            coin(BALANCE_1T_U128.u128(), DENOM_UUSD.to_string()),
            coin(BALANCE_1T_U128.u128(), DENOM_UUSDC.to_string()), // Assuming UUSDC also has a large balance for this test context
            coin(BALANCE_300T_HIGH_PREC_U128.u128(), AUSDY_DENOM.to_string()),
            coin(BALANCE_300T_HIGH_PREC_U128.u128(), PUSDC_DENOM.to_string()),
            coin(BALANCE_1T_U128.u128(), DENOM_UOM.to_string()),
        ],
        StargateMock::new(vec![coin(
            EIGHT_EIGHT_EIGHT_EIGHT_U128.u128(),
            DENOM_UOM.to_string(),
        )]),
    );
    let alice = suite.creator();
    let bob = suite.senders[1].clone();
    let carol = suite.senders[2].clone();

    let asset_denoms = vec![AUSDY_DENOM.to_string(), PUSDC_DENOM.to_string()];

    let pool_fees = PoolFee {
        protocol_fee: Fee {
            share: Decimal::zero(),
        },
        swap_fee: Fee {
            share: *SWAP_FEE_PERMILLE_5,
        },
        burn_fee: Fee {
            share: *DECIMAL_ZERO,
        },
        extra_fees: vec![],
    };

    // Create a pool
    suite.instantiate_default().add_one_epoch().create_pool(
        &alice,
        asset_denoms,
        vec![DECIMALS_18, DECIMALS_18],
        pool_fees,
        PoolType::ConstantProduct,
        None,
        vec![
            coin(ONE_THOUSAND_U128.u128(), DENOM_UUSD.to_string()),
            coin(EIGHT_EIGHT_EIGHT_EIGHT_U128.u128(), DENOM_UOM.to_string()),
        ],
        |result| {
            result.unwrap();
        },
    );

    // let's provide liquidity 300T pusdc, 300T usdy
    suite.provide_liquidity(
        &alice,
        P1_POOL_ID.to_string(),
        None,
        None,
        None,
        None,
        None,
        vec![
            Coin {
                denom: PUSDC_DENOM.to_string(),
                amount: BALANCE_300T_HIGH_PREC_U128,
            },
            Coin {
                denom: AUSDY_DENOM.to_string(),
                amount: BALANCE_300T_HIGH_PREC_U128,
            },
        ],
        |result| {
            result.unwrap();
        },
    );

    // swap 100T pusdc for usdy
    suite
        .query_balance(&bob.to_string(), AUSDY_DENOM.to_string(), |result| {
            assert_eq!(result.unwrap().amount, BALANCE_300T_HIGH_PREC_U128);
        })
        .query_balance(&bob.to_string(), PUSDC_DENOM.to_string(), |result| {
            assert_eq!(result.unwrap().amount, BALANCE_300T_HIGH_PREC_U128);
        })
        .query_simulation(
            P1_POOL_ID.to_string(),
            Coin {
                denom: PUSDC_DENOM.to_string(),
                amount: SWAP_100T_HIGH_PREC_U128,
            },
            AUSDY_DENOM.to_string(),
            |result| {
                assert_eq!(
                    result.unwrap().return_amount,
                    SIMULATED_RETURN_74_625T_HIGH_PREC_U128
                );
            },
        )
        .swap(
            &bob,
            AUSDY_DENOM.to_string(),
            None,
            Some(*DECIMAL_PERCENT_30),
            None,
            P1_POOL_ID.to_string(),
            vec![coin(
                SWAP_100T_HIGH_PREC_U128.u128(),
                PUSDC_DENOM.to_string(),
            )],
            |result| {
                result.unwrap();
            },
        )
        .query_balance(&bob.to_string(), AUSDY_DENOM.to_string(), |result| {
            assert_eq!(
                result.unwrap().amount,
                BALANCE_300T_HIGH_PREC_U128 + SIMULATED_RETURN_74_625T_HIGH_PREC_U128
            );
        })
        .query_balance(&bob.to_string(), PUSDC_DENOM.to_string(), |result| {
            assert_eq!(
                result.unwrap().amount,
                BALANCE_300T_HIGH_PREC_U128 - SWAP_100T_HIGH_PREC_U128
            );
        });

    // swap 50T usdy for pusdc
    suite
        .query_balance(&carol.to_string(), AUSDY_DENOM.to_string(), |result| {
            assert_eq!(result.unwrap().amount, BALANCE_300T_HIGH_PREC_U128);
        })
        .query_balance(&carol.to_string(), PUSDC_DENOM.to_string(), |result| {
            assert_eq!(result.unwrap().amount, BALANCE_300T_HIGH_PREC_U128);
        })
        .query_simulation(
            P1_POOL_ID.to_string(),
            Coin {
                denom: AUSDY_DENOM.to_string(),
                amount: SWAP_50T_HIGH_PREC_U128,
            },
            PUSDC_DENOM.to_string(),
            |result| {
                assert_eq!(
                    result.unwrap().return_amount,
                    SIMULATED_RETURN_72_265T_HIGH_PREC_U128
                );
            },
        )
        .swap(
            &carol,
            PUSDC_DENOM.to_string(),
            None,
            Some(Decimal::percent(20)),
            None,
            P1_POOL_ID.to_string(),
            vec![coin(
                SWAP_50T_HIGH_PREC_U128.u128(),
                AUSDY_DENOM.to_string(),
            )],
            |result| {
                result.unwrap();
            },
        )
        .query_balance(&carol.to_string(), AUSDY_DENOM.to_string(), |result| {
            assert_eq!(
                result.unwrap().amount,
                BALANCE_300T_HIGH_PREC_U128 - SWAP_50T_HIGH_PREC_U128
            );
        })
        .query_balance(&carol.to_string(), PUSDC_DENOM.to_string(), |result| {
            assert_eq!(
                result.unwrap().amount,
                BALANCE_300T_HIGH_PREC_U128 + SIMULATED_RETURN_72_265T_HIGH_PREC_U128
            );
        });
}

#[allow(clippy::inconsistent_digit_grouping)]
#[test]
fn swap_3pool_same_decimals() {
    let mut suite = TestingSuite::default_with_balances(
        vec![
            coin(BALANCE_300T_3POOL_U128.u128(), DENOM_UUSD.to_string()),
            coin(BALANCE_300T_3POOL_U128.u128(), DENOM_UUSDC.to_string()),
            coin(BALANCE_300T_3POOL_U128.u128(), DENOM_UUSDT.to_string()),
            coin(BALANCE_1T_U128.u128(), DENOM_UOM.to_string()),
        ],
        StargateMock::new(vec![coin(
            EIGHT_EIGHT_EIGHT_EIGHT_U128.u128(),
            DENOM_UOM.to_string(),
        )]),
    );
    let alice = suite.creator();
    let bob = suite.senders[1].clone();

    let asset_denoms = vec![
        DENOM_UUSD.to_string(),
        DENOM_UUSDC.to_string(),
        DENOM_UUSDT.to_string(),
    ];

    let pool_fees = PoolFee {
        protocol_fee: Fee {
            share: Decimal::zero(),
        },
        swap_fee: Fee {
            share: Decimal::zero(),
        },
        burn_fee: Fee {
            share: *DECIMAL_ZERO,
        },
        extra_fees: vec![],
    };

    // Create a pool
    suite.instantiate_default().add_one_epoch().create_pool(
        &alice,
        asset_denoms,
        vec![DECIMALS_6, DECIMALS_6, DECIMALS_6],
        pool_fees,
        PoolType::StableSwap { amp: 85 },
        None,
        vec![
            coin(ONE_THOUSAND_U128.u128(), DENOM_UUSD),
            coin(EIGHT_EIGHT_EIGHT_EIGHT_U128.u128(), DENOM_UOM),
        ],
        |result| {
            result.unwrap();
        },
    );

    // let's provide liquidity
    suite.provide_liquidity(
        &alice,
        P1_POOL_ID.to_string(),
        None,
        None,
        None,
        None,
        None,
        vec![
            Coin {
                denom: DENOM_UUSDC.to_string(),
                amount: LIQUIDITY_1B_U128,
            },
            Coin {
                denom: DENOM_UUSD.to_string(),
                amount: LIQUIDITY_1B_U128,
            },
            Coin {
                denom: DENOM_UUSDT.to_string(),
                amount: LIQUIDITY_1B_U128,
                // amount: Uint128::new(1_000_000_000_000_000_000_000u128),
            },
        ],
        |result| {
            result.unwrap();
        },
    );

    suite
        .query_balance(&bob.to_string(), DENOM_UUSDC.to_string(), |result| {
            assert_eq!(result.unwrap().amount, BALANCE_300T_3POOL_U128);
        })
        .query_balance(&bob.to_string(), DENOM_UUSDT.to_string(), |result| {
            assert_eq!(result.unwrap().amount, BALANCE_300T_3POOL_U128);
        })
        .query_simulation(
            P1_POOL_ID.to_string(),
            Coin {
                denom: DENOM_UUSDC.to_string(),
                amount: SWAP_OFFER_200M_UUSDC_U128,
            },
            DENOM_UUSDT.to_string(),
            |result| {
                assert_eq!(
                    result.unwrap().return_amount,
                    SIMULATED_RETURN_199_517_195_UUSDT_U128
                );
            },
        )
        .swap(
            &bob,
            DENOM_UUSDT.to_string(),
            None,
            Some(*DECIMAL_PERCENT_30),
            None,
            P1_POOL_ID.to_string(),
            vec![coin(
                SWAP_OFFER_200M_UUSDC_U128.u128(),
                DENOM_UUSDC.to_string(),
            )],
            |result| {
                result.unwrap();
            },
        )
        .query_balance(&bob.to_string(), DENOM_UUSDC.to_string(), |result| {
            assert_eq!(
                result.unwrap().amount,
                BALANCE_300T_3POOL_U128 - SWAP_OFFER_200M_UUSDC_U128
            );
        })
        .query_balance(&bob.to_string(), DENOM_UUSDT.to_string(), |result| {
            assert_eq!(
                result.unwrap().amount,
                BALANCE_300T_3POOL_U128 + SIMULATED_RETURN_199_517_195_UUSDT_U128
            );
        });
}

#[allow(clippy::inconsistent_digit_grouping)]
#[test]
fn swap_3pool_different_decimals() {
    let mut suite = setup_3pool_different_decimals(None, None, None, None);
    let bob = suite.senders[1].clone();

    let return_amount = RefCell::new(Uint128::zero());
    let mut current_swap_amount = Uint128::new(200_000000000000u128);

    let bob_uusdc_balance = RefCell::new(Uint128::zero());
    let bob_uusdt_balance = RefCell::new(Uint128::zero());
    let bob_uusd_balance = RefCell::new(Uint128::zero());

    suite
        .query_balance(&bob.to_string(), DENOM_UUSD.to_string(), |result| {
            bob_uusd_balance
                .borrow_mut()
                .clone_from(&result.unwrap().amount);
        })
        .query_balance(&bob.to_string(), DENOM_UUSDC.to_string(), |result| {
            bob_uusdc_balance
                .borrow_mut()
                .clone_from(&result.unwrap().amount);
        })
        .query_balance(&bob.to_string(), DENOM_UUSDT.to_string(), |result| {
            bob_uusdt_balance
                .borrow_mut()
                .clone_from(&result.unwrap().amount);
        });

    println!("****************");
    println!("uusdc -> uusdt");
    println!("swap_amount : {:?}", current_swap_amount);

    suite
        .query_simulation(
            P1_POOL_ID.to_string(),
            Coin {
                denom: DENOM_UUSDC.to_string(),
                amount: current_swap_amount,
            },
            DENOM_UUSDT.to_string(),
            |result| {
                let response: SimulationResponse = result.unwrap();
                println!("return_amount: {:?}", response.return_amount);

                return_amount
                    .borrow_mut()
                    .clone_from(&response.return_amount);
            },
        )
        .swap(
            &bob,
            DENOM_UUSDT.to_string(),
            None,
            Some(*MAX_SLIPPAGE_PERCENT_30_3POOL_DIFFERENT_DECIMALS),
            None,
            P1_POOL_ID.to_string(),
            vec![coin(current_swap_amount.u128(), DENOM_UUSDC.to_string())],
            |result| {
                result.unwrap();
            },
        )
        .query_balance(&bob.to_string(), DENOM_UUSDC.to_string(), |result| {
            let balance = result.unwrap().amount;
            assert_eq!(
                balance,
                Uint128::new(bob_uusdc_balance.borrow().u128() - current_swap_amount.u128())
            );
            bob_uusdc_balance.borrow_mut().clone_from(&balance);
        })
        .query_balance(&bob.to_string(), DENOM_UUSDT.to_string(), |result| {
            let balance = result.unwrap().amount;
            assert_eq!(
                balance,
                Uint128::new(bob_uusdt_balance.borrow().u128() + return_amount.borrow().u128())
            );
            bob_uusdt_balance.borrow_mut().clone_from(&balance);
        });

    println!("****************");
    println!("uusdt -> uusdc");
    current_swap_amount = SWAP_10K_ATTO_UUSDT_3POOL_DIFFERENT_DECIMALS;
    println!("swap_amount : {:?}", current_swap_amount);

    suite
        .query_simulation(
            P1_POOL_ID.to_string(),
            Coin {
                denom: DENOM_UUSDT.to_string(),
                amount: current_swap_amount,
            },
            DENOM_UUSDC.to_string(),
            |result| {
                let response: SimulationResponse = result.unwrap();
                println!("return_amount: {:?}", response.return_amount);

                return_amount
                    .borrow_mut()
                    .clone_from(&response.return_amount);
            },
        )
        .swap(
            &bob,
            DENOM_UUSDC.to_string(),
            None,
            Some(*MAX_SLIPPAGE_PERCENT_30_3POOL_DIFFERENT_DECIMALS),
            None,
            P1_POOL_ID.to_string(),
            vec![coin(current_swap_amount.u128(), DENOM_UUSDT.to_string())],
            |result| {
                result.unwrap();
            },
        )
        .query_balance(&bob.to_string(), DENOM_UUSDC.to_string(), |result| {
            let balance = result.unwrap().amount;
            assert_eq!(
                balance,
                Uint128::new(bob_uusdc_balance.borrow().u128() + return_amount.borrow().u128())
            );
            bob_uusdc_balance.borrow_mut().clone_from(&balance);
        })
        .query_balance(&bob.to_string(), DENOM_UUSDT.to_string(), |result| {
            let balance = result.unwrap().amount;
            assert_eq!(
                balance,
                Uint128::new(bob_uusdt_balance.borrow().u128() - current_swap_amount.u128())
            );
            bob_uusdt_balance.borrow_mut().clone_from(&balance);
        });

    println!("****************");
    println!("uusdt -> uusd");
    current_swap_amount = SWAP_10K_ATTO_UUSDT_3POOL_DIFFERENT_DECIMALS;
    println!("swap_amount : {:?}", current_swap_amount);

    suite
        .query_simulation(
            P1_POOL_ID.to_string(),
            Coin {
                denom: DENOM_UUSDT.to_string(),
                amount: current_swap_amount,
            },
            DENOM_UUSD.to_string(),
            |result| {
                let response: SimulationResponse = result.unwrap();
                println!("return_amount: {:?}", response.return_amount);

                return_amount
                    .borrow_mut()
                    .clone_from(&response.return_amount);
            },
        )
        .swap(
            &bob,
            DENOM_UUSD.to_string(),
            None,
            Some(*MAX_SLIPPAGE_PERCENT_30_3POOL_DIFFERENT_DECIMALS),
            None,
            P1_POOL_ID.to_string(),
            vec![coin(current_swap_amount.u128(), DENOM_UUSDT.to_string())],
            |result| {
                result.unwrap();
            },
        )
        .query_balance(&bob.to_string(), DENOM_UUSD.to_string(), |result| {
            let balance = result.unwrap().amount;
            assert_eq!(
                balance,
                Uint128::new(bob_uusd_balance.borrow().u128() + return_amount.borrow().u128())
            );
            bob_uusd_balance.borrow_mut().clone_from(&balance);
        })
        .query_balance(&bob.to_string(), DENOM_UUSDT.to_string(), |result| {
            let balance = result.unwrap().amount;
            assert_eq!(
                balance,
                Uint128::new(bob_uusdt_balance.borrow().u128() - current_swap_amount.u128())
            );
            bob_uusdt_balance.borrow_mut().clone_from(&balance);
        });
}

#[allow(clippy::inconsistent_digit_grouping)]
#[test]
fn swap_4pool_different_decimals() {
    let mut suite = setup_4pool_different_decimals(None, None, None, None);
    let bob = suite.senders[1].clone();

    let return_amount = RefCell::new(Uint128::zero());
    let mut swap_amount = Uint128::new(200_000000u128);

    let bob_uusdx_balance = RefCell::new(Uint128::zero());
    let bob_uusdc_balance = RefCell::new(Uint128::zero());
    let bob_uusdt_balance = RefCell::new(Uint128::zero());
    let bob_uusd_balance = RefCell::new(Uint128::zero());

    suite
        .query_balance(&bob.to_string(), DENOM_UUSD.to_string(), |result| {
            bob_uusd_balance
                .borrow_mut()
                .clone_from(&result.unwrap().amount);
        })
        .query_balance(&bob.to_string(), UUSDX_DENOM.to_string(), |result| {
            bob_uusdx_balance
                .borrow_mut()
                .clone_from(&result.unwrap().amount);
        })
        .query_balance(&bob.to_string(), DENOM_UUSDC.to_string(), |result| {
            bob_uusdc_balance
                .borrow_mut()
                .clone_from(&result.unwrap().amount);
        })
        .query_balance(&bob.to_string(), DENOM_UUSDT.to_string(), |result| {
            bob_uusdt_balance
                .borrow_mut()
                .clone_from(&result.unwrap().amount);
        });

    println!("uusd -> uusdx");
    println!("swap_amount : {:?}", swap_amount);

    suite
        .query_simulation(
            P1_POOL_ID.to_string(),
            Coin {
                denom: DENOM_UUSD.to_string(),
                amount: swap_amount,
            },
            UUSDX_DENOM.to_string(),
            |result| {
                let response: SimulationResponse = result.unwrap();
                println!("return_amount: {:?}", response.return_amount);

                return_amount
                    .borrow_mut()
                    .clone_from(&response.return_amount);
            },
        )
        .swap(
            &bob,
            UUSDX_DENOM.to_string(),
            None,
            Some(*DECIMAL_PERCENT_30),
            None,
            P1_POOL_ID.to_string(),
            vec![coin(swap_amount.u128(), DENOM_UUSD.to_string())],
            |result| {
                result.unwrap();
            },
        )
        .query_balance(&bob.to_string(), DENOM_UUSD.to_string(), |result| {
            let balance = result.unwrap().amount;
            assert_eq!(
                balance,
                Uint128::new(bob_uusd_balance.borrow().u128() - swap_amount.u128())
            );
            bob_uusd_balance.borrow_mut().clone_from(&balance);
        })
        .query_balance(&bob.to_string(), UUSDX_DENOM.to_string(), |result| {
            let balance = result.unwrap().amount;
            assert_eq!(
                balance,
                Uint128::new(bob_uusdx_balance.borrow().u128() + return_amount.borrow().u128())
            );
            bob_uusdx_balance.borrow_mut().clone_from(&balance);
        });

    println!("uusdx -> uusd");
    // swap_amount is SWAP_4POOL_VAL1 here
    println!("swap_amount : {:?}", swap_amount);

    suite
        .query_simulation(
            P1_POOL_ID.to_string(),
            Coin {
                denom: UUSDX_DENOM.to_string(),
                amount: swap_amount,
            },
            DENOM_UUSD.to_string(),
            |result| {
                let response: SimulationResponse = result.unwrap();
                println!("return_amount: {:?}", response.return_amount);

                return_amount
                    .borrow_mut()
                    .clone_from(&response.return_amount);
            },
        )
        .swap(
            &bob,
            DENOM_UUSD.to_string(),
            None,
            Some(*DECIMAL_PERCENT_30),
            None,
            P1_POOL_ID.to_string(),
            vec![coin(swap_amount.u128(), UUSDX_DENOM.to_string())],
            |result| {
                result.unwrap();
            },
        )
        .query_balance(&bob.to_string(), UUSDX_DENOM.to_string(), |result| {
            let balance = result.unwrap().amount;
            assert_eq!(
                balance,
                Uint128::new(bob_uusdx_balance.borrow().u128() - swap_amount.u128())
            );
            bob_uusdx_balance.borrow_mut().clone_from(&balance);
        })
        .query_balance(&bob.to_string(), DENOM_UUSD.to_string(), |result| {
            let balance = result.unwrap().amount;
            assert_eq!(
                balance,
                Uint128::new(bob_uusd_balance.borrow().u128() + return_amount.borrow().u128())
            );
            bob_uusd_balance.borrow_mut().clone_from(&balance);
        });

    println!("uusd -> uusdc");
    // swap_amount is SWAP_4POOL_VAL1 here
    println!("swap_amount : {:?}", swap_amount);

    suite
        .query_simulation(
            P1_POOL_ID.to_string(),
            Coin {
                denom: DENOM_UUSD.to_string(),
                amount: swap_amount,
            },
            DENOM_UUSDC.to_string(),
            |result| {
                let response: SimulationResponse = result.unwrap();
                println!("return_amount: {:?}", response.return_amount);

                return_amount
                    .borrow_mut()
                    .clone_from(&response.return_amount);
            },
        )
        .swap(
            &bob,
            DENOM_UUSDC.to_string(),
            None,
            Some(*DECIMAL_PERCENT_30),
            None,
            P1_POOL_ID.to_string(),
            vec![coin(swap_amount.u128(), DENOM_UUSD.to_string())],
            |result| {
                result.unwrap();
            },
        )
        .query_balance(&bob.to_string(), DENOM_UUSDC.to_string(), |result| {
            let balance = result.unwrap().amount;
            assert_eq!(
                balance,
                Uint128::new(bob_uusdc_balance.borrow().u128() + return_amount.borrow().u128())
            );
            bob_uusdc_balance.borrow_mut().clone_from(&balance);
        })
        .query_balance(&bob.to_string(), DENOM_UUSD.to_string(), |result| {
            let balance = result.unwrap().amount;
            assert_eq!(
                balance,
                Uint128::new(bob_uusd_balance.borrow().u128() - swap_amount.u128())
            );
            bob_uusd_balance.borrow_mut().clone_from(&balance);
        });

    println!("uusdc -> uusd");

    swap_amount = Uint128::new(200_000000000000u128);
    println!("swap_amount : {:?}", swap_amount);

    suite
        .query_simulation(
            P1_POOL_ID.to_string(),
            Coin {
                denom: DENOM_UUSDC.to_string(),
                amount: swap_amount,
            },
            DENOM_UUSD.to_string(),
            |result| {
                let response: SimulationResponse = result.unwrap();
                println!("return_amount: {:?}", response.return_amount);

                return_amount
                    .borrow_mut()
                    .clone_from(&response.return_amount);
            },
        )
        .swap(
            &bob,
            DENOM_UUSD.to_string(),
            None,
            Some(*DECIMAL_PERCENT_30),
            None,
            P1_POOL_ID.to_string(),
            vec![coin(swap_amount.u128(), DENOM_UUSDC.to_string())],
            |result| {
                result.unwrap();
            },
        )
        .query_balance(&bob.to_string(), DENOM_UUSD.to_string(), |result| {
            let balance = result.unwrap().amount;
            assert_eq!(
                balance,
                Uint128::new(bob_uusd_balance.borrow().u128() + return_amount.borrow().u128())
            );
            bob_uusd_balance.borrow_mut().clone_from(&balance);
        })
        .query_balance(&bob.to_string(), DENOM_UUSDC.to_string(), |result| {
            let balance = result.unwrap().amount;
            assert_eq!(
                balance,
                Uint128::new(bob_uusdc_balance.borrow().u128() - swap_amount.u128())
            );
            bob_uusdc_balance.borrow_mut().clone_from(&balance);
        });

    println!("uusd -> uusdt");

    swap_amount = Uint128::new(10_000_000000u128);
    println!("swap_amount : {:?}", swap_amount);

    suite
        .query_simulation(
            P1_POOL_ID.to_string(),
            Coin {
                denom: DENOM_UUSD.to_string(),
                amount: swap_amount,
            },
            DENOM_UUSDT.to_string(),
            |result| {
                let response: SimulationResponse = result.unwrap();
                println!("return_amount: {:?}", response.return_amount);

                return_amount
                    .borrow_mut()
                    .clone_from(&response.return_amount);
            },
        )
        .swap(
            &bob,
            DENOM_UUSDT.to_string(),
            None,
            Some(*DECIMAL_PERCENT_30),
            None,
            P1_POOL_ID.to_string(),
            vec![coin(swap_amount.u128(), DENOM_UUSD.to_string())],
            |result| {
                result.unwrap();
            },
        )
        .query_balance(&bob.to_string(), DENOM_UUSDT.to_string(), |result| {
            let balance = result.unwrap().amount;
            assert_eq!(
                balance,
                Uint128::new(bob_uusdt_balance.borrow().u128() + return_amount.borrow().u128())
            );
            bob_uusdt_balance.borrow_mut().clone_from(&balance);
        })
        .query_balance(&bob.to_string(), DENOM_UUSD.to_string(), |result| {
            let balance = result.unwrap().amount;
            assert_eq!(
                balance,
                Uint128::new(bob_uusd_balance.borrow().u128() - swap_amount.u128())
            );
            bob_uusd_balance.borrow_mut().clone_from(&balance);
        });

    println!("uusdt -> uusd");

    swap_amount = Uint128::new(10_000_000000000000000000u128);
    println!("swap_amount : {:?}", swap_amount);

    suite
        .query_simulation(
            P1_POOL_ID.to_string(),
            Coin {
                denom: DENOM_UUSDT.to_string(),
                amount: swap_amount,
            },
            DENOM_UUSD.to_string(),
            |result| {
                let response: SimulationResponse = result.unwrap();
                println!("return_amount: {:?}", response.return_amount);

                return_amount
                    .borrow_mut()
                    .clone_from(&response.return_amount);
            },
        )
        .swap(
            &bob,
            DENOM_UUSD.to_string(),
            None,
            Some(*DECIMAL_PERCENT_30),
            None,
            P1_POOL_ID.to_string(),
            vec![coin(swap_amount.u128(), DENOM_UUSDT.to_string())],
            |result| {
                result.unwrap();
            },
        )
        .query_balance(&bob.to_string(), DENOM_UUSD.to_string(), |result| {
            let balance = result.unwrap().amount;
            assert_eq!(
                balance,
                Uint128::new(bob_uusd_balance.borrow().u128() + return_amount.borrow().u128())
            );
            bob_uusd_balance.borrow_mut().clone_from(&balance);
        })
        .query_balance(&bob.to_string(), DENOM_UUSDT.to_string(), |result| {
            let balance = result.unwrap().amount;
            assert_eq!(
                balance,
                Uint128::new(bob_uusdt_balance.borrow().u128() - swap_amount.u128())
            );
            bob_uusdt_balance.borrow_mut().clone_from(&balance);
        });

    println!("uusdx -> uusdc");

    swap_amount = Uint128::new(1_000_000_000000u128);
    println!("swap_amount : {:?}", swap_amount);

    suite
        .query_simulation(
            P1_POOL_ID.to_string(),
            Coin {
                denom: UUSDX_DENOM.to_string(),
                amount: swap_amount,
            },
            DENOM_UUSDC.to_string(),
            |result| {
                let response: SimulationResponse = result.unwrap();
                println!("return_amount: {:?}", response.return_amount);

                return_amount
                    .borrow_mut()
                    .clone_from(&response.return_amount);
            },
        )
        .swap(
            &bob,
            DENOM_UUSDC.to_string(),
            None,
            Some(*DECIMAL_PERCENT_30),
            None,
            P1_POOL_ID.to_string(),
            vec![coin(swap_amount.u128(), UUSDX_DENOM.to_string())],
            |result| {
                result.unwrap();
            },
        )
        .query_balance(&bob.to_string(), DENOM_UUSDC.to_string(), |result| {
            let balance = result.unwrap().amount;
            assert_eq!(
                balance,
                Uint128::new(bob_uusdc_balance.borrow().u128() + return_amount.borrow().u128())
            );
            bob_uusdc_balance.borrow_mut().clone_from(&balance);
        })
        .query_balance(&bob.to_string(), UUSDX_DENOM.to_string(), |result| {
            let balance = result.unwrap().amount;
            assert_eq!(
                balance,
                Uint128::new(bob_uusdx_balance.borrow().u128() - swap_amount.u128())
            );
            bob_uusdx_balance.borrow_mut().clone_from(&balance);
        });

    println!("uusdc -> uusdt");

    swap_amount = Uint128::new(1_000_000_000000000000u128);
    println!("swap_amount : {:?}", swap_amount);

    suite
        .query_simulation(
            P1_POOL_ID.to_string(),
            Coin {
                denom: DENOM_UUSDC.to_string(),
                amount: swap_amount,
            },
            DENOM_UUSDT.to_string(),
            |result| {
                let response: SimulationResponse = result.unwrap();
                println!("return_amount: {:?}", response.return_amount);

                return_amount
                    .borrow_mut()
                    .clone_from(&response.return_amount);
            },
        )
        .swap(
            &bob,
            DENOM_UUSDT.to_string(),
            None,
            Some(*DECIMAL_PERCENT_30),
            None,
            P1_POOL_ID.to_string(),
            vec![coin(swap_amount.u128(), DENOM_UUSDC.to_string())],
            |result| {
                result.unwrap();
            },
        )
        .query_balance(&bob.to_string(), DENOM_UUSDT.to_string(), |result| {
            let balance = result.unwrap().amount;
            assert_eq!(
                balance,
                Uint128::new(bob_uusdt_balance.borrow().u128() + return_amount.borrow().u128())
            );
            bob_uusdt_balance.borrow_mut().clone_from(&balance);
        })
        .query_balance(&bob.to_string(), DENOM_UUSDC.to_string(), |result| {
            let balance = result.unwrap().amount;
            assert_eq!(
                balance,
                Uint128::new(bob_uusdc_balance.borrow().u128() - swap_amount.u128())
            );
            bob_uusdc_balance.borrow_mut().clone_from(&balance);
        });

    println!("uusdt -> uusdc");

    swap_amount = Uint128::new(5_000_000_000000000000u128);
    println!("swap_amount : {:?}", swap_amount);

    suite
        .query_simulation(
            P1_POOL_ID.to_string(),
            Coin {
                denom: DENOM_UUSDC.to_string(),
                amount: swap_amount,
            },
            DENOM_UUSDT.to_string(),
            |result| {
                let response: SimulationResponse = result.unwrap();
                println!("return_amount: {:?}", response.return_amount);

                return_amount
                    .borrow_mut()
                    .clone_from(&response.return_amount);
            },
        )
        .swap(
            &bob,
            DENOM_UUSDT.to_string(),
            None,
            Some(*DECIMAL_PERCENT_30),
            None,
            P1_POOL_ID.to_string(),
            vec![coin(swap_amount.u128(), DENOM_UUSDC.to_string())],
            |result| {
                result.unwrap();
            },
        )
        .query_balance(&bob.to_string(), DENOM_UUSDT.to_string(), |result| {
            let balance = result.unwrap().amount;
            assert_eq!(
                balance,
                Uint128::new(bob_uusdt_balance.borrow().u128() + return_amount.borrow().u128())
            );
            bob_uusdt_balance.borrow_mut().clone_from(&balance);
        })
        .query_balance(&bob.to_string(), DENOM_UUSDC.to_string(), |result| {
            let balance = result.unwrap().amount;
            assert_eq!(
                balance,
                Uint128::new(bob_uusdc_balance.borrow().u128() - swap_amount.u128())
            );
            bob_uusdc_balance.borrow_mut().clone_from(&balance);
        });

    println!("uusdc -> uusdt");

    swap_amount = Uint128::new(5_000_000_000000000000000000u128);
    println!("swap_amount : {:?}", swap_amount);

    suite
        .query_simulation(
            P1_POOL_ID.to_string(),
            Coin {
                denom: DENOM_UUSDT.to_string(),
                amount: swap_amount,
            },
            DENOM_UUSDC.to_string(),
            |result| {
                let response: SimulationResponse = result.unwrap();
                println!("return_amount: {:?}", response.return_amount);

                return_amount
                    .borrow_mut()
                    .clone_from(&response.return_amount);
            },
        )
        .swap(
            &bob,
            DENOM_UUSDC.to_string(),
            None,
            Some(*DECIMAL_PERCENT_30),
            None,
            P1_POOL_ID.to_string(),
            vec![coin(swap_amount.u128(), DENOM_UUSDT.to_string())],
            |result| {
                result.unwrap();
            },
        )
        .query_balance(&bob.to_string(), DENOM_UUSDC.to_string(), |result| {
            let balance = result.unwrap().amount;
            assert_eq!(
                balance,
                Uint128::new(bob_uusdc_balance.borrow().u128() + return_amount.borrow().u128())
            );
            bob_uusdc_balance.borrow_mut().clone_from(&balance);
        })
        .query_balance(&bob.to_string(), DENOM_UUSDT.to_string(), |result| {
            let balance = result.unwrap().amount;
            assert_eq!(
                balance,
                Uint128::new(bob_uusdt_balance.borrow().u128() - swap_amount.u128())
            );
            bob_uusdt_balance.borrow_mut().clone_from(&balance);
        });
}

#[allow(clippy::inconsistent_digit_grouping)]
#[test]
fn simulation_vs_reverse_simulation_3pool() {
    let mut suite = setup_3pool_different_decimals(None, None, None, None);

    // Test cases with different amounts and token pairs
    let test_cases = vec![
        // Small amounts
        (DENOM_UUSD, DENOM_UUSDC, Uint128::new(333_000000u128)),
        (DENOM_UUSDC, DENOM_UUSDT, Uint128::new(5_000000000000u128)),
        // Medium amounts
        (
            DENOM_UUSDT,
            DENOM_UUSD,
            Uint128::new(10_000_000000000000000000u128),
        ),
        // Large amounts
        (DENOM_UUSD, DENOM_UUSDT, Uint128::new(1_000_000_000000u128)),
    ];
    let mut decimals_by_denom: HashMap<String, u8> = HashMap::new();
    decimals_by_denom.insert(DENOM_UUSD.to_string(), DECIMALS_6);
    decimals_by_denom.insert(DENOM_UUSDC.to_string(), DECIMALS_12);
    decimals_by_denom.insert(DENOM_UUSDT.to_string(), DECIMALS_18);

    // Test each case
    for (token_in, token_out, amount_in) in test_cases {
        let token_in_decimals = decimals_by_denom[&token_in.to_string()];

        let simulated_return = RefCell::new(Uint128::zero());
        let simulated_spread = RefCell::new(Uint128::zero());
        let reverse_simulated_offer = RefCell::new(Uint128::zero());
        let reverse_simulated_spread = RefCell::new(Uint128::zero());

        // Forward simulation
        suite.query_simulation(
            P1_POOL_ID.to_string(),
            Coin {
                denom: token_in.to_string(),
                amount: amount_in,
            },
            token_out.to_string(),
            |result| {
                let response = result.unwrap();
                simulated_return
                    .borrow_mut()
                    .clone_from(&response.return_amount);
                simulated_spread
                    .borrow_mut()
                    .clone_from(&response.slippage_amount);
            },
        );

        // Reverse simulation using the forward simulation result
        suite.query_reverse_simulation(
            P1_POOL_ID.to_string(),
            Coin {
                denom: token_out.to_string(),
                amount: *simulated_return.borrow(),
            },
            token_in.to_string(),
            |result| {
                let response = result.unwrap();
                reverse_simulated_offer
                    .borrow_mut()
                    .clone_from(&response.offer_amount);
                reverse_simulated_spread
                    .borrow_mut()
                    .clone_from(&response.slippage_amount);
            },
        );

        // Compare the original amount_in with the reverse simulated offer amount
        // They should be very close, with a small difference due to rounding
        // Using 1bps tolerance
        let tolerance = *ONE_BPS_DECIMAL_TOLERANCE;
        let amount_in_decimal =
            Decimal::from_ratio(amount_in, 10u128.pow(u32::from(token_in_decimals)));
        let reverse_offer_decimal = Decimal::from_ratio(
            *reverse_simulated_offer.borrow(),
            10u128.pow(u32::from(token_in_decimals)),
        );

        let diff = amount_in_decimal.abs_diff(reverse_offer_decimal);
        let max_allowed_diff = amount_in_decimal * tolerance;

        assert!(
            diff <= max_allowed_diff,
            "Simulation mismatch for {}->{}: original={}, reverse={}, diff={}, max_allowed={}",
            token_in,
            token_out,
            amount_in,
            reverse_simulated_offer.borrow(),
            diff,
            max_allowed_diff
        );

        // Compare the spread between the forward and reverse simulations
        let spread_simulation = *simulated_spread.borrow();
        let spread_reverse_simulation = *reverse_simulated_spread.borrow();
        let spread_tolerance = (amount_in_decimal * tolerance).atomics();

        assert!(
            spread_simulation.saturating_sub(spread_reverse_simulation) <= spread_tolerance,
            "Spread mismatch for {}->{}: simulation={}, reverse={}, diff={}, max_allowed={}",
            token_in,
            token_out,
            spread_simulation,
            spread_reverse_simulation,
            spread_simulation.saturating_sub(spread_reverse_simulation),
            spread_tolerance
        );
    }
}

#[test]
fn belief_price_works_decimals_independent() {
    let mut suite = TestingSuite::default_with_balances(
        vec![
            coin(1_000_000_000u128, DENOM_UUSD.to_string()),
            coin(
                1_000_000_000_000_000000000000000000u128,
                UWETH_DENOM.to_string(),
            ),
            coin(10_000u128, DENOM_UOM.to_string()),
        ],
        StargateMock::new(vec![coin(
            EIGHT_EIGHT_EIGHT_EIGHT_U128.u128(),
            DENOM_UOM.to_string(),
        )]),
    );
    let creator = suite.creator();
    let user = suite.senders[1].clone();

    suite
        .instantiate_default()
        .add_one_epoch()
        .create_pool(
            &creator,
            vec![DENOM_UUSD.to_string(), UWETH_DENOM.to_string()],
            vec![DECIMALS_6, DECIMALS_18],
            PoolFee {
                protocol_fee: Fee {
                    share: Decimal::zero(),
                },
                swap_fee: Fee {
                    share: Decimal::zero(),
                },
                burn_fee: Fee {
                    share: *DECIMAL_ZERO,
                },
                extra_fees: vec![],
            },
            PoolType::ConstantProduct,
            Some("uusd.uweth".to_string()),
            vec![
                coin(ONE_THOUSAND_U128.u128(), DENOM_UUSD),
                coin(EIGHT_EIGHT_EIGHT_EIGHT_U128.u128(), DENOM_UOM),
            ],
            |result| {
                result.unwrap();
            },
        )
        .provide_liquidity(
            &user,
            UUSD_UWETH_POOL_ID.to_string(),
            None,
            None,
            None,
            None,
            None,
            vec![
                Coin {
                    denom: DENOM_UUSD.to_string(),
                    amount: Uint128::new(100_000_000u128),
                },
                Coin {
                    denom: UWETH_DENOM.to_string(),
                    amount: Uint128::new(100_00000000000000000000u128),
                },
            ],
            |result| {
                result.unwrap();
            },
        );

    // swap 18 decimals to 6 decimals
    let simulated_return_amount = RefCell::new(Uint128::zero());
    suite.query_simulation(
        UUSD_UWETH_POOL_ID.to_string(),
        Coin {
            denom: UWETH_DENOM.to_string(),
            amount: SWAP_OFFER_UWETH_BELIEF_PRICE,
        },
        DENOM_UUSD.to_string(),
        |result| {
            let response: SimulationResponse = result.unwrap();
            simulated_return_amount
                .borrow_mut()
                .clone_from(&response.return_amount);
        },
    );

    suite.swap(
        &user,
        DENOM_UUSD.to_string(),
        // belief_price = offer / ask
        Some(Decimal::from_ratio(
            SWAP_OFFER_UWETH_BELIEF_PRICE, // This is the offer amount in the original token's precision
            *simulated_return_amount.borrow(),
        )),
        None,
        None,
        UUSD_UWETH_POOL_ID.to_string(),
        vec![coin(
            SWAP_OFFER_UWETH_BELIEF_PRICE.u128(),
            UWETH_DENOM.to_string(),
        )],
        |result| {
            result.unwrap();
        },
    );

    // swap 6 decimals to 18 decimals
    suite.query_simulation(
        UUSD_UWETH_POOL_ID.to_string(),
        Coin {
            denom: DENOM_UUSD.to_string(),
            amount: SWAP_OFFER_UUSD_BELIEF_PRICE,
        },
        UWETH_DENOM.to_string(),
        |result| {
            let response: SimulationResponse = result.unwrap();
            simulated_return_amount
                .borrow_mut()
                .clone_from(&response.return_amount);
        },
    );

    suite.swap(
        &user,
        UWETH_DENOM.to_string(),
        // belief_price = offer / ask
        Some(Decimal::from_ratio(
            SWAP_OFFER_UUSD_BELIEF_PRICE, // This is the offer amount in the original token's precision
            *simulated_return_amount.borrow(),
        )),
        None,
        None,
        UUSD_UWETH_POOL_ID.to_string(),
        vec![coin(
            SWAP_OFFER_UUSD_BELIEF_PRICE.u128(),
            DENOM_UUSD.to_string(),
        )],
        |result| {
            result.unwrap();
        },
    );
}

#[test]
fn compute_offer_amount_floor() {
    let mut suite = TestingSuite::default_with_balances(
        vec![
            coin(ONE_HUNDRED_TRILLION, DENOM_ULUNA.to_string()),
            coin(ONE_HUNDRED_TRILLION, DENOM_UUSD.to_string()),
            coin(10_000u128, DENOM_UOM.to_string()),
        ],
        StargateMock::new(vec![coin(
            EIGHT_EIGHT_EIGHT_EIGHT_U128.u128(),
            DENOM_UOM.to_string(),
        )]),
    );
    let creator = suite.creator();
    let user = suite.senders[1].clone();
    let pool_id = ULUNA_UUSD_POOL_ID.to_string();

    suite
        .instantiate_default()
        .add_one_epoch()
        .create_pool(
            &creator,
            vec![DENOM_ULUNA.to_string(), DENOM_UUSD.to_string()],
            vec![DECIMALS_6, DECIMALS_6],
            PoolFee {
                protocol_fee: Fee {
                    share: Decimal::percent(0),
                },
                swap_fee: Fee {
                    share: Decimal::percent(0),
                },
                burn_fee: Fee {
                    share: *DECIMAL_ZERO,
                },
                extra_fees: vec![],
            },
            PoolType::ConstantProduct,
            Some(ULUNA_UUSD_POOL_RAW.to_string()),
            vec![
                coin(ONE_THOUSAND_U128.u128(), DENOM_UUSD),
                coin(EIGHT_EIGHT_EIGHT_EIGHT_U128.u128(), DENOM_UOM),
            ],
            |result| {
                result.unwrap();
            },
        )
        .provide_liquidity(
            &creator,
            pool_id.clone(),
            None,
            None,
            None,
            None,
            None,
            vec![
                Coin {
                    denom: DENOM_ULUNA.to_string(),
                    amount: Uint128::new(100_000_000_000u128),
                },
                Coin {
                    denom: DENOM_UUSD.to_string(),
                    amount: Uint128::new(100_000_000_000u128),
                },
            ],
            |result| {
                result.unwrap();
            },
        );

    // Need uusd amount (amount out)
    let needed_uusd = Uint128::new(99_900_099u128);
    let offer_uluna = std::cell::RefCell::new(Uint128::zero());

    suite
        .query_reverse_simulation(
            pool_id.clone(),
            Coin {
                denom: DENOM_UUSD.to_string(),
                amount: needed_uusd,
            },
            DENOM_ULUNA.to_string(),
            |result| {
                // Computed the amount of uluna (amount in)
                *offer_uluna.borrow_mut() = result.unwrap().offer_amount;
            },
        )
        // Swap using the computed amount in
        .swap(
            &user,
            DENOM_UUSD.to_string(),
            None,
            None,
            None,
            pool_id.clone(),
            vec![coin(
                (*offer_uluna.borrow()).into(),
                DENOM_ULUNA.to_string(),
            )],
            |result| {
                for event in result.unwrap().events {
                    if event.ty == EVENT_TYPE_WASM {
                        for attribute in event.attributes {
                            if attribute.key.as_str() == "return_amount" {
                                let return_amount = attribute.value.parse::<Uint128>().unwrap();
                                assert!(return_amount >= needed_uusd);
                            }
                        }
                    }
                }
            },
        );
}

// This function is used to setup a 3pool with different decimals.
// Default values are used if not provided.
// -- Default decimals: 6, 12, 18
// -- Default amp: 85
// -- Default initial balances: 300T for each token
// -- Default initial liquidity: 100T for each token
fn setup_3pool_different_decimals(
    asset_decimals: Option<Vec<u8>>,
    initial_balances: Option<Vec<Uint128>>,
    amp: Option<u64>,
    initial_liquidity: Option<Vec<Uint128>>,
) -> TestingSuite {
    // Default values
    let decimals = asset_decimals.unwrap_or_else(|| vec![DECIMALS_6, DECIMALS_12, DECIMALS_18]);
    let amp_val = amp.unwrap_or(85);

    // Default initial balances (300T for each token)
    // The original calculation was: 300_000_000_000_000u128 * 10u128.pow(18)
    // 300_000_000_000_000u128 is the base balance
    let initial_balance_val = Uint128::new(300_000_000_000_000u128) * Uint128::from(10u128.pow(18));
    let balances = initial_balances.unwrap_or_else(|| {
        vec![
            initial_balance_val,
            initial_balance_val,
            initial_balance_val,
        ]
    });

    // Default initial liquidity (100T for each token)
    let liquidity = initial_liquidity.unwrap_or_else(|| {
        vec![
            Uint128::new(100_000_000_000_000_000000u128),
            Uint128::new(100_000_000_000_000_000000000000u128),
            Uint128::new(100_000_000_000_000_000000000000000000u128),
        ]
    });

    let asset_denoms_str = vec![
        DENOM_UUSD.to_string(),
        DENOM_UUSDC.to_string(),
        DENOM_UUSDT.to_string(),
    ];

    // Create test suite with initial balances
    let mut suite = TestingSuite::default_with_balances(
        vec![
            coin(balances[0].u128(), DENOM_UUSD.to_string()),
            coin(balances[1].u128(), DENOM_UUSDC.to_string()),
            coin(balances[2].u128(), DENOM_UUSDT.to_string()),
            coin(BALANCE_1T_U128.u128(), DENOM_UOM.to_string()),
        ],
        StargateMock::new(vec![coin(
            EIGHT_EIGHT_EIGHT_EIGHT_U128.u128(),
            DENOM_UOM.to_string(),
        )]),
    );

    let creator = suite.creator();

    // Pool fees (zero fees for testing)
    let pool_fees = PoolFee {
        protocol_fee: Fee {
            share: Decimal::zero(),
        },
        swap_fee: Fee {
            share: Decimal::zero(),
        },
        burn_fee: Fee {
            share: *DECIMAL_ZERO,
        },
        extra_fees: vec![],
    };

    // Create pool
    suite.instantiate_default().add_one_epoch().create_pool(
        &creator,
        asset_denoms_str.clone(),
        decimals,
        pool_fees,
        PoolType::StableSwap { amp: amp_val },
        None,
        vec![
            coin(ONE_THOUSAND_U128.u128(), DENOM_UUSD),
            coin(EIGHT_EIGHT_EIGHT_EIGHT_U128.u128(), DENOM_UOM),
        ],
        |result| {
            result.unwrap();
        },
    );

    // Provide initial liquidity
    suite.provide_liquidity(
        &creator,
        P1_POOL_ID.to_string(),
        None,
        None,
        None,
        None,
        None,
        vec![
            Coin {
                denom: DENOM_UUSD.to_string(),
                amount: liquidity[0],
            },
            Coin {
                denom: DENOM_UUSDC.to_string(),
                amount: liquidity[1],
            },
            Coin {
                denom: DENOM_UUSDT.to_string(),
                amount: liquidity[2],
            },
        ],
        |result| {
            result.unwrap();
        },
    );

    suite
}

// This function is used to setup a 4pool with different decimals.
// Default values are used if not provided.
// -- Default decimals: 6, 6, 12, 18
// -- Default amp: 85
// -- Default initial balances: 300T for each token
// -- Default initial liquidity: 100T for each token
fn setup_4pool_different_decimals(
    asset_decimals: Option<Vec<u8>>,
    initial_balances: Option<Vec<Uint128>>,
    amp: Option<u64>,
    initial_liquidity: Option<Vec<Uint128>>,
) -> TestingSuite {
    // Default values
    let decimals =
        asset_decimals.unwrap_or_else(|| vec![DECIMALS_6, DECIMALS_6, DECIMALS_12, DECIMALS_18]);
    let amp = amp.unwrap_or(85);

    // Default initial balances (300T for each token)
    let initial_balance = 300_000_000_000_000u128 * 10u128.pow(18);
    let balances = initial_balances.unwrap_or_else(|| {
        vec![
            Uint128::new(initial_balance),
            Uint128::new(initial_balance),
            Uint128::new(initial_balance),
            Uint128::new(initial_balance),
        ]
    });

    // Default initial liquidity (100T for each token)
    let liquidity = initial_liquidity.unwrap_or_else(|| {
        vec![
            Uint128::new(ONE_HUNDRED_TRILLION * 10u128.pow(6)), // 100T with 6 decimals
            Uint128::new(ONE_HUNDRED_TRILLION * 10u128.pow(6)), // 100T with 6 decimals
            Uint128::new(ONE_HUNDRED_TRILLION * 10u128.pow(12)), // 100T with 12 decimals
            Uint128::new(ONE_HUNDRED_TRILLION * 10u128.pow(18)), // 100T with 18 decimals
        ]
    });

    let asset_denoms = vec![
        "uusd".to_string(),
        "uusdx".to_string(),
        "uusdc".to_string(),
        "uusdt".to_string(),
    ];

    // Create test suite with initial balances
    let mut suite = TestingSuite::default_with_balances(
        vec![
            coin(balances[0].u128(), "uusd".to_string()),
            coin(balances[1].u128(), "uusdx".to_string()),
            coin(balances[2].u128(), "uusdc".to_string()),
            coin(balances[3].u128(), "uusdt".to_string()),
            coin(1_000_000_000_000u128, "uom".to_string()),
        ],
        StargateMock::new(vec![coin(8888u128, "uom".to_string())]),
    );

    let creator = suite.creator();

    // Pool fees (zero fees for testing)
    let pool_fees = PoolFee {
        protocol_fee: Fee {
            share: Decimal::zero(),
        },
        swap_fee: Fee {
            share: Decimal::zero(),
        },
        burn_fee: Fee {
            share: *DECIMAL_ZERO,
        },
        extra_fees: vec![],
    };

    // Create pool
    suite.instantiate_default().add_one_epoch().create_pool(
        &creator,
        asset_denoms.clone(),
        decimals,
        pool_fees,
        PoolType::StableSwap { amp },
        None,
        vec![coin(1000, "uusd"), coin(8888, "uom")],
        |result| {
            result.unwrap();
        },
    );

    // Provide initial liquidity
    suite.provide_liquidity(
        &creator,
        "p.1".to_string(),
        None,
        None,
        None,
        None,
        None,
        vec![
            Coin {
                denom: "uusd".to_string(),
                amount: liquidity[0],
            },
            Coin {
                denom: "uusdx".to_string(),
                amount: liquidity[1],
            },
            Coin {
                denom: "uusdc".to_string(),
                amount: liquidity[2],
            },
            Coin {
                denom: "uusdt".to_string(),
                amount: liquidity[3],
            },
        ],
        |result| {
            result.unwrap();
        },
    );

    suite
}

#[test]
fn providing_skewed_liquidity_on_stableswap_gets_punished_same_decimals() {
    let separator = String::from("===============================================================");
    println!(
        "TEST providing_skewed_liquidity_on_stableswap_gets_punished_same_decimals {}",
        separator
    );
    // Decimals already defined as DECIMAL_PLACES
    let initial_balance = BALANCE_300T_3POOL_U128; // Reusing existing constant

    let mut suite = TestingSuite::default_with_balances(
        vec![
            coin(initial_balance.u128(), DENOM_UUSDT.to_string()),
            coin(initial_balance.u128(), DENOM_UUSDC.to_string()),
            coin(initial_balance.u128(), DENOM_UUSD.to_string()),
            coin(initial_balance.u128(), DENOM_UOM.to_string()),
        ],
        StargateMock::new(vec![coin(
            EIGHT_EIGHT_EIGHT_EIGHT_U128.u128(),
            DENOM_UOM.to_string(),
        )]),
    );
    let alice = suite.creator();
    let bob = suite.senders[1].clone();
    let asset_infos = vec![DENOM_UUSDC.to_string(), DENOM_UUSDT.to_string()];

    let pool_fees = PoolFee {
        protocol_fee: Fee {
            share: Decimal::zero(),
        },
        swap_fee: Fee {
            share: Decimal::percent(5),
        },
        burn_fee: Fee {
            share: *DECIMAL_ZERO,
        },
        extra_fees: vec![],
    };

    // Create a pool
    suite.instantiate_default().create_pool(
        &alice,
        asset_infos,
        vec![DECIMALS_6, DECIMALS_6],
        pool_fees,
        PoolType::StableSwap {
            amp: STABLE_SWAP_AMP,
        }, // Reusing existing STABLE_SWAP_AMP
        Some("uusdc.uusdt".to_string()),
        vec![
            coin(ONE_THOUSAND_U128.u128(), DENOM_UUSD),
            coin(EIGHT_EIGHT_EIGHT_EIGHT_U128.u128(), DENOM_UOM),
        ],
        |result| {
            result.unwrap();
        },
    );

    // Initial liquidity
    let alice_initial_uusdt_liquidity =
        ALICE_INITIAL_LIQ_BASE_SKEWED_TEST * Uint128::from(10u128.pow(u32::from(DECIMALS_6)));
    let alice_initial_uusdc_liquidity =
        ALICE_INITIAL_LIQ_BASE_SKEWED_TEST * Uint128::from(10u128.pow(u32::from(DECIMALS_6)));
    println!("{}", separator);
    println!("==== Alice provides 1k usdt and 1k usdc as initial liquidity to the pool");
    suite.provide_liquidity(
        &alice,
        UUSDC_UUSDT_POOL_ID.to_string(),
        None,
        None,
        None,
        None,
        None,
        vec![
            Coin {
                denom: DENOM_UUSDT.to_string(),
                amount: alice_initial_uusdt_liquidity,
            },
            Coin {
                denom: DENOM_UUSDC.to_string(),
                amount: alice_initial_uusdc_liquidity,
            },
        ],
        |result| {
            result.unwrap();
        },
    );

    println!("{}", separator);
    println!("==== Bob provides 110 usdt and 90 usdc of liquidity");
    let bob_initial_uusdt_liquidity =
        BOB_INITIAL_LIQ_USDT_BASE_SKEWED_TEST * Uint128::from(10u128.pow(u32::from(DECIMALS_6)));
    let bob_initial_uusdc_liquidity =
        BOB_INITIAL_LIQ_USDC_BASE_SKEWED_TEST * Uint128::from(10u128.pow(u32::from(DECIMALS_6)));
    suite.provide_liquidity(
        &bob,
        UUSDC_UUSDT_POOL_ID.to_string(),
        None,
        None,
        None,
        None,
        None,
        vec![
            Coin {
                denom: DENOM_UUSDT.to_string(),
                amount: bob_initial_uusdt_liquidity,
            },
            Coin {
                denom: DENOM_UUSDC.to_string(),
                amount: bob_initial_uusdc_liquidity,
            },
        ],
        |result| {
            result.unwrap();
        },
    );

    let lp_shares_alice = RefCell::new(Coin::new(0u128, "".to_string()));
    let lp_shares_bob = RefCell::new(Coin::new(0u128, "".to_string()));

    let bob_uusdc_balance = RefCell::new(Uint128::zero());
    let bob_uusdt_balance = RefCell::new(Uint128::zero());
    let alice_uusdc_balance = RefCell::new(Uint128::zero());
    let alice_uusdt_balance = RefCell::new(Uint128::zero());

    suite
        .query_all_balances(&alice.to_string(), |balances| {
            for coin in balances.unwrap().iter() {
                match coin.denom.as_str() {
                    denom if denom.contains(UUSDC_UUSDT_POOL_ID) => {
                        // Check against pool ID string
                        *lp_shares_alice.borrow_mut() = coin.clone();
                    }
                    denom if denom == DENOM_UUSDC => {
                        // Exact match for denom
                        *alice_uusdc_balance.borrow_mut() = coin.amount;
                    }
                    denom if denom == DENOM_UUSDT => {
                        // Exact match for denom
                        *alice_uusdt_balance.borrow_mut() = coin.amount;
                    }
                    _ => {}
                }
            }
        })
        .query_all_balances(&bob.to_string(), |balances| {
            for coin in balances.unwrap().iter() {
                match coin.denom.as_str() {
                    denom if denom.contains(UUSDC_UUSDT_POOL_ID) => {
                        // Check against pool ID string
                        *lp_shares_bob.borrow_mut() = coin.clone();
                    }
                    denom if denom == DENOM_UUSDC => {
                        // Exact match for denom
                        *bob_uusdc_balance.borrow_mut() = coin.amount;
                    }
                    denom if denom == DENOM_UUSDT => {
                        // Exact match for denom
                        *bob_uusdt_balance.borrow_mut() = coin.amount;
                    }
                    _ => {}
                }
            }
        });

    println!("{}", separator);
    println!("==== Withdraw Alice's and Bob's liquidity");
    suite
        .withdraw_liquidity(
            &bob,
            UUSDC_UUSDT_POOL_ID.to_string(),
            vec![lp_shares_bob.borrow().clone()],
            |result| {
                result.unwrap();
            },
        )
        .withdraw_liquidity(
            &alice,
            UUSDC_UUSDT_POOL_ID.to_string(),
            vec![lp_shares_alice.borrow().clone()],
            |result| {
                result.unwrap();
            },
        );

    // now alice should have been rewarded with the swap fees paid by bob for providing liquidity
    // with a skewed ratio
    let alice_usdc_balance_change = RefCell::new(0i128);
    let alice_usdt_balance_change = RefCell::new(0i128);
    suite.query_all_balances(&alice.to_string(), |balances| {
        for coin in balances.unwrap().iter() {
            match coin.denom.as_str() {
                denom if denom == DENOM_UUSDC => {
                    let coin_amount_i128 = i128::try_from(coin.amount.u128()).unwrap();
                    let initial_balance_i128 = i128::try_from(initial_balance.u128()).unwrap();
                    let difference = coin_amount_i128 - initial_balance_i128;
                    println!("{}", separator);
                    println!("Alice USDC balance change:");
                    println!("difference:       {}", difference);
                    println!("initial_balance:  {}", initial_balance.u128());
                    println!("coin.amount:      {}", coin.amount.u128());
                    *alice_usdc_balance_change.borrow_mut() = difference;
                    assert!(difference > 0);
                }
                denom if denom == DENOM_UUSDT => {
                    let coin_amount_i128 = i128::try_from(coin.amount.u128()).unwrap();
                    let initial_balance_i128 = i128::try_from(initial_balance.u128()).unwrap();
                    let difference = coin_amount_i128 - initial_balance_i128;
                    println!("{}", separator);
                    println!("Alice USDT balance change:");
                    println!("difference:       {}", difference);
                    println!("initial_balance:  {}", initial_balance.u128());
                    println!("coin.amount:      {}", coin.amount.u128());
                    *alice_usdt_balance_change.borrow_mut() = difference;
                    assert!(difference > 0);
                }
                _ => {}
            }
        }
    });
    let alice_nominal_balance_change =
        *alice_usdc_balance_change.borrow() + *alice_usdt_balance_change.borrow();
    println!(
        "Alice nominal difference:  {}",
        alice_nominal_balance_change
    );

    // bob on the other hand was punished
    let bob_usdc_balance_change = RefCell::new(0i128);
    let bob_usdt_balance_change = RefCell::new(0i128);
    suite.query_all_balances(&bob.to_string(), |balances| {
        for coin in balances.unwrap().iter() {
            match coin.denom.as_str() {
                denom if denom == DENOM_UUSDC => {
                    let coin_amount_i128 = i128::try_from(coin.amount.u128()).unwrap();
                    let initial_balance_i128 = i128::try_from(initial_balance.u128()).unwrap();
                    let difference = coin_amount_i128 - initial_balance_i128;
                    println!("{}", separator);
                    println!("Bob USDC balance change:");
                    println!("difference:           {}", difference);
                    println!("initial balance:      {}", initial_balance.u128());
                    println!("current amount:       {}", coin.amount.u128());
                    *bob_usdc_balance_change.borrow_mut() = difference;
                    assert!(difference < 0);
                }
                denom if denom == DENOM_UUSDT => {
                    let coin_amount_i128 = i128::try_from(coin.amount.u128()).unwrap();
                    let initial_balance_i128 = i128::try_from(initial_balance.u128()).unwrap();
                    let difference = coin_amount_i128 - initial_balance_i128;
                    println!("{}", separator);
                    println!("Bob USDT balance change:");
                    println!("difference:           {}", difference);
                    println!("initial balance:      {}", initial_balance.u128());
                    println!("current amount:       {}", coin.amount.u128());
                    *bob_usdt_balance_change.borrow_mut() = difference;
                    assert!(difference < 0);
                }
                _ => {}
            }
        }
    });
    let bob_nominal_balance_change =
        *bob_usdc_balance_change.borrow() + *bob_usdt_balance_change.borrow();
    println!("Bob nominal difference: {}", bob_nominal_balance_change);

    // check remaining assets on the pool
    println!("{}", separator);
    println!("==== Remaining assets in pool");
    suite.query_pools(
        Some(UUSDC_UUSDT_POOL_ID.to_string()),
        None,
        None,
        |result| {
            let response = result.unwrap();
            let pool = response.pools[0].pool_info.clone();
            let assets = pool.assets;
            for asset in assets {
                println!("amount:       {}", asset.amount);
                println!("denom:        {}", asset.denom);
            }
        },
    );
}
