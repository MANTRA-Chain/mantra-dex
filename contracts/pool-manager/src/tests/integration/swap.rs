use crate::math::Decimal256Helper;
use std::cell::RefCell;
use std::collections::HashMap;

use crate::tests::integration::helpers::extract_pool_reserves;
use crate::tests::suite::TestingSuite;
use cosmwasm_std::Decimal;
use cosmwasm_std::Uint128;
use cosmwasm_std::{assert_approx_eq, coin};
use cosmwasm_std::{Coin, Decimal256, Uint256};
use mantra_common_testing::multi_test::stargate_mock::StargateMock;
use mantra_dex_std::fee::Fee;
use mantra_dex_std::fee::PoolFee;
use mantra_dex_std::lp_common::MINIMUM_LIQUIDITY_AMOUNT;
use mantra_dex_std::pool_manager::{PoolType, SimulationResponse};

#[test]
fn basic_swapping_test() {
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

    let asset_infos = vec!["uwhale".to_string(), "uluna".to_string()];

    // Protocol fee is 0.01% and swap fee is 0.02% and burn fee is 0%
    let pool_fees = PoolFee {
        protocol_fee: Fee {
            share: Decimal::from_ratio(1u128, 100_000u128),
        },
        swap_fee: Fee {
            share: Decimal::from_ratio(1u128, 100_000u128),
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
        PoolType::ConstantProduct,
        Some("whale.uluna".to_string()),
        vec![coin(1000, "uusd"), coin(8888, "uom")],
        |result| {
            result.unwrap();
        },
    );

    // Query pool info to ensure the query is working fine
    suite.query_pools(Some("o.whale.uluna".to_string()), None, None, |result| {
        assert_eq!(
            result.unwrap().pools[0].pool_info.asset_decimals,
            vec![6u8, 6u8]
        );
    });

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
                // Ensure we got 999_000 in the response which is 1mil less the initial liquidity amount
                assert!(result.unwrap().events.iter().any(|event| {
                    event.attributes.iter().any(|attr| {
                        attr.key == "added_shares"
                            && attr.value
                                == (Uint128::from(1_000_000u128) - MINIMUM_LIQUIDITY_AMOUNT)
                                    .to_string()
                    })
                }));
            },
        )
        .query_pools(Some("o.whale.uluna".to_string()), None, None, |result| {
            let response = result.unwrap();
            assert_eq!(
                response.pools[0].total_share,
                Coin {
                    denom: response.pools[0].pool_info.lp_denom.clone(),
                    amount: Uint128::from(1_000_000u128),
                }
            );
        });

    let simulated_return_amount = RefCell::new(Uint128::zero());
    suite.query_simulation(
        "o.whale.uluna".to_string(),
        Coin {
            denom: "uwhale".to_string(),
            amount: Uint128::from(1000u128),
        },
        "uluna".to_string(),
        |result| {
            // Ensure that the return amount is 1_000 minus spread
            assert_eq!(
                result.as_ref().unwrap().return_amount + result.as_ref().unwrap().spread_amount,
                Uint128::from(1000u128)
            );
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
        "o.whale.uluna".to_string(),
        vec![coin(1000u128, "uwhale".to_string())],
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
    suite.query_reverse_simulation(
        "o.whale.uluna".to_string(),
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
    // Now Let's try a swap
    suite.swap(
        &creator,
        "uwhale".to_string(),
        None,
        None,
        None,
        "o.whale.uluna".to_string(),
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
}

#[test]
fn basic_swapping_pool_reserves_event_test() {
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
    // Asset infos with uwhale and uluna

    // Protocol fee is 0.01% and swap fee is 0.02% and burn fee is 0%
    let pool_fees = PoolFee {
        protocol_fee: Fee {
            share: Decimal::from_ratio(1u128, 100_000u128),
        },
        swap_fee: Fee {
            share: Decimal::from_ratio(1u128, 100_000u128),
        },
        burn_fee: Fee {
            share: Decimal::zero(),
        },
        extra_fees: vec![],
    };

    // Create a pool
    suite
        .instantiate_default()
        .add_one_epoch()
        .create_pool(
            &creator,
            vec!["uom".to_string(), "uusd".to_string()],
            vec![6u8, 6u8],
            pool_fees.clone(),
            PoolType::ConstantProduct,
            Some("uom.uusd".to_string()),
            vec![coin(1000, "uusd"), coin(8888, "uom")],
            |result| {
                result.unwrap();
            },
        )
        .create_pool(
            &creator,
            vec!["uluna".to_string(), "uusd".to_string()],
            vec![6u8, 6u8],
            pool_fees,
            PoolType::ConstantProduct,
            Some("uluna.uusd".to_string()),
            vec![coin(1000, "uusd"), coin(8888, "uom")],
            |result| {
                result.unwrap();
            },
        )
        .provide_liquidity(
            &creator,
            "o.uom.uusd".to_string(),
            None,
            None,
            None,
            None,
            None,
            vec![
                Coin {
                    denom: "uom".to_string(),
                    amount: Uint128::from(1000000u128),
                },
                Coin {
                    denom: "uusd".to_string(),
                    amount: Uint128::from(6000000u128),
                },
            ],
            |result| {
                result.unwrap();
            },
        )
        .provide_liquidity(
            &creator,
            "o.uluna.uusd".to_string(),
            None,
            None,
            None,
            None,
            None,
            vec![
                Coin {
                    denom: "uluna".to_string(),
                    amount: Uint128::from(3000000u128),
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

    let expected_pool_reserves = RefCell::<Vec<Vec<Coin>>>::new(vec![]);

    // Now Let's try a swap
    suite
        .swap(
            &creator,
            "uom".to_string(),
            None,
            None,
            None,
            "o.uom.uusd".to_string(),
            vec![coin(1000u128, "uusd".to_string())],
            |result| {
                for event in result.unwrap().events {
                    if event.ty == "wasm" {
                        for attribute in event.attributes {
                            match attribute.key.as_str() {
                                "pool_reserves" => {
                                    expected_pool_reserves.borrow_mut().clear();
                                    extract_pool_reserves(&attribute, &expected_pool_reserves);
                                    println!(
                                        "pool_reserves: {:?}",
                                        expected_pool_reserves.borrow()
                                    );
                                }
                                "pool_identifier" => {
                                    assert_eq!(attribute.value, "o.uom.uusd");
                                }
                                _ => {}
                            }
                        }
                    }
                }
            },
        )
        .query_pools(Some("o.uom.uusd".to_string()), None, None, |result| {
            let response = result.unwrap();
            let mut assets = response.pools[0].pool_info.assets.clone();
            assets.sort_by(|a, b| a.denom.cmp(&b.denom));
            assert_eq!(assets, expected_pool_reserves.borrow()[0]);
        })
        .query_pools(Some("o.uluna.uusd".to_string()), None, None, |result| {
            let response = result.unwrap();
            let mut assets = response.pools[0].pool_info.assets.clone();
            assets.sort_by(|a, b| a.denom.cmp(&b.denom));
            assert_eq!(
                assets,
                vec![
                    coin(3000000u128, "uluna".to_string()),
                    coin(1000000u128, "uusd".to_string())
                ]
            );
        });

    // now a swap via the router, single and multiswap
    let swap_operations = vec![mantra_dex_std::pool_manager::SwapOperation::MantraSwap {
        token_in_denom: "uom".to_string(),
        token_out_denom: "uusd".to_string(),
        pool_identifier: "o.uom.uusd".to_string(),
    }];

    suite
        .execute_swap_operations(
            &creator,
            swap_operations,
            None,
            None,
            Some(Decimal::percent(10)),
            vec![coin(1_000u128, "uom".to_string())],
            |result| {
                for event in result.unwrap().events {
                    if event.ty == "wasm" {
                        for attribute in event.attributes {
                            match attribute.key.as_str() {
                                "pool_reserves" => {
                                    expected_pool_reserves.borrow_mut().clear();
                                    extract_pool_reserves(&attribute, &expected_pool_reserves);
                                }
                                "pool_identifier" => {
                                    assert_eq!(attribute.value, "o.uom.uusd");
                                }
                                _ => {}
                            }
                        }
                    }
                }
            },
        )
        .query_pools(Some("o.uom.uusd".to_string()), None, None, |result| {
            let response = result.unwrap();
            let mut assets = response.pools[0].pool_info.assets.clone();
            assets.sort_by(|a, b| a.denom.cmp(&b.denom));
            assert_eq!(assets, expected_pool_reserves.borrow()[0]);
        })
        .query_pools(Some("o.uluna.uusd".to_string()), None, None, |result| {
            let response = result.unwrap();
            let mut assets = response.pools[0].pool_info.assets.clone();
            assets.sort_by(|a, b| a.denom.cmp(&b.denom));
            assert_eq!(
                assets,
                vec![
                    coin(3000000u128, "uluna".to_string()),
                    coin(1000000u128, "uusd".to_string())
                ]
            );
        });

    let swap_operations = vec![
        mantra_dex_std::pool_manager::SwapOperation::MantraSwap {
            token_in_denom: "uom".to_string(),
            token_out_denom: "uusd".to_string(),
            pool_identifier: "o.uom.uusd".to_string(),
        },
        mantra_dex_std::pool_manager::SwapOperation::MantraSwap {
            token_in_denom: "uusd".to_string(),
            token_out_denom: "uluna".to_string(),
            pool_identifier: "o.uluna.uusd".to_string(),
        },
    ];

    let expected_pool_reserves = RefCell::<Vec<Vec<Coin>>>::new(vec![]);

    suite
        .execute_swap_operations(
            &creator,
            swap_operations,
            None,
            None,
            Some(Decimal::percent(10)),
            vec![coin(1_000u128, "uom".to_string())],
            |result| {
                let mut pool_identifiers = vec![];

                expected_pool_reserves.borrow_mut().clear();
                for event in result.unwrap().events {
                    if event.ty == "wasm" {
                        for attribute in event.attributes {
                            match attribute.key.as_str() {
                                "pool_reserves" => {
                                    extract_pool_reserves(&attribute, &expected_pool_reserves);
                                }
                                "pool_identifier" => {
                                    pool_identifiers.push(attribute.value.clone());
                                }
                                _ => {}
                            }
                        }
                    }
                }
                assert_eq!(pool_identifiers, vec!["o.uom.uusd", "o.uluna.uusd"]);
            },
        )
        .query_pools(Some("o.uom.uusd".to_string()), None, None, |result| {
            let response = result.unwrap();
            let mut assets = response.pools[0].pool_info.assets.clone();
            assets.sort_by(|a, b| a.denom.cmp(&b.denom));
            assert_eq!(assets, expected_pool_reserves.borrow()[0]);
        })
        .query_pools(Some("o.uluna.uusd".to_string()), None, None, |result| {
            let response = result.unwrap();
            let mut assets = response.pools[0].pool_info.assets.clone();
            assets.sort_by(|a, b| a.denom.cmp(&b.denom));
            assert_eq!(assets, expected_pool_reserves.borrow()[1]);
        });
}

//todo remove when the stableswap issues are mitigated
//#[test]
fn basic_swapping_test_stable_swap_two_assets() {
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

    let asset_infos = vec!["uwhale".to_string(), "uluna".to_string()];

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

    // Create a stableswap pool with amp = 100
    suite.instantiate_default().add_one_epoch().create_pool(
        &creator,
        asset_infos,
        vec![6u8, 6u8],
        pool_fees,
        PoolType::StableSwap { amp: 100 },
        Some("whale.uluna".to_string()),
        vec![coin(1000, "uusd"), coin(8888, "uom")],
        |result| {
            result.unwrap();
        },
    );

    // Let's try to add liquidity
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
            // Ensure we got 999000 in the response which is 1mil less the initial liquidity amount
            for event in result.unwrap().events {
                println!("{:?}", event);
            }
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
        "o.whale.uluna".to_string(),
        vec![coin(1000u128, "uwhale".to_string())],
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
    suite.query_reverse_simulation(
        "o.whale.uluna".to_string(),
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
    // Now Let's try a swap
    suite.swap(
        &creator,
        "uwhale".to_string(),
        None,
        None,
        None,
        "o.whale.uluna".to_string(),
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
}

#[test]
fn swap_with_fees() {
    let mut suite = TestingSuite::default_with_balances(
        vec![
            coin(1_000_000_000_001u128, "uwhale".to_string()),
            coin(1_000_000_000_000u128, "uluna".to_string()),
            coin(1_000_000_000_001u128, "uusd".to_string()),
            coin(1_000_000_000_001u128, "uom".to_string()),
        ],
        StargateMock::new(vec![coin(8888u128, "uom".to_string())]),
    );
    let creator = suite.creator();
    let _other = suite.senders[1].clone();
    let _unauthorized = suite.senders[2].clone();
    // Asset infos with uwhale and uluna

    let asset_infos = vec!["uwhale".to_string(), "uluna".to_string()];

    // Protocol fee is 0.001% and swap fee is 0.002% and burn fee is 0%
    let pool_fees = PoolFee {
        protocol_fee: Fee {
            share: Decimal::from_ratio(1u128, 100_000u128),
        },
        swap_fee: Fee {
            share: Decimal::from_ratio(2u128, 100_000u128),
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
        PoolType::ConstantProduct,
        Some("whale.uluna".to_string()),
        vec![coin(1000, "uusd"), coin(8888, "uom")],
        |result| {
            result.unwrap();
        },
    );

    // Let's try to add liquidity, 1000 of each token.
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
                amount: Uint128::from(1000_000000u128),
            },
            Coin {
                denom: "uluna".to_string(),
                amount: Uint128::from(1000_000000u128),
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
        "uluna".to_string(),
        None,
        Some(Decimal::percent(1)),
        None,
        "o.whale.uluna".to_string(),
        vec![coin(10000000u128, "uwhale".to_string())],
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
                "0.011"
            );
        },
    );

    // Verify fee collection by querying the address of the fee collector and checking its balance
    // Should be 99 uLUNA
    suite.query_balance(
        &suite.fee_collector_addr.to_string(),
        "uluna".to_string(),
        |result| {
            assert_eq!(result.unwrap().amount, Uint128::from(99u128));
        },
    );
}

#[allow(clippy::inconsistent_digit_grouping)]
#[test]
fn swap_large_digits_xyk() {
    let mut suite = TestingSuite::default_with_balances(
        vec![
            coin(1_000_000_000_000u128, "uwhale".to_string()),
            coin(1_000_000_000_000u128, "uluna".to_string()),
            coin(1_000_000_000_000u128, "uosmo".to_string()),
            coin(1_000_000_000_000u128, "uusd".to_string()),
            coin(100_000_000_000_000_000000u128, "uusdc".to_string()),
            coin(
                100_000_000_000_000_000000000000000000u128,
                "ausdy".to_string(),
            ),
            coin(150_000_000_000_000_000000u128, "uom".to_string()),
        ],
        StargateMock::new(vec![coin(8888u128, "uom".to_string())]),
    );
    let alice = suite.creator();
    let bob = suite.senders[1].clone();
    let carol = suite.senders[2].clone();
    let dan = suite.senders[3].clone();

    let asset_denoms = vec!["uom".to_string(), "ausdy".to_string()];

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
        &alice,
        asset_denoms,
        vec![6u8, 18u8],
        pool_fees,
        PoolType::ConstantProduct,
        None,
        vec![coin(1000, "uusd"), coin(8888, "uom")],
        |result| {
            result.unwrap();
        },
    );

    let contract_addr = suite.pool_manager_addr.clone();
    let lp_denom = suite.get_lp_denom("p.1".to_string());

    // let's provide liquidity 150T om, 100T usdy
    suite
        .provide_liquidity(
            &bob,
            "p.1".to_string(),
            None,
            None,
            None,
            None,
            None,
            vec![
                Coin {
                    denom: "uom".to_string(),
                    amount: Uint128::new(150_000_000_000_000_000000u128),
                },
                Coin {
                    denom: "ausdy".to_string(),
                    amount: Uint128::new(100_000_000_000_000_000000000000000000u128),
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
        .query_balance(&carol.to_string(), "ausdy".to_string(), |result| {
            assert_eq!(
                result.unwrap().amount,
                Uint128::new(100_000_000_000_000_000000000000000000u128)
            );
        })
        .query_balance(&carol.to_string(), "uom".to_string(), |result| {
            assert_eq!(
                result.unwrap().amount,
                Uint128::new(150_000_000_000_000_000000u128)
            );
        })
        .query_simulation(
            "p.1".to_string(),
            Coin {
                denom: "ausdy".to_string(),
                amount: Uint128::new(2_000_000_000_000_000000000000000000u128),
            },
            "uom".to_string(),
            |result| {
                assert_eq!(
                    result.unwrap().return_amount,
                    Uint128::new(2_852_941_176_470_588236u128)
                );
            },
        )
        .swap(
            &carol,
            "uom".to_string(),
            None,
            Some(Decimal::percent(5)),
            None,
            "p.1".to_string(),
            vec![coin(
                2_000_000_000_000_000000000000000000u128,
                "ausdy".to_string(),
            )],
            |result| {
                result.unwrap();
            },
        )
        .query_balance(&carol.to_string(), "ausdy".to_string(), |result| {
            assert_eq!(
                result.unwrap().amount,
                Uint128::new(
                    100_000_000_000_000_000000000000000000u128
                        - 2_000_000_000_000_000000000000000000u128
                )
            );
        })
        .query_balance(&carol.to_string(), "uom".to_string(), |result| {
            assert_eq!(
                result.unwrap().amount,
                Uint128::new(150_000_000_000_000_000000u128 + 2_852_941_176_470_588236u128)
            );
        });

    // swap 10T om for usdy
    suite
        .query_balance(&dan.to_string(), "ausdy".to_string(), |result| {
            assert_eq!(
                result.unwrap().amount,
                Uint128::new(100_000_000_000_000_000000000000000000u128)
            );
        })
        .query_balance(&dan.to_string(), "uom".to_string(), |result| {
            assert_eq!(
                result.unwrap().amount,
                Uint128::new(150_000_000_000_000_000000u128)
            );
        })
        .query_simulation(
            "p.1".to_string(),
            Coin {
                denom: "uom".to_string(),
                amount: Uint128::new(10_000_000_000_000_000000u128),
            },
            "ausdy".to_string(),
            |result| {
                assert_eq!(
                    result.unwrap().return_amount,
                    Uint128::new(6_296_013_475_575_519371168089897701u128)
                );
            },
        )
        .swap(
            &dan,
            "ausdy".to_string(),
            None,
            Some(Decimal::percent(20)),
            None,
            "p.1".to_string(),
            vec![coin(10_000_000_000_000_000000u128, "uom".to_string())],
            |result| {
                result.unwrap();
            },
        )
        .query_balance(&dan.to_string(), "ausdy".to_string(), |result| {
            assert_eq!(
                result.unwrap().amount,
                Uint128::new(
                    100_000_000_000_000_000000000000000000u128
                        + 6_296_013_475_575_519371168089897701u128
                )
            );
        })
        .query_balance(&dan.to_string(), "uom".to_string(), |result| {
            assert_eq!(
                result.unwrap().amount,
                Uint128::new(150_000_000_000_000_000000u128 - 10_000_000_000_000_000000u128)
            );
        });
}

#[allow(clippy::inconsistent_digit_grouping)]
#[test]
fn swap_large_digits_stable() {
    let mut suite = TestingSuite::default_with_balances(
        vec![
            coin(1_000_000_000_000u128, "uwhale".to_string()),
            coin(1_000_000_000_000u128, "uluna".to_string()),
            coin(1_000_000_000_000u128, "uosmo".to_string()),
            coin(1_000_000_000_000u128, "uusd".to_string()),
            coin(100_000_000_000_000_000000u128, "uusdc".to_string()),
            coin(
                100_000_000_000_000_000000000000000000u128,
                "ausdy".to_string(),
            ),
            coin(150_000_000_000_000_000000u128, "uom".to_string()),
        ],
        StargateMock::new(vec![coin(8888u128, "uom".to_string())]),
    );
    let alice = suite.creator();
    let bob = suite.senders[1].clone();
    let carol = suite.senders[2].clone();
    let dan = suite.senders[3].clone();

    let asset_denoms = vec!["ausdy".to_string(), "uusdc".to_string()];

    let pool_fees = PoolFee {
        protocol_fee: Fee {
            share: Decimal::zero(),
        },
        swap_fee: Fee {
            share: Decimal::permille(5),
        },
        burn_fee: Fee {
            share: Decimal::zero(),
        },
        extra_fees: vec![],
    };

    // Create a pool
    suite.instantiate_default().add_one_epoch().create_pool(
        &alice,
        asset_denoms,
        vec![18u8, 6u8],
        pool_fees,
        PoolType::ConstantProduct,
        None,
        vec![coin(1000, "uusd"), coin(8888, "uom")],
        |result| {
            result.unwrap();
        },
    );

    // let's provide liquidity 200T usdc, 200T usdy
    suite
        .provide_liquidity(
            &alice,
            "p.1".to_string(),
            None,
            None,
            None,
            None,
            None,
            vec![
                Coin {
                    denom: "uusdc".to_string(),
                    amount: Uint128::new(100_000_000_000_000_000000u128),
                },
                Coin {
                    denom: "ausdy".to_string(),
                    amount: Uint128::new(100_000_000_000_000_000000000000000000u128),
                },
            ],
            |result| {
                result.unwrap();
            },
        )
        .provide_liquidity(
            &bob,
            "p.1".to_string(),
            None,
            None,
            None,
            None,
            None,
            vec![
                Coin {
                    denom: "uusdc".to_string(),
                    amount: Uint128::new(100_000_000_000_000_000000u128),
                },
                Coin {
                    denom: "ausdy".to_string(),
                    amount: Uint128::new(100_000_000_000_000_000000000000000000u128),
                },
            ],
            |result| {
                result.unwrap();
            },
        );

    // swap 10T usdc for usdy
    suite
        .query_balance(&carol.to_string(), "ausdy".to_string(), |result| {
            assert_eq!(
                result.unwrap().amount,
                Uint128::new(100_000_000_000_000_000000000000000000u128)
            );
        })
        .query_balance(&carol.to_string(), "uusdc".to_string(), |result| {
            assert_eq!(
                result.unwrap().amount,
                Uint128::new(100_000_000_000_000_000000u128)
            );
        })
        .query_simulation(
            "p.1".to_string(),
            Coin {
                denom: "uusdc".to_string(),
                amount: Uint128::new(10_000_000_000_000_000000u128),
            },
            "ausdy".to_string(),
            |result| {
                assert_eq!(
                    result.unwrap().return_amount,
                    Uint128::new(9_476_190_476_190_476190476190476190u128)
                );
            },
        )
        .swap(
            &carol,
            "ausdy".to_string(),
            None,
            Some(Decimal::percent(6)),
            None,
            "p.1".to_string(),
            vec![coin(10_000_000_000_000_000000u128, "uusdc".to_string())],
            |result| {
                result.unwrap();
            },
        )
        .query_balance(&carol.to_string(), "ausdy".to_string(), |result| {
            assert_eq!(
                result.unwrap().amount,
                Uint128::new(
                    100_000_000_000_000_000000000000000000u128
                        + 9_476_190_476_190_476190476190476190u128
                )
            );
        })
        .query_balance(&carol.to_string(), "uusdc".to_string(), |result| {
            assert_eq!(
                result.unwrap().amount,
                Uint128::new(100_000_000_000_000_000000u128 - 10_000_000_000_000_000000u128)
            );
        });

    // swap 20T usdy for usdc
    suite
        .query_balance(&dan.to_string(), "ausdy".to_string(), |result| {
            assert_eq!(
                result.unwrap().amount,
                Uint128::new(100_000_000_000_000_000000000000000000u128)
            );
        })
        .query_balance(&dan.to_string(), "uusdc".to_string(), |result| {
            assert_eq!(
                result.unwrap().amount,
                Uint128::new(100_000_000_000_000_000000u128)
            );
        })
        .query_simulation(
            "p.1".to_string(),
            Coin {
                denom: "ausdy".to_string(),
                amount: Uint128::new(20_000_000_000_000_000000000000000000u128),
            },
            "uusdc".to_string(),
            |result| {
                assert_eq!(
                    result.unwrap().return_amount,
                    Uint128::new(19_850_486_315_313_277539u128)
                );
            },
        )
        .swap(
            &dan,
            "uusdc".to_string(),
            None,
            Some(Decimal::percent(10)),
            None,
            "p.1".to_string(),
            vec![coin(
                20_000_000_000_000_000000000000000000u128,
                "ausdy".to_string(),
            )],
            |result| {
                result.unwrap();
            },
        )
        .query_balance(&dan.to_string(), "ausdy".to_string(), |result| {
            assert_eq!(
                result.unwrap().amount,
                Uint128::new(
                    100_000_000_000_000_000000000000000000u128
                        - 20_000_000_000_000_000000000000000000u128
                )
            );
        })
        .query_balance(&dan.to_string(), "uusdc".to_string(), |result| {
            assert_eq!(
                result.unwrap().amount,
                Uint128::new(100_000_000_000_000_000000u128 + 19_850_486_315_313_277539u128)
            );
        });
}

#[allow(clippy::inconsistent_digit_grouping)]
#[test]
fn swap_large_digits_stable_18_digits() {
    let mut suite = TestingSuite::default_with_balances(
        vec![
            coin(1_000_000_000_000u128, "uwhale".to_string()),
            coin(1_000_000_000_000u128, "uluna".to_string()),
            coin(1_000_000_000_000u128, "uosmo".to_string()),
            coin(1_000_000_000_000u128, "uusd".to_string()),
            coin(1_000_000_000_000u128, "uusdc".to_string()),
            coin(
                300_000_000_000_000_000000000000000000u128,
                "ausdy".to_string(),
            ),
            coin(
                300_000_000_000_000_000000000000000000u128,
                "pusdc".to_string(),
            ),
            coin(1_000_000_000_000u128, "uom".to_string()),
        ],
        StargateMock::new(vec![coin(8888u128, "uom".to_string())]),
    );
    let alice = suite.creator();
    let bob = suite.senders[1].clone();
    let carol = suite.senders[2].clone();

    let asset_denoms = vec!["ausdy".to_string(), "pusdc".to_string()];

    let pool_fees = PoolFee {
        protocol_fee: Fee {
            share: Decimal::zero(),
        },
        swap_fee: Fee {
            share: Decimal::permille(5),
        },
        burn_fee: Fee {
            share: Decimal::zero(),
        },
        extra_fees: vec![],
    };

    // Create a pool
    suite.instantiate_default().add_one_epoch().create_pool(
        &alice,
        asset_denoms,
        vec![18u8, 18u8],
        pool_fees,
        PoolType::ConstantProduct,
        None,
        vec![coin(1000, "uusd"), coin(8888, "uom")],
        |result| {
            result.unwrap();
        },
    );

    // let's provide liquidity 300T pusdc, 300T usdy
    suite.provide_liquidity(
        &alice,
        "p.1".to_string(),
        None,
        None,
        None,
        None,
        None,
        vec![
            Coin {
                denom: "pusdc".to_string(),
                amount: Uint128::new(300_000_000_000_000_000000000000000000u128),
            },
            Coin {
                denom: "ausdy".to_string(),
                amount: Uint128::new(300_000_000_000_000_000000000000000000u128),
            },
        ],
        |result| {
            result.unwrap();
        },
    );

    // swap 100T pusdc for usdy
    suite
        .query_balance(&bob.to_string(), "ausdy".to_string(), |result| {
            assert_eq!(
                result.unwrap().amount,
                Uint128::new(300_000_000_000_000_000000000000000000u128)
            );
        })
        .query_balance(&bob.to_string(), "pusdc".to_string(), |result| {
            assert_eq!(
                result.unwrap().amount,
                Uint128::new(300_000_000_000_000_000000000000000000u128)
            );
        })
        .query_simulation(
            "p.1".to_string(),
            Coin {
                denom: "pusdc".to_string(),
                amount: Uint128::new(100_000_000_000_000_000000000000000000u128),
            },
            "ausdy".to_string(),
            |result| {
                assert_eq!(
                    result.unwrap().return_amount,
                    Uint128::new(74_625_000_000_000_000000000000000000u128)
                );
            },
        )
        .swap(
            &bob,
            "ausdy".to_string(),
            None,
            Some(Decimal::percent(30)),
            None,
            "p.1".to_string(),
            vec![coin(
                100_000_000_000_000_000000000000000000u128,
                "pusdc".to_string(),
            )],
            |result| {
                result.unwrap();
            },
        )
        .query_balance(&bob.to_string(), "ausdy".to_string(), |result| {
            assert_eq!(
                result.unwrap().amount,
                Uint128::new(
                    300_000_000_000_000_000000000000000000u128
                        + 74_625_000_000_000_000000000000000000u128
                )
            );
        })
        .query_balance(&bob.to_string(), "pusdc".to_string(), |result| {
            assert_eq!(
                result.unwrap().amount,
                Uint128::new(
                    300_000_000_000_000_000000000000000000u128
                        - 100_000_000_000_000_000000000000000000u128
                )
            );
        });

    // swap 50T usdy for pusdc
    suite
        .query_balance(&carol.to_string(), "ausdy".to_string(), |result| {
            assert_eq!(
                result.unwrap().amount,
                Uint128::new(300_000_000_000_000_000000000000000000u128)
            );
        })
        .query_balance(&carol.to_string(), "pusdc".to_string(), |result| {
            assert_eq!(
                result.unwrap().amount,
                Uint128::new(300_000_000_000_000_000000000000000000u128)
            );
        })
        .query_simulation(
            "p.1".to_string(),
            Coin {
                denom: "ausdy".to_string(),
                amount: Uint128::new(50_000_000_000_000_000000000000000000u128),
            },
            "pusdc".to_string(),
            |result| {
                assert_eq!(
                    result.unwrap().return_amount,
                    Uint128::new(72_265_093_054_925_102133454380390377u128)
                );
            },
        )
        .swap(
            &carol,
            "pusdc".to_string(),
            None,
            Some(Decimal::percent(20)),
            None,
            "p.1".to_string(),
            vec![coin(
                50_000_000_000_000_000000000000000000u128,
                "ausdy".to_string(),
            )],
            |result| {
                result.unwrap();
            },
        )
        .query_balance(&carol.to_string(), "ausdy".to_string(), |result| {
            assert_eq!(
                result.unwrap().amount,
                Uint128::new(
                    300_000_000_000_000_000000000000000000u128
                        - 50_000_000_000_000_000000000000000000u128
                )
            );
        })
        .query_balance(&carol.to_string(), "pusdc".to_string(), |result| {
            assert_eq!(
                result.unwrap().amount,
                Uint128::new(
                    300_000_000_000_000_000000000000000000u128
                        + 72_265_093_054_925_102133454380390377u128
                )
            );
        });
}

#[allow(clippy::inconsistent_digit_grouping)]
#[test]
fn swap_3pool_same_decimals() {
    let mut suite = TestingSuite::default_with_balances(
        vec![
            coin(
                300_000_000_000_000_000000000000000000u128,
                "uusd".to_string(),
            ),
            coin(
                300_000_000_000_000_000000000000000000u128,
                "uusdc".to_string(),
            ),
            coin(
                300_000_000_000_000_000000000000000000u128,
                "uusdt".to_string(),
            ),
            coin(1_000_000_000_000u128, "uom".to_string()),
        ],
        StargateMock::new(vec![coin(8888u128, "uom".to_string())]),
    );
    let alice = suite.creator();
    let bob = suite.senders[1].clone();
    let carol = suite.senders[2].clone();

    let asset_denoms = vec!["uusd".to_string(), "uusdc".to_string(), "uusdt".to_string()];

    let pool_fees = PoolFee {
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
    };

    // Create a pool
    suite.instantiate_default().add_one_epoch().create_pool(
        &alice,
        asset_denoms,
        vec![6u8, 6u8, 6u8],
        pool_fees,
        PoolType::StableSwap { amp: 85 },
        None,
        vec![coin(1000, "uusd"), coin(8888, "uom")],
        |result| {
            result.unwrap();
        },
    );

    // let's provide liquidity
    suite.provide_liquidity(
        &alice,
        "p.1".to_string(),
        None,
        None,
        None,
        None,
        None,
        vec![
            Coin {
                denom: "uusdc".to_string(),
                amount: Uint128::new(1_000_000_000u128),
            },
            Coin {
                denom: "uusd".to_string(),
                amount: Uint128::new(1_000_000_000u128),
            },
            Coin {
                denom: "uusdt".to_string(),
                amount: Uint128::new(1_000_000_000u128),
                // amount: Uint128::new(1_000_000_000_000_000_000_000u128),
            },
        ],
        |result| {
            result.unwrap();
        },
    );

    suite
        .query_balance(&bob.to_string(), "uusdc".to_string(), |result| {
            assert_eq!(
                result.unwrap().amount,
                Uint128::new(300_000_000_000_000_000000000000000000u128)
            );
        })
        .query_balance(&bob.to_string(), "uusdt".to_string(), |result| {
            assert_eq!(
                result.unwrap().amount,
                Uint128::new(300_000_000_000_000_000000000000000000u128)
            );
        })
        .query_simulation(
            "p.1".to_string(),
            Coin {
                denom: "uusdc".to_string(),
                amount: Uint128::new(200_000_000u128),
            },
            "uusdt".to_string(),
            |result| {
                assert_eq!(result.unwrap().return_amount, Uint128::new(199_517_195u128));
            },
        )
        .swap(
            &bob,
            "uusdt".to_string(),
            None,
            Some(Decimal::percent(30)),
            None,
            "p.1".to_string(),
            vec![coin(200_000_000u128, "uusdc".to_string())],
            |result| {
                result.unwrap();
            },
        )
        .query_balance(&bob.to_string(), "uusdc".to_string(), |result| {
            assert_eq!(
                result.unwrap().amount,
                Uint128::new(300_000_000_000_000_000000000000000000u128 - 200_000_000u128)
            );
        })
        .query_balance(&bob.to_string(), "uusdt".to_string(), |result| {
            assert_eq!(
                result.unwrap().amount,
                Uint128::new(300_000_000_000_000_000000000000000000u128 + 199_517_195u128)
            );
        });
}

#[allow(clippy::inconsistent_digit_grouping)]
#[test]
fn swap_3pool_different_decimals() {
    let mut suite = setup_3pool_different_decimals(None, None, None, None);
    let bob = suite.senders[1].clone();

    println!("swap1");
    let return_amount = RefCell::new(Uint128::zero());
    let swap_amount = Uint128::new(200_000000000000u128);

    let bob_uusdc_balance = RefCell::new(Uint128::zero());
    let bob_uusdt_balance = RefCell::new(Uint128::zero());
    let bob_uusd_balance = RefCell::new(Uint128::zero());

    suite
        .query_balance(&bob.to_string(), "uusd".to_string(), |result| {
            bob_uusd_balance
                .borrow_mut()
                .clone_from(&result.unwrap().amount);
        })
        .query_balance(&bob.to_string(), "uusdc".to_string(), |result| {
            bob_uusdc_balance
                .borrow_mut()
                .clone_from(&result.unwrap().amount);
        })
        .query_balance(&bob.to_string(), "uusdt".to_string(), |result| {
            bob_uusdt_balance
                .borrow_mut()
                .clone_from(&result.unwrap().amount);
        });

    suite
        .query_simulation(
            "p.1".to_string(),
            Coin {
                denom: "uusdc".to_string(),
                amount: swap_amount.clone(),
            },
            "uusdt".to_string(),
            |result| {
                let response: SimulationResponse = result.unwrap();
                println!("return_amount.x: {:?}", response.return_amount);

                return_amount
                    .borrow_mut()
                    .clone_from(&response.return_amount);
            },
        )
        .swap(
            &bob,
            "uusdt".to_string(),
            None,
            Some(Decimal::percent(30)),
            None,
            "p.1".to_string(),
            vec![coin(swap_amount.u128(), "uusdc".to_string())],
            |result| {
                result.unwrap();
            },
        )
        .query_balance(&bob.to_string(), "uusdc".to_string(), |result| {
            let balance = result.unwrap().amount;
            assert_eq!(
                balance,
                Uint128::new(bob_uusdc_balance.borrow().u128() - swap_amount.u128())
            );
            bob_uusdc_balance.borrow_mut().clone_from(&balance);
        })
        .query_balance(&bob.to_string(), "uusdt".to_string(), |result| {
            let balance = result.unwrap().amount;
            assert_eq!(
                balance,
                Uint128::new(bob_uusdt_balance.borrow().u128() + return_amount.borrow().u128())
            );
            bob_uusdt_balance.borrow_mut().clone_from(&balance);
        });

    println!("swap2");
    let swap_amount = Uint128::new(10_000_000000000000000000u128);

    suite
        .query_simulation(
            "p.1".to_string(),
            Coin {
                denom: "uusdt".to_string(),
                amount: swap_amount.clone(),
            },
            "uusdc".to_string(),
            |result| {
                let response: SimulationResponse = result.unwrap();
                println!(
                    "response.xxxxx.return_amount : {:?}",
                    response.return_amount
                );

                return_amount
                    .borrow_mut()
                    .clone_from(&response.return_amount);
            },
        )
        .swap(
            &bob,
            "uusdc".to_string(),
            None,
            Some(Decimal::percent(30)),
            None,
            "p.1".to_string(),
            vec![coin(swap_amount.u128(), "uusdt".to_string())],
            |result| {
                result.unwrap();
            },
        )
        .query_balance(&bob.to_string(), "uusdc".to_string(), |result| {
            let balance = result.unwrap().amount;
            assert_eq!(
                balance,
                Uint128::new(bob_uusdc_balance.borrow().u128() + return_amount.borrow().u128())
            );
            bob_uusdc_balance.borrow_mut().clone_from(&balance);
        })
        .query_balance(&bob.to_string(), "uusdt".to_string(), |result| {
            let balance = result.unwrap().amount;
            assert_eq!(
                balance,
                Uint128::new(bob_uusdt_balance.borrow().u128() - swap_amount.u128())
            );
            bob_uusdt_balance.borrow_mut().clone_from(&balance);
        });

    println!("swap3");

    let swap_amount = Uint128::new(10_000_000000000000000000u128);

    suite
        .query_simulation(
            "p.1".to_string(),
            Coin {
                denom: "uusdt".to_string(),
                amount: swap_amount.clone(),
            },
            "uusd".to_string(),
            |result| {
                let response: SimulationResponse = result.unwrap();
                println!(
                    "response.return_amountresponse.return_amount : {:?}",
                    response.return_amount
                );
                return_amount
                    .borrow_mut()
                    .clone_from(&response.return_amount);
            },
        )
        .swap(
            &bob,
            "uusd".to_string(),
            None,
            Some(Decimal::percent(30)),
            None,
            "p.1".to_string(),
            vec![coin(swap_amount.u128(), "uusdt".to_string())],
            |result| {
                result.unwrap();
            },
        )
        .query_balance(&bob.to_string(), "uusd".to_string(), |result| {
            let balance = result.unwrap().amount;
            assert_eq!(
                balance,
                Uint128::new(bob_uusd_balance.borrow().u128() + return_amount.borrow().u128())
            );
            bob_uusd_balance.borrow_mut().clone_from(&balance);
        })
        .query_balance(&bob.to_string(), "uusdt".to_string(), |result| {
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
        (
            "uusd",
            "uusdc",
            Uint128::new(333_000000u128), // 333 USDC with 6 decimals
        ),
        (
            "uusdc",
            "uusdt",
            Uint128::new(5_000000000000u128), // 5 USDT with 12 decimals
        ),
        // Medium amounts
        (
            "uusdt",
            "uusd",
            Uint128::new(10_000_000000000000000000u128), // 10k USD with 18 decimals
        ),
        // Large amounts
        (
            "uusd",
            "uusdt",
            Uint128::new(1_000_000_000000u128), // 1M USD with 6 decimals
        ),
    ];
    let mut decimals_by_denom: HashMap<String, u8> = HashMap::new();
    decimals_by_denom.insert("uusd".to_string(), 6);
    decimals_by_denom.insert("uusdc".to_string(), 12);
    decimals_by_denom.insert("uusdt".to_string(), 18);

    // Test each case
    for (token_in, token_out, amount_in) in test_cases {
        let token_in_decimals = decimals_by_denom[&token_in.to_string()];

        let simulated_return = RefCell::new(Uint128::zero());
        let simulated_spread = RefCell::new(Uint128::zero());
        let reverse_simulated_offer = RefCell::new(Uint128::zero());
        let reverse_simulated_spread = RefCell::new(Uint128::zero());

        // Forward simulation
        suite.query_simulation(
            "p.1".to_string(),
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
                    .clone_from(&response.spread_amount);
            },
        );

        // Reverse simulation using the forward simulation result
        suite.query_reverse_simulation(
            "p.1".to_string(),
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
                    .clone_from(&response.spread_amount);
            },
        );

        // Compare the original amount_in with the reverse simulated offer amount
        // They should be very close, with a small difference due to rounding
        // Using 0.01% tolerance for comparison
        // 1bps tolerance
        let tolerance = Decimal::from_ratio(Uint128::from(1u128), Uint128::from(10000u128));
        let amount_in_decimal =
            Decimal::from_ratio(amount_in, 10u128.pow(u32::from(token_in_decimals)));
        let reverse_offer_decimal = Decimal::from_ratio(
            *reverse_simulated_offer.borrow(),
            10u128.pow(u32::from(token_in_decimals)),
        );

        let diff = if amount_in_decimal > reverse_offer_decimal {
            amount_in_decimal - reverse_offer_decimal
        } else {
            reverse_offer_decimal - amount_in_decimal
        };

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
            coin(1_000u128 * 10u128.pow(6), "uusd".to_string()),
            coin(1_000_000_000_000u128 * 10u128.pow(18), "uweth".to_string()),
            coin(10_000u128, "uom".to_string()),
        ],
        StargateMock::new(vec![coin(8888u128, "uom".to_string())]),
    );
    let creator = suite.creator();
    let user = suite.senders[1].clone();

    suite
        .instantiate_default()
        .add_one_epoch()
        .create_pool(
            &creator,
            vec!["uusd".to_string(), "uweth".to_string()],
            vec![6u8, 18u8],
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
            PoolType::ConstantProduct,
            Some("uusd.uweth".to_string()),
            vec![coin(1000, "uusd"), coin(8888, "uom")],
            |result| {
                result.unwrap();
            },
        )
        .provide_liquidity(
            &user,
            "o.uusd.uweth".to_string(),
            None,
            None,
            None,
            None,
            None,
            vec![
                Coin {
                    denom: "uusd".to_string(),
                    amount: Uint128::from(100 * 10u128.pow(6)),
                },
                Coin {
                    denom: "uweth".to_string(),
                    amount: Uint128::from(100 * 10u128.pow(18)),
                },
            ],
            |result| {
                result.unwrap();
            },
        );

    // swap 18 decimals to 6 decimals
    let simulated_return_amount = RefCell::new(Uint128::zero());
    suite.query_simulation(
        "o.uusd.uweth".to_string(),
        Coin {
            denom: "uweth".to_string(),
            amount: Uint128::from(10u128 * 10u128.pow(18)),
        },
        "uusd".to_string(),
        |result| {
            let response: SimulationResponse = result.unwrap();
            simulated_return_amount
                .borrow_mut()
                .clone_from(&response.return_amount);
        },
    );

    suite.swap(
        &user,
        "uusd".to_string(),
        // belief_price = offer / ask
        Some(Decimal::from_ratio(
            Uint128::new(10_000_000_000_000_000_000),
            *simulated_return_amount.borrow(),
        )),
        None,
        None,
        "o.uusd.uweth".to_string(),
        vec![coin(10u128 * 10u128.pow(18), "uweth".to_string())],
        |result| {
            result.unwrap();
        },
    );

    // swap 6 decimals to 18 decimals
    suite.query_simulation(
        "o.uusd.uweth".to_string(),
        Coin {
            denom: "uusd".to_string(),
            amount: Uint128::from(10u128 * 10u128.pow(6)),
        },
        "uweth".to_string(),
        |result| {
            let response: SimulationResponse = result.unwrap();
            simulated_return_amount
                .borrow_mut()
                .clone_from(&response.return_amount);
        },
    );

    suite.swap(
        &user,
        "uweth".to_string(),
        // belief_price = offer / ask
        Some(Decimal::from_ratio(
            Uint128::new(10_000_000),
            *simulated_return_amount.borrow(),
        )),
        None,
        None,
        "o.uusd.uweth".to_string(),
        vec![coin(10u128 * 10u128.pow(6), "uusd".to_string())],
        |result| {
            result.unwrap();
        },
    );
}

#[test]
fn compute_offer_amount_floor() {
    let mut suite = TestingSuite::default_with_balances(
        vec![
            coin(100_000_000u128 * 10u128.pow(6), "uluna".to_string()),
            coin(100_000_000u128 * 10u128.pow(6), "uusd".to_string()),
            coin(10_000u128, "uom".to_string()),
        ],
        StargateMock::new(vec![coin(8888u128, "uom".to_string())]),
    );
    let creator = suite.creator();
    let user = suite.senders[1].clone();
    let pool_id = "o.uluna.uusd".to_string();

    suite
        .instantiate_default()
        .add_one_epoch()
        .create_pool(
            &creator,
            vec!["uluna".to_string(), "uusd".to_string()],
            vec![6u8, 6u8],
            PoolFee {
                protocol_fee: Fee {
                    share: Decimal::percent(0),
                },
                swap_fee: Fee {
                    share: Decimal::percent(0),
                },
                burn_fee: Fee {
                    share: Decimal::percent(0),
                },
                extra_fees: vec![],
            },
            PoolType::ConstantProduct,
            Some("uluna.uusd".to_string()),
            vec![coin(1000, "uusd"), coin(8888, "uom")],
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
                    denom: "uluna".to_string(),
                    amount: Uint128::from(100_000u128 * 10u128.pow(6)),
                },
                Coin {
                    denom: "uusd".to_string(),
                    amount: Uint128::from(100_000u128 * 10u128.pow(6)),
                },
            ],
            |result| {
                result.unwrap();
            },
        );

    // Need uusd amount (amount out)
    let needed_uusd = Uint128::from(99_900_099u128);
    let offer_uluna = std::cell::RefCell::new(Uint128::zero());

    suite
        .query_reverse_simulation(
            pool_id.clone(),
            Coin {
                denom: "uusd".to_string(),
                amount: needed_uusd,
            },
            "uluna".to_string(),
            |result| {
                // Computed the amount of uluna (amount in)
                *offer_uluna.borrow_mut() = result.unwrap().offer_amount;
            },
        )
        // Swap using the computed amount in
        .swap(
            &user,
            "uusd".to_string(),
            None,
            None,
            None,
            pool_id.clone(),
            vec![coin((*offer_uluna.borrow()).into(), "uluna".to_string())],
            |result| {
                for event in result.unwrap().events {
                    if event.ty == "wasm" {
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
    let decimals = asset_decimals.unwrap_or_else(|| vec![6u8, 12u8, 18u8]);
    let amp = amp.unwrap_or(85);

    // Default initial balances (300T for each token)
    let balances = initial_balances.unwrap_or_else(|| {
        vec![
            Uint128::new(300_000_000_000_000_000000000000000000u128),
            Uint128::new(300_000_000_000_000_000000000000000000u128),
            Uint128::new(300_000_000_000_000_000000000000000000u128),
        ]
    });

    // Default initial liquidity (100T for each token)
    let liquidity = initial_liquidity.unwrap_or_else(|| {
        vec![
            Uint128::new(100_000_000_000_000_000000u128), // 100T with 6 decimals
            Uint128::new(100_000_000_000_000_000000000000u128), // 100T with 12 decimals
            Uint128::new(100_000_000_000_000_000000000000000000u128), // 100T with 18 decimals
        ]
    });

    let asset_denoms = vec!["uusd".to_string(), "uusdc".to_string(), "uusdt".to_string()];

    // Create test suite with initial balances
    let mut suite = TestingSuite::default_with_balances(
        vec![
            coin(balances[0].u128(), "uusd".to_string()),
            coin(balances[1].u128(), "uusdc".to_string()),
            coin(balances[2].u128(), "uusdt".to_string()),
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
            share: Decimal::zero(),
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
                denom: "uusdc".to_string(),
                amount: liquidity[1],
            },
            Coin {
                denom: "uusdt".to_string(),
                amount: liquidity[2],
            },
        ],
        |result| {
            result.unwrap();
        },
    );

    suite
}
