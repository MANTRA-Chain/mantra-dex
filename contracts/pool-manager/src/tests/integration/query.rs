use std::cell::RefCell;

use super::super::suite::TestingSuite;
use cosmwasm_std::{assert_approx_eq, coin, Coin, Decimal, Uint128};
use mantra_common_testing::multi_test::stargate_mock::StargateMock;
use mantra_dex_std::fee::{Fee, PoolFee};
use mantra_dex_std::pool_manager::{PoolType, SwapOperation};
use test_utils::common_constants::{
    DECIMALS_6, DENOM_ULUNA as DENOM_LUNA, DENOM_UOM as DENOM_OM, DENOM_UUSD as DENOM_USD,
    DENOM_UUSDC as DENOM_USDC, DENOM_UUSDT as DENOM_USDT, DENOM_UWHALE as DENOM_WHALE,
    INITIAL_BALANCE, INITIAL_BALANCE_PLUS_ONE, ONE_MILLION, ONE_THOUSAND, STABLESWAP_AMP_FACTOR,
    STARGATE_MOCK_UOM_AMOUNT as OM_STARGATE_BALANCE,
};

// Token amounts
const LARGE_INITIAL_BALANCE: u128 = 1_000_000_000_000;
const LARGE_INITIAL_BALANCE_PLUS_ONE: u128 = LARGE_INITIAL_BALANCE + 1;

// Pool identifiers
const POOL_ID_WHALE_LUNA: &str = "whale.uluna";
const POOL_ID_USD_USDC: &str = "uusd.uusdc";
const POOL_IDENTIFIER_WHALE_LUNA: &str = "o.whale.uluna";
const POOL_IDENTIFIER_USD_USDC: &str = "o.uusd.uusdc";

// Fee percentages
const PROTOCOL_FEE_PERCENT: u64 = 1;
const SWAP_FEE_PERCENT: u64 = 2;
const BURN_FEE_PERCENT: u64 = 3;
const EXTRA_FEE_PERCENT: u64 = 4;
const SLIPPAGE_TOLERANCE_PERCENT: u64 = 10;

// Expected values
const EXPECTED_PROTOCOL_FEE: u128 = 10;
const EXPECTED_SWAP_FEE: u128 = 20;
const EXPECTED_BURN_FEE: u128 = 30;
const EXPECTED_EXTRA_FEE: u128 = 40;

// Simulation parameters
const SIMULATION_TOLERANCE: &str = "0.1";
const RETURN_AMOUNT_TOLERANCE: &str = "0.00000001";

// Additional constants for simulate_swap_operations_query_verification test
const POOL_ID_OM_USDT: &str = "uom.uusdt";
const POOL_ID_USDT_USDC: &str = "uusdt.uusdc";
const POOL_IDENTIFIER_OM_USDT: &str = "o.uom.uusdt";
const POOL_IDENTIFIER_USDT_USDC: &str = "o.uusdt.uusdc";
const OM_LIQUIDITY_AMOUNT: u128 = 1_000_000_000;
const USDT_OM_POOL_LIQUIDITY: u128 = 4_000_000_000;
const USDT_USDC_POOL_LIQUIDITY: u128 = 1_000_000_000;

// Expected values for simulate_swap_operations_query_verification

// Additional constants for reverse_simulation_queries_fees_verification test
const REVERSE_SIMULATION_AMOUNT_WHALE: u128 = 903;
const REVERSE_SIMULATION_AMOUNT_USD: u128 = 900;

const OFFER_AMOUNT_TOLERANCE: &str = "0.002";
const RETURN_AMOUNT_TOLERANCE_WHALE: &str = "0.002";
const WHALE_LIQUIDITY_AMOUNT: u128 = 1_000_000_000;

// Additional constants for reverse_simulate_swap_operations_query_verification test

#[test]
fn simulation_queries_fees_verification() {
    let mut suite = TestingSuite::default_with_balances(
        vec![
            coin(INITIAL_BALANCE_PLUS_ONE, DENOM_WHALE.to_string()),
            coin(INITIAL_BALANCE, DENOM_LUNA.to_string()),
            coin(INITIAL_BALANCE_PLUS_ONE, DENOM_USD.to_string()),
            coin(INITIAL_BALANCE_PLUS_ONE, DENOM_USDC.to_string()),
            coin(INITIAL_BALANCE_PLUS_ONE, DENOM_OM.to_string()),
        ],
        StargateMock::new(vec![coin(OM_STARGATE_BALANCE, DENOM_OM.to_string())]),
    );
    let creator = suite.creator();
    let _other = suite.senders[1].clone();
    let _unauthorized = suite.senders[2].clone();
    // Asset infos with uwhale and uluna

    let asset_infos = vec![DENOM_WHALE.to_string(), DENOM_LUNA.to_string()];

    // protocol fee 1%
    // swap fee 2%
    // burn fee 3%
    // extra fee 4%
    let pool_fees = PoolFee {
        protocol_fee: Fee {
            share: Decimal::percent(PROTOCOL_FEE_PERCENT),
        },
        swap_fee: Fee {
            share: Decimal::percent(SWAP_FEE_PERCENT),
        },
        burn_fee: Fee {
            share: Decimal::percent(BURN_FEE_PERCENT),
        },
        extra_fees: vec![Fee {
            share: Decimal::percent(EXTRA_FEE_PERCENT),
        }],
    };

    // Create a pool
    suite
        .instantiate_default()
        .add_one_epoch()
        .create_pool(
            &creator,
            asset_infos,
            vec![DECIMALS_6, DECIMALS_6],
            pool_fees.clone(),
            PoolType::ConstantProduct,
            Some(POOL_ID_WHALE_LUNA.to_string()),
            vec![
                coin(ONE_THOUSAND, DENOM_USD),
                coin(OM_STARGATE_BALANCE, DENOM_OM),
            ],
            |result| {
                result.unwrap();
            },
        )
        .create_pool(
            &creator,
            vec![DENOM_USD.to_string(), DENOM_USDC.to_string()],
            vec![DECIMALS_6, DECIMALS_6],
            pool_fees,
            PoolType::StableSwap {
                amp: STABLESWAP_AMP_FACTOR,
            },
            Some(POOL_ID_USD_USDC.to_string()),
            vec![
                coin(ONE_THOUSAND, DENOM_USD),
                coin(OM_STARGATE_BALANCE, DENOM_OM),
            ],
            |result| {
                result.unwrap();
            },
        );

    // Let's try to add liquidity
    suite
        .provide_liquidity(
            &creator,
            POOL_IDENTIFIER_WHALE_LUNA.to_string(),
            None,
            None,
            None,
            None,
            None,
            vec![
                Coin {
                    denom: DENOM_WHALE.to_string(),
                    amount: Uint128::from(ONE_MILLION),
                },
                Coin {
                    denom: DENOM_LUNA.to_string(),
                    amount: Uint128::from(ONE_MILLION),
                },
            ],
            |result| {
                result.unwrap();
            },
        )
        .provide_liquidity(
            &creator,
            POOL_IDENTIFIER_USD_USDC.to_string(),
            None,
            None,
            None,
            None,
            None,
            vec![
                Coin {
                    denom: DENOM_USDC.to_string(),
                    amount: Uint128::from(ONE_MILLION),
                },
                Coin {
                    denom: DENOM_USD.to_string(),
                    amount: Uint128::from(ONE_MILLION),
                },
            ],
            |result| {
                result.unwrap();
            },
        );

    let simulated_return_amount = RefCell::new(Uint128::zero());
    suite.query_simulation(
        POOL_IDENTIFIER_WHALE_LUNA.to_string(),
        Coin {
            denom: DENOM_WHALE.to_string(),
            amount: Uint128::from(ONE_THOUSAND),
        },
        DENOM_LUNA.to_string(),
        |result| {
            let response = result.as_ref().unwrap();

            // the protocol fee is 1% of the output amount
            assert_approx_eq!(
                response.protocol_fee_amount,
                Uint128::new(EXPECTED_PROTOCOL_FEE),
                SIMULATION_TOLERANCE
            );

            // the swap fee is 2% of the output amount
            assert_approx_eq!(
                response.swap_fee_amount,
                Uint128::new(EXPECTED_SWAP_FEE),
                SIMULATION_TOLERANCE
            );

            // the burn fee is 3% of the output amount
            assert_approx_eq!(
                response.burn_fee_amount,
                Uint128::new(EXPECTED_BURN_FEE),
                SIMULATION_TOLERANCE
            );

            // the extra fees are 4% of the output amount
            assert_approx_eq!(
                response.extra_fees_amount,
                Uint128::new(EXPECTED_EXTRA_FEE),
                SIMULATION_TOLERANCE
            );

            *simulated_return_amount.borrow_mut() = response.return_amount;
        },
    );

    // Now Let's try a swap
    suite.swap(
        &creator,
        DENOM_LUNA.to_string(),
        None,
        Some(Decimal::percent(SLIPPAGE_TOLERANCE_PERCENT)),
        None,
        POOL_IDENTIFIER_WHALE_LUNA.to_string(),
        vec![coin(ONE_THOUSAND, DENOM_WHALE.to_string())],
        |result| {
            let mut return_amount = String::new();
            for event in result.unwrap().events {
                if event.ty == "wasm" {
                    for attribute in event.attributes {
                        if attribute.key.as_str() == "return_amount" {
                            return_amount = attribute.value
                        }
                    }
                }
            }

            // return amount must be approximately equal to the value returned by the simulation
            assert_approx_eq!(
                simulated_return_amount.borrow().u128(),
                return_amount.parse::<u128>().unwrap(),
                RETURN_AMOUNT_TOLERANCE
            );
        },
    );

    // now on the stable pool

    suite.query_simulation(
        POOL_IDENTIFIER_USD_USDC.to_string(),
        Coin {
            denom: DENOM_USD.to_string(),
            amount: Uint128::from(ONE_THOUSAND),
        },
        DENOM_USDC.to_string(),
        |result| {
            let response = result.as_ref().unwrap();

            assert_eq!(response.slippage_amount, Uint128::new(100u128));

            // the protocol fee is 1% of the output amount
            assert_approx_eq!(
                response.protocol_fee_amount,
                Uint128::new(EXPECTED_PROTOCOL_FEE),
                SIMULATION_TOLERANCE
            );

            // the swap fee is 2% of the output amount
            assert_approx_eq!(
                response.swap_fee_amount,
                Uint128::new(EXPECTED_SWAP_FEE),
                SIMULATION_TOLERANCE
            );

            // the burn fee is 3% of the output amount
            assert_approx_eq!(
                response.burn_fee_amount,
                Uint128::new(EXPECTED_BURN_FEE),
                SIMULATION_TOLERANCE
            );

            // the extra fees are 4% of the output amount
            assert_approx_eq!(
                response.extra_fees_amount,
                Uint128::new(EXPECTED_EXTRA_FEE),
                SIMULATION_TOLERANCE
            );

            *simulated_return_amount.borrow_mut() = response.return_amount;
        },
    );

    suite.swap(
        &creator,
        DENOM_USDC.to_string(),
        None,
        Some(Decimal::percent(SLIPPAGE_TOLERANCE_PERCENT)),
        None,
        POOL_IDENTIFIER_USD_USDC.to_string(),
        vec![coin(ONE_THOUSAND, DENOM_USD.to_string())],
        |result| {
            let mut return_amount = String::new();
            for event in result.unwrap().events {
                if event.ty == "wasm" {
                    for attribute in event.attributes {
                        if attribute.key.as_str() == "return_amount" {
                            return_amount = attribute.value
                        }
                    }
                }
            }

            // return amount must be approximately equal to the value returned by the simulation
            assert_approx_eq!(
                simulated_return_amount.borrow().u128(),
                return_amount.parse::<u128>().unwrap(),
                RETURN_AMOUNT_TOLERANCE
            );
        },
    );
}

#[test]
fn simulate_swap_operations_query_verification() {
    let mut suite = TestingSuite::default_with_balances(
        vec![
            coin(LARGE_INITIAL_BALANCE_PLUS_ONE, DENOM_OM.to_string()),
            coin(LARGE_INITIAL_BALANCE, DENOM_USDT.to_string()),
            coin(LARGE_INITIAL_BALANCE_PLUS_ONE, DENOM_USD.to_string()),
            coin(LARGE_INITIAL_BALANCE_PLUS_ONE, DENOM_USDC.to_string()),
        ],
        StargateMock::new(vec![coin(OM_STARGATE_BALANCE, DENOM_OM.to_string())]),
    );
    let creator = suite.creator();

    let asset_infos = vec![DENOM_OM.to_string(), DENOM_USDT.to_string()];

    // protocol fee 1%
    // swap fee 2%
    // burn fee 3%
    // extra fee 4%
    let pool_fees = PoolFee {
        protocol_fee: Fee {
            share: Decimal::percent(PROTOCOL_FEE_PERCENT),
        },
        swap_fee: Fee {
            share: Decimal::percent(SWAP_FEE_PERCENT),
        },
        burn_fee: Fee {
            share: Decimal::percent(BURN_FEE_PERCENT),
        },
        extra_fees: vec![Fee {
            share: Decimal::percent(EXTRA_FEE_PERCENT),
        }],
    };

    // Create a pool
    suite
        .instantiate_default()
        .add_one_epoch()
        .create_pool(
            &creator,
            asset_infos,
            vec![DECIMALS_6, DECIMALS_6],
            pool_fees.clone(),
            PoolType::ConstantProduct,
            Some(POOL_ID_OM_USDT.to_string()),
            vec![
                coin(ONE_THOUSAND, DENOM_USD),
                coin(OM_STARGATE_BALANCE, DENOM_OM),
            ],
            |result| {
                result.unwrap();
            },
        )
        .create_pool(
            &creator,
            vec![DENOM_USDT.to_string(), DENOM_USDC.to_string()],
            vec![DECIMALS_6, DECIMALS_6],
            pool_fees,
            PoolType::StableSwap {
                amp: STABLESWAP_AMP_FACTOR,
            },
            Some(POOL_ID_USDT_USDC.to_string()),
            vec![
                coin(ONE_THOUSAND, DENOM_USD),
                coin(OM_STARGATE_BALANCE, DENOM_OM),
            ],
            |result| {
                result.unwrap();
            },
        );

    // Let's try to add liquidity
    suite
        .provide_liquidity(
            &creator,
            POOL_IDENTIFIER_OM_USDT.to_string(),
            None,
            None,
            None,
            None,
            None,
            vec![
                Coin {
                    denom: DENOM_OM.to_string(),
                    amount: Uint128::from(OM_LIQUIDITY_AMOUNT),
                },
                Coin {
                    denom: DENOM_USDT.to_string(),
                    amount: Uint128::from(USDT_OM_POOL_LIQUIDITY),
                },
            ],
            |result| {
                result.unwrap();
            },
        )
        .provide_liquidity(
            &creator,
            POOL_IDENTIFIER_USDT_USDC.to_string(),
            None,
            None,
            None,
            None,
            None,
            vec![
                Coin {
                    denom: DENOM_USDT.to_string(),
                    amount: Uint128::from(USDT_USDC_POOL_LIQUIDITY),
                },
                Coin {
                    denom: DENOM_USDC.to_string(),
                    amount: Uint128::from(USDT_USDC_POOL_LIQUIDITY),
                },
            ],
            |result| {
                result.unwrap();
            },
        );

    let simulated_return_amount = RefCell::new(Uint128::zero());
    suite.query_simulate_swap_operations(
        Uint128::from(ONE_THOUSAND),
        vec![
            SwapOperation::MantraSwap {
                token_in_denom: DENOM_OM.to_string(),
                token_out_denom: DENOM_USDT.to_string(),
                pool_identifier: POOL_IDENTIFIER_OM_USDT.to_string(),
            },
            SwapOperation::MantraSwap {
                token_in_denom: DENOM_USDT.to_string(),
                token_out_denom: DENOM_USDC.to_string(),
                pool_identifier: POOL_IDENTIFIER_USDT_USDC.to_string(),
            },
        ],
        |result| {
            println!("{:?}", result);
            let response = result.unwrap();

            assert_eq!(response.return_amount, Uint128::from(3243u128));
            assert_eq!(
                response.slippage_amounts,
                vec![
                    coin(360u128, DENOM_USDC.to_string()),
                    coin(397u128, DENOM_USDT.to_string())
                ]
            );
            assert_eq!(
                response.swap_fees,
                vec![
                    coin(72u128, DENOM_USDC.to_string()),
                    coin(79u128, DENOM_USDT.to_string()),
                ]
            );
            assert_eq!(
                response.protocol_fees,
                vec![
                    coin(36u128, DENOM_USDC.to_string()),
                    coin(39u128, DENOM_USDT.to_string()),
                ]
            );
            assert_eq!(
                response.burn_fees,
                vec![
                    coin(108u128, DENOM_USDC.to_string()),
                    coin(119u128, DENOM_USDT.to_string()),
                ]
            );
            assert_eq!(
                response.extra_fees,
                vec![
                    coin(144u128, DENOM_USDC.to_string()),
                    coin(159u128, DENOM_USDT.to_string()),
                ]
            );

            simulated_return_amount
                .borrow_mut()
                .clone_from(&response.return_amount);
        },
    );

    // Now Let's try a swap
    suite.execute_swap_operations(
        &creator,
        vec![
            SwapOperation::MantraSwap {
                token_in_denom: DENOM_OM.to_string(),
                token_out_denom: DENOM_USDT.to_string(),
                pool_identifier: POOL_IDENTIFIER_OM_USDT.to_string(),
            },
            SwapOperation::MantraSwap {
                token_in_denom: DENOM_USDT.to_string(),
                token_out_denom: DENOM_USDC.to_string(),
                pool_identifier: POOL_IDENTIFIER_USDT_USDC.to_string(),
            },
        ],
        None,
        None,
        Some(Decimal::percent(SLIPPAGE_TOLERANCE_PERCENT)),
        vec![coin(ONE_THOUSAND, DENOM_OM.to_string())],
        |result| {
            println!("{:?}", result);
            let mut return_amount = String::new();
            for event in result.unwrap().events {
                if event.ty == "wasm" {
                    for attribute in event.attributes {
                        if attribute.key.as_str() == "return_amount" {
                            return_amount = attribute.value
                        }
                    }
                }
            }

            // return amount must be approximately equal to the value returned by the simulation
            assert_approx_eq!(
                simulated_return_amount.borrow().u128(),
                return_amount.parse::<u128>().unwrap(),
                RETURN_AMOUNT_TOLERANCE
            );
        },
    );
}

#[test]
fn reverse_simulation_queries_fees_verification() {
    let mut suite = TestingSuite::default_with_balances(
        vec![
            coin(LARGE_INITIAL_BALANCE_PLUS_ONE, DENOM_WHALE.to_string()),
            coin(LARGE_INITIAL_BALANCE, DENOM_LUNA.to_string()),
            coin(LARGE_INITIAL_BALANCE_PLUS_ONE, DENOM_USD.to_string()),
            coin(LARGE_INITIAL_BALANCE_PLUS_ONE, DENOM_USDC.to_string()),
            coin(LARGE_INITIAL_BALANCE_PLUS_ONE, DENOM_OM.to_string()),
        ],
        StargateMock::new(vec![coin(OM_STARGATE_BALANCE, DENOM_OM.to_string())]),
    );
    let creator = suite.creator();

    let asset_infos = vec![DENOM_WHALE.to_string(), DENOM_LUNA.to_string()];

    // protocol fee 1%
    // swap fee 2%
    // burn fee 3%
    // extra fee 4%
    let pool_fees = PoolFee {
        protocol_fee: Fee {
            share: Decimal::percent(PROTOCOL_FEE_PERCENT),
        },
        swap_fee: Fee {
            share: Decimal::percent(SWAP_FEE_PERCENT),
        },
        burn_fee: Fee {
            share: Decimal::percent(BURN_FEE_PERCENT),
        },
        extra_fees: vec![Fee {
            share: Decimal::percent(EXTRA_FEE_PERCENT),
        }],
    };

    // Create a pool
    suite
        .instantiate_default()
        .add_one_epoch()
        .create_pool(
            &creator,
            asset_infos,
            vec![DECIMALS_6, DECIMALS_6],
            pool_fees.clone(),
            PoolType::ConstantProduct,
            Some(POOL_ID_WHALE_LUNA.to_string()),
            vec![
                coin(ONE_THOUSAND, DENOM_USD),
                coin(OM_STARGATE_BALANCE, DENOM_OM),
            ],
            |result| {
                result.unwrap();
            },
        )
        .create_pool(
            &creator,
            vec![DENOM_USD.to_string(), DENOM_USDC.to_string()],
            vec![DECIMALS_6, DECIMALS_6],
            pool_fees,
            PoolType::StableSwap {
                amp: STABLESWAP_AMP_FACTOR,
            },
            Some(POOL_ID_USD_USDC.to_string()),
            vec![
                coin(ONE_THOUSAND, DENOM_USD),
                coin(OM_STARGATE_BALANCE, DENOM_OM),
            ],
            |result| {
                result.unwrap();
            },
        );

    // Let's try to add liquidity
    suite
        .provide_liquidity(
            &creator,
            POOL_IDENTIFIER_WHALE_LUNA.to_string(),
            None,
            None,
            None,
            None,
            None,
            vec![
                Coin {
                    denom: DENOM_WHALE.to_string(),
                    amount: Uint128::from(WHALE_LIQUIDITY_AMOUNT),
                },
                Coin {
                    denom: DENOM_LUNA.to_string(),
                    amount: Uint128::from(WHALE_LIQUIDITY_AMOUNT),
                },
            ],
            |result| {
                result.unwrap();
            },
        )
        .provide_liquidity(
            &creator,
            POOL_IDENTIFIER_USD_USDC.to_string(),
            None,
            None,
            None,
            None,
            None,
            vec![
                Coin {
                    denom: DENOM_USDC.to_string(),
                    amount: Uint128::from(WHALE_LIQUIDITY_AMOUNT),
                },
                Coin {
                    denom: DENOM_USD.to_string(),
                    amount: Uint128::from(WHALE_LIQUIDITY_AMOUNT),
                },
            ],
            |result| {
                result.unwrap();
            },
        );

    let simulated_offer_amount = RefCell::new(Uint128::zero());
    suite.query_reverse_simulation(
        POOL_IDENTIFIER_WHALE_LUNA.to_string(),
        Coin {
            denom: DENOM_WHALE.to_string(),
            // reuse the value of the previous test
            amount: Uint128::from(REVERSE_SIMULATION_AMOUNT_WHALE),
        },
        DENOM_LUNA.to_string(),
        |result| {
            println!(">>>> {:?}", result);
            let response = result.as_ref().unwrap();

            // the fees should be the same as the previous test, as we requested
            // the reverse simulation for the value we obtained before

            assert_approx_eq!(
                response.protocol_fee_amount,
                Uint128::new(EXPECTED_PROTOCOL_FEE),
                SIMULATION_TOLERANCE
            );

            assert_approx_eq!(
                response.swap_fee_amount,
                Uint128::new(EXPECTED_SWAP_FEE),
                SIMULATION_TOLERANCE
            );

            assert_approx_eq!(
                response.burn_fee_amount,
                Uint128::new(EXPECTED_BURN_FEE),
                SIMULATION_TOLERANCE
            );

            assert_approx_eq!(
                response.extra_fees_amount,
                Uint128::new(EXPECTED_EXTRA_FEE),
                SIMULATION_TOLERANCE
            );

            *simulated_offer_amount.borrow_mut() = response.offer_amount;
        },
    );

    // Another swap but this time the other way around
    suite.swap(
        &creator,
        DENOM_WHALE.to_string(),
        None,
        Some(Decimal::percent(11)),
        None,
        POOL_IDENTIFIER_WHALE_LUNA.to_string(),
        vec![coin(
            simulated_offer_amount.borrow().u128(),
            DENOM_LUNA.to_string(),
        )],
        |result| {
            let mut return_amount = String::new();
            let mut offer_amount = String::new();

            for event in result.unwrap().events {
                if event.ty == "wasm" {
                    for attribute in event.attributes {
                        match attribute.key.as_str() {
                            "return_amount" => return_amount = attribute.value,
                            "offer_amount" => offer_amount = attribute.value,
                            _ => {}
                        }
                    }
                }
            }
            assert_approx_eq!(
                simulated_offer_amount.borrow().u128(),
                offer_amount.parse::<u128>().unwrap(),
                OFFER_AMOUNT_TOLERANCE
            );

            assert_approx_eq!(
                REVERSE_SIMULATION_AMOUNT_WHALE,
                return_amount.parse::<u128>().unwrap(),
                RETURN_AMOUNT_TOLERANCE_WHALE
            );
        },
    );

    // now on the stable pool
    suite.query_reverse_simulation(
        POOL_IDENTIFIER_USD_USDC.to_string(),
        Coin {
            denom: DENOM_USD.to_string(),
            // reuse the value of the previous test
            amount: Uint128::from(REVERSE_SIMULATION_AMOUNT_USD),
        },
        DENOM_USDC.to_string(),
        |result| {
            println!(">>>> {:?}", result);
            let response = result.as_ref().unwrap();

            // the fees should be the same as the previous test, as we requested
            // the reverse simulation for the value we obtained before

            assert_approx_eq!(
                response.protocol_fee_amount,
                Uint128::new(EXPECTED_PROTOCOL_FEE),
                SIMULATION_TOLERANCE
            );

            assert_approx_eq!(
                response.swap_fee_amount,
                Uint128::new(EXPECTED_SWAP_FEE),
                SIMULATION_TOLERANCE
            );

            assert_approx_eq!(
                response.burn_fee_amount,
                Uint128::new(EXPECTED_BURN_FEE),
                SIMULATION_TOLERANCE
            );

            assert_approx_eq!(
                response.extra_fees_amount,
                Uint128::new(EXPECTED_EXTRA_FEE),
                SIMULATION_TOLERANCE
            );

            *simulated_offer_amount.borrow_mut() = response.offer_amount;
        },
    );

    // Another swap but this time the other way around
    suite.swap(
        &creator,
        DENOM_USD.to_string(),
        None,
        Some(Decimal::percent(SLIPPAGE_TOLERANCE_PERCENT)),
        None,
        POOL_IDENTIFIER_USD_USDC.to_string(),
        vec![coin(
            simulated_offer_amount.borrow().u128(),
            DENOM_USDC.to_string(),
        )],
        |result| {
            let mut return_amount = String::new();
            let mut offer_amount = String::new();

            for event in result.unwrap().events {
                if event.ty == "wasm" {
                    for attribute in event.attributes {
                        match attribute.key.as_str() {
                            "return_amount" => return_amount = attribute.value,
                            "offer_amount" => offer_amount = attribute.value,
                            _ => {}
                        }
                    }
                }
            }
            assert_approx_eq!(
                simulated_offer_amount.borrow().u128(),
                offer_amount.parse::<u128>().unwrap(),
                OFFER_AMOUNT_TOLERANCE
            );

            assert_approx_eq!(
                REVERSE_SIMULATION_AMOUNT_USD,
                return_amount.parse::<u128>().unwrap(),
                RETURN_AMOUNT_TOLERANCE_WHALE
            );
        },
    );
}

#[test]
fn reverse_simulate_swap_operations_query_verification() {
    let mut suite = TestingSuite::default_with_balances(
        vec![
            coin(LARGE_INITIAL_BALANCE_PLUS_ONE, DENOM_OM.to_string()),
            coin(LARGE_INITIAL_BALANCE, DENOM_USDT.to_string()),
            coin(LARGE_INITIAL_BALANCE_PLUS_ONE, DENOM_USD.to_string()),
            coin(LARGE_INITIAL_BALANCE_PLUS_ONE, DENOM_USDC.to_string()),
        ],
        StargateMock::new(vec![coin(OM_STARGATE_BALANCE, DENOM_OM.to_string())]),
    );
    let creator = suite.creator();

    let asset_infos = vec![DENOM_OM.to_string(), DENOM_USDT.to_string()];

    // protocol fee 1%
    // swap fee 2%
    // burn fee 3%
    // extra fee 4%
    let pool_fees = PoolFee {
        protocol_fee: Fee {
            share: Decimal::percent(PROTOCOL_FEE_PERCENT),
        },
        swap_fee: Fee {
            share: Decimal::percent(SWAP_FEE_PERCENT),
        },
        burn_fee: Fee {
            share: Decimal::percent(BURN_FEE_PERCENT),
        },
        extra_fees: vec![Fee {
            share: Decimal::percent(EXTRA_FEE_PERCENT),
        }],
    };

    // Create a pool
    suite
        .instantiate_default()
        .add_one_epoch()
        .create_pool(
            &creator,
            asset_infos,
            vec![DECIMALS_6, DECIMALS_6],
            pool_fees.clone(),
            PoolType::ConstantProduct,
            Some(POOL_ID_OM_USDT.to_string()),
            vec![
                coin(ONE_THOUSAND, DENOM_USD),
                coin(OM_STARGATE_BALANCE, DENOM_OM),
            ],
            |result| {
                result.unwrap();
            },
        )
        .create_pool(
            &creator,
            vec![DENOM_USDT.to_string(), DENOM_USDC.to_string()],
            vec![DECIMALS_6, DECIMALS_6],
            pool_fees,
            PoolType::StableSwap {
                amp: STABLESWAP_AMP_FACTOR,
            },
            Some(POOL_ID_USDT_USDC.to_string()),
            vec![
                coin(ONE_THOUSAND, DENOM_USD),
                coin(OM_STARGATE_BALANCE, DENOM_OM),
            ],
            |result| {
                result.unwrap();
            },
        );

    // Let's try to add liquidity
    suite
        .provide_liquidity(
            &creator,
            POOL_IDENTIFIER_OM_USDT.to_string(),
            None,
            None,
            None,
            None,
            None,
            vec![
                Coin {
                    denom: DENOM_OM.to_string(),
                    amount: Uint128::from(OM_LIQUIDITY_AMOUNT),
                },
                Coin {
                    denom: DENOM_USDT.to_string(),
                    amount: Uint128::from(USDT_OM_POOL_LIQUIDITY),
                },
            ],
            |result| {
                result.unwrap();
            },
        )
        .provide_liquidity(
            &creator,
            POOL_IDENTIFIER_USDT_USDC.to_string(),
            None,
            None,
            None,
            None,
            None,
            vec![
                Coin {
                    denom: DENOM_USDT.to_string(),
                    amount: Uint128::from(USDT_USDC_POOL_LIQUIDITY),
                },
                Coin {
                    denom: DENOM_USDC.to_string(),
                    amount: Uint128::from(USDT_USDC_POOL_LIQUIDITY),
                },
            ],
            |result| {
                result.unwrap();
            },
        );

    let simulated_input_amount = RefCell::new(Uint128::zero());
    let desired_output_amount = Uint128::from(3240u128);
    suite.query_reverse_simulate_swap_operations(
        desired_output_amount,
        vec![
            SwapOperation::MantraSwap {
                token_in_denom: DENOM_OM.to_string(),
                token_out_denom: DENOM_USDT.to_string(),
                pool_identifier: POOL_IDENTIFIER_OM_USDT.to_string(),
            },
            SwapOperation::MantraSwap {
                token_in_denom: DENOM_USDT.to_string(),
                token_out_denom: DENOM_USDC.to_string(),
                pool_identifier: POOL_IDENTIFIER_USDT_USDC.to_string(),
            },
        ],
        |result| {
            let response = result.unwrap();

            // this is the value we got in the previous test for the regular simulation
            assert_approx_eq!(response.offer_amount, Uint128::from(ONE_THOUSAND), "0.001");
            assert_eq!(
                response.slippage_amounts,
                vec![
                    coin(1u128, DENOM_USDC.to_string()),
                    coin(1u128, DENOM_USDT.to_string()),
                ]
            );
            assert_eq!(
                response.swap_fees,
                vec![
                    coin(71u128, DENOM_USDC.to_string()),
                    coin(79u128, DENOM_USDT.to_string()),
                ]
            );
            assert_eq!(
                response.protocol_fees,
                vec![
                    coin(35u128, DENOM_USDC.to_string()),
                    coin(39u128, DENOM_USDT.to_string()),
                ]
            );
            assert_eq!(
                response.burn_fees,
                vec![
                    coin(107u128, DENOM_USDC.to_string()),
                    coin(119u128, DENOM_USDT.to_string()),
                ]
            );
            assert_eq!(
                response.extra_fees,
                vec![
                    coin(143u128, DENOM_USDC.to_string()),
                    coin(159u128, DENOM_USDT.to_string()),
                ]
            );

            simulated_input_amount
                .borrow_mut()
                .clone_from(&response.offer_amount);
        },
    );

    // Now Let's try a swap
    suite.execute_swap_operations(
        &creator,
        vec![
            SwapOperation::MantraSwap {
                token_in_denom: DENOM_OM.to_string(),
                token_out_denom: DENOM_USDT.to_string(),
                pool_identifier: POOL_IDENTIFIER_OM_USDT.to_string(),
            },
            SwapOperation::MantraSwap {
                token_in_denom: DENOM_USDT.to_string(),
                token_out_denom: DENOM_USDC.to_string(),
                pool_identifier: POOL_IDENTIFIER_USDT_USDC.to_string(),
            },
        ],
        None,
        None,
        Some(Decimal::percent(SLIPPAGE_TOLERANCE_PERCENT)),
        vec![coin(
            simulated_input_amount.borrow().u128(),
            DENOM_OM.to_string(),
        )],
        |result| {
            let mut return_amount = String::new();
            for event in result.unwrap().events {
                if event.ty == "wasm" {
                    for attribute in event.attributes {
                        if attribute.key.as_str() == "return_amount" {
                            return_amount = attribute.value
                        }
                    }
                }
            }

            // return amount must be approximately equal (a bit higher) to the value returned by the simulation
            assert_approx_eq!(
                desired_output_amount.u128(),
                return_amount.parse::<u128>().unwrap(),
                "0.001"
            );
        },
    );
}
