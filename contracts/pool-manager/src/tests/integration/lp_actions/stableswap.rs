use std::cell::RefCell;

use cosmwasm_std::{assert_approx_eq, coin, Coin, Decimal, Uint128};
use mantra_common_testing::multi_test::stargate_mock::StargateMock;
use mantra_dex_std::{
    fee::{Fee, PoolFee},
    lp_common::MINIMUM_LIQUIDITY_AMOUNT,
    pool_manager::PoolType,
};

use crate::{tests::suite::TestingSuite, ContractError};

#[test]
fn provide_liquidity_stable_swap() {
    let mut suite = TestingSuite::default_with_balances(
        vec![
            coin(1_000_000_001u128, "uwhale".to_string()),
            coin(1_000_000_000u128, "uluna".to_string()),
            coin(1_000_000_001u128, "uusd".to_string()),
            coin(1_000_000_001u128, "uom".to_string()),
        ],
        StargateMock::new(vec![coin(8888u128, "uom".to_string())]),
    );
    let creator = suite.creator();
    let _other = suite.senders[1].clone();
    let _unauthorized = suite.senders[2].clone();
    // Asset infos with uwhale and uluna

    let asset_infos = vec![
        "uwhale".to_string(),
        "uluna".to_string(),
        "uusd".to_string(),
    ];

    // Protocol fee is 0.01% and swap fee is 0.02% and burn fee is 0%
    let pool_fees = PoolFee {
        protocol_fee: Fee {
            share: Decimal::from_ratio(1u128, 1000u128),
        },
        swap_fee: Fee {
            share: Decimal::from_ratio(1u128, 10_000_u128),
        },
        burn_fee: Fee {
            share: Decimal::zero(),
        },
        extra_fees: vec![],
    };

    // Create a pool
    suite.instantiate_default().create_pool(
        &creator,
        asset_infos,
        vec![6u8, 6u8, 6u8],
        pool_fees,
        PoolType::StableSwap { amp: 100 },
        Some("whale.uluna.uusd".to_string()),
        vec![coin(1000, "uusd"), coin(8888, "uom")],
        |result| {
            result.unwrap();
        },
    );

    // Let's try to add liquidity
    suite.provide_liquidity(
        &creator,
        "o.whale.uluna.uusd".to_string(),
        None,
        None,
        None,
        None,
        None,
        vec![
            Coin {
                denom: "uwhale".to_string(),
                amount: Uint128::from(1_000_000u128),
            },
            Coin {
                denom: "uluna".to_string(),
                amount: Uint128::from(1_000_000u128),
            },
            Coin {
                denom: "uusd".to_string(),
                amount: Uint128::from(1_000_000u128),
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
        "o.whale.uluna.uusd".to_string(),
        Coin {
            denom: "uwhale".to_string(),
            amount: Uint128::from(1_000u128),
        },
        "uluna".to_string(),
        |result| {
            *simulated_return_amount.borrow_mut() = result.unwrap().return_amount;
        },
    );

    // Now Let's try a swap
    suite.swap(
        &creator,
        "uluna".to_string(),
        None,
        None,
        None,
        "o.whale.uluna.uusd".to_string(),
        vec![coin(1_000u128, "uwhale".to_string())],
        |result| {
            // Find the key with 'offer_amount' and the key with 'return_amount'
            // Ensure that the offer amount is 1000 and the return amount is greater than 0
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
            // Because the Pool was created and 1_000_000 of each token has been provided as liquidity
            // Assuming no fees we should expect a small swap of 1000 to result in not too much slippage
            // Expect 1000 give or take 0.002 difference
            // Once fees are added and being deducted properly only the "0.002" should be changed.
            assert_approx_eq!(
                offer_amount.parse::<u128>().unwrap(),
                return_amount.parse::<u128>().unwrap(),
                "0.002"
            );
            assert_approx_eq!(
                simulated_return_amount.borrow().u128(),
                return_amount.parse::<u128>().unwrap(),
                "0.002"
            );
        },
    );

    let simulated_offer_amount = RefCell::new(Uint128::zero());
    // Now Let's try a reverse simulation by swapping uluna to uwhale
    suite.query_reverse_simulation(
        "o.whale.uluna.uusd".to_string(),
        Coin {
            denom: "uwhale".to_string(),
            amount: Uint128::from(1000u128),
        },
        "uluna".to_string(),
        |result| {
            *simulated_offer_amount.borrow_mut() = result.unwrap().offer_amount;
        },
    );

    // Another swap but this time the other way around
    suite.swap(
        &creator,
        "uwhale".to_string(),
        None,
        None,
        None,
        "o.whale.uluna.uusd".to_string(),
        vec![coin(
            simulated_offer_amount.borrow().u128(),
            "uluna".to_string(),
        )],
        |result| {
            // Find the key with 'offer_amount' and the key with 'return_amount'
            // Ensure that the offer amount is 1000 and the return amount is greater than 0
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

            assert_approx_eq!(1000u128, return_amount.parse::<u128>().unwrap(), "0.003");
        },
    );

    // And now uwhale to uusd
    suite.query_reverse_simulation(
        "o.whale.uluna.uusd".to_string(),
        Coin {
            denom: "uusd".to_string(),
            amount: Uint128::from(1000u128),
        },
        "uwhale".to_string(),
        |result| {
            *simulated_return_amount.borrow_mut() = result.unwrap().offer_amount;
        },
    );
    // Another swap but this time uwhale to uusd
    suite.swap(
        &creator,
        "uusd".to_string(),
        None,
        None,
        None,
        "o.whale.uluna.uusd".to_string(),
        vec![coin(
            simulated_return_amount.borrow().u128(),
            "uwhale".to_string(),
        )],
        |result| {
            // Find the key with 'offer_amount' and the key with 'return_amount'
            // Ensure that the offer amount is 1000 and the return amount is greater than 0
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
                simulated_return_amount.borrow().u128(),
                return_amount.parse::<u128>().unwrap(),
                "0.002"
            );
            assert_approx_eq!(1000u128, offer_amount.parse::<u128>().unwrap(), "0.003");
        },
    );

    // And now uusd to uluna
    suite.query_reverse_simulation(
        "o.whale.uluna.uusd".to_string(),
        Coin {
            denom: "uluna".to_string(),
            amount: Uint128::from(1000u128),
        },
        "uusd".to_string(),
        |result| {
            *simulated_offer_amount.borrow_mut() = result.unwrap().offer_amount;
        },
    );
    // Another swap but this time uusd to uluna
    suite.swap(
        &creator,
        "uluna".to_string(),
        None,
        None,
        None,
        "o.whale.uluna.uusd".to_string(),
        vec![coin(
            simulated_offer_amount.borrow().u128(),
            "uusd".to_string(),
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

            assert_approx_eq!(1000u128, return_amount.parse::<u128>().unwrap(), "0.003");
        },
    );
}

#[test]
fn provide_liquidity_stable_swap_shouldnt_double_count_deposits_or_inflate_lp() {
    let mut suite = TestingSuite::default_with_balances(
        vec![
            coin(1_000_000_001u128, "uusd".to_string()),
            coin(1_000_000_001u128, "uusdc".to_string()),
            coin(1_000_000_000u128, "uusdt".to_string()),
            coin(1_000_000_001u128, "uusdy".to_string()),
            coin(1_000_000_001u128, "uom".to_string()),
        ],
        StargateMock::new(vec![coin(8888u128, "uom".to_string())]),
    );
    let creator = suite.creator();
    let alice = suite.senders[1].clone();

    let asset_infos = vec![
        "uusdc".to_string(),
        "uusdt".to_string(),
        "uusdy".to_string(),
    ];

    // Protocol fee is 0.01% and swap fee is 0.02% and burn fee is 0%
    let pool_fees = PoolFee {
        protocol_fee: Fee {
            share: Decimal::from_ratio(1u128, 1000u128),
        },
        swap_fee: Fee {
            share: Decimal::from_ratio(1u128, 10_000_u128),
        },
        burn_fee: Fee {
            share: Decimal::zero(),
        },
        extra_fees: vec![],
    };

    // Create a pool
    suite.instantiate_default().create_pool(
        &creator,
        asset_infos,
        vec![6u8, 6u8, 6u8],
        pool_fees,
        PoolType::StableSwap { amp: 100 },
        Some("uusdc.uusdt.uusdy".to_string()),
        vec![coin(1000, "uusd"), coin(8888, "uom")],
        |result| {
            result.unwrap();
        },
    );

    let lp_denom = suite.get_lp_denom("o.uusdc.uusdt.uusdy".to_string());

    // Let's try to add liquidity
    suite
        .provide_liquidity(
            &creator,
            "o.uusdc.uusdt.uusdy".to_string(),
            None,
            None,
            None,
            None,
            None,
            vec![
                Coin {
                    denom: "uusdc".to_string(),
                    amount: Uint128::from(500_000u128),
                },
                Coin {
                    denom: "uusdt".to_string(),
                    amount: Uint128::from(500_000u128),
                },
                Coin {
                    denom: "uusdy".to_string(),
                    amount: Uint128::from(500_000u128),
                },
            ],
            |result| {
                result.unwrap();
            },
        )
        .query_balance(&creator.to_string(), &lp_denom, |result| {
            assert_eq!(
                result.unwrap().amount,
                // liquidity provided - MINIMUM_LIQUIDITY_AMOUNT
                Uint128::from(1_500_000u128 - 1_000u128)
            );
        });

    // let's try providing liquidity again
    suite
        .provide_liquidity(
            &creator,
            "o.uusdc.uusdt.uusdy".to_string(),
            None,
            None,
            None,
            None,
            None,
            vec![
                Coin {
                    denom: "uusdc".to_string(),
                    amount: Uint128::from(500_000u128),
                },
                Coin {
                    denom: "uusdt".to_string(),
                    amount: Uint128::from(500_000u128),
                },
                Coin {
                    denom: "uusdy".to_string(),
                    amount: Uint128::from(500_000u128),
                },
            ],
            |result| {
                result.unwrap();
            },
        )
        .query_balance(&creator.to_string(), &lp_denom, |result| {
            assert_eq!(
                result.unwrap().amount,
                // we should expect another ~1_500_000
                Uint128::from(1_500_000u128 + 1_500_000u128 - 1_000u128)
            );
        });

    let simulated_return_amount = RefCell::new(Uint128::zero());
    suite.query_simulation(
        "o.uusdc.uusdt.uusdy".to_string(),
        Coin {
            denom: "uusdc".to_string(),
            amount: Uint128::from(1_000u128),
        },
        "uusdt".to_string(),
        |result| {
            *simulated_return_amount.borrow_mut() = result.unwrap().return_amount;
        },
    );

    // Now Let's try a swap
    suite.swap(
        &creator,
        "uusdt".to_string(),
        None,
        None,
        None,
        "o.uusdc.uusdt.uusdy".to_string(),
        vec![coin(1_000u128, "uusdc".to_string())],
        |result| {
            // Find the key with 'offer_amount' and the key with 'return_amount'
            // Ensure that the offer amount is 1000 and the return amount is greater than 0
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
            // Because the Pool was created and 1_000_000 of each token has been provided as liquidity
            // Assuming no fees we should expect a small swap of 1000 to result in not too much slippage
            // Expect 1000 give or take 0.002 difference
            // Once fees are added and being deducted properly only the "0.002" should be changed.
            assert_approx_eq!(
                offer_amount.parse::<u128>().unwrap(),
                return_amount.parse::<u128>().unwrap(),
                "0.002"
            );
            assert_approx_eq!(
                simulated_return_amount.borrow().u128(),
                return_amount.parse::<u128>().unwrap(),
                "0.002"
            );
        },
    );

    let simulated_offer_amount = RefCell::new(Uint128::zero());
    // Now Let's try a reverse simulation by swapping uluna to uwhale
    suite.query_reverse_simulation(
        "o.uusdc.uusdt.uusdy".to_string(),
        Coin {
            denom: "uusdc".to_string(),
            amount: Uint128::from(1000u128),
        },
        "uusdt".to_string(),
        |result| {
            *simulated_offer_amount.borrow_mut() = result.unwrap().offer_amount;
        },
    );

    // Another swap but this time the other way around
    suite.swap(
        &creator,
        "uusdc".to_string(),
        None,
        None,
        None,
        "o.uusdc.uusdt.uusdy".to_string(),
        vec![coin(
            simulated_offer_amount.borrow().u128(),
            "uusdt".to_string(),
        )],
        |result| {
            // Find the key with 'offer_amount' and the key with 'return_amount'
            // Ensure that the offer amount is 1000 and the return amount is greater than 0
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

            assert_approx_eq!(1000u128, return_amount.parse::<u128>().unwrap(), "0.003");
        },
    );

    // And now uusdc to uusdy
    suite.query_reverse_simulation(
        "o.uusdc.uusdt.uusdy".to_string(),
        Coin {
            denom: "uusdy".to_string(),
            amount: Uint128::from(1000u128),
        },
        "uusdc".to_string(),
        |result| {
            *simulated_return_amount.borrow_mut() = result.unwrap().offer_amount;
        },
    );
    // Another swap but this time uusdc to uusdy
    suite.swap(
        &creator,
        "uusdy".to_string(),
        None,
        None,
        None,
        "o.uusdc.uusdt.uusdy".to_string(),
        vec![coin(
            simulated_return_amount.borrow().u128(),
            "uusdc".to_string(),
        )],
        |result| {
            // Find the key with 'offer_amount' and the key with 'return_amount'
            // Ensure that the offer amount is 1000 and the return amount is greater than 0
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
                simulated_return_amount.borrow().u128(),
                return_amount.parse::<u128>().unwrap(),
                "0.002"
            );
            assert_approx_eq!(1000u128, offer_amount.parse::<u128>().unwrap(), "0.003");
        },
    );

    // And now uusdy to uusdt
    suite.query_reverse_simulation(
        "o.uusdc.uusdt.uusdy".to_string(),
        Coin {
            denom: "uusdt".to_string(),
            amount: Uint128::from(1000u128),
        },
        "uusdy".to_string(),
        |result| {
            *simulated_offer_amount.borrow_mut() = result.unwrap().offer_amount;
        },
    );
    // Another swap but this time uusdy to uusdt
    suite.swap(
        &creator,
        "uusdt".to_string(),
        None,
        None,
        None,
        "o.uusdc.uusdt.uusdy".to_string(),
        vec![coin(
            simulated_offer_amount.borrow().u128(),
            "uusdy".to_string(),
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

            assert_approx_eq!(1000u128, return_amount.parse::<u128>().unwrap(), "0.003");
        },
    );

    // now creator provides even more liquidity
    suite
        .provide_liquidity(
            &creator,
            "o.uusdc.uusdt.uusdy".to_string(),
            None,
            None,
            None,
            None,
            None,
            vec![
                Coin {
                    denom: "uusdc".to_string(),
                    amount: Uint128::from(10_000_000u128),
                },
                Coin {
                    denom: "uusdt".to_string(),
                    amount: Uint128::from(10_000_000u128),
                },
                Coin {
                    denom: "uusdy".to_string(),
                    amount: Uint128::from(10_000_000u128),
                },
            ],
            |result| {
                result.unwrap();
            },
        )
        .query_balance(&creator.to_string(), &lp_denom, |result| {
            assert_approx_eq!(
                result.unwrap().amount,
                Uint128::from(30_000_000u128 + 1_500_000u128 + 1_500_000u128 - 1_000u128),
                "0.0000012425"
            );
        });

    // now alice provides liquidity
    suite
        .provide_liquidity(
            &alice,
            "o.uusdc.uusdt.uusdy".to_string(),
            None,
            None,
            None,
            None,
            None,
            vec![
                Coin {
                    denom: "uusdc".to_string(),
                    amount: Uint128::from(10_000u128),
                },
                Coin {
                    denom: "uusdt".to_string(),
                    amount: Uint128::from(10_000u128),
                },
                Coin {
                    denom: "uusdy".to_string(),
                    amount: Uint128::from(10_000u128),
                },
            ],
            |result| {
                result.unwrap();
            },
        )
        .query_balance(&alice.to_string(), &lp_denom, |result| {
            // shares are not inflated, alice should have 30_000 LP shares
            assert_eq!(result.unwrap().amount, Uint128::from(30_000u128));
        });
}

// This test is to ensure that the edge case of providing liquidity with 3 assets
#[test]
fn provide_liquidity_stable_swap_edge_case() {
    let mut suite = TestingSuite::default_with_balances(
        vec![
            coin(1_000_000_001u128, "uwhale".to_string()),
            coin(1_000_000_000u128, "uluna".to_string()),
            coin(1_000_000_001u128, "uusd".to_string()),
            coin(1_000_000_001u128, "uom".to_string()),
        ],
        StargateMock::new(vec![coin(8888u128, "uom".to_string())]),
    );
    let creator = suite.creator();
    let _other = suite.senders[1].clone();
    let _unauthorized = suite.senders[2].clone();
    // Asset infos with uwhale and uluna

    let asset_infos = vec![
        "uwhale".to_string(),
        "uluna".to_string(),
        "uusd".to_string(),
    ];

    // Protocol fee is 0.01% and swap fee is 0.02% and burn fee is 0%
    let pool_fees = PoolFee {
        protocol_fee: Fee {
            share: Decimal::from_ratio(1u128, 1000u128),
        },
        swap_fee: Fee {
            share: Decimal::from_ratio(1u128, 10_000_u128),
        },
        burn_fee: Fee {
            share: Decimal::zero(),
        },
        extra_fees: vec![],
    };

    // Create a pool with 3 assets
    suite.instantiate_default().create_pool(
        &creator,
        asset_infos,
        vec![6u8, 6u8, 6u8],
        pool_fees,
        PoolType::StableSwap { amp: 100 },
        Some("whale.uluna.uusd".to_string()),
        vec![coin(1000, "uusd"), coin(8888, "uom")],
        |result| {
            result.unwrap();
        },
    );

    // Adding liquidity with less than the minimum liquidity amount should fail
    suite.provide_liquidity(
        &creator,
        "o.whale.uluna.uusd".to_string(),
        None,
        None,
        None,
        None,
        None,
        vec![
            Coin {
                denom: "uwhale".to_string(),
                amount: MINIMUM_LIQUIDITY_AMOUNT
                    .checked_div(Uint128::new(3u128))
                    .unwrap(),
            },
            Coin {
                denom: "uluna".to_string(),
                amount: MINIMUM_LIQUIDITY_AMOUNT
                    .checked_div(Uint128::new(3u128))
                    .unwrap(),
            },
            Coin {
                denom: "uusd".to_string(),
                amount: MINIMUM_LIQUIDITY_AMOUNT
                    .checked_div(Uint128::new(3u128))
                    .unwrap(),
            },
        ],
        |result| {
            assert_eq!(
                result.unwrap_err().downcast_ref::<ContractError>(),
                Some(&ContractError::InvalidInitialLiquidityAmount(
                    MINIMUM_LIQUIDITY_AMOUNT
                ))
            );
        },
    );

    // Let's try to add liquidity with the correct amount (1_000_000 of each asset)
    suite.provide_liquidity(
        &creator,
        "o.whale.uluna.uusd".to_string(),
        None,
        None,
        None,
        None,
        None,
        vec![
            Coin {
                denom: "uwhale".to_string(),
                amount: Uint128::from(1_000_000u128),
            },
            Coin {
                denom: "uluna".to_string(),
                amount: Uint128::from(1_000_000u128),
            },
            Coin {
                denom: "uusd".to_string(),
                amount: Uint128::from(1_000_000u128),
            },
        ],
        |result| {
            // Ensure we got 999000 in the response which is 1mil less the initial liquidity amount
            for event in result.unwrap().events {
                for attribute in event.attributes {
                    if attribute.key == "share" {
                        assert_approx_eq!(
                            attribute.value.parse::<u128>().unwrap(),
                            1_000_000u128 * 3,
                            "0.002"
                        );
                    }
                }
            }
        },
    );

    let simulated_return_amount = RefCell::new(Uint128::zero());
    suite.query_simulation(
        "o.whale.uluna.uusd".to_string(),
        Coin {
            denom: "uwhale".to_string(),
            amount: Uint128::from(1_000u128),
        },
        "uluna".to_string(),
        |result| {
            *simulated_return_amount.borrow_mut() = result.unwrap().return_amount;
        },
    );

    // Now Let's try a swap
    suite.swap(
        &creator,
        "uluna".to_string(),
        None,
        None,
        None,
        "o.whale.uluna.uusd".to_string(),
        vec![coin(1_000u128, "uwhale".to_string())],
        |result| {
            // Find the key with 'offer_amount' and the key with 'return_amount'
            // Ensure that the offer amount is 1000 and the return amount is greater than 0
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
            // Because the Pool was created and 1_000_000 of each token has been provided as liquidity
            // Assuming no fees we should expect a small swap of 1000 to result in not too much slippage
            // Expect 1000 give or take 0.002 difference
            // Once fees are added and being deducted properly only the "0.002" should be changed.
            assert_approx_eq!(
                offer_amount.parse::<u128>().unwrap(),
                return_amount.parse::<u128>().unwrap(),
                "0.002"
            );
            assert_approx_eq!(
                simulated_return_amount.borrow().u128(),
                return_amount.parse::<u128>().unwrap(),
                "0.002"
            );
        },
    );
}

#[test]
fn provide_incomplete_liquidity_fails_on_stableswaps() {
    let mut suite = TestingSuite::default_with_balances(
        vec![
            coin(1_000_000_001u128, "uwhale".to_string()),
            coin(1_000_000_000u128, "uluna".to_string()),
            coin(1_000_000_001u128, "uusd".to_string()),
            coin(1_000_000_001u128, "uom".to_string()),
        ],
        StargateMock::new(vec![coin(8888u128, "uom".to_string())]),
    );
    let creator = suite.creator();

    let asset_infos = vec![
        "uwhale".to_string(),
        "uluna".to_string(),
        "uusd".to_string(),
    ];

    // Protocol fee is 0.01% and swap fee is 0.02% and burn fee is 0%
    let pool_fees = PoolFee {
        protocol_fee: Fee {
            share: Decimal::from_ratio(1u128, 1000u128),
        },
        swap_fee: Fee {
            share: Decimal::from_ratio(1u128, 10_000_u128),
        },
        burn_fee: Fee {
            share: Decimal::zero(),
        },
        extra_fees: vec![],
    };

    // Create a pool
    suite.instantiate_default().create_pool(
        &creator,
        asset_infos,
        vec![6u8, 6u8, 6u8],
        pool_fees,
        PoolType::StableSwap { amp: 100 },
        Some("whale.uluna.uusd".to_string()),
        vec![coin(1000, "uusd"), coin(8888, "uom")],
        |result| {
            result.unwrap();
        },
    );

    // Add liquidity with only 2 tokens
    suite
        .provide_liquidity(
            &creator,
            "o.whale.uluna.uusd".to_string(),
            None,
            None,
            None,
            None,
            None,
            vec![
                Coin {
                    denom: "uwhale".to_string(),
                    amount: Uint128::from(1_000_000u128),
                },
                Coin {
                    denom: "uluna".to_string(),
                    amount: Uint128::from(1_000_000u128),
                },
            ],
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();
                match err {
                    ContractError::AssetMismatch => {}
                    _ => panic!("Wrong error type, should return ContractError::AssetMismatch"),
                }
            },
        )
        // Add liquidity with 3 tokens but one of them's amount is zero
        .provide_liquidity(
            &creator,
            "o.whale.uluna.uusd".to_string(),
            None,
            None,
            None,
            None,
            None,
            vec![
                Coin {
                    denom: "uwhale".to_string(),
                    amount: Uint128::from(1_000_000u128),
                },
                Coin {
                    denom: "uluna".to_string(),
                    amount: Uint128::from(1_000_000u128),
                },
                Coin {
                    denom: "uusd".to_string(),
                    amount: Uint128::zero(),
                },
            ],
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();
                match err {
                    ContractError::AssetMismatch => {}
                    _ => panic!("Wrong error type, should return ContractError::AssetMismatch"),
                }
            },
        )
        // Add liquidity with 3 tokens but one of them is not part of the pool
        .provide_liquidity(
            &creator,
            "o.whale.uluna.uusd".to_string(),
            None,
            None,
            None,
            None,
            None,
            vec![
                Coin {
                    denom: "uwhale".to_string(),
                    amount: Uint128::from(1_000_000u128),
                },
                Coin {
                    denom: "uluna".to_string(),
                    amount: Uint128::from(1_000_000u128),
                },
                Coin {
                    denom: "uom".to_string(),
                    amount: Uint128::from(1_000_000u128),
                },
            ],
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();
                match err {
                    ContractError::AssetMismatch => {}
                    _ => panic!("Wrong error type, should return ContractError::AssetMismatch"),
                }
            },
        );
}

#[test]
fn provide_liquidity_stable_invalid_slippage_check() {
    let mut suite = TestingSuite::default_with_balances(
        vec![
            coin(1_000_000_000_000u128, "uwhale".to_string()),
            coin(1_000_000_000_000u128, "uluna".to_string()),
            coin(1_000_000_000_000u128, "uosmo".to_string()),
            coin(1_000_000_000_000u128, "uusd".to_string()),
            coin(
                200_000_000_000_000_000_000_000_000_000_000_000_u128,
                "uusdc".to_string(),
            ),
            coin(
                200_000_000_000_000_000_000_000_000_000_000_u128,
                "ausdy".to_string(),
            ),
            coin(150_000_000_000_000_000_000_u128, "uom".to_string()),
        ],
        StargateMock::new(vec![coin(8888u128, "uom".to_string())]),
    );
    let creator = suite.creator();
    let bob = suite.senders[1].clone();
    // Asset infos with uwhale and uluna

    let asset_infos = vec!["uwhale".to_string(), "uluna".to_string()];

    let pool_fees = PoolFee {
        protocol_fee: Fee {
            share: Decimal::zero(),
        },
        swap_fee: Fee {
            share: Decimal::permille(30),
        },
        burn_fee: Fee {
            share: Decimal::zero(),
        },
        extra_fees: vec![],
    };

    // Create a pool
    suite.instantiate_default().add_one_epoch().create_pool(
        &creator,
        asset_infos,
        vec![6u8, 6u8],
        pool_fees,
        PoolType::StableSwap { amp: 10 },
        Some("whale.uluna".to_string()),
        vec![coin(1000, "uusd"), coin(8888, "uom")],
        |result| {
            result.unwrap();
        },
    );

    // set the pool state
    suite.provide_liquidity(
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
    );

    //user sets 0.01% strict slippage tolerance, should fail
    let slippage: Decimal = Decimal::bps(1);
    suite.provide_liquidity(
        &bob,
        "o.whale.uluna".to_string(),
        None,
        None,
        Some(slippage),
        None,
        None,
        vec![
            Coin {
                denom: "uwhale".to_string(),
                amount: Uint128::from(1000000u128),
            },
            Coin {
                denom: "uluna".to_string(),
                amount: Uint128::from(200000u128),
            },
        ],
        |result| {
            let err = result.unwrap_err().downcast::<ContractError>().unwrap();
            match err {
                ContractError::MaxSlippageAssertion => {}
                _ => panic!("Wrong error type, should return ContractError::MaxSlippageAssertion"),
            }
        },
    );
}
