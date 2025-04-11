use std::cell::RefCell;

use cosmwasm_std::{coin, Coin, Decimal, Uint128};
use mantra_common_testing::multi_test::stargate_mock::StargateMock;
use mantra_dex_std::{
    farm_manager::{Position, PositionsBy},
    fee::{Fee, PoolFee},
    lp_common::MINIMUM_LIQUIDITY_AMOUNT,
    pool_manager::PoolType,
};

use crate::tests::suite::TestingSuite;

#[test]
fn provide_liquidity_locking_lp_no_lock_position_identifier() {
    let mut suite = TestingSuite::default_with_balances(
        vec![
            coin(10_000_000u128, "uwhale".to_string()),
            coin(10_000_000u128, "uluna".to_string()),
            coin(10_000u128, "uusd".to_string()),
            coin(10_000u128, "uom".to_string()),
        ],
        StargateMock::new(vec![coin(8888u128, "uom".to_string())]),
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
        vec![coin(1000, "uusd"), coin(8888, "uom")],
        |result| {
            result.unwrap();
        },
    );

    let contract_addr = suite.pool_manager_addr.clone();
    let farm_manager_addr = suite.farm_manager_addr.clone();
    let lp_denom = suite.get_lp_denom("o.whale.uluna".to_string());

    // Let's try to add liquidity
    suite
        .provide_liquidity(
            &creator,
            "o.whale.uluna".to_string(),
            Some(86_400u64),
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
                // Ensure we got 999_000 in the response which is 1_000_000 less the initial liquidity amount
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
        .query_all_balances(&creator.to_string(), |result| {
            let balances = result.unwrap();
            // the lp tokens should have gone to the farm manager
            assert!(!balances
                .iter()
                .any(|coin| { coin.denom == lp_denom.clone() }));
        })
        // contract should have 1_000 LP shares (MINIMUM_LIQUIDITY_AMOUNT)
        .query_all_balances(&contract_addr.to_string(), |result| {
            let balances = result.unwrap();
            assert!(balances.iter().any(|coin| {
                coin.denom == lp_denom.clone() && coin.amount == MINIMUM_LIQUIDITY_AMOUNT
            }));
        })
        // check the LP went to the farm manager
        .query_all_balances(&farm_manager_addr.to_string(), |result| {
            let balances = result.unwrap();
            assert!(balances.iter().any(|coin| {
                coin.denom == lp_denom && coin.amount == Uint128::from(999_000u128)
            }));
        });

    suite.query_farm_positions(Some(PositionsBy::Receiver(creator.to_string())), None, None, None, |result| {
        let positions = result.unwrap().positions;
        assert_eq!(positions.len(), 1);
        assert_eq!(positions[0], Position {
            identifier: "p-1".to_string(),
            lp_asset: Coin { denom: "factory/mantra1zwv6feuzhy6a9wekh96cd57lsarmqlwxdypdsplw6zhfncqw6ftqlydlr9/o.whale.uluna.LP".to_string(), amount: Uint128::from(999_000u128) },
            unlocking_duration: 86_400,
            open: true,
            expiring_at: None,
            receiver: creator.clone(),
        });
    });

    // let's do it again, it should create another position on the farm manager

    let farm_manager_lp_amount = RefCell::new(Uint128::zero());

    suite
        .query_all_balances(&farm_manager_addr.to_string(), |result| {
            let balances = result.unwrap();
            let lp_balance = balances.iter().find(|coin| coin.denom == lp_denom).unwrap();
            *farm_manager_lp_amount.borrow_mut() = lp_balance.amount;
        })
        .provide_liquidity(
            &creator,
            "o.whale.uluna".to_string(),
            Some(200_000u64),
            None,
            None,
            None,
            None,
            vec![
                Coin {
                    denom: "uwhale".to_string(),
                    amount: Uint128::from(2_000u128),
                },
                Coin {
                    denom: "uluna".to_string(),
                    amount: Uint128::from(2_000u128),
                },
            ],
            |result| {
                result.unwrap();
            },
        )
        .query_all_balances(&creator.to_string(), |result| {
            let balances = result.unwrap();
            // the lp tokens should have gone to the farm manager
            assert!(!balances
                .iter()
                .any(|coin| { coin.denom == lp_denom.clone() }));
        })
        // check the LP went to the farm manager
        .query_all_balances(&farm_manager_addr.to_string(), |result| {
            let balances = result.unwrap();
            // the LP tokens should have gone to the farm manager
            // the new minted LP tokens should be 2_000
            assert!(balances.iter().any(|coin| {
                coin.denom == lp_denom
                    && coin.amount
                        == farm_manager_lp_amount
                            .borrow()
                            .checked_add(Uint128::from(2_000u128))
                            .unwrap()
            }));

            let lp_balance = balances.iter().find(|coin| coin.denom == lp_denom).unwrap();
            *farm_manager_lp_amount.borrow_mut() = lp_balance.amount;
        });

    suite.query_farm_positions(Some(PositionsBy::Receiver(creator.to_string())), None, None, None, |result| {
        let positions = result.unwrap().positions;
        assert_eq!(positions.len(), 2);
        assert_eq!(positions[0], Position {
            identifier: "p-1".to_string(),
            lp_asset: Coin { denom: "factory/mantra1zwv6feuzhy6a9wekh96cd57lsarmqlwxdypdsplw6zhfncqw6ftqlydlr9/o.whale.uluna.LP".to_string(), amount: Uint128::from(999_000u128) },
            unlocking_duration: 86_400,
            open: true,
            expiring_at: None,
            receiver: creator.clone(),
        });
        assert_eq!(positions[1], Position {
            identifier: "p-2".to_string(),
            lp_asset: Coin { denom: "factory/mantra1zwv6feuzhy6a9wekh96cd57lsarmqlwxdypdsplw6zhfncqw6ftqlydlr9/o.whale.uluna.LP".to_string(), amount: Uint128::from(2_000u128) },
            unlocking_duration: 200_000,
            open: true,
            expiring_at: None,
            receiver: creator.clone(),
        });
    });
}

#[test]
fn provide_liquidity_locking_lp_reusing_position_identifier() {
    let mut suite = TestingSuite::default_with_balances(
        vec![
            coin(10_000_000u128, "uwhale".to_string()),
            coin(10_000_000u128, "uluna".to_string()),
            coin(10_000u128, "uusd".to_string()),
            coin(10_000u128, "uom".to_string()),
        ],
        StargateMock::new(vec![coin(8888u128, "uom".to_string())]),
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
        vec![coin(1000, "uusd"), coin(8888, "uom")],
        |result| {
            result.unwrap();
        },
    );

    let contract_addr = suite.pool_manager_addr.clone();
    let farm_manager_addr = suite.farm_manager_addr.clone();
    let lp_denom = suite.get_lp_denom("o.whale.uluna".to_string());

    // Let's try to add liquidity
    suite
        .provide_liquidity(
            &creator,
            "o.whale.uluna".to_string(),
            Some(86_400u64),
            Some("farm_identifier".to_string()),
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
                // Ensure we got 999_000 in the response which is 1_000_000 less the initial liquidity amount
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
        .query_all_balances(&creator.to_string(), |result| {
            let balances = result.unwrap();
            // the lp tokens should have gone to the farm manager
            assert!(!balances
                .iter()
                .any(|coin| { coin.denom == lp_denom.clone() }));
        })
        // contract should have 1_000 LP shares (MINIMUM_LIQUIDITY_AMOUNT)
        .query_all_balances(&contract_addr.to_string(), |result| {
            let balances = result.unwrap();
            assert!(balances.iter().any(|coin| {
                coin.denom == lp_denom.clone() && coin.amount == MINIMUM_LIQUIDITY_AMOUNT
            }));
        })
        // check the LP went to the farm manager
        .query_all_balances(&farm_manager_addr.to_string(), |result| {
            let balances = result.unwrap();
            assert!(balances.iter().any(|coin| {
                coin.denom == lp_denom && coin.amount == Uint128::from(999_000u128)
            }));
        });

    suite.query_farm_positions(Some(PositionsBy::Receiver(creator.to_string())), None, None, None, |result| {
        let positions = result.unwrap().positions;
        assert_eq!(positions.len(), 1);
        assert_eq!(positions[0], Position {
            identifier: "u-farm_identifier".to_string(),
            lp_asset: Coin { denom: "factory/mantra1zwv6feuzhy6a9wekh96cd57lsarmqlwxdypdsplw6zhfncqw6ftqlydlr9/o.whale.uluna.LP".to_string(), amount: Uint128::from(999_000u128) },
            unlocking_duration: 86_400,
            open: true,
            expiring_at: None,
            receiver: creator.clone(),
        });
    });

    // let's do it again, reusing the same farm identifier

    let farm_manager_lp_amount = RefCell::new(Uint128::zero());

    suite
        .query_all_balances(&farm_manager_addr.to_string(), |result| {
            let balances = result.unwrap();
            let lp_balance = balances.iter().find(|coin| coin.denom == lp_denom).unwrap();
            *farm_manager_lp_amount.borrow_mut() = lp_balance.amount;
        })
        .provide_liquidity(
            &creator,
            "o.whale.uluna".to_string(),
            Some(200_000u64),
            Some("u-farm_identifier".to_string()),
            None,
            None,
            None,
            vec![
                Coin {
                    denom: "uwhale".to_string(),
                    amount: Uint128::from(2_000u128),
                },
                Coin {
                    denom: "uluna".to_string(),
                    amount: Uint128::from(2_000u128),
                },
            ],
            |result| {
                result.unwrap();
            },
        )
        .query_all_balances(&creator.to_string(), |result| {
            let balances = result.unwrap();
            // the lp tokens should have gone to the farm manager
            assert!(!balances
                .iter()
                .any(|coin| { coin.denom == lp_denom.clone() }));
        })
        // check the LP went to the farm manager
        .query_all_balances(&farm_manager_addr.to_string(), |result| {
            let balances = result.unwrap();
            // the LP tokens should have gone to the farm manager
            // the new minted LP tokens should be 2_000
            assert!(balances.iter().any(|coin| {
                coin.denom == lp_denom
                    && coin.amount
                        == farm_manager_lp_amount
                            .borrow()
                            .checked_add(Uint128::from(2_000u128))
                            .unwrap()
            }));

            let lp_balance = balances.iter().find(|coin| coin.denom == lp_denom).unwrap();
            *farm_manager_lp_amount.borrow_mut() = lp_balance.amount;
        });

    suite.query_farm_positions(Some(PositionsBy::Receiver(creator.to_string())), None, None, None, |result| {
        let positions = result.unwrap().positions;
        // the position should be updated
        assert_eq!(positions.len(), 1);
        assert_eq!(positions[0], Position {
            identifier: "u-farm_identifier".to_string(),
            lp_asset: Coin { denom: "factory/mantra1zwv6feuzhy6a9wekh96cd57lsarmqlwxdypdsplw6zhfncqw6ftqlydlr9/o.whale.uluna.LP".to_string(), amount: *farm_manager_lp_amount.borrow() },
            unlocking_duration: 86_400,
            open: true,
            expiring_at: None,
            receiver: creator.clone(),
        });
    });
}

#[test]
fn provide_liquidity_locking_lp_reusing_position_identifier_2() {
    let mut suite = TestingSuite::default_with_balances(
        vec![
            coin(10_000_000u128, "uwhale".to_string()),
            coin(10_000_000u128, "uluna".to_string()),
            coin(10_000u128, "uusd".to_string()),
            coin(10_000u128, "uom".to_string()),
        ],
        StargateMock::new(vec![coin(8888u128, "uom".to_string())]),
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
        vec![coin(1000, "uusd"), coin(8888, "uom")],
        |result| {
            result.unwrap();
        },
    );

    let contract_addr = suite.pool_manager_addr.clone();
    let farm_manager_addr = suite.farm_manager_addr.clone();
    let lp_denom = suite.get_lp_denom("o.whale.uluna".to_string());

    // Let's try to add liquidity
    suite
        .provide_liquidity(
            &creator,
            "o.whale.uluna".to_string(),
            Some(86_400u64),
            Some("farm_identifier".to_string()),
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
                // Ensure we got 999_000 in the response which is 1_000_000 less the initial liquidity amount
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
        .query_all_balances(&creator.to_string(), |result| {
            let balances = result.unwrap();
            // the lp tokens should have gone to the farm manager
            assert!(!balances
                .iter()
                .any(|coin| { coin.denom == lp_denom.clone() }));
        })
        // contract should have 1_000 LP shares (MINIMUM_LIQUIDITY_AMOUNT)
        .query_all_balances(&contract_addr.to_string(), |result| {
            let balances = result.unwrap();
            assert!(balances.iter().any(|coin| {
                coin.denom == lp_denom.clone() && coin.amount == MINIMUM_LIQUIDITY_AMOUNT
            }));
        })
        // check the LP went to the farm manager
        .query_all_balances(&farm_manager_addr.to_string(), |result| {
            let balances = result.unwrap();
            assert!(balances.iter().any(|coin| {
                coin.denom == lp_denom && coin.amount == Uint128::from(999_000u128)
            }));
        });

    suite.query_farm_positions(Some(PositionsBy::Receiver(creator.to_string())), None, None, None, |result| {
        let positions = result.unwrap().positions;
        assert_eq!(positions.len(), 1);
        assert_eq!(positions[0], Position {
            identifier: "u-farm_identifier".to_string(),
            lp_asset: Coin { denom: "factory/mantra1zwv6feuzhy6a9wekh96cd57lsarmqlwxdypdsplw6zhfncqw6ftqlydlr9/o.whale.uluna.LP".to_string(), amount: Uint128::from(999_000u128) },
            unlocking_duration: 86_400,
            open: true,
            expiring_at: None,
            receiver: creator.clone(),
        });
    });

    // let's do it again, this time no identifier is used

    let farm_manager_lp_amount = RefCell::new(Uint128::zero());

    suite
        .query_all_balances(&farm_manager_addr.to_string(), |result| {
            let balances = result.unwrap();
            let lp_balance = balances.iter().find(|coin| coin.denom == lp_denom).unwrap();
            *farm_manager_lp_amount.borrow_mut() = lp_balance.amount;
        })
        .provide_liquidity(
            &creator,
            "o.whale.uluna".to_string(),
            Some(200_000u64),
            None,
            None,
            None,
            None,
            vec![
                Coin {
                    denom: "uwhale".to_string(),
                    amount: Uint128::from(2_000u128),
                },
                Coin {
                    denom: "uluna".to_string(),
                    amount: Uint128::from(2_000u128),
                },
            ],
            |result| {
                result.unwrap();
            },
        )
        .query_all_balances(&creator.to_string(), |result| {
            let balances = result.unwrap();
            // the lp tokens should have gone to the farm manager
            assert!(!balances
                .iter()
                .any(|coin| { coin.denom == lp_denom.clone() }));
        })
        // check the LP went to the farm manager
        .query_all_balances(&farm_manager_addr.to_string(), |result| {
            let balances = result.unwrap();
            // the LP tokens should have gone to the farm manager
            // the new minted LP tokens should be 2_000
            assert!(balances.iter().any(|coin| {
                coin.denom == lp_denom
                    && coin.amount
                        == farm_manager_lp_amount
                            .borrow()
                            .checked_add(Uint128::from(2_000u128))
                            .unwrap()
            }));

            let lp_balance = balances.iter().find(|coin| coin.denom == lp_denom).unwrap();
            *farm_manager_lp_amount.borrow_mut() = lp_balance.amount;
        });

    suite.query_farm_positions(Some(PositionsBy::Receiver(creator.to_string())), None, None, None, |result| {
        let positions = result.unwrap().positions;
        // the position should be updated
        assert_eq!(positions.len(), 2);
        assert_eq!(positions[0], Position {
            identifier: "p-1".to_string(),
            lp_asset: Coin { denom: "factory/mantra1zwv6feuzhy6a9wekh96cd57lsarmqlwxdypdsplw6zhfncqw6ftqlydlr9/o.whale.uluna.LP".to_string(), amount: Uint128::new(2_000u128) },
            unlocking_duration: 200_000,
            open: true,
            expiring_at: None,
            receiver: creator.clone(),
        });
        assert_eq!(positions[1], Position {
            identifier: "u-farm_identifier".to_string(),
            lp_asset: Coin { denom: "factory/mantra1zwv6feuzhy6a9wekh96cd57lsarmqlwxdypdsplw6zhfncqw6ftqlydlr9/o.whale.uluna.LP".to_string(), amount: Uint128::new(999_000u128) },
            unlocking_duration: 86_400,
            open: true,
            expiring_at: None,
            receiver: creator.clone(),
        });
    });
}
