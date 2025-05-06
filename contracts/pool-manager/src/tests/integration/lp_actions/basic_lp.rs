use std::cell::RefCell;

use cosmwasm_std::{assert_approx_eq, coin, Coin, Decimal, Uint128};
use mantra_common_testing::multi_test::stargate_mock::StargateMock;
use mantra_dex_std::{
    fee::{Fee, PoolFee},
    lp_common::MINIMUM_LIQUIDITY_AMOUNT,
    pool_manager::PoolType,
};

use crate::tests::{integration::helpers::extract_pool_reserves, suite::TestingSuite};

#[test]
fn deposit_and_withdraw_sanity_check() {
    let mut suite = TestingSuite::default_with_balances(
        vec![
            coin(1_000_000u128, "uwhale".to_string()),
            coin(1_000_000u128, "uluna".to_string()),
            coin(1_000u128, "uusd".to_string()),
            coin(10_000u128, "uom".to_string()),
            coin(10_000u128, "utest".to_string()),
        ],
        StargateMock::new(vec![coin(1000u128, "utest".to_string())]),
    );
    let creator = suite.creator();
    let _other = suite.senders[1].clone();
    let _unauthorized = suite.senders[2].clone();

    // Asset denoms with uwhale and uluna
    let asset_denoms = vec!["uwhale".to_string(), "uluna".to_string()];

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
        &creator,
        asset_denoms,
        vec![6u8, 6u8],
        pool_fees,
        PoolType::ConstantProduct,
        Some("whale.uluna".to_string()),
        vec![coin(1000, "uusd"), coin(1000, "utest")],
        |result| {
            result.unwrap();
        },
    );

    let contract_addr = suite.pool_manager_addr.clone();
    let lp_denom = suite.get_lp_denom("o.whale.uluna".to_string());
    let expected_pool_reserves = RefCell::<Vec<Vec<Coin>>>::new(vec![]);

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
                    amount: Uint128::from(1_000_000u128),
                },
                Coin {
                    denom: "uluna".to_string(),
                    amount: Uint128::from(1_000_000u128),
                },
            ],
            |result| {
                let events = result.unwrap().events;
                // Ensure we got 999_000 in the response which is 1_000_000 less the initial liquidity amount
                assert!(events.iter().any(|event| {
                    if event.ty == "wasm" {
                        for attribute in &event.attributes {
                            if attribute.key.as_str() == "pool_reserves" {
                                extract_pool_reserves(attribute, &expected_pool_reserves);
                            }
                        }
                    }
                    event.attributes.iter().any(|attr| {
                        attr.key == "added_shares"
                            && attr.value
                                == (Uint128::from(1_000_000u128) - MINIMUM_LIQUIDITY_AMOUNT)
                                    .to_string()
                    })
                }));
            },
        )
        // creator should have 999_000 LP shares (1M - MINIMUM_LIQUIDITY_AMOUNT)
        .query_all_balances(&creator.to_string(), |result| {
            let balances = result.unwrap();

            assert!(balances.iter().any(|coin| {
                coin.denom == lp_denom && coin.amount == Uint128::from(999_000u128)
            }));
        })
        // contract should have 1_000 LP shares (MINIMUM_LIQUIDITY_AMOUNT)
        .query_all_balances(&contract_addr.to_string(), |result| {
            let balances = result.unwrap();
            // check that balances has 999_000 factory/mantra1zwv6feuzhy6a9wekh96cd57lsarmqlwxdypdsplw6zhfncqw6ftqlydlr9/o.whale.uluna.LP
            assert!(balances.iter().any(|coin| {
                coin.denom == lp_denom.clone() && coin.amount == MINIMUM_LIQUIDITY_AMOUNT
            }));
        })
        .query_pools(Some("o.whale.uluna".to_string()), None, None, |result| {
            let response = result.unwrap();
            let mut assets = response.pools[0].pool_info.assets.clone();
            assets.sort_by(|a, b| a.denom.cmp(&b.denom));
            assert_eq!(assets, expected_pool_reserves.borrow()[0]);
        });

    // Let's try to withdraw liquidity
    expected_pool_reserves.borrow_mut().clear();
    suite
        .withdraw_liquidity(
            &creator,
            "o.whale.uluna".to_string(),
            vec![Coin {
                denom: lp_denom.clone(),
                amount: Uint128::from(999_000u128),
            }],
            |result| {
                let events = result.unwrap().events;
                for event in &events {
                    if event.ty.as_str() == "wasm" {
                        for attribute in &event.attributes {
                            match attribute.key.as_str() {
                                "pool_reserves" => {
                                    extract_pool_reserves(attribute, &expected_pool_reserves);
                                }
                                "pool_identifier" => {
                                    assert_eq!(attribute.value, "o.whale.uluna");
                                }
                                _ => {}
                            }
                        }
                    }
                }
                // we're trading 999_000 shares for 1_000_000 of our liquidity
                assert!(events.iter().any(|event| {
                    event.attributes.iter().any(|attr| {
                        attr.key == "withdrawn_shares"
                            && attr.value
                                == (Uint128::from(1_000_000u128) - MINIMUM_LIQUIDITY_AMOUNT)
                                    .to_string()
                    })
                }));
            },
        )
        // creator should have 0 LP shares in the contract and 0 LP shares in their account balance
        .query_amount_of_lp_token(
            "o.whale.uluna".to_string(),
            &creator.to_string(),
            |result| {
                assert_eq!(result.unwrap(), Uint128::zero());
            },
        )
        .query_balance(&creator.to_string(), lp_denom, |result| {
            assert_eq!(result.unwrap().amount, Uint128::zero());
        })
        // creator should 999_000 uwhale and 999_000 uluna (1M - MINIMUM_LIQUIDITY_AMOUNT)
        .query_all_balances(&creator.to_string(), |result| {
            let balances = result.unwrap();
            assert!(balances.iter().any(|coin| {
                coin.denom == *"uwhale"
                    && coin.amount == Uint128::from(1_000_000u128) - MINIMUM_LIQUIDITY_AMOUNT
            }));
            assert!(balances.iter().any(|coin| {
                coin.denom == *"uluna"
                    && coin.amount == Uint128::from(1_000_000u128) - MINIMUM_LIQUIDITY_AMOUNT
            }));
        })
        .query_pools(Some("o.whale.uluna".to_string()), None, None, |result| {
            let response = result.unwrap();
            let mut assets = response.pools[0].pool_info.assets.clone();
            assets.sort_by(|a, b| a.denom.cmp(&b.denom));
            assert_eq!(assets, expected_pool_reserves.borrow()[0]);
        });
}

#[test]
fn lp_mint_stableswap_different_decimals_scaling_min_liquidity() {
    let mut suite = TestingSuite::default_with_balances(
        vec![
            coin(1_000_00000000000000u128, "uusdc".to_string()),
            coin(1_000_00000000000000u128, "uusdt".to_string()),
            coin(1_000u128, "uusd".to_string()),
            coin(10_000u128, "uom".to_string()),
            coin(10_000u128, "utest".to_string()),
        ],
        StargateMock::new(vec![coin(1000u128, "utest".to_string())]),
    );
    let alice = suite.creator();
    let bob = suite.senders[1].clone();
    let asset_denoms = vec!["uusdc".to_string(), "uusdt".to_string()];

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
        vec![6u8, 18u8],
        pool_fees,
        PoolType::StableSwap { amp: 85 },
        None,
        vec![coin(1000, "uusd"), coin(1000, "utest")],
        |result| {
            result.unwrap();
        },
    );

    // Let's try to add liquidity
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
                coin(10u128.pow(3), "uusdc".to_string()),
                coin(10u128.pow(15), "uusdt".to_string()),
            ],
            |result| {
                println!("result: {:?}", result);
                result.unwrap();
                println!("---");
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
                coin(10u128.pow(3), "uusdc".to_string()),
                coin(10u128.pow(15), "uusdt".to_string()),
            ],
            |result| {
                println!("result: {:?}", result);
                result.unwrap();
            },
        );

    let lp_shares_alice = RefCell::new(Coin::new(0u128, "".to_string()));
    let uusdc_alice = RefCell::new(Coin::new(0u128, "".to_string()));
    let uusdt_alice = RefCell::new(Coin::new(0u128, "".to_string()));
    suite.query_all_balances(&alice.to_string(), |balances| {
        for coin in balances.unwrap().iter() {
            if coin.denom.contains("p.1") {
                *lp_shares_alice.borrow_mut() = coin.clone();
                println!("lp_shares_alice: {:?}", lp_shares_alice.borrow());
            }
            if coin.denom.contains("uusdc") {
                *uusdc_alice.borrow_mut() = coin.clone();
                println!("uusdc_alice: {:?}", uusdc_alice.borrow());
            }
            if coin.denom.contains("uusdt") {
                *uusdt_alice.borrow_mut() = coin.clone();
                println!("uusdt_alice: {:?}", uusdt_alice.borrow());
            }
        }
    });

    let lp_shares_bob = RefCell::new(Coin::new(0u128, "".to_string()));
    let uusdc_bob = RefCell::new(Coin::new(0u128, "".to_string()));
    let uusdt_bob = RefCell::new(Coin::new(0u128, "".to_string()));
    suite.query_all_balances(&bob.to_string(), |balances| {
        for coin in balances.unwrap().iter() {
            if coin.denom.contains("p.1") {
                *lp_shares_bob.borrow_mut() = coin.clone();
                println!("lp_shares_bob: {:?}", lp_shares_bob.borrow());
            }
            if coin.denom.contains("uusdc") {
                *uusdc_bob.borrow_mut() = coin.clone();
                println!("uusdc_bob: {:?}", uusdc_bob.borrow());
            }
            if coin.denom.contains("uusdt") {
                *uusdt_bob.borrow_mut() = coin.clone();
                println!("uusdt_bob: {:?}", uusdt_bob.borrow());
            }
        }
    });

    suite
        .withdraw_liquidity(
            &bob,
            "p.1".to_string(),
            vec![lp_shares_bob.borrow().clone()],
            |result| {
                println!("result: {:?}", result);
                result.unwrap();
            },
        )
        .query_all_balances(&bob.to_string(), |balances| {
            for coin in balances.unwrap().iter() {
                if coin.denom.contains("uusdc") {
                    assert_eq!(
                        coin.amount,
                        uusdc_bob.borrow().clone().amount + Uint128::new(10u128.pow(3))
                    );
                }
                if coin.denom.contains("uusdt") {
                    assert_eq!(
                        coin.amount,
                        uusdt_bob.borrow().clone().amount + Uint128::new(10u128.pow(15))
                    );
                }
            }
        });

    suite
        .withdraw_liquidity(
            &alice,
            "p.1".to_string(),
            vec![lp_shares_alice.borrow().clone()],
            |result| {
                println!("result: {:?}", result);
                result.unwrap();
            },
        )
        .query_all_balances(&alice.to_string(), |balances| {
            for coin in balances.unwrap().iter() {
                if coin.denom.contains("uusdc") {
                    assert_approx_eq!(
                        coin.amount,
                        uusdc_alice.borrow().clone().amount + Uint128::new(10u128.pow(3)),
                        "50"
                    );
                }
                if coin.denom.contains("uusdt") {
                    assert_approx_eq!(
                        coin.amount,
                        uusdt_alice.borrow().clone().amount + Uint128::new(10u128.pow(15)),
                        "50"
                    );
                }
            }
        });
}

#[test]
fn lp_mint_stableswap_low_decimals_scaling_min_liquidity() {
    let mut suite = TestingSuite::default_with_balances(
        vec![
            coin(1_000_00000000000000u128, "uusdc".to_string()),
            coin(1_000_00000000000000u128, "uusdt".to_string()),
            coin(1_000u128, "uusd".to_string()),
            coin(10_000u128, "uom".to_string()),
            coin(10_000u128, "utest".to_string()),
        ],
        StargateMock::new(vec![coin(1000u128, "utest".to_string())]),
    );
    let alice = suite.creator();
    let bob = suite.senders[1].clone();
    let asset_denoms = vec!["uusdc".to_string(), "uusdt".to_string()];

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
        vec![3u8, 6u8],
        pool_fees,
        PoolType::StableSwap { amp: 85 },
        None,
        vec![coin(1000, "uusd"), coin(1000, "utest")],
        |result| {
            result.unwrap();
        },
    );

    // Let's try to add liquidity
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
                coin(1_000u128, "uusdc".to_string()),
                coin(1_000_000, "uusdt".to_string()),
            ],
            |result| {
                println!("result: {:?}", result);
                result.unwrap();
                println!("---");
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
                coin(1_000u128, "uusdc".to_string()),
                coin(1_000_000, "uusdt".to_string()),
            ],
            |result| {
                println!("result: {:?}", result);
                result.unwrap();
            },
        );

    let lp_shares_alice = RefCell::new(Coin::new(0u128, "".to_string()));
    let uusdc_alice = RefCell::new(Coin::new(0u128, "".to_string()));
    let uusdt_alice = RefCell::new(Coin::new(0u128, "".to_string()));
    suite.query_all_balances(&alice.to_string(), |balances| {
        for coin in balances.unwrap().iter() {
            if coin.denom.contains("p.1") {
                *lp_shares_alice.borrow_mut() = coin.clone();
                println!("lp_shares_alice: {:?}", lp_shares_alice.borrow());
            }
            if coin.denom.contains("uusdc") {
                *uusdc_alice.borrow_mut() = coin.clone();
                println!("uusdc_alice: {:?}", uusdc_alice.borrow());
            }
            if coin.denom.contains("uusdt") {
                *uusdt_alice.borrow_mut() = coin.clone();
                println!("uusdt_alice: {:?}", uusdt_alice.borrow());
            }
        }
    });

    let lp_shares_bob = RefCell::new(Coin::new(0u128, "".to_string()));
    let uusdc_bob = RefCell::new(Coin::new(0u128, "".to_string()));
    let uusdt_bob = RefCell::new(Coin::new(0u128, "".to_string()));
    suite.query_all_balances(&bob.to_string(), |balances| {
        for coin in balances.unwrap().iter() {
            if coin.denom.contains("p.1") {
                *lp_shares_bob.borrow_mut() = coin.clone();
                println!("lp_shares_bob: {:?}", lp_shares_bob.borrow());
            }
            if coin.denom.contains("uusdc") {
                *uusdc_bob.borrow_mut() = coin.clone();
                println!("uusdc_bob: {:?}", uusdc_bob.borrow());
            }
            if coin.denom.contains("uusdt") {
                *uusdt_bob.borrow_mut() = coin.clone();
                println!("uusdt_bob: {:?}", uusdt_bob.borrow());
            }
        }
    });

    suite
        .withdraw_liquidity(
            &bob,
            "p.1".to_string(),
            vec![lp_shares_bob.borrow().clone()],
            |result| {
                println!("result: {:?}", result);
                result.unwrap();
            },
        )
        .query_all_balances(&bob.to_string(), |balances| {
            for coin in balances.unwrap().iter() {
                if coin.denom.contains("uusdc") {
                    assert_eq!(
                        coin.amount,
                        uusdc_bob.borrow().clone().amount + Uint128::new(1000u128)
                    );
                }
                if coin.denom.contains("uusdt") {
                    assert_eq!(
                        coin.amount,
                        uusdt_bob.borrow().clone().amount + Uint128::new(1000u128 * 10u128.pow(3))
                    );
                }
            }
        });

    suite
        .withdraw_liquidity(
            &alice,
            "p.1".to_string(),
            vec![lp_shares_alice.borrow().clone()],
            |result| {
                println!("result: {:?}", result);
                result.unwrap();
            },
        )
        .query_all_balances(&alice.to_string(), |balances| {
            for coin in balances.unwrap().iter() {
                if coin.denom.contains("uusdc") {
                    assert_approx_eq!(
                        coin.amount,
                        uusdc_alice.borrow().clone().amount + Uint128::new(1000u128),
                        "50"
                    );
                }
                if coin.denom.contains("uusdt") {
                    assert_approx_eq!(
                        coin.amount,
                        uusdt_alice.borrow().clone().amount
                            + Uint128::new(1000u128 * 10u128.pow(3)),
                        "50"
                    );
                }
            }
        });
}

#[allow(clippy::inconsistent_digit_grouping)]
#[test]
fn provide_and_remove_liquidity_18_decimals() {
    let mut suite = TestingSuite::default_with_balances(
        vec![
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

    let lp_shares = RefCell::new(Coin::new(0u128, "".to_string()));
    suite.query_all_balances(&alice.to_string(), |balances| {
        for coin in balances.unwrap().iter() {
            if coin.denom.contains("p.1") {
                *lp_shares.borrow_mut() = coin.clone();
            }
        }
    });

    suite
        .query_balance(&alice.to_string(), "pusdc".to_string(), |result| {
            assert_eq!(result.unwrap().amount, Uint128::zero());
        })
        .query_balance(&alice.to_string(), "usdy".to_string(), |result| {
            assert_eq!(result.unwrap().amount, Uint128::zero());
        })
        .withdraw_liquidity(
            &alice,
            "p.1".to_string(),
            vec![lp_shares.borrow().clone()],
            |result| {
                result.unwrap();
            },
        )
        .query_balance(&alice.to_string(), "pusdc".to_string(), |result| {
            assert_approx_eq!(
                result.unwrap().amount,
                Uint128::new(300_000_000_000_000_000000000000000000u128),
                "0.000000000000000001"
            );
        })
        .query_balance(&alice.to_string(), "ausdy".to_string(), |result| {
            assert_approx_eq!(
                result.unwrap().amount,
                Uint128::new(300_000_000_000_000_000000000000000000u128),
                "0.000000000000000001"
            );
        });
}
