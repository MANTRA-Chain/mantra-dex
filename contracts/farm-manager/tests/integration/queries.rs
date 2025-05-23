extern crate core;

use std::cell::RefCell;

use cosmwasm_std::{coin, coins, Coin, Uint128};
use farm_manager::state::{MAX_FARMS_LIMIT, MAX_POSITIONS_LIMIT};
use mantra_dex_std::constants::LP_SYMBOL;
use mantra_dex_std::farm_manager::{
    FarmAction, FarmParams, PositionAction, PositionsBy, RewardsResponse,
};

use crate::common::suite::TestingSuite;
use crate::common::MOCK_CONTRACT_ADDR_1;
use test_utils::common_constants::{
    DEFAULT_UNLOCKING_DURATION_SECONDS, DENOM_UOM as UOM_DENOM, DENOM_UOSMO as UOSMO_DENOM,
    DENOM_UUSDY as UUSDY_DENOM, INITIAL_BALANCE, UOM_FARM_CREATION_FEE,
};

const FARM_START_EPOCH: u64 = 12;
const FARM_END_EPOCH: u64 = 16;
const POSITION_LP_AMOUNT: u128 = 1_000;
const PAGE_LIMIT_5: u32 = 5;
const LP_DENOM_1_INITIAL_BALANCE: u128 = 1_000_000_000_000;
const FARM_UUSDY_ASSET_AMOUNT: u128 = 3333;
const CREATOR_ANOTHER_POSITION_LP_AMOUNT: u128 = 2_000;
const LP_DENOM_2_INITIAL_BALANCE: u128 = 1_000_000_000_000;
const FARM_1_UUSDY_ASSET_AMOUNT: u128 = 8_888;
const FARM_2_UUSDY_ASSET_AMOUNT: u128 = 666_666;
const FARM_2_END_EPOCH: u64 = 20;

#[test]
fn test_rewards_query_overlapping_farms() {
    let lp_denom_1 = format!("factory/{MOCK_CONTRACT_ADDR_1}/1.{LP_SYMBOL}").to_string();
    let lp_denom_2 = format!("factory/{MOCK_CONTRACT_ADDR_1}/2.{LP_SYMBOL}").to_string();

    let mut suite = TestingSuite::default_with_balances(vec![
        coin(INITIAL_BALANCE, UOM_DENOM.to_string()),
        coin(INITIAL_BALANCE, UUSDY_DENOM.to_string()),
        coin(INITIAL_BALANCE, UOSMO_DENOM.to_string()),
        coin(INITIAL_BALANCE, lp_denom_1.clone()),
        coin(INITIAL_BALANCE, lp_denom_2.clone()),
    ]);

    let creator = suite.creator();
    let other = suite.senders[1].clone();

    suite.instantiate_default();

    for _ in 0..10 {
        suite.add_one_epoch();
    }

    // create 4 farms with 2 different LPs
    suite
        .query_current_epoch(|result| {
            let epoch_response = result.unwrap();
            assert_eq!(epoch_response.epoch.id, 10);
        })
        .manage_farm(
            &creator,
            FarmAction::Fill {
                params: FarmParams {
                    lp_denom: lp_denom_1.clone(),
                    start_epoch: Some(FARM_START_EPOCH),
                    preliminary_end_epoch: Some(FARM_END_EPOCH),
                    curve: None,
                    farm_asset: Coin {
                        denom: UUSDY_DENOM.to_string(),
                        amount: Uint128::new(80_000u128),
                    },
                    farm_identifier: Some("farm_1".to_string()),
                },
            },
            vec![
                coin(80_000u128, UUSDY_DENOM.to_string()),
                coin(UOM_FARM_CREATION_FEE, UOM_DENOM.to_string()),
            ],
            |result| {
                result.unwrap();
            },
        )
        .manage_farm(
            &creator,
            FarmAction::Fill {
                params: FarmParams {
                    lp_denom: lp_denom_1.clone(),
                    start_epoch: Some(FARM_START_EPOCH),
                    preliminary_end_epoch: Some(FARM_END_EPOCH),
                    curve: None,
                    farm_asset: Coin {
                        denom: UOSMO_DENOM.to_string(),
                        amount: Uint128::new(10_000u128),
                    },
                    farm_identifier: Some("farm_2".to_string()),
                },
            },
            vec![
                coin(10_000u128, UOSMO_DENOM.to_string()),
                coin(UOM_FARM_CREATION_FEE, UOM_DENOM.to_string()),
            ],
            |result| {
                result.unwrap();
            },
        )
        .manage_farm(
            &other,
            FarmAction::Fill {
                params: FarmParams {
                    lp_denom: lp_denom_2.clone(),
                    start_epoch: Some(FARM_START_EPOCH),
                    preliminary_end_epoch: Some(FARM_END_EPOCH),
                    curve: None,
                    farm_asset: Coin {
                        denom: UOM_DENOM.to_string(),
                        amount: Uint128::new(30_000u128),
                    },
                    farm_identifier: Some("farm_3".to_string()),
                },
            },
            vec![coin(31_000u128, UOM_DENOM.to_string())], // 30_000 + 1_000 fee
            |result| {
                result.unwrap();
            },
        )
        .manage_farm(
            &other,
            FarmAction::Fill {
                params: FarmParams {
                    lp_denom: lp_denom_2.clone(),
                    start_epoch: Some(FARM_START_EPOCH),
                    preliminary_end_epoch: Some(FARM_END_EPOCH),
                    curve: None,
                    farm_asset: Coin {
                        denom: UUSDY_DENOM.to_string(),
                        amount: Uint128::new(70_000u128),
                    },
                    farm_identifier: Some("farm_4".to_string()),
                },
            },
            vec![
                coin(70_000u128, UUSDY_DENOM.to_string()),
                coin(UOM_FARM_CREATION_FEE, UOM_DENOM.to_string()),
            ],
            |result| {
                result.unwrap();
            },
        );

    // creator fills a position
    suite
        .manage_position(
            &creator,
            PositionAction::Create {
                identifier: Some("creator_pos_1".to_string()),
                unlocking_duration: DEFAULT_UNLOCKING_DURATION_SECONDS,
                receiver: None,
            },
            vec![coin(35_000, lp_denom_1.clone())],
            |result| {
                result.unwrap();
            },
        )
        .manage_position(
            &creator,
            PositionAction::Create {
                identifier: Some("creator_pos_2".to_string()),
                unlocking_duration: DEFAULT_UNLOCKING_DURATION_SECONDS,
                receiver: None,
            },
            vec![coin(70_000, lp_denom_2.clone())],
            |result| {
                result.unwrap();
            },
        );

    suite.add_epochs(3).query_current_epoch(|result| {
        let epoch_response = result.unwrap();
        assert_eq!(epoch_response.epoch.id, 13);
    });

    suite.query_rewards(&creator, None, |result| {
        let rewards_response = result.unwrap();

        assert_eq!(
            rewards_response,
            RewardsResponse::RewardsResponse {
                total_rewards: vec![
                    coin(15000, UOM_DENOM.to_string()),
                    coin(5000, UOSMO_DENOM.to_string()),
                    coin(75000, UUSDY_DENOM.to_string()),
                ],
                rewards_per_lp_denom: vec![
                    (
                        lp_denom_1.clone(),
                        vec![
                            coin(5000, UOSMO_DENOM.to_string()),
                            coin(40000, UUSDY_DENOM.to_string())
                        ]
                    ),
                    (
                        lp_denom_2.clone(),
                        vec![
                            coin(15000, UOM_DENOM.to_string()),
                            coin(35000, UUSDY_DENOM.to_string())
                        ]
                    ),
                ],
            }
        );
    });
}

#[test]
fn test_positions_query_filters_and_pagination() {
    let mut balances = vec![
        coin(INITIAL_BALANCE, UOM_DENOM.to_string()),
        coin(INITIAL_BALANCE, UUSDY_DENOM.to_string()),
        coin(INITIAL_BALANCE, UOSMO_DENOM.to_string()),
    ];

    // prepare lp denoms
    for i in 1..MAX_FARMS_LIMIT * 2 {
        let lp_denom = format!("factory/{MOCK_CONTRACT_ADDR_1}/{i}.{LP_SYMBOL}");
        balances.push(coin(INITIAL_BALANCE, lp_denom.clone()));
    }

    let mut suite = TestingSuite::default_with_balances(balances);

    let alice = suite.senders[1].clone();
    suite.instantiate_default();

    // open positions
    for i in 1..=MAX_POSITIONS_LIMIT {
        suite.manage_position(
            &alice,
            PositionAction::Create {
                identifier: Some(format!("position{}", i)),
                unlocking_duration: DEFAULT_UNLOCKING_DURATION_SECONDS,
                receiver: None,
            },
            vec![coin(
                POSITION_LP_AMOUNT,
                format!("factory/{MOCK_CONTRACT_ADDR_1}/{i}.{LP_SYMBOL}"),
            )],
            |result| {
                result.unwrap();
            },
        );
    }

    let position_a_id = RefCell::new("".to_string());
    let position_b_id = RefCell::new("".to_string());

    suite
        .query_positions(
            Some(PositionsBy::Receiver(alice.to_string())),
            Some(true),
            None,
            Some(MAX_POSITIONS_LIMIT),
            |result| {
                let response = result.unwrap();
                assert_eq!(response.positions.len(), MAX_POSITIONS_LIMIT as usize);
            },
        )
        .query_positions(
            Some(PositionsBy::Receiver(alice.to_string())),
            Some(true),
            None,
            Some(PAGE_LIMIT_5),
            |result| {
                let response = result.unwrap();
                assert_eq!(response.positions.len(), PAGE_LIMIT_5 as usize);
                position_a_id.replace(response.positions[4].identifier.clone());
            },
        )
        .query_positions(
            Some(PositionsBy::Receiver(alice.to_string())),
            Some(true),
            Some(position_a_id.borrow().clone()),
            Some(PAGE_LIMIT_5),
            |result| {
                let response = result.unwrap();
                assert_eq!(response.positions.len(), PAGE_LIMIT_5 as usize);
                position_b_id.replace(response.positions[0].identifier.clone());
            },
        );

    // query with filters
    suite.query_positions(
        Some(PositionsBy::Identifier(position_b_id.borrow().clone())),
        None,
        None,
        None,
        |result| {
            let response = result.unwrap();
            assert_eq!(response.positions.len(), 1usize);
            assert_eq!(
                response.positions[0].identifier,
                position_b_id.borrow().clone()
            );
        },
    );
}

// This is to cover for the following edge case:
// Single user in the system opens a position, claims some rewards, and then closes the
// position in full (making the total_lp_weight zero for the subsequent epoch).
// The LAST_CLAIMED_EPOCH is set to the epoch where the user closed the position (let's call
// it EC).
// At EC + 1, the total_lp_weight will be zero.
// Then, the user opens another position.
// The LAST_CLAIMED_EPOCH remains unchanged.
// When the user tries to query the rewards or claim the rewards with the new position,
// it would get a DivideByZero error, as the algorithm will try to iterate from EC + 1,
// where the total_lp_weight is zero.
// This scenario could have been fixed by skipping the rewards calculation if total_lp_weight was zero,
// but clearing up the LAST_CLAIMED_EPOCH and the LP_WEIGHT_HISTORY for the user was more correct
#[test]
fn test_query_rewards_divide_by_zero() {
    let lp_denom_1 = format!("factory/{MOCK_CONTRACT_ADDR_1}/1.{LP_SYMBOL}").to_string();

    let mut suite = TestingSuite::default_with_balances(vec![
        coin(INITIAL_BALANCE, UOM_DENOM.to_string()),
        coin(INITIAL_BALANCE, UUSDY_DENOM.to_string()),
        coin(INITIAL_BALANCE, UOSMO_DENOM.to_string()),
        coin(LP_DENOM_1_INITIAL_BALANCE, lp_denom_1.clone()),
    ]);

    let creator = suite.creator();

    suite.instantiate_default();

    let farm_manager = suite.farm_manager_addr.clone();

    suite.manage_farm(
        &creator,
        FarmAction::Fill {
            params: FarmParams {
                lp_denom: lp_denom_1.clone(),
                start_epoch: None,
                preliminary_end_epoch: None,
                curve: None,
                farm_asset: Coin {
                    denom: UUSDY_DENOM.to_string(),
                    amount: Uint128::new(FARM_UUSDY_ASSET_AMOUNT),
                },
                farm_identifier: None,
            },
        },
        vec![
            coin(FARM_UUSDY_ASSET_AMOUNT, UUSDY_DENOM.to_string()),
            coin(UOM_FARM_CREATION_FEE, UOM_DENOM.to_string()),
        ],
        |result| {
            result.unwrap();
        },
    );

    // creator and other fill a position
    suite.manage_position(
        &creator,
        PositionAction::Create {
            identifier: Some("creator_position".to_string()),
            unlocking_duration: DEFAULT_UNLOCKING_DURATION_SECONDS,
            receiver: None,
        },
        vec![coin(POSITION_LP_AMOUNT, lp_denom_1.clone())],
        |result| {
            result.unwrap();
        },
    );

    suite.add_epochs(5).query_current_epoch(|result| {
        let epoch_response = result.unwrap();
        assert_eq!(epoch_response.epoch.id, 5);
    });

    suite.query_rewards(&creator, None, |result| {
        result.unwrap();
    });

    suite
        .claim(&creator, vec![], None, |result| {
            result.unwrap();
        })
        .manage_position(
            &creator,
            PositionAction::Close {
                identifier: "u-creator_position".to_string(),
                lp_asset: None,
            },
            vec![],
            |result| {
                result.unwrap();
            },
        )
        .query_lp_weight(&farm_manager, &lp_denom_1, 6, |result| {
            result.unwrap();
        })
        .query_rewards(&creator, None, |result| {
            let rewards_response = result.unwrap();
            match rewards_response {
                RewardsResponse::RewardsResponse { total_rewards, .. } => {
                    assert!(total_rewards.is_empty());
                }
                _ => {
                    panic!("Wrong response type, should return RewardsResponse::RewardsResponse")
                }
            }
        })
        .query_lp_weight(&creator, &lp_denom_1, 4, |result| {
            result.unwrap_err();
        })
        .query_lp_weight(&creator, &lp_denom_1, 5, |result| {
            result.unwrap_err();
        })
        .query_lp_weight(&creator, &lp_denom_1, 6, |result| {
            result.unwrap_err();
        })
        .query_lp_weight(&creator, &lp_denom_1, 7, |result| {
            result.unwrap_err();
        });

    suite.add_epochs(2).query_rewards(&creator, None, |result| {
        let rewards_response = result.unwrap();
        match rewards_response {
            RewardsResponse::RewardsResponse { total_rewards, .. } => {
                assert!(total_rewards.is_empty());
            }
            _ => {
                panic!("Wrong response type, should return RewardsResponse::RewardsResponse")
            }
        }
    });

    // open a new position
    suite.manage_position(
        &creator,
        PositionAction::Create {
            identifier: Some("creator_another_position".to_string()),
            unlocking_duration: DEFAULT_UNLOCKING_DURATION_SECONDS,
            receiver: None,
        },
        vec![coin(CREATOR_ANOTHER_POSITION_LP_AMOUNT, lp_denom_1.clone())],
        |result| {
            result.unwrap();
        },
    );

    suite.add_one_epoch().query_current_epoch(|result| {
        let epoch_response = result.unwrap();
        assert_eq!(epoch_response.epoch.id, 8);
    });

    // this would normally fail as in some point of the reward calculation the total_lp_weight
    // would be zero.
    // This is a case that the contract shouldn't compute rewards for anyway, so the epoch is skipped.
    suite
        .query_rewards(&creator, None, |result| {
            let rewards_response = result.unwrap();
            match rewards_response {
                RewardsResponse::RewardsResponse { total_rewards, .. } => {
                    assert!(!total_rewards.is_empty());
                }
                _ => {
                    panic!("Wrong response type, should return RewardsResponse::RewardsResponse")
                }
            }
        })
        .claim(&creator, vec![], None, |result| {
            result.unwrap();
        })
        .query_lp_weight(&creator, &lp_denom_1, 7, |result| {
            result.unwrap_err();
        })
        .query_lp_weight(&creator, &lp_denom_1, 8, |result| {
            let lp_weight_response = result.unwrap();
            assert_eq!(
                lp_weight_response.lp_weight,
                Uint128::new(CREATOR_ANOTHER_POSITION_LP_AMOUNT)
            );
        })
        .query_lp_weight(&creator, &lp_denom_1, 9, |result| {
            result.unwrap_err();
        });

    suite.add_epochs(2).query_rewards(&creator, None, |result| {
        let rewards_response = result.unwrap();
        match rewards_response {
            RewardsResponse::RewardsResponse { total_rewards, .. } => {
                assert!(!total_rewards.is_empty());
            }
            _ => {
                panic!("Wrong response type, should return RewardsResponse::RewardsResponse")
            }
        }
    });

    // let's emergency withdraw the new position
    suite
        .manage_position(
            &creator,
            PositionAction::Withdraw {
                identifier: "u-creator_another_position".to_string(),
                emergency_unlock: Some(true),
            },
            vec![],
            |result| {
                result.unwrap();
            },
        )
        .query_lp_weight(&creator, &lp_denom_1, 9, |result| {
            result.unwrap_err();
        })
        .query_lp_weight(&creator, &lp_denom_1, 10, |result| {
            result.unwrap_err();
        })
        .query_lp_weight(&creator, &lp_denom_1, 11, |result| {
            result.unwrap_err();
        })
        .query_rewards(&creator, None, |result| {
            let rewards_response = result.unwrap();
            match rewards_response {
                RewardsResponse::RewardsResponse { total_rewards, .. } => {
                    assert!(total_rewards.is_empty());
                }
                _ => {
                    panic!("Wrong response type, should return RewardsResponse::RewardsResponse")
                }
            }
        });
}

/*
Suppose a user has two positions - position1 (lp_denom1), position2(lp_denom2).

- user calls close_position to close position1.
- In close_position→ update_weights, contract weight for lp_denom1 becomes 0 in the following epoch.
- In close_position→ reconcile_user_state, LAST_CLAIMED_EPOCH.remove is skipped due to user has other positions.
- after a few epochs, user create position3(lp_denom1).
- after a few epochs, user call claims. claim tx revert due to division by zero.
- Now all the rewards of the user would be locked.
*/
#[test]
fn test_query_rewards_divide_by_zero_mitigated() {
    let lp_denom_1 = format!("factory/{MOCK_CONTRACT_ADDR_1}/1.{LP_SYMBOL}").to_string();
    let lp_denom_2 = format!("factory/{MOCK_CONTRACT_ADDR_1}/2.{LP_SYMBOL}").to_string();

    let mut suite = TestingSuite::default_with_balances(vec![
        coin(INITIAL_BALANCE, UOM_DENOM.to_string()),
        coin(INITIAL_BALANCE, UUSDY_DENOM.to_string()),
        coin(INITIAL_BALANCE, UOSMO_DENOM.to_string()),
        coin(LP_DENOM_1_INITIAL_BALANCE, lp_denom_1.clone()),
        coin(LP_DENOM_2_INITIAL_BALANCE, lp_denom_2.clone()),
    ]);

    let alice = suite.creator();
    let bob = suite.senders[1].clone();

    suite.instantiate_default();

    // create overlapping farms
    suite
        .manage_farm(
            &alice,
            FarmAction::Fill {
                params: FarmParams {
                    lp_denom: lp_denom_1.clone(),
                    start_epoch: None,
                    preliminary_end_epoch: None,
                    curve: None,
                    farm_asset: Coin {
                        denom: UUSDY_DENOM.to_string(),
                        amount: Uint128::new(FARM_1_UUSDY_ASSET_AMOUNT),
                    },
                    farm_identifier: None,
                },
            },
            vec![
                coin(FARM_1_UUSDY_ASSET_AMOUNT, UUSDY_DENOM.to_string()),
                coin(UOM_FARM_CREATION_FEE, UOM_DENOM.to_string()),
            ],
            |result| {
                result.unwrap();
            },
        )
        .manage_farm(
            &alice,
            FarmAction::Fill {
                params: FarmParams {
                    lp_denom: lp_denom_2.clone(),
                    start_epoch: None,
                    preliminary_end_epoch: Some(FARM_2_END_EPOCH),
                    curve: None,
                    farm_asset: Coin {
                        denom: UUSDY_DENOM.to_string(),
                        amount: Uint128::new(FARM_2_UUSDY_ASSET_AMOUNT),
                    },
                    farm_identifier: None,
                },
            },
            vec![
                coin(FARM_2_UUSDY_ASSET_AMOUNT, UUSDY_DENOM.to_string()),
                coin(UOM_FARM_CREATION_FEE, UOM_DENOM.to_string()),
            ],
            |result| {
                result.unwrap();
            },
        );

    // creator and other fill two positions - one in a different lp_denom farm.
    suite.manage_position(
        &bob,
        PositionAction::Create {
            identifier: Some("creator_position".to_string()),
            unlocking_duration: DEFAULT_UNLOCKING_DURATION_SECONDS,
            receiver: None,
        },
        vec![coin(POSITION_LP_AMOUNT, lp_denom_1.clone())],
        |result| {
            result.unwrap();
        },
    );

    suite.manage_position(
        &bob,
        PositionAction::Create {
            identifier: Some("creator_another_position".to_string()),
            unlocking_duration: DEFAULT_UNLOCKING_DURATION_SECONDS,
            receiver: None,
        },
        vec![coin(POSITION_LP_AMOUNT, lp_denom_2.clone())],
        |result| {
            result.unwrap();
        },
    );

    suite.add_epochs(5).query_current_epoch(|result| {
        let epoch_response = result.unwrap();
        assert_eq!(epoch_response.epoch.id, 5);
    });

    suite.query_rewards(&bob, None, |result| {
        result.unwrap();
    });
    let farm_manager = suite.farm_manager_addr.clone();
    suite
        .claim(&bob, vec![], None, |result| {
            result.unwrap();
        })
        .manage_position(
            &bob,
            PositionAction::Close {
                identifier: "u-creator_position".to_string(),
                lp_asset: None,
            },
            vec![],
            |result| {
                result.unwrap();
            },
        )
        .query_lp_weight(&farm_manager, &lp_denom_1, 6, |result| {
            result.unwrap();
        })
        .query_rewards(&bob, None, |result| {
            let rewards_response = result.unwrap();
            match rewards_response {
                RewardsResponse::RewardsResponse { total_rewards, .. } => {
                    assert!(total_rewards.is_empty());
                }
                _ => {
                    panic!("Wrong response type, should return RewardsResponse::RewardsResponse")
                }
            }
        })
        .query_lp_weight(&bob, &lp_denom_1, 4, |result| {
            result.unwrap_err();
        })
        .query_lp_weight(&bob, &lp_denom_1, 5, |result| {
            result.unwrap_err();
        })
        .query_lp_weight(&bob, &lp_denom_1, 6, |result| {
            result.unwrap_err();
        })
        .query_lp_weight(&bob, &lp_denom_1, 7, |result| {
            result.unwrap_err();
        });

    suite.add_epochs(2); //6 & 7

    // open a new position
    suite.manage_position(
        &bob,
        PositionAction::Create {
            identifier: Some("creator_a_third_position".to_string()),
            unlocking_duration: DEFAULT_UNLOCKING_DURATION_SECONDS,
            receiver: None,
        },
        vec![coin(CREATOR_ANOTHER_POSITION_LP_AMOUNT, lp_denom_1.clone())],
        |result| {
            result.unwrap();
        },
    );
    suite.add_one_epoch().query_current_epoch(|result| {
        let epoch_response = result.unwrap();
        assert_eq!(epoch_response.epoch.id, 8);
    });

    // this would have failed if the divide by zero was not mitigated
    suite.query_rewards(&bob, None, |result| {
        let rewards_response = result.unwrap();
        assert_eq!(
            rewards_response,
            RewardsResponse::RewardsResponse {
                total_rewards: vec![coin(105895u128, UUSDY_DENOM.to_string())],
                rewards_per_lp_denom: vec![
                    (lp_denom_1.clone(), coins(634u128, UUSDY_DENOM.to_string())),
                    (
                        lp_denom_2.clone(),
                        coins(105261u128, UUSDY_DENOM.to_string())
                    ),
                ],
            }
        );
    });
}
