use std::cell::RefCell;

use cosmwasm_std::{assert_approx_eq, coin, Addr, Coin, Decimal, Uint128};
use mantra_common_testing::multi_test::stargate_mock::StargateMock;
use mantra_dex_std::{
    fee::{Fee, PoolFee},
    lp_common::MINIMUM_LIQUIDITY_AMOUNT,
    pool_manager::{PoolType, PoolsResponse},
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

// This function is used to setup the test suite for the stable swap tests

fn setup_stable_swap() -> (TestingSuite, Addr, Addr, String) {
    // 2T uluna, 2T uusd, 2T uweth
    let uluna_amount = 2_000_000_000_000u128 * 10u128.pow(6);
    // 2T uusd
    let uusd_amount = 2_000_000_000_000u128 * 10u128.pow(6);
    // 2T uweth
    let uweth_amount = 2_000_000_000_000u128 * 10u128.pow(18);
    let mut suite = TestingSuite::default_with_balances(
        vec![
            coin(uluna_amount, "uluna".to_string()), // 2T
            coin(uusd_amount, "uusd".to_string()),   // 2T
            coin(uweth_amount, "uweth".to_string()), // 2T
            coin(10_000u128, "uom".to_string()),
        ],
        StargateMock::new(vec![coin(8888u128, "uom".to_string())]),
    );

    let creator = suite.creator();
    let user = suite.senders[1].clone();

    suite.instantiate_default().add_one_epoch().create_pool(
        &creator,
        vec!["uluna".to_string(), "uusd".to_string(), "uweth".to_string()],
        vec![6u8, 6u8, 18u8],
        PoolFee {
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
        PoolType::StableSwap { amp: 100 },
        Some("uluna.uusd.uweth".to_string()),
        vec![coin(1000, "uusd"), coin(8888, "uom")],
        |result| {
            result.unwrap();
        },
    );

    let lp_denom = suite.get_lp_denom("o.uluna.uusd.uweth".to_string());

    suite.provide_liquidity(
        &creator,
        "o.uluna.uusd.uweth".to_string(),
        None,
        None,
        None,
        None,
        None,
        vec![
            Coin {
                denom: "uluna".to_string(),
                amount: Uint128::from(10u128 * 10u128.pow(6)),
            },
            Coin {
                denom: "uusd".to_string(),
                amount: Uint128::from(10u128 * 10u128.pow(6)),
            },
            Coin {
                denom: "uweth".to_string(),
                amount: Uint128::from(10u128 * 10u128.pow(18)),
            },
        ],
        |result| {
            result.unwrap();
        },
    );

    (suite, creator, user, lp_denom)
}

#[test]
fn equal_handling_of_decimals_on_stableswap_deposit() {
    // Setup with the same asset configuration as Python simulation
    // 2T uluna (6 decimals), 2T uusd (6 decimals), 2T uweth (18 decimals)
    let uluna_amount = 2_000_000_000_000u128 * 10u128.pow(6);
    let uusd_amount = 2_000_000_000_000u128 * 10u128.pow(6);
    let uweth_amount = 2_000_000_000_000u128 * 10u128.pow(18);

    let mut suite = TestingSuite::default_with_balances(
        vec![
            coin(uluna_amount, "uluna".to_string()),
            coin(uusd_amount, "uusd".to_string()),
            coin(uweth_amount, "uweth".to_string()),
            coin(10_000u128, "uom".to_string()),
        ],
        StargateMock::new(vec![coin(8888u128, "uom".to_string())]),
    );

    let creator = suite.creator();
    let user = suite.senders[1].clone();

    // Create a pool with 3 assets, same decimals as Python simulation
    suite.instantiate_default().add_one_epoch().create_pool(
        &creator,
        vec!["uluna".to_string(), "uusd".to_string(), "uweth".to_string()],
        vec![6u8, 6u8, 18u8], // Explicitly set decimals to match Python
        PoolFee {
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
        PoolType::StableSwap { amp: 100 }, // Same amplification as Python
        Some("uluna.uusd.uweth".to_string()),
        vec![coin(1000, "uusd"), coin(8888, "uom")],
        |result| {
            result.unwrap();
        },
    );

    let lp_denom = suite.get_lp_denom("o.uluna.uusd.uweth".to_string());

    // Initial liquidity provision - same as Python
    // 10*10^6 uluna, 10*10^6 uusd, 10*10^18 uweth
    println!("--- Initial Liquidity Provision ---");
    suite.provide_liquidity(
        &creator,
        "o.uluna.uusd.uweth".to_string(),
        None,
        None,
        None,
        None,
        None,
        vec![
            Coin {
                denom: "uluna".to_string(),
                amount: Uint128::from(10u128 * 10u128.pow(6)),
            },
            Coin {
                denom: "uusd".to_string(),
                amount: Uint128::from(10u128 * 10u128.pow(6)),
            },
            Coin {
                denom: "uweth".to_string(),
                amount: Uint128::from(10u128 * 10u128.pow(18)),
            },
        ],
        |result| {
            result.unwrap();
        },
    );

    // Get initial LP shares amount
    let addr = creator.to_string();
    let contract = suite.pool_manager_addr.to_string();
    let initial_lp_supply = RefCell::new(Uint128::zero());
    suite
        .query_balance(&addr, lp_denom.clone(), |result| {
            let amount = result.unwrap().amount;
            *initial_lp_supply.borrow_mut() = amount;
        })
        .query_balance(&contract, lp_denom.clone(), |result| {
            let amount = result.unwrap().amount;
            let x = initial_lp_supply.borrow_mut().clone();
            *initial_lp_supply.borrow_mut() = amount + x;

            println!("Initial LP Supply: {}", *initial_lp_supply.borrow());
            println!();
        });

    // Case 1: Deposit uluna + uweth
    println!(
        "--- Test Case 1: Deposit {} uluna + {} uweth ---",
        2u128 * 10u128.pow(6),
        2u128 * 10u128.pow(18)
    );

    let lp_shares_case_1 = RefCell::new(Uint128::zero());
    suite
        .provide_liquidity(
            &user,
            "o.uluna.uusd.uweth".to_string(),
            None,
            None,
            None,
            None,
            None,
            vec![
                Coin {
                    denom: "uluna".to_string(),
                    amount: Uint128::from(2u128 * 10u128.pow(6)),
                },
                Coin {
                    denom: "uweth".to_string(),
                    amount: Uint128::from(2u128 * 10u128.pow(18)),
                },
            ],
            |result| {
                result.unwrap();
            },
        )
        .query_balance(&user.to_string(), lp_denom.clone(), |result| {
            let lp_shares_received = result.unwrap().amount;
            println!("Current Total Supply: {}", initial_lp_supply.borrow());
            println!("Depositing: [2000000, 0, 2000000000000000000]");
            println!("LP Minted in Case 1: {}", lp_shares_received);
            *lp_shares_case_1.borrow_mut() = lp_shares_received;
        });

    // Reset state to run Case 2 separately
    let mut suite = TestingSuite::default_with_balances(
        vec![
            coin(uluna_amount, "uluna".to_string()),
            coin(uusd_amount, "uusd".to_string()),
            coin(uweth_amount, "uweth".to_string()),
            coin(10_000u128, "uom".to_string()),
        ],
        StargateMock::new(vec![coin(8888u128, "uom".to_string())]),
    );

    let creator = suite.creator();
    let user = suite.senders[1].clone();

    // Recreate the same pool
    suite.instantiate_default().add_one_epoch().create_pool(
        &creator,
        vec!["uluna".to_string(), "uusd".to_string(), "uweth".to_string()],
        vec![6u8, 6u8, 18u8],
        PoolFee {
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
        PoolType::StableSwap { amp: 100 },
        Some("uluna.uusd.uweth".to_string()),
        vec![coin(1000, "uusd"), coin(8888, "uom")],
        |result| {
            result.unwrap();
        },
    );

    let lp_denom = suite.get_lp_denom("o.uluna.uusd.uweth".to_string());

    // Initial liquidity provision again
    suite.provide_liquidity(
        &creator,
        "o.uluna.uusd.uweth".to_string(),
        None,
        None,
        None,
        None,
        None,
        vec![
            Coin {
                denom: "uluna".to_string(),
                amount: Uint128::from(10u128 * 10u128.pow(6)),
            },
            Coin {
                denom: "uusd".to_string(),
                amount: Uint128::from(10u128 * 10u128.pow(6)),
            },
            Coin {
                denom: "uweth".to_string(),
                amount: Uint128::from(10u128 * 10u128.pow(18)),
            },
        ],
        |result| {
            result.unwrap();
        },
    );

    // Case 2: Deposit uluna + uusd
    println!(
        "--- Test Case 2: Deposit {} uluna + {} uusd ---",
        2u128 * 10u128.pow(6),
        2u128 * 10u128.pow(6)
    );
    let lp_shares_case_2 = RefCell::new(Uint128::zero());
    suite
        .provide_liquidity(
            &user,
            "o.uluna.uusd.uweth".to_string(),
            None,
            None,
            None,
            None,
            None,
            vec![
                Coin {
                    denom: "uluna".to_string(),
                    amount: Uint128::from(2u128 * 10u128.pow(6)),
                },
                Coin {
                    denom: "uusd".to_string(),
                    amount: Uint128::from(2u128 * 10u128.pow(6)),
                },
            ],
            |result| {
                result.unwrap();
            },
        )
        .query_balance(&user.to_string(), lp_denom.clone(), |result| {
            let lp_shares_received = result.unwrap().amount;
            println!("Depositing: [2000000, 2000000, 0]");
            println!("LP Minted in Case 2: {}", lp_shares_received);
            *lp_shares_case_2.borrow_mut() = lp_shares_received;
        });

    // Print summary
    println!("--- Summary ---");
    println!(
        "Case 1 LP Minted (uluna + uweth): {}",
        lp_shares_case_1.borrow()
    );
    println!(
        "Case 2 LP Minted (uluna + uusd): {}",
        lp_shares_case_2.borrow()
    );

    assert_eq!(
        lp_shares_case_1.borrow().u128(),
        lp_shares_case_2.borrow().u128()
    );

    assert_eq!(
        lp_shares_case_1.borrow().u128(),
        lp_shares_case_2.borrow().u128()
    );
}

#[test]
// similar to the above, but with exotic amounts
// low amount (1), medium amount (333), high amount (1T)
fn handling_of_lp_shares_exotic_amounts() {
    let (mut suite, _, user, lp_denom) = setup_stable_swap();
    // query the pool assets
    let pool_assets: RefCell<PoolsResponse> = RefCell::new(PoolsResponse { pools: vec![] });
    suite.query_pools(
        Some("o.uluna.uusd.uweth".to_string()),
        None,
        None,
        |result| {
            pool_assets.borrow_mut().pools = result.unwrap().pools;
        },
    );

    // low amount (1)
    println!("low amount (1)");
    suite
        .provide_liquidity(
            &user,
            "o.uluna.uusd.uweth".to_string(),
            None,
            None,
            None,
            None,
            None,
            vec![
                Coin {
                    denom: "uluna".to_string(),
                    amount: Uint128::from(10u128.pow(6)),
                },
                Coin {
                    denom: "uusd".to_string(),
                    amount: Uint128::from(10u128.pow(6)),
                },
                Coin {
                    denom: "uweth".to_string(),
                    amount: Uint128::from(10u128.pow(18)),
                },
            ],
            |result| {
                result.unwrap();
            },
        )
        .query_balance(&user.to_string(), lp_denom.clone(), |result| {
            let new_lp_balance = result.unwrap().amount;
            println!("new_lp_balance: {}", new_lp_balance);
            let expected = 9486810555483u128;
            let tolerance = expected / 1000; // 0.1%
            assert!(
                expected.abs_diff(new_lp_balance.u128()) <= tolerance,
                "LP balance exceeds tolerance"
            );
        });

    // medium amount (333)
    println!("medium amount (333)");
    suite
        .provide_liquidity(
            &user,
            "o.uluna.uusd.uweth".to_string(),
            None,
            None,
            None,
            None,
            None,
            vec![
                Coin {
                    denom: "uluna".to_string(),
                    amount: Uint128::from(333u128 * 10u128.pow(6)),
                },
                Coin {
                    denom: "uusd".to_string(),
                    amount: Uint128::from(333u128 * 10u128.pow(6)),
                },
                Coin {
                    denom: "uweth".to_string(),
                    amount: Uint128::from(333u128 * 10u128.pow(18)),
                },
            ],
            |result| {
                result.unwrap();
            },
        )
        .query_balance(&user.to_string(), lp_denom.clone(), |result| {
            let new_lp_balance = result.unwrap().amount;
            println!("new_lp_balance: {}", new_lp_balance);
            let expected = 3168594725531412u128;
            let tolerance = expected / 1000; // 0.1%
            assert!(
                expected.abs_diff(new_lp_balance.u128()) <= tolerance,
                "LP balance exceeds tolerance"
            );
        });

    println!("high amount (1T)");
    // high amount (1T)
    suite
        .provide_liquidity(
            &user,
            "o.uluna.uusd.uweth".to_string(),
            None,
            None,
            None,
            None,
            None,
            vec![
                Coin {
                    denom: "uluna".to_string(),
                    amount: Uint128::from(1_000_000_000_000u128 * 10u128.pow(6)),
                },
                Coin {
                    denom: "uusd".to_string(),
                    amount: Uint128::from(1_000_000_000_000u128 * 10u128.pow(6)),
                },
                Coin {
                    denom: "uweth".to_string(),
                    amount: Uint128::from(1_000_000_000_000u128 * 10u128.pow(18)),
                },
            ],
            |result| {
                result.unwrap();
            },
        )
        .query_balance(&user.to_string(), lp_denom.clone(), |result| {
            let new_lp_balance = result.unwrap().amount;
            println!("new_lp_balance: {}", new_lp_balance);
            let expected = 9486810558699405934993014u128;
            let tolerance = expected / 1000; // 0.1%
            assert!(
                expected.abs_diff(new_lp_balance.u128()) <= tolerance,
                "LP balance exceeds tolerance"
            );
        });
}

#[test]
// similar to the above, but with exotic amounts
fn handling_of_lp_shares_extreme_cases() {
    let (mut suite, _, user, lp_denom) = setup_stable_swap();
    // query the pool assets
    let pool_assets: RefCell<PoolsResponse> = RefCell::new(PoolsResponse { pools: vec![] });
    suite.query_pools(
        Some("o.uluna.uusd.uweth".to_string()),
        None,
        None,
        |result| {
            pool_assets.borrow_mut().pools = result.unwrap().pools;
        },
    );

    println!("nothing (1)");
    suite
        .provide_liquidity(
            &user,
            "o.uluna.uusd.uweth".to_string(),
            None,
            None,
            None,
            None,
            None,
            vec![
                Coin {
                    denom: "uluna".to_string(),
                    amount: Uint128::new(1),
                },
                Coin {
                    denom: "uusd".to_string(),
                    amount: Uint128::new(1),
                },
                Coin {
                    denom: "uweth".to_string(),
                    amount: Uint128::new(1000000000),
                },
            ],
            |result| {
                result.unwrap();
            },
        )
        .query_balance(&user.to_string(), lp_denom.clone(), |result| {
            let new_lp_balance = result.unwrap().amount;
            println!("new_lp_balance: {}", new_lp_balance);
            let expected = 3168594725531412u128;
            let tolerance = expected / 1000; // 0.1%
            assert!(
                expected.abs_diff(new_lp_balance.u128()) <= tolerance,
                "LP balance exceeds tolerance"
            );
        });

    println!("high amount (200T)");
    suite
        .provide_liquidity(
            &user,
            "o.uluna.uusd.uweth".to_string(),
            None,
            None,
            None,
            None,
            None,
            vec![
                Coin {
                    denom: "uluna".to_string(),
                    amount: Uint128::from(200_000_000_000_000u128 * 10u128.pow(6)),
                },
                Coin {
                    denom: "uusd".to_string(),
                    amount: Uint128::from(200_000_000_000_000u128 * 10u128.pow(6)),
                },
                Coin {
                    denom: "uweth".to_string(),
                    amount: Uint128::from(200_000_000_000_000u128 * 10u128.pow(18)),
                },
            ],
            |result| {
                result.unwrap();
            },
        )
        .query_balance(&user.to_string(), lp_denom.clone(), |result| {
            let new_lp_balance = result.unwrap().amount;
            println!("new_lp_balance: {}", new_lp_balance);
            let expected = 9486810558699405934993014u128;
            let tolerance = expected / 1000; // 0.1%
            assert!(
                expected.abs_diff(new_lp_balance.u128()) <= tolerance,
                "LP balance exceeds tolerance"
            );
        });
}
