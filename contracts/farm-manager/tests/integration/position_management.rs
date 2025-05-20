extern crate core;

use std::cell::RefCell;

use cosmwasm_std::{coin, Coin, StdResult, Timestamp, Uint128};
use farm_manager::state::{MAX_FARMS_LIMIT, MAX_POSITIONS_LIMIT};
use farm_manager::ContractError;
use mantra_dex_std::constants::LP_SYMBOL;
use mantra_dex_std::farm_manager::{
    FarmAction, FarmParams, LpWeightResponse, Position, PositionAction, PositionsBy,
    PositionsResponse, RewardsResponse,
};

use crate::common::suite::TestingSuite;
use crate::common::{MOCK_CONTRACT_ADDR_1, MOCK_CONTRACT_ADDR_2};

#[test]
#[allow(clippy::inconsistent_digit_grouping)]
pub fn test_manage_position() {
    let lp_denom = format!("factory/{MOCK_CONTRACT_ADDR_1}/{LP_SYMBOL}").to_string();
    let another_lp = format!("factory/{MOCK_CONTRACT_ADDR_1}/2.{LP_SYMBOL}").to_string();
    let invalid_lp_denom = format!("factory/{MOCK_CONTRACT_ADDR_2}/{LP_SYMBOL}").to_string();

    let mut suite = TestingSuite::default_with_balances(vec![
        coin(1_000_000_000u128, "uom"),
        coin(1_000_000_000u128, "uusdy"),
        coin(1_000_000_000u128, "uosmo"),
        coin(1_000_000_000u128, lp_denom.clone()),
        coin(1_000_000_000u128, invalid_lp_denom.clone()),
        coin(1_000_000_000u128, another_lp.clone()),
    ]);

    let creator = suite.creator();
    let other = suite.senders[1].clone();
    let another = suite.senders[2].clone();

    suite.instantiate_default();

    let fee_collector = suite.fee_collector_addr.clone();
    let farm_manager = suite.farm_manager_addr.clone();
    let pool_manager = suite.pool_manager_addr.clone();

    // send some lp tokens to the pool manager, to simulate later the creation of a position
    // on behalf of a user by the pool manager
    suite.send_tokens(&creator, &pool_manager, &[coin(100_000, lp_denom.clone())]);

    suite
        .manage_farm(
            &creator,
            FarmAction::Fill {
                params: FarmParams {
                    lp_denom: lp_denom.clone(),
                    start_epoch: Some(2),
                    preliminary_end_epoch: Some(6),
                    curve: None,
                    farm_asset: Coin {
                        denom: "uusdy".to_string(),
                        amount: Uint128::new(8_000u128),
                    },
                    farm_identifier: None,
                },
            },
            vec![coin(8_000, "uusdy"), coin(1_000, "uom")],
            |result| {
                result.unwrap();
            },
        )
        .query_lp_weight(&farm_manager, &lp_denom, 0, |result| {
            let err = result.unwrap_err().to_string();

            assert_eq!(
                err,
                "Generic error: Querier contract error: There's no snapshot of the LP \
           weight in the contract for the epoch 0"
            );
        })
        .manage_position(
            &creator,
            PositionAction::Create {
                identifier: Some("creator_position".to_string()),
                unlocking_duration: 80_400,
                receiver: None,
            },
            vec![coin(1_000, lp_denom.clone())],
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();
                match err {
                    ContractError::InvalidUnlockingDuration { .. } => {}
                    _ => panic!(
                        "Wrong error type, should return ContractError::InvalidUnlockingDuration"
                    ),
                }
            },
        )
        .manage_position(
            &creator,
            PositionAction::Create {
                identifier: Some("creator_position".to_string()),
                unlocking_duration: 32_536_000,
                receiver: None,
            },
            vec![coin(1_000, lp_denom.clone())],
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();
                match err {
                    ContractError::InvalidUnlockingDuration { .. } => {}
                    _ => panic!(
                        "Wrong error type, should return ContractError::InvalidUnlockingDuration"
                    ),
                }
            },
        )
        .manage_position(
            &creator,
            PositionAction::Create {
                identifier: Some("creator_position".to_string()),
                unlocking_duration: 32_536_000,
                receiver: None,
            },
            vec![],
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();
                match err {
                    ContractError::PaymentError { .. } => {}
                    _ => panic!("Wrong error type, should return ContractError::PaymentError"),
                }
            },
        )
        .manage_position(
            &creator,
            PositionAction::Create {
                identifier: Some("creator_position".to_string()),
                unlocking_duration: 86_400,
                receiver: None,
            },
            vec![coin(1_000, lp_denom.clone())],
            |result| {
                result.unwrap();
            },
        )
        .query_lp_weight(&farm_manager, &lp_denom, 1, |result| {
            let lp_weight = result.unwrap();
            assert_eq!(
                lp_weight,
                LpWeightResponse {
                    lp_weight: Uint128::new(1_000),
                    epoch_id: 1,
                }
            );
        })
        // refilling the position with a different LP asset should fail
        .manage_position(
            &creator,
            PositionAction::Expand {
                identifier: "u-creator_position".to_string(),
            },
            vec![coin(1_000, another_lp.clone())],
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();
                match err {
                    ContractError::AssetMismatch => {}
                    _ => panic!("Wrong error type, should return ContractError::AssetMismatch"),
                }
            },
        )
        .query_positions(
            Some(PositionsBy::Receiver(creator.to_string())),
            Some(true),
            None,
            None,
            |result| {
                let positions = result.unwrap();
                assert_eq!(positions.positions.len(), 1);
                assert_eq!(
                    positions.positions[0],
                    Position {
                        identifier: "u-creator_position".to_string(),
                        lp_asset: Coin {
                            denom: format!("factory/{MOCK_CONTRACT_ADDR_1}/{LP_SYMBOL}")
                                .to_string(),
                            amount: Uint128::new(1_000),
                        },
                        unlocking_duration: 86400,
                        open: true,
                        expiring_at: None,
                        receiver: creator.clone(),
                    }
                );
            },
        )
        .manage_position(
            &creator,
            PositionAction::Expand {
                identifier: "u-creator_position".to_string(),
            },
            vec![coin(5_000, lp_denom.clone())],
            |result| {
                result.unwrap();
            },
        )
        .manage_position(
            &creator,
            PositionAction::Withdraw {
                identifier: "u-creator_position".to_string(),
                emergency_unlock: None,
            },
            vec![],
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();
                // the position is not closed or hasn't expired yet
                match err {
                    ContractError::Unauthorized => {}
                    _ => panic!("Wrong error type, should return ContractError::Unauthorized"),
                }
            },
        )
        .query_lp_weight(&farm_manager, &lp_denom, 1, |result| {
            let lp_weight = result.unwrap();
            assert_eq!(
                lp_weight,
                LpWeightResponse {
                    lp_weight: Uint128::new(6_000),
                    epoch_id: 1,
                }
            );
        })
        .query_positions(
            Some(PositionsBy::Receiver(creator.to_string())),
            Some(true),
            None,
            None,
            |result| {
                let positions = result.unwrap();
                assert_eq!(positions.positions.len(), 1);
                assert_eq!(
                    positions.positions[0],
                    Position {
                        identifier: "u-creator_position".to_string(),
                        lp_asset: Coin {
                            denom: format!("factory/{MOCK_CONTRACT_ADDR_1}/{LP_SYMBOL}")
                                .to_string(),
                            amount: Uint128::new(6_000),
                        },
                        unlocking_duration: 86400,
                        open: true,
                        expiring_at: None,
                        receiver: creator.clone(),
                    }
                );
            },
        )
        .query_lp_weight(&farm_manager, &lp_denom, 1, |result| {
            let lp_weight = result.unwrap();
            assert_eq!(
                lp_weight,
                LpWeightResponse {
                    lp_weight: Uint128::new(6_000),
                    epoch_id: 1,
                }
            );
        })
        .add_one_epoch()
        .query_current_epoch(|result| {
            let epoch_response = result.unwrap();
            assert_eq!(epoch_response.epoch.id, 1);
        });

    // make sure snapshots are working correctly
    suite
        .add_one_epoch()
        .query_current_epoch(|result| {
            let epoch_response = result.unwrap();
            assert_eq!(epoch_response.epoch.id, 2);
        })
        .manage_position(
            &creator,
            PositionAction::Expand {
                //refill position
                identifier: "u-creator_position".to_string(),
            },
            vec![coin(1_000, lp_denom.clone())],
            |result| {
                result.unwrap();
            },
        );

    suite.query_current_epoch(|result| {
        let epoch_response = result.unwrap();
        assert_eq!(epoch_response.epoch.id, 2);
    });

    suite
        .manage_position(
            &creator,
            PositionAction::Close {
                identifier: "u-creator_position".to_string(),
                lp_asset: Some(Coin {
                    denom: lp_denom.clone(),
                    amount: Uint128::new(4_000),
                }),
            },
            vec![coin(4_000, lp_denom.clone())],
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();
                match err {
                    ContractError::PaymentError { .. } => {}
                    _ => panic!("Wrong error type, should return ContractError::PaymentError"),
                }
            },
        )
        .manage_position(
            &creator,
            PositionAction::Close {
                // remove 4_000 from the 7_000 position
                identifier: "u-creator_position".to_string(),
                lp_asset: Some(Coin {
                    denom: lp_denom.clone(),
                    amount: Uint128::new(4_000),
                }),
            },
            vec![],
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();
                match err {
                    ContractError::PendingRewards => {}
                    _ => panic!("Wrong error type, should return ContractError::PendingRewards"),
                }
            },
        )
        .claim(
            &creator,
            vec![coin(4_000, lp_denom.clone())],
            None,
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();
                match err {
                    ContractError::PaymentError { .. } => {}
                    _ => panic!("Wrong error type, should return ContractError::PaymentError"),
                }
            },
        )
        .claim(&other, vec![], None, |result| {
            let err = result.unwrap_err().downcast::<ContractError>().unwrap();
            match err {
                ContractError::NoOpenPositions => {}
                _ => panic!("Wrong error type, should return ContractError::NoOpenPositions"),
            }
        })
        .query_balance("uusdy".to_string(), &creator, |balance| {
            assert_eq!(balance, Uint128::new(999_992_000));
        })
        .claim(&creator, vec![], None, |result| {
            result.unwrap();
        })
        .query_balance("uusdy".to_string(), &creator, |balance| {
            assert_eq!(balance, Uint128::new(999_994_000));
        })
        .query_farms(None, None, None, |result| {
            let farms_response = result.unwrap();
            assert_eq!(farms_response.farms.len(), 1);
            assert_eq!(farms_response.farms[0].claimed_amount, Uint128::new(2_000));
        })
        .manage_position(
            &creator,
            PositionAction::Close {
                identifier: "non_existent__position".to_string(),
                lp_asset: Some(Coin {
                    denom: lp_denom.clone(),
                    amount: Uint128::new(4_000),
                }),
            },
            vec![],
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();
                match err {
                    ContractError::NoPositionFound { .. } => {}
                    _ => panic!("Wrong error type, should return ContractError::NoPositionFound"),
                }
            },
        )
        .manage_position(
            &other,
            PositionAction::Close {
                identifier: "u-creator_position".to_string(),
                lp_asset: Some(Coin {
                    denom: lp_denom.clone(),
                    amount: Uint128::new(4_000),
                }),
            },
            vec![],
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();
                match err {
                    ContractError::Unauthorized => {}
                    _ => panic!("Wrong error type, should return ContractError::Unauthorized"),
                }
            },
        )
        .manage_position(
            &creator,
            PositionAction::Close {
                identifier: "u-creator_position".to_string(),
                lp_asset: Some(Coin {
                    denom: another_lp.clone(),
                    amount: Uint128::new(4_000),
                }),
            },
            vec![],
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();
                match err {
                    ContractError::AssetMismatch => {}
                    _ => panic!("Wrong error type, should return ContractError::AssetMismatch"),
                }
            },
        )
        .manage_position(
            &creator, // someone tries to close the creator's position
            PositionAction::Close {
                identifier: "u-creator_position".to_string(),
                lp_asset: Some(Coin {
                    denom: lp_denom.to_string(),
                    amount: Uint128::new(10_000),
                }),
            },
            vec![],
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();
                match err {
                    ContractError::InvalidLpAmount { expected, actual } => {
                        assert_eq!(expected, Uint128::new(7_000));
                        assert_eq!(actual, Uint128::new(10_000));
                    }
                    _ => panic!("Wrong error type, should return ContractError::InvalidLpAmount"),
                }
            },
        )
        .manage_position(
            &creator,
            PositionAction::Close {
                // remove 5_000 from the 7_000 position
                identifier: "u-creator_position".to_string(),
                lp_asset: Some(Coin {
                    denom: lp_denom.clone(),
                    amount: Uint128::new(5_000),
                }),
            },
            vec![],
            |result| {
                result.unwrap();
            },
        )
        .manage_position(
            &creator,
            PositionAction::Withdraw {
                identifier: "p-1".to_string(),
                emergency_unlock: None,
            },
            vec![],
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();
                match err {
                    ContractError::PositionNotExpired => {}
                    _ => {
                        panic!("Wrong error type, should return ContractError::PositionNotExpired")
                    }
                }
            },
        )
        .query_lp_weight(&farm_manager, &lp_denom, 3, |result| {
            let lp_weight = result.unwrap();
            assert_eq!(
                lp_weight,
                LpWeightResponse {
                    // should be the same for epoch 2, as the weight for new positions is added
                    // to the next epoch
                    lp_weight: Uint128::new(2_000),
                    epoch_id: 3,
                }
            );
        })
        // create a few epochs without any changes in the weight
        .add_one_epoch()
        //after a day the closed position should be able to be withdrawn
        .manage_position(
            &other,
            PositionAction::Withdraw {
                identifier: "u-creator_position".to_string(),
                emergency_unlock: None,
            },
            vec![coin(5_000, lp_denom.clone())],
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();
                match err {
                    ContractError::PaymentError { .. } => {}
                    _ => panic!("Wrong error type, should return ContractError::PaymentError"),
                }
            },
        )
        .manage_position(
            &creator,
            PositionAction::Withdraw {
                identifier: "non_existent_position".to_string(),
                emergency_unlock: None,
            },
            vec![],
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();
                match err {
                    ContractError::NoPositionFound { .. } => {}
                    _ => panic!("Wrong error type, should return ContractError::NoPositionFound"),
                }
            },
        )
        .manage_position(
            &other,
            PositionAction::Withdraw {
                identifier: "p-1".to_string(),
                emergency_unlock: None,
            },
            vec![],
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();
                match err {
                    ContractError::Unauthorized => {}
                    _ => panic!("Wrong error type, should return ContractError::Unauthorized"),
                }
            },
        )
        .add_one_epoch()
        .add_one_epoch()
        .query_current_epoch(|result| {
            let epoch_response = result.unwrap();
            assert_eq!(epoch_response.epoch.id, 5);
        })
        .add_one_epoch()
        .query_rewards(&creator, None, |result| {
            let rewards_response = result.unwrap();
            match rewards_response {
                RewardsResponse::RewardsResponse { total_rewards, .. } => {
                    assert_eq!(total_rewards.len(), 1);
                    assert_eq!(
                        total_rewards[0],
                        Coin {
                            denom: "uusdy".to_string(),
                            amount: Uint128::new(6_000),
                        }
                    );
                }
                _ => panic!("shouldn't return this but RewardsResponse"),
            }
        })
        .query_farms(None, None, None, |result| {
            let farms_response = result.unwrap();
            assert_eq!(farms_response.farms[0].claimed_amount, Uint128::new(2_000));
        })
        .manage_position(
            &creator,
            PositionAction::Withdraw {
                identifier: "p-1".to_string(),
                emergency_unlock: None,
            },
            vec![],
            |result| {
                result.unwrap();
            },
        )
        .query_balance("uusdy".to_string(), &creator, |balance| {
            assert_eq!(balance, Uint128::new(999_994_000));
        })
        .claim(&creator, vec![], None, |result| {
            result.unwrap();
        })
        .query_farms(None, None, None, |result| {
            let farms_response = result.unwrap();
            assert_eq!(
                farms_response.farms[0].farm_asset.amount,
                farms_response.farms[0].claimed_amount
            );
        })
        .query_rewards(&creator, None, |result| {
            let rewards_response = result.unwrap();
            match rewards_response {
                RewardsResponse::RewardsResponse { total_rewards, .. } => {
                    assert!(total_rewards.is_empty());
                }
                _ => panic!("shouldn't return this but RewardsResponse"),
            }
        })
        .query_balance("uusdy".to_string(), &creator, |balance| {
            assert_eq!(balance, Uint128::new(1_000_000_000));
        })
        .query_positions(
            Some(PositionsBy::Receiver(other.to_string())),
            Some(false),
            None,
            None,
            |result| {
                let positions = result.unwrap();
                assert!(positions.positions.is_empty());
            },
        )
        .manage_position(
            &creator,
            PositionAction::Create {
                identifier: None,
                unlocking_duration: 86_400,
                receiver: Some(another.to_string()),
            },
            vec![coin(5_000, lp_denom.clone())],
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();
                match err {
                    ContractError::Unauthorized => {}
                    _ => panic!("Wrong error type, should return ContractError::Unauthorized"),
                }
            },
        )
        .manage_position(
            &pool_manager,
            PositionAction::Create {
                identifier: None,
                unlocking_duration: 86_400,
                receiver: Some(another.to_string()),
            },
            vec![coin(5_000, lp_denom.clone())],
            |result| {
                result.unwrap();
            },
        )
        .query_positions(
            Some(PositionsBy::Receiver(another.to_string())),
            Some(true),
            None,
            None,
            |result| {
                let positions = result.unwrap();
                assert_eq!(positions.positions.len(), 1);
                assert_eq!(
                    positions.positions[0],
                    Position {
                        identifier: "p-2".to_string(),
                        lp_asset: Coin {
                            denom: format!("factory/{MOCK_CONTRACT_ADDR_1}/{LP_SYMBOL}")
                                .to_string(),
                            amount: Uint128::new(5_000),
                        },
                        unlocking_duration: 86400,
                        open: true,
                        expiring_at: None,
                        receiver: another.clone(),
                    }
                );
            },
        )
        .manage_position(
            &creator,
            PositionAction::Close {
                identifier: "p-2".to_string(),
                lp_asset: None,
            },
            vec![],
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();
                match err {
                    ContractError::Unauthorized => {}
                    _ => panic!("Wrong error type, should return ContractError::Unauthorized"),
                }
            },
        )
        .manage_position(
            &another,
            PositionAction::Close {
                identifier: "p-2".to_string(),
                lp_asset: None, //close in full
            },
            vec![],
            |result| {
                result.unwrap();
            },
        )
        .query_positions(
            Some(PositionsBy::Receiver(another.to_string())),
            Some(true),
            None,
            None,
            |result| {
                let positions = result.unwrap();
                assert!(positions.positions.is_empty());
            },
        )
        .query_positions(
            Some(PositionsBy::Receiver(another.to_string())),
            Some(false),
            None,
            None,
            |result| {
                let positions = result.unwrap();
                assert_eq!(positions.positions.len(), 1);
                assert_eq!(
                    positions.positions[0],
                    Position {
                        identifier: "p-2".to_string(),
                        lp_asset: Coin {
                            denom: format!("factory/{MOCK_CONTRACT_ADDR_1}/{LP_SYMBOL}")
                                .to_string(),
                            amount: Uint128::new(5_000),
                        },
                        unlocking_duration: 86400,
                        open: false,
                        expiring_at: Some(1712847600),
                        receiver: another.clone(),
                    }
                );
            },
        );

    suite
        .add_one_epoch()
        .add_one_epoch()
        .query_current_epoch(|result| {
            let epoch_response = result.unwrap();
            assert_eq!(epoch_response.epoch.id, 8);
        });

    // try emergency exit a position that is closed
    suite
        .manage_position(
            &another,
            PositionAction::Create {
                identifier: Some("special_position".to_string()),
                unlocking_duration: 100_000,
                receiver: None,
            },
            vec![coin(5_000, lp_denom.clone())],
            |result| {
                result.unwrap();
            },
        )
        .query_lp_weight(&farm_manager, &lp_denom, 9, |result| {
            let lp_weight = result.unwrap();
            assert_eq!(
                lp_weight,
                LpWeightResponse {
                    lp_weight: Uint128::new(7_002),
                    epoch_id: 9,
                }
            );
        });

    suite.add_one_epoch().query_current_epoch(|result| {
        let epoch_response = result.unwrap();
        assert_eq!(epoch_response.epoch.id, 9);
    });

    // close the position
    suite
        .manage_position(
            &another,
            PositionAction::Close {
                identifier: "u-special_position".to_string(),
                lp_asset: None,
            },
            vec![],
            |result| {
                result.unwrap();
            },
        )
        .query_lp_weight(&farm_manager, &lp_denom, 10, |result| {
            let lp_weight = result.unwrap();
            assert_eq!(
                lp_weight,
                LpWeightResponse {
                    // the weight went back to what it was before the position was opened
                    lp_weight: Uint128::new(2_000),
                    epoch_id: 10,
                }
            );
        });

    // emergency exit
    suite
        .query_balance(lp_denom.clone().to_string(), &fee_collector, |balance| {
            assert_eq!(balance, Uint128::zero());
        })
        .manage_position(
            &another,
            PositionAction::Close {
                identifier: "u-special_position".to_string(),
                lp_asset: None,
            },
            vec![],
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();
                match err {
                    ContractError::PositionAlreadyClosed { .. } => {}
                    _ => panic!(
                        "Wrong error type, should return ContractError::PositionAlreadyClosed"
                    ),
                }
            },
        )
        .manage_position(
            &another,
            PositionAction::Withdraw {
                identifier: "u-special_position".to_string(),
                emergency_unlock: Some(true),
            },
            vec![],
            |result| {
                result.unwrap();
            },
        )
        .query_balance(lp_denom.clone().to_string(), &fee_collector, |balance| {
            assert_eq!(balance, Uint128::new(500));
        });

    // trying to open a position with an invalid lp which has not been created by the pool manager
    // should fail
    suite.manage_position(
        &other,
        PositionAction::Create {
            identifier: Some("a_new_position_with_invalid_lp".to_string()),
            unlocking_duration: 86_400,
            receiver: None,
        },
        vec![coin(5_000, invalid_lp_denom.clone())],
        |result| {
            let err = result.unwrap_err().downcast::<ContractError>().unwrap();
            match err {
                ContractError::AssetMismatch => {}
                _ => panic!("Wrong error type, should return ContractError::AssetMismatch"),
            }
        },
    );

    suite.manage_position(
        &another,
        PositionAction::Withdraw {
            identifier: "p-2".to_string(),
            emergency_unlock: None,
        },
        vec![],
        |result| {
            result.unwrap();
        },
    );

    // create a position and close it in full by specifying the total amount of LP to close
    suite
        .manage_position(
            &another,
            PositionAction::Create {
                identifier: Some("to_be_closed_in_full".to_string()),
                unlocking_duration: 86_400,
                receiver: None,
            },
            vec![coin(5_000, lp_denom.clone())],
            |result| {
                result.unwrap();
            },
        )
        .query_positions(
            Some(PositionsBy::Receiver(another.to_string())),
            Some(true),
            None,
            None,
            |result| {
                let positions = result.unwrap();
                assert_eq!(positions.positions.len(), 1);
                assert_eq!(
                    positions.positions[0],
                    Position {
                        identifier: "u-to_be_closed_in_full".to_string(),
                        lp_asset: Coin {
                            denom: format!("factory/{MOCK_CONTRACT_ADDR_1}/{LP_SYMBOL}")
                                .to_string(),
                            amount: Uint128::new(5_000),
                        },
                        unlocking_duration: 86400,
                        open: true,
                        expiring_at: None,
                        receiver: another.clone(),
                    }
                );
            },
        )
        .manage_position(
            &another,
            PositionAction::Close {
                identifier: "u-to_be_closed_in_full".to_string(),
                lp_asset: Some(Coin {
                    denom: lp_denom.clone(),
                    amount: Uint128::new(5_000),
                }),
            },
            vec![],
            |result| {
                result.unwrap();
            },
        )
        .query_positions(
            Some(PositionsBy::Receiver(another.to_string())),
            Some(false),
            None,
            None,
            |result| {
                let positions = result.unwrap();
                assert_eq!(positions.positions.len(), 1);
                assert_eq!(
                    positions.positions[0],
                    Position {
                        identifier: "u-to_be_closed_in_full".to_string(),
                        lp_asset: Coin {
                            denom: format!("factory/{MOCK_CONTRACT_ADDR_1}/{LP_SYMBOL}")
                                .to_string(),
                            amount: Uint128::new(5_000),
                        },
                        unlocking_duration: 86400,
                        open: false,
                        expiring_at: Some(1_713_106_800),
                        receiver: another.clone(),
                    }
                );
            },
        )
        .query_positions(
            Some(PositionsBy::Receiver(another.to_string())),
            Some(true),
            None,
            None,
            |result| {
                let positions = result.unwrap();
                assert!(positions.positions.is_empty());
            },
        );
}

#[test]
#[allow(clippy::inconsistent_digit_grouping)]
pub fn test_withdrawing_open_positions_updates_weight() {
    let lp_denom = format!("factory/{MOCK_CONTRACT_ADDR_1}/{LP_SYMBOL}").to_string();
    let another_lp = format!("factory/{MOCK_CONTRACT_ADDR_1}/2.{LP_SYMBOL}").to_string();
    let invalid_lp_denom = format!("factory/{MOCK_CONTRACT_ADDR_2}/{LP_SYMBOL}").to_string();

    let mut suite = TestingSuite::default_with_balances(vec![
        coin(1_000_000_000u128, "uom"),
        coin(1_000_000_000u128, "uusdy"),
        coin(1_000_000_000u128, "uosmo"),
        coin(1_000_000_000u128, lp_denom.clone()),
        coin(1_000_000_000u128, invalid_lp_denom.clone()),
        coin(1_000_000_000u128, another_lp.clone()),
    ]);

    let creator = suite.creator();

    suite.instantiate_default();

    let farm_manager = suite.farm_manager_addr.clone();
    let pool_manager = suite.pool_manager_addr.clone();

    // send some lp tokens to the pool manager, to simulate later the creation of a position
    // on behalf of a user by the pool manager
    suite.send_tokens(&creator, &pool_manager, &[coin(100_000, lp_denom.clone())]);

    suite
        .manage_farm(
            &creator,
            FarmAction::Fill {
                params: FarmParams {
                    lp_denom: lp_denom.clone(),
                    start_epoch: Some(2),
                    preliminary_end_epoch: Some(6),
                    curve: None,
                    farm_asset: Coin {
                        denom: "uusdy".to_string(),
                        amount: Uint128::new(8_000u128),
                    },
                    farm_identifier: None,
                },
            },
            vec![coin(8_000, "uusdy"), coin(1_000, "uom")],
            |result| {
                result.unwrap();
            },
        )
        .query_lp_weight(&farm_manager, &lp_denom, 0, |result| {
            let err = result.unwrap_err().to_string();

            assert_eq!(
                err,
                "Generic error: Querier contract error: There's no snapshot of the LP \
           weight in the contract for the epoch 0"
            );
        })
        .manage_position(
            &creator,
            PositionAction::Create {
                identifier: Some("creator_position".to_string()),
                unlocking_duration: 86_400,
                receiver: None,
            },
            vec![coin(2_000, lp_denom.clone())],
            |result| {
                result.unwrap();
            },
        )
        .query_lp_weight(&farm_manager, &lp_denom, 1, |result| {
            let lp_weight = result.unwrap();
            assert_eq!(
                lp_weight,
                LpWeightResponse {
                    lp_weight: Uint128::new(2_000),
                    epoch_id: 1,
                }
            );
        });

    suite
        .add_one_epoch()
        .add_one_epoch()
        .query_current_epoch(|result| {
            let epoch_response = result.unwrap();
            assert_eq!(epoch_response.epoch.id, 2);
        })
        .query_rewards(&creator, None, |result| {
            let rewards_response = result.unwrap();
            match rewards_response {
                RewardsResponse::RewardsResponse { total_rewards, .. } => {
                    assert_eq!(total_rewards.len(), 1);
                    assert_eq!(
                        total_rewards[0],
                        Coin {
                            denom: "uusdy".to_string(),
                            amount: Uint128::new(2_000),
                        }
                    );
                }
                _ => panic!("shouldn't return this but RewardsResponse"),
            }
        })
        .query_farms(None, None, None, |result| {
            let farms_response = result.unwrap();
            assert_eq!(farms_response.farms.len(), 1);
            assert_eq!(farms_response.farms[0].claimed_amount, Uint128::zero());
        });

    // withdraw the position
    suite
        .manage_position(
            &creator,
            PositionAction::Withdraw {
                identifier: "u-creator_position".to_string(),
                emergency_unlock: None,
            },
            vec![],
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();
                match err {
                    ContractError::Unauthorized => {}
                    _ => panic!("Wrong error type, should return ContractError::Unauthorized"),
                }
            },
        )
        .manage_position(
            &creator,
            PositionAction::Withdraw {
                identifier: "u-creator_position".to_string(),
                emergency_unlock: Some(true),
            },
            vec![],
            |result| {
                result.unwrap();
            },
        )
        // the weight is updated after the position is withdrawn with the emergency flag
        .query_lp_weight(&farm_manager, &lp_denom, 3, |result| {
            let lp_weight = result.unwrap();
            assert_eq!(
                lp_weight,
                LpWeightResponse {
                    lp_weight: Uint128::zero(),
                    epoch_id: 3,
                }
            );
        })
        .query_farms(None, None, None, |result| {
            let farms_response = result.unwrap();
            assert_eq!(farms_response.farms.len(), 1);
            assert_eq!(farms_response.farms[0].claimed_amount, Uint128::zero());
        });
}

#[test]
#[allow(clippy::inconsistent_digit_grouping)]
pub fn test_expand_position_unsuccessfully() {
    let lp_denom = format!("factory/{MOCK_CONTRACT_ADDR_1}/{LP_SYMBOL}").to_string();
    let another_lp = format!("factory/{MOCK_CONTRACT_ADDR_1}/2.{LP_SYMBOL}").to_string();
    let invalid_lp_denom = format!("factory/{MOCK_CONTRACT_ADDR_2}/{LP_SYMBOL}").to_string();

    let mut suite = TestingSuite::default_with_balances(vec![
        coin(1_000_000_000u128, "uom"),
        coin(1_000_000_000u128, "uusdy"),
        coin(1_000_000_000u128, "uosmo"),
        coin(1_000_000_000u128, lp_denom.clone()),
        coin(1_000_000_000u128, invalid_lp_denom.clone()),
        coin(1_000_000_000u128, another_lp.clone()),
    ]);

    let creator = suite.creator();

    suite.instantiate_default();

    suite
        // open position
        .manage_position(
            &creator,
            PositionAction::Create {
                identifier: Some("creator_position".to_string()),
                unlocking_duration: 86_400,
                receiver: None,
            },
            vec![coin(10_000, &lp_denom)],
            |result| {
                result.unwrap();
            },
        )
        .query_positions(
            Some(PositionsBy::Receiver(creator.to_string())),
            None,
            None,
            None,
            |result| {
                let positions = result.unwrap();
                assert_eq!(positions.positions.len(), 1);
                assert_eq!(
                    positions.positions[0],
                    Position {
                        identifier: "u-creator_position".to_string(),
                        lp_asset: Coin {
                            denom: lp_denom.clone(),
                            amount: Uint128::new(10_000),
                        },
                        unlocking_duration: 86400,
                        open: true,
                        expiring_at: None,
                        receiver: creator.clone(),
                    }
                );
            },
        )
        .add_one_epoch()
        // close position
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
        .query_positions(
            Some(PositionsBy::Receiver(creator.to_string())),
            None,
            None,
            None,
            |result| {
                let positions = result.unwrap();
                assert_eq!(positions.positions.len(), 1);
                assert_eq!(
                    positions.positions[0],
                    Position {
                        identifier: "u-creator_position".to_string(),
                        lp_asset: Coin {
                            denom: lp_denom.clone(),
                            amount: Uint128::new(10_000),
                        },
                        unlocking_duration: 86400,
                        open: false,
                        expiring_at: Some(1_712_415_600),
                        receiver: creator.clone(),
                    }
                );
            },
        )
        // try refilling the closed position should err
        .manage_position(
            &creator,
            PositionAction::Expand {
                identifier: "u-creator_position".to_string(),
            },
            vec![coin(10_000, &lp_denom)],
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();
                match err {
                    ContractError::PositionAlreadyClosed { .. } => {}
                    _ => panic!(
                        "Wrong error type, should return ContractError::PositionAlreadyClosed"
                    ),
                }
            },
        );
}

#[test]
#[allow(clippy::inconsistent_digit_grouping)]
pub fn cant_create_position_with_overlapping_identifier() {
    let lp_denom = format!("factory/{MOCK_CONTRACT_ADDR_1}/{LP_SYMBOL}").to_string();
    let another_lp = format!("factory/{MOCK_CONTRACT_ADDR_1}/2.{LP_SYMBOL}").to_string();
    let invalid_lp_denom = format!("factory/{MOCK_CONTRACT_ADDR_2}/{LP_SYMBOL}").to_string();

    let mut suite = TestingSuite::default_with_balances(vec![
        coin(1_000_000_000u128, "uom"),
        coin(1_000_000_000u128, "uusdy"),
        coin(1_000_000_000u128, "uosmo"),
        coin(1_000_000_000u128, lp_denom.clone()),
        coin(1_000_000_000u128, invalid_lp_denom.clone()),
        coin(1_000_000_000u128, another_lp.clone()),
    ]);

    let alice = suite.creator();
    let bob = suite.senders[1].clone();

    suite.instantiate_default();

    suite
        // open position
        .manage_position(
            &alice,
            PositionAction::Create {
                identifier: Some("u-2".to_string()),
                unlocking_duration: 86_400,
                receiver: None,
            },
            vec![coin(10_000, &lp_denom)],
            |result| {
                result.unwrap();
            },
        )
        .query_positions(
            Some(PositionsBy::Receiver(alice.to_string())),
            None,
            None,
            None,
            |result| {
                let positions = result.unwrap();
                assert_eq!(positions.positions.len(), 1);
                assert_eq!(
                    positions.positions[0],
                    Position {
                        identifier: "u-u-2".to_string(),
                        lp_asset: Coin {
                            denom: lp_denom.clone(),
                            amount: Uint128::new(10_000),
                        },
                        unlocking_duration: 86400,
                        open: true,
                        expiring_at: None,
                        receiver: alice.clone(),
                    }
                );
            },
        )
        .manage_position(
            &bob,
            PositionAction::Create {
                // this would normally overlap with the previous position, as the identifier the contract will
                // assign would be "2". It doesn't fail now as the position identifiers have a
                // prefix
                identifier: None,
                unlocking_duration: 86_400,
                receiver: None,
            },
            vec![coin(10_000, &lp_denom)],
            |result| {
                result.unwrap();
            },
        )
        .query_positions(
            Some(PositionsBy::Receiver(bob.to_string())),
            None,
            None,
            None,
            |result| {
                let positions = result.unwrap();
                assert_eq!(positions.positions.len(), 1);
                assert_eq!(
                    positions.positions[0],
                    Position {
                        identifier: "p-1".to_string(),
                        lp_asset: Coin {
                            denom: lp_denom.clone(),
                            amount: Uint128::new(10_000),
                        },
                        unlocking_duration: 86400,
                        open: true,
                        expiring_at: None,
                        receiver: bob.clone(),
                    }
                );
            },
        );
}

#[test]
fn test_fill_closed_position() {
    let lp_denom_1 =
        format!("factory/{MOCK_CONTRACT_ADDR_1}/pool.identifier.{LP_SYMBOL}").to_string();
    let lp_denom_2 = format!("factory/{MOCK_CONTRACT_ADDR_1}/2.{LP_SYMBOL}").to_string();

    let mut suite = TestingSuite::default_with_balances(vec![
        coin(1_000_000_000u128, "uom".to_string()),
        coin(1_000_000_000u128, "uusdy".to_string()),
        coin(1_000_000_000u128, "uosmo".to_string()),
        coin(1_000_000_000u128, lp_denom_1.clone()),
        coin(1_000_000_000u128, lp_denom_2.clone()),
    ]);

    let creator = suite.creator();

    suite.instantiate_default();

    for _ in 0..10 {
        suite.add_one_epoch();
    }

    let farm_manager_addr = suite.farm_manager_addr.clone();

    suite.query_current_epoch(|result| {
        let epoch_response = result.unwrap();
        assert_eq!(epoch_response.epoch.id, 10);
    });

    let time = RefCell::new(Timestamp::default());
    let time2 = RefCell::new(Timestamp::default());

    // open a position
    // close a position (partially and fully)
    // try to top up the same (closed) position, should err
    suite
        .query_balance(lp_denom_1.to_string(), &farm_manager_addr, |balance| {
            assert_eq!(balance, Uint128::zero());
        })
        .manage_position(
            &creator,
            PositionAction::Create {
                identifier: Some("creator_position".to_string()),
                unlocking_duration: 86_400,
                receiver: None,
            },
            vec![coin(1_000, lp_denom_1.clone())],
            |result| {
                result.unwrap();
            },
        )
        .query_positions(
            Some(PositionsBy::Receiver(creator.to_string())),
            None,
            None,
            None,
            |result| {
                let response = result.unwrap();
                assert_eq!(response.positions.len(), 1);
                assert_eq!(
                    response.positions[0],
                    Position {
                        identifier: "u-creator_position".to_string(),
                        lp_asset: coin(1_000, lp_denom_1.clone()),
                        unlocking_duration: 86_400,
                        open: true,
                        expiring_at: None,
                        receiver: creator.clone(),
                    }
                );
            },
        )
        .get_time(|result| {
            *time.borrow_mut() = result;
        })
        .manage_position(
            &creator,
            PositionAction::Close {
                identifier: "u-creator_position".to_string(),
                lp_asset: Some(coin(600, lp_denom_1.clone())),
            },
            vec![],
            |result| {
                result.unwrap();
            },
        )
        .query_positions(
            Some(PositionsBy::Receiver(creator.to_string())),
            None,
            None,
            None,
            |result| {
                let response = result.unwrap();
                assert_eq!(response.positions.len(), 2);
                assert_eq!(
                    response.positions[0],
                    Position {
                        identifier: "p-1".to_string(),
                        lp_asset: coin(600, lp_denom_1.clone()),
                        unlocking_duration: 86_400,
                        open: false,
                        expiring_at: Some(time.borrow().plus_seconds(86_400).seconds()),
                        receiver: creator.clone(),
                    }
                );
                assert_eq!(
                    response.positions[1],
                    Position {
                        identifier: "u-creator_position".to_string(),
                        lp_asset: coin(400, lp_denom_1.clone()),
                        unlocking_duration: 86_400,
                        open: true,
                        expiring_at: None,
                        receiver: creator.clone(),
                    }
                );
            },
        )
        // try to refill the closed position, i.e. "2"
        .manage_position(
            &creator,
            PositionAction::Expand {
                identifier: "p-1".to_string(),
            },
            vec![coin(10_000, lp_denom_1.clone())],
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();
                match err {
                    ContractError::PositionAlreadyClosed { identifier } => {
                        assert_eq!(identifier, "p-1".to_string())
                    }
                    _ => panic!(
                        "Wrong error type, should return ContractError::PositionAlreadyClosed"
                    ),
                }
            },
        )
        .query_lp_weight(&creator, &lp_denom_1, 11, |result| {
            let response = result.unwrap();
            assert_eq!(response.lp_weight, Uint128::new(400));
        })
        .manage_position(
            &creator,
            PositionAction::Expand {
                identifier: "u-creator_position".to_string(),
            },
            vec![coin(10_000, lp_denom_1.clone())],
            |result| {
                result.unwrap();
            },
        )
        .query_positions(
            Some(PositionsBy::Receiver(creator.to_string())),
            None,
            None,
            None,
            |result| {
                let response = result.unwrap();
                assert_eq!(response.positions.len(), 2);
                assert_eq!(
                    response.positions[0],
                    Position {
                        identifier: "p-1".to_string(),
                        lp_asset: coin(600, lp_denom_1.clone()),
                        unlocking_duration: 86_400,
                        open: false,
                        expiring_at: Some(time.borrow().plus_seconds(86_400).seconds()),
                        receiver: creator.clone(),
                    }
                );
                assert_eq!(
                    response.positions[1],
                    Position {
                        identifier: "u-creator_position".to_string(),
                        lp_asset: coin(10_400, lp_denom_1.clone()),
                        unlocking_duration: 86_400,
                        open: true,
                        expiring_at: None,
                        receiver: creator.clone(),
                    }
                );
            },
        )
        .query_lp_weight(&creator, &lp_denom_1, 11, |result| {
            let response = result.unwrap();
            assert_eq!(response.lp_weight, Uint128::new(10_400));
        })
        .add_one_epoch()
        .query_current_epoch(|result| {
            let epoch_response = result.unwrap();
            assert_eq!(epoch_response.epoch.id, 11);
        })
        .get_time(|result| {
            *time2.borrow_mut() = result;
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
        .query_positions(
            Some(PositionsBy::Receiver(creator.to_string())),
            None,
            None,
            None,
            |result| {
                let response = result.unwrap();
                assert_eq!(response.positions.len(), 2);
                assert_eq!(
                    response.positions[0],
                    Position {
                        identifier: "p-1".to_string(),
                        lp_asset: coin(600, lp_denom_1.clone()),
                        unlocking_duration: 86_400,
                        open: false,
                        expiring_at: Some(time.borrow().plus_seconds(86_400).seconds()),
                        receiver: creator.clone(),
                    }
                );
                assert_eq!(
                    response.positions[1],
                    Position {
                        identifier: "u-creator_position".to_string(),
                        lp_asset: coin(10_400, lp_denom_1.clone()),
                        unlocking_duration: 86_400,
                        open: false,
                        expiring_at: Some(time2.borrow().plus_seconds(86_400).seconds()),
                        receiver: creator.clone(),
                    }
                );
            },
        )
        .query_lp_weight(&creator, &lp_denom_1, 12, |result| {
            // as the user closed the position in full, shouldn't have any lp weight registered
            result.unwrap_err();
        });
}

#[test]
fn test_refill_position_uses_current_position_unlocking_period() {
    let lp_denom_1 =
        format!("factory/{MOCK_CONTRACT_ADDR_1}/pool.identifier.{LP_SYMBOL}").to_string();
    let lp_denom_2 = format!("factory/{MOCK_CONTRACT_ADDR_1}/2.{LP_SYMBOL}").to_string();

    let mut suite = TestingSuite::default_with_balances(vec![
        coin(1_000_000_000u128, "uom".to_string()),
        coin(1_000_000_000u128, "uusdy".to_string()),
        coin(1_000_000_000u128, "uosmo".to_string()),
        coin(1_000_000_000u128, lp_denom_1.clone()),
        coin(1_000_000_000u128, lp_denom_2.clone()),
    ]);

    let creator = suite.creator();

    suite.instantiate_default();

    for _ in 0..10 {
        suite.add_one_epoch();
    }

    let farm_manager_addr = suite.farm_manager_addr.clone();

    suite.query_current_epoch(|result| {
        let epoch_response = result.unwrap();
        assert_eq!(epoch_response.epoch.id, 10);
    });

    // open a position with the minimum unlocking period
    // try to refill the same position with the maximum unlocking period
    // the weight should remain unaffected, i.e. the refilling should use the
    // unlocking period of the current position
    suite
        .query_balance(lp_denom_1.to_string(), &farm_manager_addr, |balance| {
            assert_eq!(balance, Uint128::zero());
        })
        .manage_position(
            &creator,
            PositionAction::Create {
                identifier: Some("creator_position".to_string()),
                unlocking_duration: 86_400,
                receiver: None,
            },
            vec![coin(1_000, lp_denom_1.clone())],
            |result| {
                result.unwrap();
            },
        )
        .query_positions(
            Some(PositionsBy::Receiver(creator.to_string())),
            None,
            None,
            None,
            |result| {
                let response = result.unwrap();
                assert_eq!(response.positions.len(), 1);
                assert_eq!(
                    response.positions[0],
                    Position {
                        identifier: "u-creator_position".to_string(),
                        lp_asset: coin(1_000, lp_denom_1.clone()),
                        unlocking_duration: 86_400,
                        open: true,
                        expiring_at: None,
                        receiver: creator.clone(),
                    }
                );
            },
        )
        .query_lp_weight(&creator, &lp_denom_1, 11, |result| {
            let response = result.unwrap();
            assert_eq!(response.lp_weight, Uint128::new(1_000));
        })
        .manage_position(
            &creator,
            PositionAction::Expand {
                // this shouldn't inflate the lp weight
                identifier: "u-creator_position".to_string(),
            },
            vec![coin(1_000, lp_denom_1.clone())],
            |result| {
                result.unwrap();
            },
        )
        .query_positions(
            Some(PositionsBy::Receiver(creator.to_string())),
            None,
            None,
            None,
            |result| {
                let response = result.unwrap();
                assert_eq!(response.positions.len(), 1);
                assert_eq!(
                    response.positions[0],
                    Position {
                        identifier: "u-creator_position".to_string(),
                        lp_asset: coin(2_000, lp_denom_1.clone()),
                        unlocking_duration: 86_400,
                        open: true,
                        expiring_at: None,
                        receiver: creator.clone(),
                    }
                );
            },
        )
        .query_lp_weight(&creator, &lp_denom_1, 11, |result| {
            let response = result.unwrap();
            // the weight shouldn't be affected by the large unlocking period used in the refill
            assert_eq!(response.lp_weight, Uint128::new(2_000));
        });

    // let's do the reverse, using the maximum unlocking period
    // and then refilling with the minimum unlocking period
    suite
        .query_balance(lp_denom_2.to_string(), &farm_manager_addr, |balance| {
            assert_eq!(balance, Uint128::zero());
        })
        .manage_position(
            &creator,
            PositionAction::Create {
                identifier: Some("lp_denom_2_position".to_string()),
                unlocking_duration: 31_556_926,
                receiver: None,
            },
            vec![coin(1_000, lp_denom_2.clone())],
            |result| {
                result.unwrap();
            },
        )
        .query_positions(
            Some(PositionsBy::Receiver(creator.to_string())),
            None,
            None,
            None,
            |result| {
                let response = result.unwrap();
                assert_eq!(response.positions.len(), 2);
                assert_eq!(
                    response.positions[0],
                    Position {
                        identifier: "u-creator_position".to_string(),
                        lp_asset: coin(2_000, lp_denom_1.clone()),
                        unlocking_duration: 86_400,
                        open: true,
                        expiring_at: None,
                        receiver: creator.clone(),
                    }
                );
                assert_eq!(
                    response.positions[1],
                    Position {
                        identifier: "u-lp_denom_2_position".to_string(),
                        lp_asset: coin(1_000, lp_denom_2.clone()),
                        unlocking_duration: 31_556_926,
                        open: true,
                        expiring_at: None,
                        receiver: creator.clone(),
                    }
                );
            },
        )
        .query_lp_weight(&creator, &lp_denom_2, 11, |result| {
            let response = result.unwrap();
            // ~16x multiplier for the large unlocking period with an 1_000 lp position
            assert_eq!(response.lp_weight, Uint128::new(15_999));
        })
        .manage_position(
            &creator,
            PositionAction::Expand {
                // this shouldn't deflate the lp weight
                identifier: "u-lp_denom_2_position".to_string(),
            },
            vec![coin(1_000, lp_denom_2.clone())],
            |result| {
                result.unwrap();
            },
        )
        .query_positions(
            Some(PositionsBy::Receiver(creator.to_string())),
            None,
            None,
            None,
            |result| {
                let response = result.unwrap();
                assert_eq!(response.positions.len(), 2);
                assert_eq!(
                    response.positions[0],
                    Position {
                        identifier: "u-creator_position".to_string(),
                        lp_asset: coin(2_000, lp_denom_1.clone()),
                        unlocking_duration: 86_400,
                        open: true,
                        expiring_at: None,
                        receiver: creator.clone(),
                    }
                );
                assert_eq!(
                    response.positions[1],
                    Position {
                        identifier: "u-lp_denom_2_position".to_string(),
                        lp_asset: coin(2_000, lp_denom_2.clone()),
                        unlocking_duration: 31_556_926,
                        open: true,
                        expiring_at: None,
                        receiver: creator.clone(),
                    }
                );
            },
        )
        .query_lp_weight(&creator, &lp_denom_2, 11, |result| {
            let response = result.unwrap();
            // the weight shouldn't be affected by the low unlocking period used in the refill
            assert_eq!(response.lp_weight, Uint128::new(31_998));
        });
}

#[test]
fn position_fill_attack_is_not_possible() {
    let lp_denom = format!("factory/{MOCK_CONTRACT_ADDR_1}/{LP_SYMBOL}").to_string();
    let mut suite = TestingSuite::default_with_balances(vec![
        coin(1_000_000_000u128, "uom"),
        coin(1_000_000_000u128, "uusdy"),
        coin(1_000_000_000u128, "uosmo"),
        coin(1_000_000_000u128, lp_denom.clone()),
        coin(1_000_000_000u128, "invalid_lp"),
    ]);

    let creator = suite.creator();
    let victim_not_victim = suite.senders[1].clone();
    let attacker = suite.senders[2].clone();
    suite.instantiate_default();

    // Prepare the farm and victim's position
    suite
        .manage_farm(
            &creator,
            FarmAction::Fill {
                params: FarmParams {
                    lp_denom: lp_denom.clone(),
                    start_epoch: Some(12),
                    preliminary_end_epoch: Some(16),
                    curve: None,
                    farm_asset: Coin {
                        denom: "uusdy".to_string(),
                        amount: Uint128::new(8_000u128),
                    },
                    farm_identifier: None,
                },
            },
            vec![coin(8_000, "uusdy"), coin(1_000, "uom")],
            |result| {
                result.unwrap();
            },
        )
        .manage_position(
            &victim_not_victim,
            PositionAction::Create {
                identifier: Some("nice_position".to_string()),
                // 1 day unlocking duration
                unlocking_duration: 86_400,
                // No receiver means the user is the owner of the position receiver: None,
                receiver: None,
            },
            vec![coin(5_000, lp_denom.clone())],
            |result| {
                result.unwrap();
            },
        )
        // Check that the position is created
        .query_positions(
            Some(PositionsBy::Receiver(victim_not_victim.to_string())),
            Some(true),
            None,
            None,
            |result| {
                let positions = result.unwrap();
                assert_eq!(positions.positions.len(), 1);
                assert_eq!(
                    positions.positions[0],
                    Position {
                        identifier: "u-nice_position".to_string(),
                        lp_asset: Coin {
                            denom: lp_denom.clone(),
                            amount: Uint128::new(5_000),
                        },
                        unlocking_duration: 86_400,
                        open: true,
                        expiring_at: None,
                        receiver: victim_not_victim.clone(),
                    }
                );
            },
        );

    // The attacker tries to create 100 positions with minimal amounts
    // and sets the receiver to the victim
    for i in 0..100 {
        suite.manage_position(
            &attacker,
            PositionAction::Create {
                identifier: Some(format!("nasty{}", i)),
                // change to this line to see how sorting matters:
                // identifier: Some(format!("nice_position{}", i)),
                // Set unlocking duration to 1 year (maximum)
                unlocking_duration: 31_556_926u64,
                // Receiver is set to the user, making the user the owner of these positions
                receiver: Some(victim_not_victim.to_string()),
            },
            vec![coin(1, lp_denom.clone())],
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();
                match err {
                    ContractError::Unauthorized => {}
                    _ => panic!("Wrong error type, should return ContractError::Unauthorized"),
                }
            },
        );
    }

    // Query positions for the user again
    suite.query_positions(
        Some(PositionsBy::Receiver(victim_not_victim.to_string())),
        Some(true),
        None,
        None,
        |result| {
            let positions = result.unwrap();
            // the attacker couldn't create any positions for the user
            assert_eq!(positions.positions.len(), 1);
        },
    );

    suite.query_positions(
        Some(PositionsBy::Receiver(victim_not_victim.to_string())),
        Some(true),
        None,
        None,
        |result| {
            let positions = result.unwrap();
            // The original position must be visible
            assert!(positions
                .positions
                .iter()
                .any(|p| p.identifier == "u-nice_position"));
        },
    );
}

#[test]
fn positions_can_handled_by_pool_manager_for_the_user() {
    let lp_denom = format!("factory/{MOCK_CONTRACT_ADDR_1}/{LP_SYMBOL}").to_string();
    let mut suite = TestingSuite::default_with_balances(vec![
        coin(1_000_000_000u128, "uom"),
        coin(1_000_000_000u128, "uusdy"),
        coin(1_000_000_000u128, "uosmo"),
        coin(1_000_000_000u128, lp_denom.clone()),
        coin(1_000_000_000u128, "invalid_lp"),
    ]);

    let creator = suite.creator();
    let alice = suite.senders[1].clone();
    let attacker = suite.senders[2].clone();
    suite.instantiate_default();

    let pool_manager = suite.pool_manager_addr.clone();

    // send some lp tokens to the pool manager
    suite.send_tokens(
        &creator,
        &pool_manager,
        &[coin(1_000_000, lp_denom.clone())],
    );

    // the pool manager creates a position on behalf of alice
    suite
        .manage_position(
            &pool_manager,
            PositionAction::Create {
                identifier: Some("nice_position".to_string()),
                unlocking_duration: 86_400,
                receiver: Some(alice.to_string()),
            },
            vec![coin(5_000, lp_denom.clone())],
            |result| {
                result.unwrap();
            },
        )
        // Check that the position is created
        .query_positions(
            Some(PositionsBy::Receiver(alice.to_string())),
            Some(true),
            None,
            None,
            |result| {
                let positions = result.unwrap();
                assert_eq!(positions.positions.len(), 1);
                assert_eq!(
                    positions.positions[0],
                    Position {
                        identifier: "u-nice_position".to_string(),
                        lp_asset: Coin {
                            denom: lp_denom.clone(),
                            amount: Uint128::new(5_000),
                        },
                        unlocking_duration: 86_400,
                        open: true,
                        expiring_at: None,
                        receiver: alice.clone(),
                    }
                );
            },
        );

    // the pool manager refills that position
    suite
        .manage_position(
            &pool_manager,
            PositionAction::Expand {
                identifier: "u-nice_position".to_string(),
            },
            vec![coin(5_000, lp_denom.clone())],
            |result| {
                result.unwrap();
            },
        )
        // Check that the position was expanded
        .query_positions(
            Some(PositionsBy::Receiver(alice.to_string())),
            Some(true),
            None,
            None,
            |result| {
                let positions = result.unwrap();
                assert_eq!(positions.positions.len(), 1);
                assert_eq!(
                    positions.positions[0],
                    Position {
                        identifier: "u-nice_position".to_string(),
                        lp_asset: Coin {
                            denom: lp_denom.clone(),
                            amount: Uint128::new(10_000),
                        },
                        unlocking_duration: 86_400,
                        open: true,
                        expiring_at: None,
                        receiver: alice.clone(),
                    }
                );
            },
        );

    // an attacker tries to do the same
    suite
        .manage_position(
            &attacker,
            PositionAction::Create {
                identifier: Some("spam_position_for_alice".to_string()),
                unlocking_duration: 86_400,
                receiver: Some(alice.to_string()),
            },
            vec![coin(5_000, lp_denom.clone())],
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();
                match err {
                    ContractError::Unauthorized => {}
                    _ => panic!("Wrong error type, should return ContractError::Unauthorized"),
                }
            },
        )
        .manage_position(
            &attacker,
            PositionAction::Expand {
                identifier: "u-nice_position".to_string(),
            },
            vec![coin(5_000, lp_denom.clone())],
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();
                match err {
                    ContractError::Unauthorized => {}
                    _ => panic!("Wrong error type, should return ContractError::Unauthorized"),
                }
            },
        )
        // Check that alice has still the same position
        .query_positions(
            Some(PositionsBy::Receiver(alice.to_string())),
            Some(true),
            None,
            None,
            |result| {
                let positions = result.unwrap();
                assert_eq!(positions.positions.len(), 1);
                assert_eq!(
                    positions.positions[0],
                    Position {
                        identifier: "u-nice_position".to_string(),
                        lp_asset: Coin {
                            denom: lp_denom.clone(),
                            amount: Uint128::new(10_000),
                        },
                        unlocking_duration: 86_400,
                        open: true,
                        expiring_at: None,
                        receiver: alice.clone(),
                    }
                );
            },
        );
}

/// creates a MAX_ITEMS_LIMIT number of positions and farms. A user will claim for all the farms.
/// This shouldn't leave any unclaimed amount, as the user shouldn't be able to participate in more farms
/// than what the rewards calculation function iterates over.
#[test]
fn test_positions_limits() {
    let mut balances = vec![
        coin(1_000_000_000u128, "uom"),
        coin(1_000_000_000u128, "uusdy"),
        coin(1_000_000_000u128, "uosmo"),
    ];

    // prepare lp denoms
    for i in 1..MAX_FARMS_LIMIT * 2 {
        let lp_denom = format!("factory/{MOCK_CONTRACT_ADDR_1}/{i}.{LP_SYMBOL}");
        balances.push(coin(1_000_000_000u128, lp_denom.clone()));
    }

    let mut suite = TestingSuite::default_with_balances(balances);

    let creator = suite.creator();
    let alice = suite.senders[1].clone();
    suite.instantiate_default();

    // prepare farms, create more than the user could participate on
    for i in 1..MAX_POSITIONS_LIMIT * 2 {
        suite.manage_farm(
            &creator,
            FarmAction::Fill {
                params: FarmParams {
                    lp_denom: format!("factory/{MOCK_CONTRACT_ADDR_1}/{i}.{LP_SYMBOL}"),
                    start_epoch: Some(1),
                    preliminary_end_epoch: Some(2),
                    curve: None,
                    farm_asset: Coin {
                        denom: "uusdy".to_string(),
                        amount: Uint128::new(1_000u128),
                    },
                    farm_identifier: None,
                },
            },
            vec![coin(1_000, "uusdy"), coin(1_000, "uom")],
            |result| {
                result.unwrap();
            },
        );
    }

    // open positions
    for i in 1..=MAX_POSITIONS_LIMIT {
        suite.manage_position(
            &alice,
            PositionAction::Create {
                identifier: Some(format!("position{}", i)),
                unlocking_duration: 86_400,
                receiver: None,
            },
            vec![coin(
                1_000,
                format!("factory/{MOCK_CONTRACT_ADDR_1}/{i}.{LP_SYMBOL}"),
            )],
            |result| {
                result.unwrap();
            },
        );
    }

    suite.query_positions(
        Some(PositionsBy::Receiver(alice.to_string())),
        Some(true),
        None,
        Some(MAX_POSITIONS_LIMIT),
        |result| {
            let response = result.unwrap();
            assert_eq!(response.positions.len(), MAX_POSITIONS_LIMIT as usize);
        },
    );

    // alice can't create additional positions, as it hit the limit on open positions
    suite.manage_position(
        &alice,
        PositionAction::Create {
            identifier: Some("aditional_position".to_string()),
            unlocking_duration: 86_400,
            receiver: None,
        },
        vec![coin(
            1_000,
            format!("factory/{MOCK_CONTRACT_ADDR_1}/102.{LP_SYMBOL}"),
        )],
        |result| {
            let err = result.unwrap_err().downcast::<ContractError>().unwrap();
            match err {
                ContractError::MaxPositionsPerUserExceeded { .. } => {}
                _ => panic!(
                    "Wrong error type, should return ContractError::MaxPositionsPerUserExceeded"
                ),
            }
        },
    );

    // move an epoch and claim
    suite
        .add_one_epoch()
        .query_balance("uusdy".to_string(), &alice, |balance| {
            assert_eq!(balance, Uint128::new(1_000_000_000u128));
        })
        .claim(&alice, vec![], None, |result| {
            result.unwrap();
        })
        .query_balance("uusdy".to_string(), &alice, |balance| {
            // all the rewards were claimed, 1000 uusdy * 10
            assert_eq!(balance, Uint128::new(1_000_010_000u128));
        })
        .query_positions(
            Some(PositionsBy::Receiver(alice.to_string())),
            Some(true),
            None,
            Some(MAX_POSITIONS_LIMIT),
            |result| {
                let response = result.unwrap();
                assert_eq!(response.positions.len(), MAX_POSITIONS_LIMIT as usize);
            },
        );

    // now let's try closing positions
    for i in 1..=MAX_POSITIONS_LIMIT {
        suite.manage_position(
            &alice,
            PositionAction::Close {
                identifier: format!("u-position{}", i),
                lp_asset: None,
            },
            vec![],
            |result| {
                result.unwrap();
            },
        );
    }

    // no open positions are left, instead there are MAX_ITEMS_LIMIT closed positions
    suite
        .query_positions(
            Some(PositionsBy::Receiver(alice.to_string())),
            Some(true),
            None,
            Some(MAX_FARMS_LIMIT),
            |result| {
                let response = result.unwrap();
                assert!(response.positions.is_empty());
            },
        )
        .query_positions(
            Some(PositionsBy::Receiver(alice.to_string())),
            Some(false),
            None,
            Some(MAX_POSITIONS_LIMIT),
            |result| {
                let response = result.unwrap();
                assert_eq!(response.positions.len(), MAX_POSITIONS_LIMIT as usize);
            },
        );

    // try opening more positions
    for i in 1..=MAX_POSITIONS_LIMIT {
        suite.manage_position(
            &alice,
            PositionAction::Create {
                identifier: Some(format!("new_position{}", i)),
                unlocking_duration: 86_400,
                receiver: None,
            },
            vec![coin(
                1_000,
                format!("factory/{MOCK_CONTRACT_ADDR_1}/{i}.{LP_SYMBOL}"),
            )],
            |result| {
                result.unwrap();
            },
        );
    }

    // alice has MAX_ITEMS_LIMIT open positions and MAX_ITEMS_LIMIT closed positions
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
            Some(false),
            None,
            Some(MAX_POSITIONS_LIMIT),
            |result| {
                let response = result.unwrap();
                assert_eq!(response.positions.len(), MAX_POSITIONS_LIMIT as usize);
            },
        );

    // trying to close another position should err
    suite
        .manage_position(
            &alice,
            PositionAction::Close {
                identifier: "u-new_position1".to_string(),
                lp_asset: None,
            },
            vec![],
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();
                match err {
                    ContractError::MaxPositionsPerUserExceeded { .. } => {}
                    _ => panic!(
                        "Wrong error type, should return ContractError::MaxPositionsPerUserExceeded"
                    ),
                }
            },
        )
        // try closing partially
        .manage_position(
            &alice,
            PositionAction::Close {
                identifier: "u-new_position1".to_string(),
                lp_asset: Some(coin(
                    500,
                    format!("factory/{MOCK_CONTRACT_ADDR_1}/1.{LP_SYMBOL}"),
                )),
            },
            vec![],
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();
                match err {
                    ContractError::MaxPositionsPerUserExceeded { .. } => {}
                    _ => panic!(
                        "Wrong error type, should return ContractError::MaxPositionsPerUserExceeded"
                    ),
                }
            },
        );

    // let's move time so alice can withdraw a few positions and open some slots to close additional positions
    suite
        .add_one_epoch()
        .manage_position(
            &alice,
            PositionAction::Withdraw {
                identifier: "u-position1".to_string(),
                emergency_unlock: None,
            },
            vec![],
            |result| {
                result.unwrap();
            },
        )
        .query_positions(
            Some(PositionsBy::Receiver(alice.to_string())),
            Some(false),
            None,
            Some(MAX_POSITIONS_LIMIT),
            |result| {
                let response = result.unwrap();
                assert_eq!(response.positions.len(), (MAX_POSITIONS_LIMIT - 1) as usize);
            },
        )
        // try closing it a position partially
        .manage_position(
            &alice,
            PositionAction::Close {
                identifier: "u-new_position1".to_string(),
                lp_asset: Some(coin(
                    500,
                    format!("factory/{MOCK_CONTRACT_ADDR_1}/1.{LP_SYMBOL}"),
                )),
            },
            vec![],
            |result| {
                result.unwrap();
            },
        )
        .query_positions(
            Some(PositionsBy::Receiver(alice.to_string())),
            Some(false),
            None,
            Some(MAX_POSITIONS_LIMIT),
            |result| {
                let response = result.unwrap();
                assert_eq!(response.positions.len(), MAX_POSITIONS_LIMIT as usize);
            },
        );
}

#[test]
// fails until the issue is fixed
fn test_overwriting_position_is_not_possible() {
    let lp_denom = format!("factory/{MOCK_CONTRACT_ADDR_1}/{LP_SYMBOL}").to_string();
    let mut suite = TestingSuite::default_with_balances(vec![
        coin(1_000_000_000u128, "uom"),
        coin(1_000_000_000u128, "uusdy"),
        coin(1_000_000_000u128, "uosmo"),
        coin(1_000_000_000u128, lp_denom.clone()),
        coin(1_000_000_000u128, "invalid_lp"),
    ]);
    let creator = suite.creator();
    let victim = suite.senders[1].clone();
    let explicit_id = "10";
    let is_as_expected = |result: StdResult<PositionsResponse>| {
        let positions = result.unwrap();
        assert_eq!(positions.positions.len(), 1);
        assert_eq!(
            positions.positions[0],
            Position {
                identifier: format!("u-{explicit_id}"),
                lp_asset: Coin {
                    denom: format!("factory/{MOCK_CONTRACT_ADDR_1}/{LP_SYMBOL}").to_string(),
                    amount: Uint128::new(5_000),
                },
                unlocking_duration: 86400,
                open: true,
                expiring_at: None,
                receiver: victim.clone(),
            }
        );
    };

    suite.instantiate_default();

    // Prepare the farm and victim's position
    suite
        .manage_farm(
            &creator,
            FarmAction::Fill {
                params: FarmParams {
                    lp_denom: lp_denom.clone(),
                    start_epoch: Some(12),
                    preliminary_end_epoch: Some(16),
                    curve: None,
                    farm_asset: Coin {
                        denom: "uusdy".to_string(),
                        amount: Uint128::new(8_000u128),
                    },
                    farm_identifier: None,
                },
            },
            vec![coin(8_000, "uusdy"), coin(1_000, "uom")],
            |result| {
                result.unwrap();
            },
        )
        // Create a user position with the explicitly provided identifier
        .manage_position(
            &victim,
            PositionAction::Create {
                identifier: Some(explicit_id.to_string()),
                unlocking_duration: 86_400,
                receiver: None,
            },
            vec![coin(5_000, lp_denom.clone())],
            |result| {
                result.unwrap();
            },
        )
        // Check that the position is created
        .query_positions(
            Some(PositionsBy::Receiver(victim.to_string())),
            None,
            None,
            Some(MAX_FARMS_LIMIT),
            is_as_expected,
        );

    // Generate positions to catch up the counter
    for _ in 0..9 {
        suite.manage_position(
            &creator,
            PositionAction::Create {
                // No identifier means the contract will generate one
                identifier: None,
                unlocking_duration: 86_400,
                receiver: None,
            },
            vec![coin(5_000, lp_denom.clone())],
            |result| {
                result.unwrap();
            },
        );
    }

    // The original position must be visible
    suite.query_positions(
        Some(PositionsBy::Receiver(victim.to_string())),
        None,
        None,
        Some(MAX_FARMS_LIMIT),
        is_as_expected,
    );
}

#[test]
fn providing_custom_position_id_doesnt_increment_position_counter() {
    let lp_denom = format!("factory/{MOCK_CONTRACT_ADDR_1}/{LP_SYMBOL}").to_string();
    let mut suite = TestingSuite::default_with_balances(vec![
        coin(1_000_000_000u128, "uom"),
        coin(1_000_000_000u128, "uusdy"),
        coin(1_000_000_000u128, "uosmo"),
        coin(1_000_000_000u128, lp_denom.clone()),
        coin(1_000_000_000u128, "invalid_lp"),
    ]);
    let creator = suite.creator();

    suite.instantiate_default();

    suite
        .manage_position(
            &creator,
            PositionAction::Create {
                identifier: Some("custom_id_1".to_string()),
                unlocking_duration: 86_400,
                receiver: None,
            },
            vec![coin(10_000, lp_denom.clone())],
            |result| {
                result.unwrap();
            },
        )
        .manage_position(
            &creator,
            PositionAction::Create {
                identifier: Some("custom_id_2".to_string()),
                unlocking_duration: 86_400,
                receiver: None,
            },
            vec![coin(10_000, lp_denom.clone())],
            |result| {
                result.unwrap();
            },
        )
        .manage_position(
            &creator,
            PositionAction::Create {
                identifier: None,
                unlocking_duration: 86_400,
                receiver: None,
            },
            vec![coin(10_000, lp_denom.clone())],
            |result| {
                result.unwrap();
            },
        )
        .query_positions(
            Some(PositionsBy::Receiver(creator.to_string())),
            None,
            None,
            Some(MAX_FARMS_LIMIT),
            |result| {
                let response = result.unwrap();
                assert_eq!(response.positions.len(), 3);
                assert_eq!(response.positions[0].identifier, "p-1");
                assert_eq!(response.positions[1].identifier, "u-custom_id_1");
                assert_eq!(response.positions[2].identifier, "u-custom_id_2");
            },
        );
}
