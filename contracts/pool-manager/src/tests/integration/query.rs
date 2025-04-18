use std::cell::RefCell;

use cosmwasm_std::{assert_approx_eq, coin, Coin, Decimal, Uint128};
use mantra_common_testing::multi_test::stargate_mock::StargateMock;
use mantra_dex_std::fee::{Fee, PoolFee};
use mantra_dex_std::pool_manager::{PoolType, SwapOperation};

use crate::tests::suite::TestingSuite;

#[test]
fn simulation_queries_fees_verification() {
    let mut suite = TestingSuite::default_with_balances(
        vec![
            coin(1_000_000_001u128, "uwhale".to_string()),
            coin(1_000_000_000u128, "uluna".to_string()),
            coin(1_000_000_001u128, "uusd".to_string()),
            coin(1_000_000_001u128, "uusdc".to_string()),
            coin(1_000_000_001u128, "uom".to_string()),
        ],
        StargateMock::new(vec![coin(8888u128, "uom".to_string())]),
    );
    let creator = suite.creator();
    let _other = suite.senders[1].clone();
    let _unauthorized = suite.senders[2].clone();
    // Asset infos with uwhale and uluna

    let asset_infos = vec!["uwhale".to_string(), "uluna".to_string()];

    // protocol fee 1%
    // swap fee 2%
    // burn fee 3%
    // extra fee 4%
    let pool_fees = PoolFee {
        protocol_fee: Fee {
            share: Decimal::percent(1u64),
        },
        swap_fee: Fee {
            share: Decimal::percent(2u64),
        },
        burn_fee: Fee {
            share: Decimal::percent(3u64),
        },
        extra_fees: vec![Fee {
            share: Decimal::percent(4u64),
        }],
    };

    // Create a pool
    suite
        .instantiate_default()
        .add_one_epoch()
        .create_pool(
            &creator,
            asset_infos,
            vec![6u8, 6u8],
            pool_fees.clone(),
            PoolType::ConstantProduct,
            Some("whale.uluna".to_string()),
            vec![coin(1000, "uusd"), coin(8888, "uom")],
            |result| {
                result.unwrap();
            },
        )
        .create_pool(
            &creator,
            vec!["uusd".to_string(), "uusdc".to_string()],
            vec![6u8, 6u8],
            pool_fees,
            PoolType::StableSwap { amp: 85 },
            Some("uusd.uusdc".to_string()),
            vec![coin(1000, "uusd"), coin(8888, "uom")],
            |result| {
                result.unwrap();
            },
        );

    // Let's try to add liquidity
    suite
        .provide_liquidity(
            &creator,
            "o.whale.uluna".to_string(),
            None,
            None,
            None,
            None,
            None,
            vec![
                Coin {
                    denom: "uwhale".to_string(),
                    amount: Uint128::from(1000000u128),
                },
                Coin {
                    denom: "uluna".to_string(),
                    amount: Uint128::from(1000000u128),
                },
            ],
            |result| {
                result.unwrap();
            },
        )
        .provide_liquidity(
            &creator,
            "o.uusd.uusdc".to_string(),
            None,
            None,
            None,
            None,
            None,
            vec![
                Coin {
                    denom: "uusdc".to_string(),
                    amount: Uint128::from(1000000u128),
                },
                Coin {
                    denom: "uusd".to_string(),
                    amount: Uint128::from(1000000u128),
                },
            ],
            |result| {
                result.unwrap();
            },
        );

    let simulated_return_amount = RefCell::new(Uint128::zero());
    suite.query_simulation(
        "o.whale.uluna".to_string(),
        Coin {
            denom: "uwhale".to_string(),
            amount: Uint128::from(1000u128),
        },
        "uluna".to_string(),
        |result| {
            let response = result.as_ref().unwrap();

            // the protocol fee is 1% of the output amount
            assert_approx_eq!(response.protocol_fee_amount, Uint128::new(10u128), "0.1");

            // the swap fee is 2% of the output amount
            assert_approx_eq!(response.swap_fee_amount, Uint128::new(20u128), "0.1");

            // the burn fee is 3% of the output amount
            assert_approx_eq!(response.burn_fee_amount, Uint128::new(30u128), "0.1");

            // the extra fees are 4% of the output amount
            assert_approx_eq!(response.extra_fees_amount, Uint128::new(40u128), "0.1");

            *simulated_return_amount.borrow_mut() = response.return_amount;
        },
    );

    // Now Let's try a swap
    suite.swap(
        &creator,
        "uluna".to_string(),
        None,
        Some(Decimal::percent(10)),
        None,
        "o.whale.uluna".to_string(),
        vec![coin(1000u128, "uwhale".to_string())],
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
                "0.00000001"
            );
        },
    );

    // now on the stable pool

    suite.query_simulation(
        "o.uusd.uusdc".to_string(),
        Coin {
            denom: "uusd".to_string(),
            amount: Uint128::from(1000u128),
        },
        "uusdc".to_string(),
        |result| {
            let response = result.as_ref().unwrap();

            assert_eq!(response.spread_amount, Uint128::new(100u128));

            // the protocol fee is 1% of the output amount
            assert_approx_eq!(response.protocol_fee_amount, Uint128::new(10u128), "0.1");

            // the swap fee is 2% of the output amount
            assert_approx_eq!(response.swap_fee_amount, Uint128::new(20u128), "0.1");

            // the burn fee is 3% of the output amount
            assert_approx_eq!(response.burn_fee_amount, Uint128::new(30u128), "0.1");

            // the extra fees are 4% of the output amount
            assert_approx_eq!(response.extra_fees_amount, Uint128::new(40u128), "0.1");

            *simulated_return_amount.borrow_mut() = response.return_amount;
        },
    );

    suite.swap(
        &creator,
        "uusdc".to_string(),
        None,
        Some(Decimal::percent(10)),
        None,
        "o.uusd.uusdc".to_string(),
        vec![coin(1000u128, "uusd".to_string())],
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
                "0.00000001"
            );
        },
    );
}

#[test]
fn simulate_swap_operations_query_verification() {
    let mut suite = TestingSuite::default_with_balances(
        vec![
            coin(1_000_000_000_001u128, "uom".to_string()),
            coin(1_000_000_000_000u128, "uusdt".to_string()),
            coin(1_000_000_000_001u128, "uusd".to_string()),
            coin(1_000_000_000_001u128, "uusdc".to_string()),
        ],
        StargateMock::new(vec![coin(8888u128, "uom".to_string())]),
    );
    let creator = suite.creator();

    let asset_infos = vec!["uom".to_string(), "uusdt".to_string()];

    // protocol fee 1%
    // swap fee 2%
    // burn fee 3%
    // extra fee 4%
    let pool_fees = PoolFee {
        protocol_fee: Fee {
            share: Decimal::percent(1u64),
        },
        swap_fee: Fee {
            share: Decimal::percent(2u64),
        },
        burn_fee: Fee {
            share: Decimal::percent(3u64),
        },
        extra_fees: vec![Fee {
            share: Decimal::percent(4u64),
        }],
    };

    // Create a pool
    suite
        .instantiate_default()
        .add_one_epoch()
        .create_pool(
            &creator,
            asset_infos,
            vec![6u8, 6u8],
            pool_fees.clone(),
            PoolType::ConstantProduct,
            Some("uom.uusdt".to_string()),
            vec![coin(1000, "uusd"), coin(8888, "uom")],
            |result| {
                result.unwrap();
            },
        )
        .create_pool(
            &creator,
            vec!["uusdt".to_string(), "uusdc".to_string()],
            vec![6u8, 6u8],
            pool_fees,
            PoolType::StableSwap { amp: 85 },
            Some("uusdt.uusdc".to_string()),
            vec![coin(1000, "uusd"), coin(8888, "uom")],
            |result| {
                result.unwrap();
            },
        );

    // Let's try to add liquidity
    suite
        .provide_liquidity(
            &creator,
            "o.uom.uusdt".to_string(),
            None,
            None,
            None,
            None,
            None,
            vec![
                Coin {
                    denom: "uom".to_string(),
                    amount: Uint128::from(1_000_000_000u128),
                },
                Coin {
                    denom: "uusdt".to_string(),
                    amount: Uint128::from(4_000_000_000u128),
                },
            ],
            |result| {
                result.unwrap();
            },
        )
        .provide_liquidity(
            &creator,
            "o.uusdt.uusdc".to_string(),
            None,
            None,
            None,
            None,
            None,
            vec![
                Coin {
                    denom: "uusdt".to_string(),
                    amount: Uint128::from(1_000_000_000u128),
                },
                Coin {
                    denom: "uusdc".to_string(),
                    amount: Uint128::from(1_000_000_000u128),
                },
            ],
            |result| {
                result.unwrap();
            },
        );

    let simulated_return_amount = RefCell::new(Uint128::zero());
    suite.query_simulate_swap_operations(
        Uint128::from(1000u128),
        vec![
            SwapOperation::MantraSwap {
                token_in_denom: "uom".to_string(),
                token_out_denom: "uusdt".to_string(),
                pool_identifier: "o.uom.uusdt".to_string(),
            },
            SwapOperation::MantraSwap {
                token_in_denom: "uusdt".to_string(),
                token_out_denom: "uusdc".to_string(),
                pool_identifier: "o.uusdt.uusdc".to_string(),
            },
        ],
        |result| {
            println!("{:?}", result);
            let response = result.unwrap();

            assert_eq!(response.return_amount, Uint128::from(3243u128));
            assert_eq!(
                response.spreads,
                vec![
                    coin(360u128, "uusdc".to_string()),
                    coin(397u128, "uusdt".to_string())
                ]
            );
            assert_eq!(
                response.swap_fees,
                vec![
                    coin(72u128, "uusdc".to_string()),
                    coin(79u128, "uusdt".to_string()),
                ]
            );
            assert_eq!(
                response.protocol_fees,
                vec![
                    coin(36u128, "uusdc".to_string()),
                    coin(39u128, "uusdt".to_string()),
                ]
            );
            assert_eq!(
                response.burn_fees,
                vec![
                    coin(108u128, "uusdc".to_string()),
                    coin(119u128, "uusdt".to_string()),
                ]
            );
            assert_eq!(
                response.extra_fees,
                vec![
                    coin(144u128, "uusdc".to_string()),
                    coin(159u128, "uusdt".to_string()),
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
                token_in_denom: "uom".to_string(),
                token_out_denom: "uusdt".to_string(),
                pool_identifier: "o.uom.uusdt".to_string(),
            },
            SwapOperation::MantraSwap {
                token_in_denom: "uusdt".to_string(),
                token_out_denom: "uusdc".to_string(),
                pool_identifier: "o.uusdt.uusdc".to_string(),
            },
        ],
        None,
        None,
        Some(Decimal::percent(10)),
        vec![coin(1000u128, "uom".to_string())],
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
                "0.00000001"
            );
        },
    );
}

#[test]
fn reverse_simulation_queries_fees_verification() {
    let mut suite = TestingSuite::default_with_balances(
        vec![
            coin(1_000_000_000_001u128, "uwhale".to_string()),
            coin(1_000_000_000_000u128, "uluna".to_string()),
            coin(1_000_000_000_001u128, "uusd".to_string()),
            coin(1_000_000_000_001u128, "uusdc".to_string()),
            coin(1_000_000_000_001u128, "uom".to_string()),
        ],
        StargateMock::new(vec![coin(8888u128, "uom".to_string())]),
    );
    let creator = suite.creator();

    let asset_infos = vec!["uwhale".to_string(), "uluna".to_string()];

    // protocol fee 1%
    // swap fee 2%
    // burn fee 3%
    // extra fee 4%
    let pool_fees = PoolFee {
        protocol_fee: Fee {
            share: Decimal::percent(1u64),
        },
        swap_fee: Fee {
            share: Decimal::percent(2u64),
        },
        burn_fee: Fee {
            share: Decimal::percent(3u64),
        },
        extra_fees: vec![Fee {
            share: Decimal::percent(4u64),
        }],
    };

    // Create a pool
    suite
        .instantiate_default()
        .add_one_epoch()
        .create_pool(
            &creator,
            asset_infos,
            vec![6u8, 6u8],
            pool_fees.clone(),
            PoolType::ConstantProduct,
            Some("whale.uluna".to_string()),
            vec![coin(1000, "uusd"), coin(8888, "uom")],
            |result| {
                result.unwrap();
            },
        )
        .create_pool(
            &creator,
            vec!["uusd".to_string(), "uusdc".to_string()],
            vec![6u8, 6u8],
            pool_fees,
            PoolType::StableSwap { amp: 85 },
            Some("uusd.uusdc".to_string()),
            vec![coin(1000, "uusd"), coin(8888, "uom")],
            |result| {
                result.unwrap();
            },
        );

    // Let's try to add liquidity
    suite
        .provide_liquidity(
            &creator,
            "o.whale.uluna".to_string(),
            None,
            None,
            None,
            None,
            None,
            vec![
                Coin {
                    denom: "uwhale".to_string(),
                    amount: Uint128::from(1000000000u128),
                },
                Coin {
                    denom: "uluna".to_string(),
                    amount: Uint128::from(1000000000u128),
                },
            ],
            |result| {
                result.unwrap();
            },
        )
        .provide_liquidity(
            &creator,
            "o.uusd.uusdc".to_string(),
            None,
            None,
            None,
            None,
            None,
            vec![
                Coin {
                    denom: "uusdc".to_string(),
                    amount: Uint128::from(1000000000u128),
                },
                Coin {
                    denom: "uusd".to_string(),
                    amount: Uint128::from(1000000000u128),
                },
            ],
            |result| {
                result.unwrap();
            },
        );

    let simulated_offer_amount = RefCell::new(Uint128::zero());
    suite.query_reverse_simulation(
        "o.whale.uluna".to_string(),
        Coin {
            denom: "uwhale".to_string(),
            // reuse the value of the previous test
            amount: Uint128::from(903u128),
        },
        "uluna".to_string(),
        |result| {
            println!(">>>> {:?}", result);
            let response = result.as_ref().unwrap();

            // the fees should be the same as the previous test, as we requested
            // the reverse simulation for the value we obtained before

            assert_approx_eq!(response.protocol_fee_amount, Uint128::new(10u128), "0.1");

            assert_approx_eq!(response.swap_fee_amount, Uint128::new(20u128), "0.1");

            assert_approx_eq!(response.burn_fee_amount, Uint128::new(30u128), "0.1");

            assert_approx_eq!(response.extra_fees_amount, Uint128::new(40u128), "0.1");

            *simulated_offer_amount.borrow_mut() = response.offer_amount;
        },
    );

    // Another swap but this time the other way around
    suite.swap(
        &creator,
        "uwhale".to_string(),
        None,
        Some(Decimal::percent(11)),
        None,
        "o.whale.uluna".to_string(),
        vec![coin(
            simulated_offer_amount.borrow().u128(),
            "uluna".to_string(),
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
                "0.002"
            );

            assert_approx_eq!(903u128, return_amount.parse::<u128>().unwrap(), "0.002");
        },
    );

    // now on the stable pool
    suite.query_reverse_simulation(
        "o.uusd.uusdc".to_string(),
        Coin {
            denom: "uusd".to_string(),
            // reuse the value of the previous test
            amount: Uint128::from(900u128),
        },
        "uusdc".to_string(),
        |result| {
            println!(">>>> {:?}", result);
            let response = result.as_ref().unwrap();

            // the fees should be the same as the previous test, as we requested
            // the reverse simulation for the value we obtained before

            assert_approx_eq!(response.protocol_fee_amount, Uint128::new(10u128), "0.1");

            assert_approx_eq!(response.swap_fee_amount, Uint128::new(20u128), "0.1");

            assert_approx_eq!(response.burn_fee_amount, Uint128::new(30u128), "0.1");

            assert_approx_eq!(response.extra_fees_amount, Uint128::new(40u128), "0.1");

            *simulated_offer_amount.borrow_mut() = response.offer_amount;
        },
    );

    // Another swap but this time the other way around
    suite.swap(
        &creator,
        "uusd".to_string(),
        None,
        Some(Decimal::percent(10)),
        None,
        "o.uusd.uusdc".to_string(),
        vec![coin(
            simulated_offer_amount.borrow().u128(),
            "uusdc".to_string(),
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
                "0.002"
            );

            assert_approx_eq!(900u128, return_amount.parse::<u128>().unwrap(), "0.002");
        },
    );
}

#[test]
fn reverse_simulate_swap_operations_query_verification() {
    let mut suite = TestingSuite::default_with_balances(
        vec![
            coin(1_000_000_000_001u128, "uom".to_string()),
            coin(1_000_000_000_000u128, "uusdt".to_string()),
            coin(1_000_000_000_001u128, "uusd".to_string()),
            coin(1_000_000_000_001u128, "uusdc".to_string()),
        ],
        StargateMock::new(vec![coin(8888u128, "uom".to_string())]),
    );
    let creator = suite.creator();

    let asset_infos = vec!["uom".to_string(), "uusdt".to_string()];

    // protocol fee 1%
    // swap fee 2%
    // burn fee 3%
    // extra fee 4%
    let pool_fees = PoolFee {
        protocol_fee: Fee {
            share: Decimal::percent(1u64),
        },
        swap_fee: Fee {
            share: Decimal::percent(2u64),
        },
        burn_fee: Fee {
            share: Decimal::percent(3u64),
        },
        extra_fees: vec![Fee {
            share: Decimal::percent(4u64),
        }],
    };

    // Create a pool
    suite
        .instantiate_default()
        .add_one_epoch()
        .create_pool(
            &creator,
            asset_infos,
            vec![6u8, 6u8],
            pool_fees.clone(),
            PoolType::ConstantProduct,
            Some("uom.uusdt".to_string()),
            vec![coin(1000, "uusd"), coin(8888, "uom")],
            |result| {
                result.unwrap();
            },
        )
        .create_pool(
            &creator,
            vec!["uusdt".to_string(), "uusdc".to_string()],
            vec![6u8, 6u8],
            pool_fees,
            PoolType::StableSwap { amp: 85 },
            Some("uusdt.uusdc".to_string()),
            vec![coin(1000, "uusd"), coin(8888, "uom")],
            |result| {
                result.unwrap();
            },
        );

    // Let's try to add liquidity
    suite
        .provide_liquidity(
            &creator,
            "o.uom.uusdt".to_string(),
            None,
            None,
            None,
            None,
            None,
            vec![
                Coin {
                    denom: "uom".to_string(),
                    amount: Uint128::from(1000000000u128),
                },
                Coin {
                    denom: "uusdt".to_string(),
                    amount: Uint128::from(4000000000u128),
                },
            ],
            |result| {
                result.unwrap();
            },
        )
        .provide_liquidity(
            &creator,
            "o.uusdt.uusdc".to_string(),
            None,
            None,
            None,
            None,
            None,
            vec![
                Coin {
                    denom: "uusdt".to_string(),
                    amount: Uint128::from(1000000000u128),
                },
                Coin {
                    denom: "uusdc".to_string(),
                    amount: Uint128::from(1000000000u128),
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
                token_in_denom: "uom".to_string(),
                token_out_denom: "uusdt".to_string(),
                pool_identifier: "o.uom.uusdt".to_string(),
            },
            SwapOperation::MantraSwap {
                token_in_denom: "uusdt".to_string(),
                token_out_denom: "uusdc".to_string(),
                pool_identifier: "o.uusdt.uusdc".to_string(),
            },
        ],
        |result| {
            let response = result.unwrap();

            // this is the value we got in the previous test for the regular simulation
            assert_approx_eq!(response.offer_amount, Uint128::from(1000u128), "0.001");
            assert_eq!(
                response.spreads,
                vec![
                    coin(1u128, "uusdc".to_string()),
                    coin(1u128, "uusdt".to_string()),
                ]
            );
            assert_eq!(
                response.swap_fees,
                vec![
                    coin(71u128, "uusdc".to_string()),
                    coin(79u128, "uusdt".to_string()),
                ]
            );
            assert_eq!(
                response.protocol_fees,
                vec![
                    coin(35u128, "uusdc".to_string()),
                    coin(39u128, "uusdt".to_string()),
                ]
            );
            assert_eq!(
                response.burn_fees,
                vec![
                    coin(107u128, "uusdc".to_string()),
                    coin(119u128, "uusdt".to_string()),
                ]
            );
            assert_eq!(
                response.extra_fees,
                vec![
                    coin(143u128, "uusdc".to_string()),
                    coin(159u128, "uusdt".to_string()),
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
                token_in_denom: "uom".to_string(),
                token_out_denom: "uusdt".to_string(),
                pool_identifier: "o.uom.uusdt".to_string(),
            },
            SwapOperation::MantraSwap {
                token_in_denom: "uusdt".to_string(),
                token_out_denom: "uusdc".to_string(),
                pool_identifier: "o.uusdt.uusdc".to_string(),
            },
        ],
        None,
        None,
        Some(Decimal::percent(10)),
        vec![coin(
            simulated_input_amount.borrow().u128(),
            "uom".to_string(),
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
