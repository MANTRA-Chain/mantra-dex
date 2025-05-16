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

// Denoms
const UWHALE_DENOM: &str = "uwhale";
const ULUNA_DENOM: &str = "uluna";
const UUSD_DENOM: &str = "uusd";
const UOM_DENOM: &str = "uom";

// Amounts
const INITIAL_LARGE_BALANCE: u128 = 10_000_000u128;
const INITIAL_SMALL_BALANCE: u128 = 10_000u128;
const STARGATE_MOCK_UOM_AMOUNT: u128 = 8888u128;
const POOL_CREATION_FEE_UUSD_AMOUNT: u128 = 1000u128;
// const POOL_CREATION_FEE_UOM_AMOUNT: u128 = 8888u128; // This is same as STARGATE_MOCK_UOM_AMOUNT

const LIQUIDITY_AMOUNT_1M: u128 = 1_000_000u128;
const LIQUIDITY_AMOUNT_2K: u128 = 2_000u128;
const EXPECTED_SHARES_AFTER_1M_LIQUIDITY: u128 = 999_000u128; // 1_000_000 - MINIMUM_LIQUIDITY_AMOUNT (1000)

// Pool Parameters
const DEFAULT_ASSET_DECIMALS: u8 = 6u8;
const WHALE_ULUNA_POOL_LABEL: &str = "whale.uluna";
const ORIGINAL_POOL_IDENTIFIER_WHALE_ULUNA: &str = "o.whale.uluna"; // Used to derive LP token name

// Locking & Position Parameters
const UNLOCK_DURATION_ONE_DAY: u64 = 86_400u64;
const UNLOCK_DURATION_OTHER: u64 = 200_000u64;
const POSITION_IDENTIFIER_1: &str = "p-1";
const POSITION_IDENTIFIER_2: &str = "p-2";
const FARM_IDENTIFIER: &str = "farm_identifier";
const USER_FARM_IDENTIFIER: &str = "u-farm_identifier";

#[test]
fn provide_liquidity_locking_lp_no_lock_position_identifier() {
    let mut suite = TestingSuite::default_with_balances(
        vec![
            coin(INITIAL_LARGE_BALANCE, UWHALE_DENOM.to_string()),
            coin(INITIAL_LARGE_BALANCE, ULUNA_DENOM.to_string()),
            coin(INITIAL_SMALL_BALANCE, UUSD_DENOM.to_string()),
            coin(INITIAL_SMALL_BALANCE, UOM_DENOM.to_string()),
        ],
        StargateMock::new(vec![coin(STARGATE_MOCK_UOM_AMOUNT, UOM_DENOM.to_string())]),
    );
    let creator = suite.creator();
    let _other = suite.senders[1].clone();
    let _unauthorized = suite.senders[2].clone();

    // Asset denoms with uwhale and uluna
    let asset_denoms = vec![UWHALE_DENOM.to_string(), ULUNA_DENOM.to_string()];

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
        vec![DEFAULT_ASSET_DECIMALS, DEFAULT_ASSET_DECIMALS],
        pool_fees,
        PoolType::ConstantProduct,
        Some(WHALE_ULUNA_POOL_LABEL.to_string()),
        vec![coin(POOL_CREATION_FEE_UUSD_AMOUNT, UUSD_DENOM), coin(STARGATE_MOCK_UOM_AMOUNT, UOM_DENOM)],
        |result| {
            result.unwrap();
        },
    );

    let contract_addr = suite.pool_manager_addr.clone();
    let farm_manager_addr = suite.farm_manager_addr.clone();
    let lp_denom = suite.get_lp_denom(ORIGINAL_POOL_IDENTIFIER_WHALE_ULUNA.to_string());

    // Let's try to add liquidity
    suite
        .provide_liquidity(
            &creator,
            ORIGINAL_POOL_IDENTIFIER_WHALE_ULUNA.to_string(),
            Some(UNLOCK_DURATION_ONE_DAY),
            None,
            None,
            None,
            None,
            vec![
                Coin {
                    denom: UWHALE_DENOM.to_string(),
                    amount: Uint128::from(LIQUIDITY_AMOUNT_1M),
                },
                Coin {
                    denom: ULUNA_DENOM.to_string(),
                    amount: Uint128::from(LIQUIDITY_AMOUNT_1M),
                },
            ],
            |result| {
                // Ensure we got 999_000 in the response which is 1_000_000 less the initial liquidity amount
                assert!(result.unwrap().events.iter().any(|event| {
                    event.attributes.iter().any(|attr| {
                        attr.key == "added_shares"
                            && attr.value
                                == (Uint128::from(LIQUIDITY_AMOUNT_1M) - MINIMUM_LIQUIDITY_AMOUNT)
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
                coin.denom == lp_denom && coin.amount == Uint128::from(EXPECTED_SHARES_AFTER_1M_LIQUIDITY)
            }));
        });

    suite.query_farm_positions(Some(PositionsBy::Receiver(creator.to_string())), None, None, None, |result| {
        let positions = result.unwrap().positions;
        assert_eq!(positions.len(), 1);
        assert_eq!(positions[0], Position {
            identifier: POSITION_IDENTIFIER_1.to_string(),
            lp_asset: Coin { denom: lp_denom.to_string(), amount: Uint128::from(EXPECTED_SHARES_AFTER_1M_LIQUIDITY) },
            unlocking_duration: UNLOCK_DURATION_ONE_DAY,
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
            ORIGINAL_POOL_IDENTIFIER_WHALE_ULUNA.to_string(),
            Some(UNLOCK_DURATION_OTHER),
            None,
            None,
            None,
            None,
            vec![
                Coin {
                    denom: UWHALE_DENOM.to_string(),
                    amount: Uint128::from(LIQUIDITY_AMOUNT_2K),
                },
                Coin {
                    denom: ULUNA_DENOM.to_string(),
                    amount: Uint128::from(LIQUIDITY_AMOUNT_2K),
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
                            .checked_add(Uint128::from(LIQUIDITY_AMOUNT_2K))
                            .unwrap()
            }));

            let lp_balance = balances.iter().find(|coin| coin.denom == lp_denom).unwrap();
            *farm_manager_lp_amount.borrow_mut() = lp_balance.amount;
        });

    suite.query_farm_positions(Some(PositionsBy::Receiver(creator.to_string())), None, None, None, |result| {
        let positions = result.unwrap().positions;
        assert_eq!(positions.len(), 2);
        assert_eq!(positions[0], Position {
            identifier: POSITION_IDENTIFIER_1.to_string(),
            lp_asset: Coin { denom: lp_denom.to_string(), amount: Uint128::from(EXPECTED_SHARES_AFTER_1M_LIQUIDITY) },
            unlocking_duration: UNLOCK_DURATION_ONE_DAY,
            open: true,
            expiring_at: None,
            receiver: creator.clone(),
        });
        assert_eq!(positions[1], Position {
            identifier: POSITION_IDENTIFIER_2.to_string(),
            lp_asset: Coin { denom: lp_denom.to_string(), amount: Uint128::from(LIQUIDITY_AMOUNT_2K) },
            unlocking_duration: UNLOCK_DURATION_OTHER,
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
            coin(INITIAL_LARGE_BALANCE, UWHALE_DENOM.to_string()),
            coin(INITIAL_LARGE_BALANCE, ULUNA_DENOM.to_string()),
            coin(INITIAL_SMALL_BALANCE, UUSD_DENOM.to_string()),
            coin(INITIAL_SMALL_BALANCE, UOM_DENOM.to_string()),
        ],
        StargateMock::new(vec![coin(STARGATE_MOCK_UOM_AMOUNT, UOM_DENOM.to_string())]),
    );
    let creator = suite.creator();
    let _other = suite.senders[1].clone();
    let _unauthorized = suite.senders[2].clone();

    // Asset denoms with uwhale and uluna
    let asset_denoms = vec![UWHALE_DENOM.to_string(), ULUNA_DENOM.to_string()];

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
        vec![DEFAULT_ASSET_DECIMALS, DEFAULT_ASSET_DECIMALS],
        pool_fees,
        PoolType::ConstantProduct,
        Some(WHALE_ULUNA_POOL_LABEL.to_string()),
        vec![coin(POOL_CREATION_FEE_UUSD_AMOUNT, UUSD_DENOM), coin(STARGATE_MOCK_UOM_AMOUNT, UOM_DENOM)],
        |result| {
            result.unwrap();
        },
    );

    let contract_addr = suite.pool_manager_addr.clone();
    let farm_manager_addr = suite.farm_manager_addr.clone();
    let lp_denom = suite.get_lp_denom(ORIGINAL_POOL_IDENTIFIER_WHALE_ULUNA.to_string());

    // Let's try to add liquidity
    suite
        .provide_liquidity(
            &creator,
            ORIGINAL_POOL_IDENTIFIER_WHALE_ULUNA.to_string(),
            Some(UNLOCK_DURATION_ONE_DAY),
            Some(FARM_IDENTIFIER.to_string()),
            None,
            None,
            None,
            vec![
                Coin {
                    denom: UWHALE_DENOM.to_string(),
                    amount: Uint128::from(LIQUIDITY_AMOUNT_1M),
                },
                Coin {
                    denom: ULUNA_DENOM.to_string(),
                    amount: Uint128::from(LIQUIDITY_AMOUNT_1M),
                },
            ],
            |result| {
                // Ensure we got 999_000 in the response which is 1_000_000 less the initial liquidity amount
                assert!(result.unwrap().events.iter().any(|event| {
                    event.attributes.iter().any(|attr| {
                        attr.key == "added_shares"
                            && attr.value
                                == (Uint128::from(LIQUIDITY_AMOUNT_1M) - MINIMUM_LIQUIDITY_AMOUNT)
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
                coin.denom == lp_denom && coin.amount == Uint128::from(EXPECTED_SHARES_AFTER_1M_LIQUIDITY)
            }));
        });

    suite.query_farm_positions(Some(PositionsBy::Receiver(creator.to_string())), None, None, None, |result| {
        let positions = result.unwrap().positions;
        assert_eq!(positions.len(), 1);
        assert_eq!(positions[0], Position {
            identifier: USER_FARM_IDENTIFIER.to_string(),
            lp_asset: Coin { denom: lp_denom.to_string(), amount: Uint128::from(EXPECTED_SHARES_AFTER_1M_LIQUIDITY) },
            unlocking_duration: UNLOCK_DURATION_ONE_DAY,
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
            ORIGINAL_POOL_IDENTIFIER_WHALE_ULUNA.to_string(),
            Some(UNLOCK_DURATION_OTHER),
            Some(USER_FARM_IDENTIFIER.to_string()),
            None,
            None,
            None,
            vec![
                Coin {
                    denom: UWHALE_DENOM.to_string(),
                    amount: Uint128::from(LIQUIDITY_AMOUNT_2K),
                },
                Coin {
                    denom: ULUNA_DENOM.to_string(),
                    amount: Uint128::from(LIQUIDITY_AMOUNT_2K),
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
                            .checked_add(Uint128::from(LIQUIDITY_AMOUNT_2K))
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
            identifier: USER_FARM_IDENTIFIER.to_string(),
            lp_asset: Coin { denom: lp_denom.to_string(), amount: *farm_manager_lp_amount.borrow() },
            unlocking_duration: UNLOCK_DURATION_OTHER,
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
            coin(INITIAL_LARGE_BALANCE, UWHALE_DENOM.to_string()),
            coin(INITIAL_LARGE_BALANCE, ULUNA_DENOM.to_string()),
            coin(INITIAL_SMALL_BALANCE, UUSD_DENOM.to_string()),
            coin(INITIAL_SMALL_BALANCE, UOM_DENOM.to_string()),
        ],
        StargateMock::new(vec![coin(STARGATE_MOCK_UOM_AMOUNT, UOM_DENOM.to_string())]),
    );
    let creator = suite.creator();
    let _other = suite.senders[1].clone();
    let _unauthorized = suite.senders[2].clone();

    // Asset denoms with uwhale and uluna
    let asset_denoms = vec![UWHALE_DENOM.to_string(), ULUNA_DENOM.to_string()];

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
        vec![DEFAULT_ASSET_DECIMALS, DEFAULT_ASSET_DECIMALS],
        pool_fees,
        PoolType::ConstantProduct,
        Some(WHALE_ULUNA_POOL_LABEL.to_string()),
        vec![coin(POOL_CREATION_FEE_UUSD_AMOUNT, UUSD_DENOM), coin(STARGATE_MOCK_UOM_AMOUNT, UOM_DENOM)],
        |result| {
            result.unwrap();
        },
    );

    let contract_addr = suite.pool_manager_addr.clone();
    let farm_manager_addr = suite.farm_manager_addr.clone();
    let lp_denom = suite.get_lp_denom(ORIGINAL_POOL_IDENTIFIER_WHALE_ULUNA.to_string());

    // Let's try to add liquidity
    suite
        .provide_liquidity(
            &creator,
            ORIGINAL_POOL_IDENTIFIER_WHALE_ULUNA.to_string(),
            Some(UNLOCK_DURATION_ONE_DAY),
            Some(FARM_IDENTIFIER.to_string()),
            None,
            None,
            None,
            vec![
                Coin {
                    denom: UWHALE_DENOM.to_string(),
                    amount: Uint128::from(LIQUIDITY_AMOUNT_1M),
                },
                Coin {
                    denom: ULUNA_DENOM.to_string(),
                    amount: Uint128::from(LIQUIDITY_AMOUNT_1M),
                },
            ],
            |result| {
                // Ensure we got 999_000 in the response which is 1_000_000 less the initial liquidity amount
                assert!(result.unwrap().events.iter().any(|event| {
                    event.attributes.iter().any(|attr| {
                        attr.key == "added_shares"
                            && attr.value
                                == (Uint128::from(LIQUIDITY_AMOUNT_1M) - MINIMUM_LIQUIDITY_AMOUNT)
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
                coin.denom == lp_denom && coin.amount == Uint128::from(EXPECTED_SHARES_AFTER_1M_LIQUIDITY)
            }));
        });

    suite.query_farm_positions(Some(PositionsBy::Receiver(creator.to_string())), None, None, None, |result| {
        let positions = result.unwrap().positions;
        assert_eq!(positions.len(), 1);
        assert_eq!(positions[0], Position {
            identifier: USER_FARM_IDENTIFIER.to_string(),
            lp_asset: Coin { denom: lp_denom.to_string(), amount: Uint128::from(EXPECTED_SHARES_AFTER_1M_LIQUIDITY) },
            unlocking_duration: UNLOCK_DURATION_ONE_DAY,
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
            ORIGINAL_POOL_IDENTIFIER_WHALE_ULUNA.to_string(),
            Some(UNLOCK_DURATION_OTHER),
            None,
            None,
            None,
            None,
            vec![
                Coin {
                    denom: UWHALE_DENOM.to_string(),
                    amount: Uint128::from(LIQUIDITY_AMOUNT_2K),
                },
                Coin {
                    denom: ULUNA_DENOM.to_string(),
                    amount: Uint128::from(LIQUIDITY_AMOUNT_2K),
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
                            .checked_add(Uint128::from(LIQUIDITY_AMOUNT_2K))
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
            identifier: POSITION_IDENTIFIER_1.to_string(),
            lp_asset: Coin { denom: lp_denom.to_string(), amount: Uint128::from(LIQUIDITY_AMOUNT_2K) },
            unlocking_duration: UNLOCK_DURATION_OTHER,
            open: true,
            expiring_at: None,
            receiver: creator.clone(),
        });
        assert_eq!(positions[1], Position {
            identifier: USER_FARM_IDENTIFIER.to_string(),
            lp_asset: Coin { denom: lp_denom.to_string(), amount: Uint128::from(EXPECTED_SHARES_AFTER_1M_LIQUIDITY) },
            unlocking_duration: UNLOCK_DURATION_ONE_DAY,
            open: true,
            expiring_at: None,
            receiver: creator.clone(),
        });
    });
}
