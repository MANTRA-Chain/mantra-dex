extern crate core;

use cosmwasm_std::{coin, coins, Coin, Uint128};
use farm_manager::ContractError;
use mantra_dex_std::constants::LP_SYMBOL;
use mantra_dex_std::farm_manager::{
    Curve, Farm, FarmAction, FarmParams, LpWeightResponse, Position, PositionAction, PositionsBy,
    RewardsResponse,
};

use crate::common::suite::TestingSuite;
use crate::common::MOCK_CONTRACT_ADDR_1;

#[test]
fn claim_expired_farm_returns_nothing() {
    let lp_denom = format!("factory/{MOCK_CONTRACT_ADDR_1}/{LP_SYMBOL}").to_string();

    let mut suite = TestingSuite::default_with_balances(vec![
        coin(1_000_000_000u128, "uom"),
        coin(1_000_000_000u128, "uusdy"),
        coin(1_000_000_000u128, "uosmo"),
        coin(1_000_000_000u128, lp_denom.clone()),
        coin(1_000_000_000u128, "invalid_lp"),
    ]);

    let creator = suite.creator();
    let other = suite.senders[1].clone();

    suite.instantiate_default();

    for _ in 0..10 {
        suite.add_one_epoch();
    }

    let farm_manager = suite.farm_manager_addr.clone();

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
            &other,
            PositionAction::Create {
                identifier: Some("creator_position".to_string()),
                unlocking_duration: 86_400,
                receiver: None,
            },
            vec![coin(5_000, lp_denom.clone())],
            |result| {
                result.unwrap();
            },
        )
        .query_lp_weight(&farm_manager, &lp_denom, 11, |result| {
            let lp_weight = result.unwrap();
            assert_eq!(
                lp_weight,
                LpWeightResponse {
                    lp_weight: Uint128::new(5_000),
                    epoch_id: 11,
                }
            );
        })
        .query_positions(
            Some(PositionsBy::Receiver(other.to_string())),
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
                            amount: Uint128::new(5_000),
                        },
                        unlocking_duration: 86400,
                        open: true,
                        expiring_at: None,
                        receiver: other.clone(),
                    }
                );
            },
        );

    // create a couple of epochs to make the farm active

    suite
        .add_epochs(4)
        .query_current_epoch(|result| {
            let epoch_response = result.unwrap();
            assert_eq!(epoch_response.epoch.id, 14);
        })
        .query_balance("uusdy".to_string(), &other, |balance| {
            assert_eq!(balance, Uint128::new(1_000_000_000u128));
        })
        .claim(&other, vec![], None, |result| {
            result.unwrap();
        })
        .query_balance("uusdy".to_string(), &other, |balance| {
            assert_eq!(balance, Uint128::new(1_000_006_000u128));
        });

    // create a bunch of epochs to make the farm expire
    for _ in 0..15 {
        suite.add_one_epoch();
    }

    // there shouldn't be anything to claim as the farm has expired, even though it still has some funds
    suite
        .query_rewards(&creator, None, |result| {
            let rewards_response = result.unwrap();
            match rewards_response {
                RewardsResponse::RewardsResponse { total_rewards, .. } => {
                    assert!(total_rewards.is_empty());
                }
                _ => panic!("shouldn't return this but RewardsResponse"),
            }
        })
        .claim(&other, vec![], None, |result| {
            result.unwrap();
        })
        .query_balance("uusdy".to_string(), &other, |balance| {
            // the balance hasn't changed
            assert_eq!(balance, Uint128::new(1_000_008_000u128));
        })
        .query_farms(None, None, None, |result| {
            let farms_response = result.unwrap();
            assert_eq!(farms_response.farms.len(), 1usize);
            assert_eq!(farms_response.farms[0].claimed_amount, Uint128::new(8_000));

            let farm_debt =
                farms_response.farms[0].farm_asset.amount - farms_response.farms[0].claimed_amount;
            assert_eq!(farm_debt, Uint128::zero());
        });
}

#[test]
fn claiming_rewards_with_multiple_positions_arent_inflated() {
    let lp_denom = format!("factory/{MOCK_CONTRACT_ADDR_1}/{LP_SYMBOL}").to_string();
    let lp_denom_2 = format!("factory/{MOCK_CONTRACT_ADDR_1}/2.{LP_SYMBOL}").to_string();

    let mut suite = TestingSuite::default_with_balances(vec![
        coin(1_000_000_000u128, "uom"),
        coin(1_000_000_000u128, "uusdy"),
        coin(1_000_000_000u128, "uosmo"),
        coin(1_000_000_000u128, lp_denom.clone()),
        coin(1_000_000_000u128, lp_denom_2.clone()),
    ]);

    let creator = suite.creator();
    let other = suite.senders[1].clone();
    let another = suite.senders[2].clone();

    suite.instantiate_default();

    for _ in 0..10 {
        suite.add_one_epoch();
    }

    let farm_manager = suite.farm_manager_addr.clone();

    suite
        .manage_farm(
            &creator,
            FarmAction::Fill {
                params: FarmParams {
                    lp_denom: lp_denom.clone(),
                    start_epoch: Some(12),
                    preliminary_end_epoch: Some(15),
                    curve: None,
                    farm_asset: Coin {
                        denom: "uusdy".to_string(),
                        amount: Uint128::new(12_000u128),
                    },
                    farm_identifier: None,
                },
            },
            vec![coin(12_000u128, "uusdy"), coin(1_000, "uom")],
            |result| {
                result.unwrap();
            },
        )
        .manage_position(
            &other,
            PositionAction::Create {
                identifier: Some("other_position_1".to_string()),
                unlocking_duration: 86_400,
                receiver: None,
            },
            vec![coin(1_000, lp_denom.clone())],
            |result| {
                result.unwrap();
            },
        )
        .manage_position(
            &other,
            PositionAction::Create {
                identifier: Some("other_position_2".to_string()),
                unlocking_duration: 86_400,
                receiver: None,
            },
            vec![coin(1_000, lp_denom.clone())],
            |result| {
                result.unwrap();
            },
        )
        .manage_position(
            &other,
            PositionAction::Create {
                identifier: Some("other_position_3".to_string()),
                unlocking_duration: 86_400,
                receiver: None,
            },
            vec![coin(1_000, lp_denom.clone())],
            |result| {
                result.unwrap();
            },
        )
        .manage_position(
            &other,
            PositionAction::Create {
                identifier: Some("other_position_4".to_string()),
                unlocking_duration: 86_400,
                receiver: None,
            },
            vec![coin(1_000, lp_denom.clone())],
            |result| {
                result.unwrap();
            },
        )
        .manage_position(
            &other,
            PositionAction::Create {
                identifier: Some("other_position_5".to_string()),
                unlocking_duration: 86_400,
                receiver: None,
            },
            vec![coin(1_000, lp_denom.clone())],
            |result| {
                result.unwrap();
            },
        )
        .query_lp_weight(&other, &lp_denom, 11, |result| {
            let lp_weight = result.unwrap();
            assert_eq!(
                lp_weight,
                LpWeightResponse {
                    lp_weight: Uint128::new(5_000),
                    epoch_id: 11,
                }
            );
        })
        .query_positions(
            Some(PositionsBy::Receiver(other.to_string())),
            Some(true),
            None,
            None,
            |result| {
                let positions = result.unwrap();
                assert_eq!(positions.positions.len(), 5);
                assert_eq!(
                    positions.positions,
                    vec![
                        Position {
                            identifier: "u-other_position_1".to_string(),
                            lp_asset: Coin {
                                denom: format!("factory/{MOCK_CONTRACT_ADDR_1}/{LP_SYMBOL}")
                                    .to_string(),
                                amount: Uint128::new(1_000),
                            },
                            unlocking_duration: 86400,
                            open: true,
                            expiring_at: None,
                            receiver: other.clone(),
                        },
                        Position {
                            identifier: "u-other_position_2".to_string(),
                            lp_asset: Coin {
                                denom: format!("factory/{MOCK_CONTRACT_ADDR_1}/{LP_SYMBOL}")
                                    .to_string(),
                                amount: Uint128::new(1_000),
                            },
                            unlocking_duration: 86400,
                            open: true,
                            expiring_at: None,
                            receiver: other.clone(),
                        },
                        Position {
                            identifier: "u-other_position_3".to_string(),
                            lp_asset: Coin {
                                denom: format!("factory/{MOCK_CONTRACT_ADDR_1}/{LP_SYMBOL}")
                                    .to_string(),
                                amount: Uint128::new(1_000),
                            },
                            unlocking_duration: 86400,
                            open: true,
                            expiring_at: None,
                            receiver: other.clone(),
                        },
                        Position {
                            identifier: "u-other_position_4".to_string(),
                            lp_asset: Coin {
                                denom: format!("factory/{MOCK_CONTRACT_ADDR_1}/{LP_SYMBOL}")
                                    .to_string(),
                                amount: Uint128::new(1_000),
                            },
                            unlocking_duration: 86400,
                            open: true,
                            expiring_at: None,
                            receiver: other.clone(),
                        },
                        Position {
                            identifier: "u-other_position_5".to_string(),
                            lp_asset: Coin {
                                denom: format!("factory/{MOCK_CONTRACT_ADDR_1}/{LP_SYMBOL}")
                                    .to_string(),
                                amount: Uint128::new(1_000),
                            },
                            unlocking_duration: 86400,
                            open: true,
                            expiring_at: None,
                            receiver: other.clone(),
                        },
                    ]
                );
            },
        )
        .manage_position(
            &another,
            PositionAction::Create {
                identifier: Some("another_position".to_string()),
                unlocking_duration: 86_400,
                receiver: None,
            },
            vec![coin(5_000, lp_denom.clone())],
            |result| {
                result.unwrap();
            },
        )
        .query_lp_weight(&another, &lp_denom, 11, |result| {
            let lp_weight = result.unwrap();
            assert_eq!(
                lp_weight,
                LpWeightResponse {
                    lp_weight: Uint128::new(5_000),
                    epoch_id: 11,
                }
            );
        })
        .query_positions(
            Some(PositionsBy::Receiver(another.to_string())),
            Some(true),
            None,
            None,
            |result| {
                let positions = result.unwrap();
                assert_eq!(positions.positions.len(), 1);
                assert_eq!(
                    positions.positions,
                    vec![Position {
                        identifier: "u-another_position".to_string(),
                        lp_asset: Coin {
                            denom: format!("factory/{MOCK_CONTRACT_ADDR_1}/{LP_SYMBOL}")
                                .to_string(),
                            amount: Uint128::new(5_000),
                        },
                        unlocking_duration: 86400,
                        open: true,
                        expiring_at: None,
                        receiver: another.clone(),
                    },]
                );
            },
        )
        .query_lp_weight(&farm_manager, &lp_denom, 11, |result| {
            let lp_weight = result.unwrap();
            assert_eq!(
                lp_weight,
                LpWeightResponse {
                    // 5k of other (with 5 positions) and 5k of another (with 1 position)
                    lp_weight: Uint128::new(10_000),
                    epoch_id: 11,
                }
            );
        });

    // create a couple of epochs to make the farm active
    // claim rewards.
    // other has 50% of the weight, distributed along 5 positions
    // another has 50% of the weight, with only 1 position
    // both should get an equal amount of rewards when claiming
    suite
        .add_epochs(3)
        .query_current_epoch(|result| {
            let epoch_response = result.unwrap();
            assert_eq!(epoch_response.epoch.id, 13);
        })
        .query_balance("uusdy".to_string(), &other, |balance| {
            assert_eq!(balance, Uint128::new(1_000_000_000u128));
        })
        .claim(&other, vec![], None, |result| {
            result.unwrap();
        })
        .query_balance("uusdy".to_string(), &other, |balance| {
            assert_eq!(balance, Uint128::new(1_000_004_000u128));
        })
        .query_balance("uusdy".to_string(), &another, |balance| {
            assert_eq!(balance, Uint128::new(1_000_000_000u128));
        })
        .claim(&another, vec![], None, |result| {
            result.unwrap();
        })
        .query_balance("uusdy".to_string(), &another, |balance| {
            assert_eq!(balance, Uint128::new(1_000_004_000u128));
        });

    // let's do two more farms for a different LP denom
    suite
        .manage_farm(
            &creator,
            FarmAction::Fill {
                params: FarmParams {
                    lp_denom: lp_denom.clone(),
                    start_epoch: Some(15),
                    preliminary_end_epoch: Some(16),
                    curve: None,
                    farm_asset: Coin {
                        denom: "uom".to_string(),
                        amount: Uint128::new(10_000u128),
                    },
                    farm_identifier: None,
                },
            },
            vec![coin(11_000u128, "uom")],
            |result| {
                result.unwrap();
            },
        )
        .manage_farm(
            &creator,
            FarmAction::Fill {
                params: FarmParams {
                    lp_denom: lp_denom_2.clone(),
                    start_epoch: Some(14),
                    preliminary_end_epoch: Some(20),
                    curve: None,
                    farm_asset: Coin {
                        denom: "uusdy".to_string(),
                        amount: Uint128::new(8_000u128),
                    },
                    farm_identifier: None,
                },
            },
            vec![coin(8_000u128, "uusdy"), coin(1_000, "uom")],
            |result| {
                result.unwrap();
            },
        )
        .manage_farm(
            &creator,
            FarmAction::Fill {
                params: FarmParams {
                    lp_denom: lp_denom_2.clone(),
                    start_epoch: Some(16),
                    preliminary_end_epoch: Some(20),
                    curve: None,
                    farm_asset: Coin {
                        denom: "uosmo".to_string(),
                        amount: Uint128::new(10_000u128),
                    },
                    farm_identifier: None,
                },
            },
            vec![coin(10_000u128, "uosmo"), coin(1_000, "uom")],
            |result| {
                result.unwrap();
            },
        )
        .query_farms(None, None, None, |result| {
            let farms_response = result.unwrap();
            assert_eq!(farms_response.farms.len(), 4);
            assert_eq!(
                farms_response.farms,
                vec![
                    Farm {
                        identifier: "f-1".to_string(),
                        owner: creator.clone(),
                        lp_denom: lp_denom.clone(),
                        farm_asset: Coin {
                            denom: "uusdy".to_string(),
                            amount: Uint128::new(12_000u128),
                        },
                        claimed_amount: Uint128::new(8_000u128),
                        emission_rate: Uint128::new(4_000u128),
                        curve: Curve::Linear,
                        start_epoch: 12u64,
                        preliminary_end_epoch: 15u64,
                    },
                    Farm {
                        identifier: "f-2".to_string(),
                        owner: creator.clone(),
                        lp_denom: lp_denom.clone(),
                        farm_asset: Coin {
                            denom: "uom".to_string(),
                            amount: Uint128::new(10_000u128),
                        },
                        claimed_amount: Uint128::zero(),
                        emission_rate: Uint128::new(10_000),
                        curve: Curve::Linear,
                        start_epoch: 15u64,
                        preliminary_end_epoch: 16u64,
                    },
                    Farm {
                        identifier: "f-3".to_string(),
                        owner: creator.clone(),
                        lp_denom: lp_denom_2.clone(),
                        farm_asset: Coin {
                            denom: "uusdy".to_string(),
                            amount: Uint128::new(8_000u128),
                        },
                        claimed_amount: Uint128::zero(),
                        emission_rate: Uint128::new(1_333),
                        curve: Curve::Linear,
                        start_epoch: 14u64,
                        preliminary_end_epoch: 20u64,
                    },
                    Farm {
                        identifier: "f-4".to_string(),
                        owner: creator.clone(),
                        lp_denom: lp_denom_2.clone(),
                        farm_asset: Coin {
                            denom: "uosmo".to_string(),
                            amount: Uint128::new(10_000u128),
                        },
                        claimed_amount: Uint128::zero(),
                        emission_rate: Uint128::new(2_500),
                        curve: Curve::Linear,
                        start_epoch: 16u64,
                        preliminary_end_epoch: 20u64,
                    },
                ]
            );
        });

    // other will have 75% of the weight for lp_denom_2, distributed along 2 positions
    // another will have the remaining 25%
    suite
        .manage_position(
            &other,
            PositionAction::Create {
                identifier: Some("other_position_6".to_string()),
                unlocking_duration: 86_400,
                receiver: None,
            },
            vec![coin(5_000, lp_denom_2.clone())],
            |result| {
                result.unwrap();
            },
        )
        .manage_position(
            &other,
            PositionAction::Create {
                identifier: Some("other_position_7".to_string()),
                unlocking_duration: 86_400,
                receiver: None,
            },
            vec![coin(2_500, lp_denom_2.clone())],
            |result| {
                result.unwrap();
            },
        )
        .manage_position(
            &another,
            PositionAction::Create {
                identifier: Some("another_position_lp_2".to_string()),
                unlocking_duration: 86_400,
                receiver: None,
            },
            vec![coin(2_500, lp_denom_2.clone())],
            |result| {
                result.unwrap();
            },
        )
        .query_lp_weight(&other, &lp_denom_2, 14, |result| {
            let lp_weight = result.unwrap();
            assert_eq!(
                lp_weight,
                LpWeightResponse {
                    lp_weight: Uint128::new(7_500),
                    epoch_id: 14,
                }
            );
        })
        .query_lp_weight(&another, &lp_denom_2, 14, |result| {
            let lp_weight = result.unwrap();
            assert_eq!(
                lp_weight,
                LpWeightResponse {
                    lp_weight: Uint128::new(2_500),
                    epoch_id: 14,
                }
            );
        });

    suite.add_epochs(3).query_current_epoch(|result| {
        let epoch_response = result.unwrap();
        assert_eq!(epoch_response.epoch.id, 16);
    });

    // other claims
    suite
        .query_balance("uusdy".to_string(), &other, |balance| {
            assert_eq!(balance, Uint128::new(1_000_004_000u128));
        })
        .query_balance("uom".to_string(), &other, |balance| {
            assert_eq!(balance, Uint128::new(1_000_000_000u128));
        })
        .query_balance("uosmo".to_string(), &other, |balance| {
            assert_eq!(balance, Uint128::new(1_000_000_000u128));
        })
        .claim(&other, vec![], None, |result| {
            result.unwrap();
        })
        .query_balance("uusdy".to_string(), &other, |balance| {
            assert_eq!(
                balance,
                Uint128::new(1_000_004_000u128) + Uint128::new(4_000) + Uint128::new(997) // + Uint128::new(1_333)
                                                                                          //     .checked_multiply_ratio(75u128, 100u128)
                                                                                          //     .unwrap()
            );
        })
        .query_balance("uom".to_string(), &other, |balance| {
            assert_eq!(
                balance,
                Uint128::new(1_000_000_000u128) + Uint128::new(5_000)
            );
        })
        .query_balance("uosmo".to_string(), &other, |balance| {
            assert_eq!(
                balance,
                Uint128::new(1_000_000_000u128)
                    + Uint128::new(2_500)
                        .checked_multiply_ratio(75u128, 100u128)
                        .unwrap()
            );
        });

    // another claims the rest
    suite
        .query_balance("uusdy".to_string(), &another, |balance| {
            assert_eq!(balance, Uint128::new(1_000_004_000u128));
        })
        .query_balance("uom".to_string(), &another, |balance| {
            assert_eq!(balance, Uint128::new(1_000_000_000u128));
        })
        .query_balance("uosmo".to_string(), &another, |balance| {
            assert_eq!(balance, Uint128::new(1_000_000_000u128));
        })
        .claim(&another, vec![], None, |result| {
            result.unwrap();
        })
        .query_balance("uusdy".to_string(), &another, |balance| {
            assert_eq!(
                balance,
                Uint128::new(1_000_004_000u128) + Uint128::new(2_000) + Uint128::new(999) // + Uint128::new(1_333)
                                                                                          //     .checked_multiply_ratio(25u128, 100u128)
                                                                                          //     .unwrap()
            );
        })
        .query_balance("uom".to_string(), &another, |balance| {
            assert_eq!(
                balance,
                Uint128::new(1_000_000_000u128) + Uint128::new(5_000)
            );
        })
        .query_balance("uosmo".to_string(), &another, |balance| {
            assert_eq!(
                balance,
                Uint128::new(1_000_000_000u128)
                    + Uint128::new(2_500)
                        .checked_multiply_ratio(25u128, 100u128)
                        .unwrap()
            );
        });
}

#[test]
fn user_can_claim_expired_epochs() {
    let lp_denom = format!("factory/{MOCK_CONTRACT_ADDR_1}/{LP_SYMBOL}").to_string();

    let mut suite = TestingSuite::default_with_balances(vec![
        coin(2_000_000_000u128, "uom".to_string()),
        coin(2_000_000_000u128, "uusdy".to_string()),
        coin(2_000_000_000u128, "uosmo".to_string()),
        coin(2_000_000_000u128, lp_denom.clone()),
    ]);

    let other = suite.senders[1].clone();
    let alice = suite.senders[2].clone();

    suite.instantiate_default();

    suite
        .manage_farm(
            &other,
            FarmAction::Fill {
                params: FarmParams {
                    lp_denom: lp_denom.clone(),
                    start_epoch: Some(10),
                    preliminary_end_epoch: Some(20),
                    curve: None,
                    farm_asset: Coin {
                        denom: "uusdy".to_string(),
                        amount: Uint128::new(4_000u128),
                    },
                    farm_identifier: Some("farm".to_string()),
                },
            },
            vec![coin(4_000, "uusdy"), coin(1_000, "uom")],
            |result| {
                result.unwrap();
            },
        )
        .query_farms(None, None, None, |result| {
            let farms_response = result.unwrap();
            assert_eq!(farms_response.farms.len(), 1);
            assert_eq!(
                farms_response.farms[0],
                Farm {
                    identifier: "m-farm".to_string(),
                    owner: other.clone(),
                    lp_denom: lp_denom.clone(),
                    farm_asset: Coin {
                        denom: "uusdy".to_string(),
                        amount: Uint128::new(4_000u128),
                    },
                    claimed_amount: Uint128::zero(),
                    emission_rate: Uint128::new(400),
                    curve: Curve::Linear,
                    start_epoch: 10u64,
                    preliminary_end_epoch: 20u64,
                }
            );
        })
        .manage_position(
            &alice,
            PositionAction::Create {
                identifier: Some("position".to_string()),
                unlocking_duration: 86_400,
                receiver: None,
            },
            vec![coin(1_000, lp_denom.clone())],
            |result| {
                result.unwrap();
            },
        );

    // create enough epochs to make the farm expire
    // should expire at epoch 16 + config.farm_expiration_time, i.e. 16 + 30 = 46
    suite.add_epochs(100);

    suite
        .query_current_epoch(|result| {
            let epoch_response = result.unwrap();
            assert_eq!(epoch_response.epoch.id, 100);
        })
        // the farm expired, can't be refilled
        .manage_farm(
            &other,
            FarmAction::Fill {
                params: FarmParams {
                    lp_denom: lp_denom.clone(),
                    start_epoch: None,
                    preliminary_end_epoch: None,
                    curve: None,
                    farm_asset: Coin {
                        denom: "uusdy".to_string(),
                        amount: Uint128::new(8_000u128),
                    },
                    farm_identifier: Some("m-farm".to_string()),
                },
            },
            vec![coin(8_000u128, "uusdy")],
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();
                match err {
                    ContractError::FarmAlreadyExpired => {}
                    _ => {
                        panic!("Wrong error type, should return ContractError::FarmAlreadyExpired")
                    }
                }
            },
        );

    // let's claim the rewards

    suite
        .query_farms(None, None, None, |result| {
            let farms_response = result.unwrap();
            assert_eq!(farms_response.farms.len(), 1);
            assert_eq!(
                farms_response.farms[0],
                Farm {
                    identifier: "m-farm".to_string(),
                    owner: other.clone(),
                    lp_denom: lp_denom.clone(),
                    farm_asset: Coin {
                        denom: "uusdy".to_string(),
                        amount: Uint128::new(4_000u128),
                    },
                    claimed_amount: Uint128::zero(),
                    emission_rate: Uint128::new(400),
                    curve: Curve::Linear,
                    start_epoch: 10u64,
                    preliminary_end_epoch: 20u64,
                }
            );
        })
        .query_balance("uusdy".to_string(), &alice, |balance| {
            assert_eq!(balance, Uint128::new(2_000_000_000));
        })
        .claim(&alice, vec![], None, |result| {
            result.unwrap();
        })
        .query_balance("uusdy".to_string(), &alice, |balance| {
            assert_eq!(balance, Uint128::new(2_000_004_000));
        })
        .query_farms(None, None, None, |result| {
            let farms_response = result.unwrap();
            assert_eq!(farms_response.farms.len(), 1);
            assert_eq!(
                farms_response.farms[0],
                Farm {
                    identifier: "m-farm".to_string(),
                    owner: other.clone(),
                    lp_denom: lp_denom.clone(),
                    farm_asset: Coin {
                        denom: "uusdy".to_string(),
                        amount: Uint128::new(4_000u128),
                    },
                    claimed_amount: Uint128::new(4_000u128),
                    emission_rate: Uint128::new(400),
                    curve: Curve::Linear,
                    start_epoch: 10u64,
                    preliminary_end_epoch: 20u64,
                }
            );
        });
}

#[test]
fn farm_owners_get_penalty_fees() {
    let lp_denom_1 = format!("factory/{MOCK_CONTRACT_ADDR_1}/1.{LP_SYMBOL}").to_string();
    let lp_denom_2 = format!("factory/{MOCK_CONTRACT_ADDR_1}/2.{LP_SYMBOL}").to_string();
    let lp_denom_3 = format!("factory/{MOCK_CONTRACT_ADDR_1}/3.{LP_SYMBOL}").to_string();

    let mut suite = TestingSuite::default_with_balances(vec![
        coin(1_000_000_000u128, "uom".to_string()),
        coin(1_000_000_000u128, "uusdy".to_string()),
        coin(1_000_000_000u128, "uosmo".to_string()),
        coin(1_000_000_000u128, lp_denom_1.clone()),
        coin(1_000_000_000u128, lp_denom_2.clone()),
        coin(1_000_000_000u128, lp_denom_3.clone()),
    ]);

    let alice = suite.senders[0].clone();
    let bob = suite.senders[1].clone();
    let carol = suite.senders[2].clone();
    let dan = suite.senders[3].clone();

    suite.instantiate_default();

    let fee_collector = suite.fee_collector_addr.clone();

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
                        denom: "uusdy".to_string(),
                        amount: Uint128::new(4_000u128),
                    },
                    farm_identifier: None,
                },
            },
            vec![coin(4_000, "uusdy"), coin(1_000, "uom")],
            |result| {
                result.unwrap();
            },
        )
        .manage_farm(
            &bob,
            FarmAction::Fill {
                params: FarmParams {
                    lp_denom: lp_denom_1.clone(),
                    start_epoch: None,
                    preliminary_end_epoch: None,
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
        .manage_farm(
            &carol,
            FarmAction::Fill {
                params: FarmParams {
                    lp_denom: lp_denom_2.clone(),
                    start_epoch: None,
                    preliminary_end_epoch: None,
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
            &dan,
            PositionAction::Create {
                identifier: Some("dan_position_lp_1".to_string()),
                unlocking_duration: 86_400,
                receiver: None,
            },
            vec![coin(1_000, lp_denom_1.clone())],
            |result| {
                result.unwrap();
            },
        )
        .manage_position(
            &dan,
            PositionAction::Create {
                identifier: Some("dan_position_lp_2".to_string()),
                unlocking_duration: 86_400,
                receiver: None,
            },
            vec![coin(1_000, lp_denom_2.clone())],
            |result| {
                result.unwrap();
            },
        )
        .manage_position(
            &dan,
            PositionAction::Create {
                identifier: Some("dan_position_lp_3".to_string()),
                unlocking_duration: 86_400,
                receiver: None,
            },
            vec![coin(1_000, lp_denom_3.clone())],
            |result| {
                result.unwrap();
            },
        )
        .query_positions(
            Some(PositionsBy::Receiver(dan.to_string())),
            Some(true),
            None,
            None,
            |result| {
                let positions = result.unwrap();
                assert_eq!(positions.positions.len(), 3);
                assert_eq!(
                    positions.positions,
                    vec![
                        Position {
                            identifier: "u-dan_position_lp_1".to_string(),
                            lp_asset: Coin {
                                denom: format!("factory/{MOCK_CONTRACT_ADDR_1}/1.{LP_SYMBOL}")
                                    .to_string(),
                                amount: Uint128::new(1_000),
                            },
                            unlocking_duration: 86400,
                            open: true,
                            expiring_at: None,
                            receiver: dan.clone(),
                        },
                        Position {
                            identifier: "u-dan_position_lp_2".to_string(),
                            lp_asset: Coin {
                                denom: format!("factory/{MOCK_CONTRACT_ADDR_1}/2.{LP_SYMBOL}")
                                    .to_string(),
                                amount: Uint128::new(1_000),
                            },
                            unlocking_duration: 86400,
                            open: true,
                            expiring_at: None,
                            receiver: dan.clone(),
                        },
                        Position {
                            identifier: "u-dan_position_lp_3".to_string(),
                            lp_asset: Coin {
                                denom: format!("factory/{MOCK_CONTRACT_ADDR_1}/3.{LP_SYMBOL}")
                                    .to_string(),
                                amount: Uint128::new(1_000),
                            },
                            unlocking_duration: 86400,
                            open: true,
                            expiring_at: None,
                            receiver: dan.clone(),
                        }
                    ]
                );
            },
        );

    suite.add_one_epoch().add_one_epoch();

    suite.query_rewards(&dan, None, |result| {
        let rewards_response = result.unwrap();
        match rewards_response {
            RewardsResponse::RewardsResponse { total_rewards, .. } => {
                assert_eq!(total_rewards.len(), 1);
                assert_eq!(total_rewards[0], coin(2_854u128, "uusdy"));
            }
            _ => {
                panic!("Wrong response type, should return RewardsResponse::RewardsResponse")
            }
        }
    });

    // dan emergency withdraws the position for the lp_3, which doesn't have any farm.
    // in that case, the full penalty fee should go to the fee collector
    suite
        .query_balance(lp_denom_3.clone(), &fee_collector, |balance| {
            assert_eq!(balance, Uint128::zero());
        })
        .manage_position(
            &dan,
            PositionAction::Withdraw {
                identifier: "u-dan_position_lp_3".to_string(),
                emergency_unlock: Some(true),
            },
            vec![],
            |result| {
                result.unwrap();
            },
        )
        .query_balance(lp_denom_3.clone(), &fee_collector, |balance| {
            assert_eq!(balance, Uint128::new(100u128));
        });

    // dan emergency withdraws the position for the lp_2, which has a single farm.
    // in that case, half of the penalty fee should go to the fee collector and the other half
    // to the only farm owner (carol)
    suite
        .query_balance(lp_denom_2.clone(), &fee_collector, |balance| {
            assert_eq!(balance, Uint128::zero());
        })
        .query_balance(lp_denom_2.clone(), &carol, |balance| {
            assert_eq!(balance, Uint128::new(1_000_000_000u128));
        })
        .manage_position(
            &dan,
            PositionAction::Withdraw {
                identifier: "u-dan_position_lp_2".to_string(),
                emergency_unlock: Some(true),
            },
            vec![],
            |result| {
                result.unwrap();
            },
        )
        .query_balance(lp_denom_2.clone(), &fee_collector, |balance| {
            assert_eq!(balance, Uint128::new(50u128));
        })
        .query_balance(lp_denom_2.clone(), &carol, |balance| {
            assert_eq!(balance, Uint128::new(1_000_000_000u128 + 50u128));
        });

    // dan emergency withdraws the position for the lp_1, which has two farms.
    // in that case, half of the penalty fee should go to the fee collector and the other half
    // to the two farm owners (alice and bob)
    suite
        .query_balance(lp_denom_1.clone(), &fee_collector, |balance| {
            assert_eq!(balance, Uint128::zero());
        })
        .query_balance(lp_denom_1.clone(), &alice, |balance| {
            assert_eq!(balance, Uint128::new(1_000_000_000u128));
        })
        .query_balance(lp_denom_1.clone(), &bob, |balance| {
            assert_eq!(balance, Uint128::new(1_000_000_000u128));
        })
        .manage_position(
            &dan,
            PositionAction::Withdraw {
                identifier: "u-dan_position_lp_1".to_string(),
                emergency_unlock: Some(true),
            },
            vec![],
            |result| {
                result.unwrap();
            },
        )
        .query_balance(lp_denom_1.clone(), &fee_collector, |balance| {
            assert_eq!(balance, Uint128::new(50u128));
        })
        .query_balance(lp_denom_1.clone(), &alice, |balance| {
            assert_eq!(balance, Uint128::new(1_000_000_000u128 + 25u128));
        })
        .query_balance(lp_denom_1.clone(), &bob, |balance| {
            assert_eq!(balance, Uint128::new(1_000_000_000u128 + 25u128));
        });

    // now let's create a new position with such a small amount that the penalty fee could go
    // (rounded down) to zero

    suite
        .manage_position(
            &dan,
            PositionAction::Create {
                identifier: Some("dan_position_lp_1".to_string()),
                unlocking_duration: 86_400,
                receiver: None,
            },
            vec![coin(20, lp_denom_1.clone())],
            |result| {
                result.unwrap();
            },
        )
        .manage_position(
            &dan,
            PositionAction::Create {
                identifier: Some("dan_position_lp_2".to_string()),
                unlocking_duration: 86_400,
                receiver: None,
            },
            vec![coin(10, lp_denom_2.clone())],
            |result| {
                result.unwrap();
            },
        )
        .manage_position(
            &dan,
            PositionAction::Create {
                identifier: Some("dan_position_lp_3".to_string()),
                unlocking_duration: 86_400,
                receiver: None,
            },
            vec![coin(5, lp_denom_3.clone())],
            |result| {
                result.unwrap();
            },
        )
        .query_positions(
            Some(PositionsBy::Receiver(dan.to_string())),
            Some(true),
            None,
            None,
            |result| {
                let positions = result.unwrap();
                assert_eq!(positions.positions.len(), 3);
                assert_eq!(
                    positions.positions,
                    vec![
                        Position {
                            identifier: "u-dan_position_lp_1".to_string(),
                            lp_asset: Coin {
                                denom: format!("factory/{MOCK_CONTRACT_ADDR_1}/1.{LP_SYMBOL}")
                                    .to_string(),
                                amount: Uint128::new(20),
                            },
                            unlocking_duration: 86400,
                            open: true,
                            expiring_at: None,
                            receiver: dan.clone(),
                        },
                        Position {
                            identifier: "u-dan_position_lp_2".to_string(),
                            lp_asset: Coin {
                                denom: format!("factory/{MOCK_CONTRACT_ADDR_1}/2.{LP_SYMBOL}")
                                    .to_string(),
                                amount: Uint128::new(10),
                            },
                            unlocking_duration: 86400,
                            open: true,
                            expiring_at: None,
                            receiver: dan.clone(),
                        },
                        Position {
                            identifier: "u-dan_position_lp_3".to_string(),
                            lp_asset: Coin {
                                denom: format!("factory/{MOCK_CONTRACT_ADDR_1}/3.{LP_SYMBOL}")
                                    .to_string(),
                                amount: Uint128::new(5),
                            },
                            unlocking_duration: 86400,
                            open: true,
                            expiring_at: None,
                            receiver: dan.clone(),
                        }
                    ]
                );
            },
        );

    // dan emergency withdraws the position for the lp_3, which doesn't have any farm.
    // in that case, the full penalty fee should go to the fee collector, but it won't since the penalty
    // will go to zero
    suite
        .query_balance(lp_denom_3.clone(), &fee_collector, |balance| {
            assert_eq!(balance, Uint128::new(100u128));
        })
        .query_balance(lp_denom_3.clone(), &dan, |balance| {
            assert_eq!(balance, Uint128::new(999_999_895u128));
        })
        .manage_position(
            &dan,
            PositionAction::Withdraw {
                identifier: "u-dan_position_lp_3".to_string(),
                emergency_unlock: Some(true),
            },
            vec![],
            |result| {
                result.unwrap();
            },
        )
        .query_balance(lp_denom_3.clone(), &fee_collector, |balance| {
            assert_eq!(balance, Uint128::new(100u128));
        })
        .query_balance(lp_denom_3.clone(), &dan, |balance| {
            assert_eq!(balance, Uint128::new(999_999_900u128));
        });

    // dan emergency withdraws the position for the lp_2, which has a single farm.
    // in that case, the full amount of the penalty will go to the fee collector because if split in
    // half it would approximate to zero
    suite
        .query_balance(lp_denom_2.clone(), &fee_collector, |balance| {
            assert_eq!(balance, Uint128::new(50u128));
        })
        .query_balance(lp_denom_2.clone(), &carol, |balance| {
            assert_eq!(balance, Uint128::new(1_000_000_000u128 + 50u128));
        })
        .manage_position(
            &dan,
            PositionAction::Withdraw {
                identifier: "u-dan_position_lp_2".to_string(),
                emergency_unlock: Some(true),
            },
            vec![],
            |result| {
                result.unwrap();
            },
        )
        .query_balance(lp_denom_2.clone(), &fee_collector, |balance| {
            assert_eq!(balance, Uint128::new(51u128));
        })
        .query_balance(lp_denom_2.clone(), &carol, |balance| {
            assert_eq!(balance, Uint128::new(1_000_000_000u128 + 50u128));
        });

    // dan emergency withdraws the position for the lp_1, which has two farms.
    // in that case, the whole penalty will go to the fee collector because the second half going to
    // the owners will approximate to zero
    suite
        .query_balance(lp_denom_1.clone(), &fee_collector, |balance| {
            assert_eq!(balance, Uint128::new(50u128));
        })
        .query_balance(lp_denom_1.clone(), &alice, |balance| {
            assert_eq!(balance, Uint128::new(1_000_000_000u128 + 25u128));
        })
        .query_balance(lp_denom_1.clone(), &bob, |balance| {
            assert_eq!(balance, Uint128::new(1_000_000_000u128 + 25u128));
        })
        .manage_position(
            &dan,
            PositionAction::Withdraw {
                identifier: "u-dan_position_lp_1".to_string(),
                emergency_unlock: Some(true),
            },
            vec![],
            |result| {
                result.unwrap();
            },
        )
        .query_balance(lp_denom_1.clone(), &fee_collector, |balance| {
            assert_eq!(balance, Uint128::new(52u128));
        })
        .query_balance(lp_denom_1.clone(), &alice, |balance| {
            assert_eq!(balance, Uint128::new(1_000_000_000u128 + 25u128));
        })
        .query_balance(lp_denom_1.clone(), &bob, |balance| {
            assert_eq!(balance, Uint128::new(1_000_000_000u128 + 25u128));
        });
}

#[test]
fn test_claim_rewards_divide_by_zero_mitigated() {
    let lp_denom_1 = format!("factory/{MOCK_CONTRACT_ADDR_1}/1.{LP_SYMBOL}").to_string();
    let lp_denom_2 = format!("factory/{MOCK_CONTRACT_ADDR_1}/2.{LP_SYMBOL}").to_string();

    let mut suite = TestingSuite::default_with_balances(vec![
        coin(1_000_000_000u128, "uom".to_string()),
        coin(1_000_000_000u128, "uusdy".to_string()),
        coin(1_000_000_000u128, "uosmo".to_string()),
        coin(1_000_000_000_000, lp_denom_1.clone()),
        coin(1_000_000_000_000, lp_denom_2.clone()),
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
                        denom: "uusdy".to_string(),
                        amount: Uint128::new(8_888u128),
                    },
                    farm_identifier: None,
                },
            },
            vec![coin(8_888u128, "uusdy"), coin(1_000, "uom")],
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
                    preliminary_end_epoch: Some(20),
                    curve: None,
                    farm_asset: Coin {
                        denom: "uusdy".to_string(),
                        amount: Uint128::new(666_666u128),
                    },
                    farm_identifier: None,
                },
            },
            vec![coin(666_666u128, "uusdy"), coin(1_000, "uom")],
            |result| {
                result.unwrap();
            },
        );

    // creator and other fill two positions - one in a different lp_denom farm.
    suite.manage_position(
        &bob,
        PositionAction::Create {
            identifier: Some("creator_position".to_string()),
            unlocking_duration: 86_400,
            receiver: None,
        },
        vec![coin(1_000, lp_denom_1.clone())],
        |result| {
            result.unwrap();
        },
    );

    suite.manage_position(
        &bob,
        PositionAction::Create {
            identifier: Some("creator_another_position".to_string()),
            unlocking_duration: 86_400,
            receiver: None,
        },
        vec![coin(1_000, lp_denom_2.clone())],
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
            unlocking_duration: 86_400,
            receiver: None,
        },
        vec![coin(2_000, lp_denom_1.clone())],
        |result| {
            result.unwrap();
        },
    );
    suite.add_one_epoch().query_current_epoch(|result| {
        let epoch_response = result.unwrap();
        assert_eq!(epoch_response.epoch.id, 8);
    });

    // this would have failed if the divide by zero was not mitigated
    suite.claim(&bob, vec![], None, |result| {
        result.unwrap();
    });
}

#[test]
fn test_claim_until_epoch() {
    let lp_denom_1 = format!("factory/{MOCK_CONTRACT_ADDR_1}/1.{LP_SYMBOL}").to_string();
    let lp_denom_2 = format!("factory/{MOCK_CONTRACT_ADDR_1}/2.{LP_SYMBOL}").to_string();

    let mut suite = TestingSuite::default_with_balances(vec![
        coin(1_000_000_000u128, "uom".to_string()),
        coin(1_000_000_000u128, "uusdy".to_string()),
        coin(1_000_000_000u128, "uosmo".to_string()),
        coin(1_000_000_000_000, lp_denom_1.clone()),
        coin(1_000_000_000_000, lp_denom_2.clone()),
    ]);

    let alice = suite.creator();
    let bob = suite.senders[1].clone();

    suite.instantiate_default();

    suite.add_one_epoch().add_one_epoch();
    // epoch 2

    // create overlapping farms
    suite
        .manage_farm(
            &alice,
            FarmAction::Fill {
                params: FarmParams {
                    lp_denom: lp_denom_1.clone(),
                    start_epoch: Some(4),
                    preliminary_end_epoch: Some(14),
                    curve: None,
                    farm_asset: Coin {
                        denom: "uusdy".to_string(),
                        amount: Uint128::new(10_000u128),
                    },
                    farm_identifier: None,
                },
            },
            vec![coin(10_000u128, "uusdy"), coin(1_000, "uom")],
            |result| {
                result.unwrap();
            },
        )
        .manage_farm(
            &alice,
            FarmAction::Fill {
                params: FarmParams {
                    lp_denom: lp_denom_2.clone(),
                    start_epoch: Some(10),
                    preliminary_end_epoch: Some(20),
                    curve: None,
                    farm_asset: Coin {
                        denom: "uusdy".to_string(),
                        amount: Uint128::new(10_000u128),
                    },
                    farm_identifier: None,
                },
            },
            vec![coin(10_000u128, "uusdy"), coin(1_000, "uom")],
            |result| {
                result.unwrap();
            },
        );

    // bob fills two positions in epoch 2
    suite.manage_position(
        &bob,
        PositionAction::Create {
            identifier: Some("creator_position".to_string()),
            unlocking_duration: 86_400,
            receiver: None,
        },
        vec![coin(1_000, lp_denom_1.clone())],
        |result| {
            result.unwrap();
        },
    );

    suite.manage_position(
        &bob,
        PositionAction::Create {
            identifier: Some("creator_another_position".to_string()),
            unlocking_duration: 86_400,
            receiver: None,
        },
        vec![coin(1_000, lp_denom_2.clone())],
        |result| {
            result.unwrap();
        },
    );

    suite.add_one_epoch().query_current_epoch(|result| {
        let epoch_response = result.unwrap();
        assert_eq!(epoch_response.epoch.id, 3);
    });

    //epoch 3

    // try claiming in the future
    suite.claim(&bob, vec![], Some(4), |result| {
        let err = result.unwrap_err().downcast::<ContractError>().unwrap();
        match err {
            ContractError::InvalidUntilEpoch { until_epoch } => {
                assert_eq!(until_epoch, 4)
            }
            _ => {
                panic!("Wrong error type, should return ContractError::InvalidUntilEpoch")
            }
        }
    });

    // try claiming in the past, in an epoch before the positions were created
    suite
        .query_rewards(&bob, Some(0), |result| {
            let rewards_response: RewardsResponse = result.unwrap();

            match rewards_response {
                RewardsResponse::RewardsResponse { total_rewards, .. } => {
                    assert!(total_rewards.is_empty());
                }
                _ => {
                    panic!("Wrong response type, should return RewardsResponse::RewardsResponse")
                }
            }
        })
        .query_balance("uusdy".to_string(), &bob, |balance| {
            assert_eq!(balance, Uint128::new(1_000_000_000));
        });

    suite
        .claim(&bob, vec![], Some(0), |result| {
            result.unwrap();
        })
        .query_balance("uusdy".to_string(), &bob, |balance| {
            assert_eq!(balance, Uint128::new(1_000_000_000));
        });

    // try claiming before the farms start
    suite
        .query_rewards(&bob, None, |result| {
            let rewards_response: RewardsResponse = result.unwrap();

            match rewards_response {
                RewardsResponse::RewardsResponse { total_rewards, .. } => {
                    assert!(total_rewards.is_empty());
                }
                _ => {
                    panic!("Wrong response type, should return RewardsResponse::RewardsResponse")
                }
            }
        })
        .query_balance("uusdy".to_string(), &bob, |balance| {
            assert_eq!(balance, Uint128::new(1_000_000_000));
        });

    suite
        .claim(&bob, vec![], None, |result| {
            result.unwrap();
        })
        .query_balance("uusdy".to_string(), &bob, |balance| {
            assert_eq!(balance, Uint128::new(1_000_000_000));
        });

    suite.add_epochs(5).query_current_epoch(|result| {
        let epoch_response = result.unwrap();
        assert_eq!(epoch_response.epoch.id, 8);
    });

    // epoch 8

    suite.query_rewards(&bob, None, |result| {
        let rewards_response = result.unwrap();
        match rewards_response {
            RewardsResponse::RewardsResponse { total_rewards, .. } => {
                assert_eq!(total_rewards, coins(5_000, "uusdy"));
            }
            _ => {
                panic!("Wrong response type, should return RewardsResponse::RewardsResponse")
            }
        }
    });
    let _farm_manager = suite.farm_manager_addr.clone();

    // claim only until epoch 5, so two epochs should be claimed
    suite
        .query_balance("uusdy".to_string(), &bob, |balance| {
            assert_eq!(balance, Uint128::new(1_000_000_000));
        })
        .claim(&bob, vec![], Some(5), |result| {
            result.unwrap();
        })
        .query_balance("uusdy".to_string(), &bob, |balance| {
            assert_eq!(balance, Uint128::new(1_000_000_000 + 1000 * 2));
        })
        // query rewards for epoch 5, which was claimed
        .query_rewards(&bob, Some(5), |result| {
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
        .query_lp_weight(&bob, &lp_denom_1, 2, |result| {
            result.unwrap_err();
        })
        .query_lp_weight(&bob, &lp_denom_1, 3, |result| {
            result.unwrap_err();
        });

    // try claiming in the past again
    suite
        .query_balance("uusdy".to_string(), &bob, |balance| {
            assert_eq!(balance, Uint128::new(1_000_000_000 + 1000 * 2));
        })
        .claim(&bob, vec![], Some(4), |result| {
            let err = result.unwrap_err().downcast::<ContractError>().unwrap();
            match err {
                ContractError::InvalidUntilEpoch { until_epoch } => {
                    assert_eq!(until_epoch, 4)
                }
                _ => {
                    panic!("Wrong error type, should return ContractError::InvalidUntilEpoch")
                }
            }
        })
        .claim(&bob, vec![], Some(5), |result| {
            result.unwrap();
        })
        // nothing was transferred when trying to claim until epoch 5 again
        .query_balance("uusdy".to_string(), &bob, |balance| {
            assert_eq!(balance, Uint128::new(1_000_000_000 + 1000 * 2));
        });

    // try claiming in the future
    suite.claim(&bob, vec![], Some(9), |result| {
        let err = result.unwrap_err().downcast::<ContractError>().unwrap();
        match err {
            ContractError::InvalidUntilEpoch { until_epoch } => {
                assert_eq!(until_epoch, 9)
            }
            _ => {
                panic!("Wrong error type, should return ContractError::InvalidUntilEpoch")
            }
        }
    });

    suite.add_epochs(6).query_current_epoch(|result| {
        let epoch_response = result.unwrap();
        assert_eq!(epoch_response.epoch.id, 14);
    });

    suite
        .query_rewards(&bob, None, |result| {
            let rewards_response = result.unwrap();
            match rewards_response {
                RewardsResponse::RewardsResponse { total_rewards, .. } => {
                    assert_eq!(total_rewards, coins(8_000 + 5_000, "uusdy"));
                }
                _ => {
                    panic!("Wrong response type, should return RewardsResponse::RewardsResponse")
                }
            }
        })
        .claim(&bob, vec![], None, |result| {
            result.unwrap();
        })
        .query_balance("uusdy".to_string(), &bob, |balance| {
            assert_eq!(
                balance,
                Uint128::new(1_000_000_000 + (1000 * 2) + 8_000 + 5_000)
            );
        });
}

#[test]
fn test_claim_until_epoch_closing_positions() {
    let lp_denom_1 = format!("factory/{MOCK_CONTRACT_ADDR_1}/1.{LP_SYMBOL}").to_string();
    let lp_denom_2 = format!("factory/{MOCK_CONTRACT_ADDR_1}/2.{LP_SYMBOL}").to_string();

    let mut suite = TestingSuite::default_with_balances(vec![
        coin(1_000_000_000u128, "uom".to_string()),
        coin(1_000_000_000u128, "uusdy".to_string()),
        coin(1_000_000_000u128, "uosmo".to_string()),
        coin(1_000_000_000_000, lp_denom_1.clone()),
        coin(1_000_000_000_000, lp_denom_2.clone()),
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
                    preliminary_end_epoch: Some(11),
                    curve: None,
                    farm_asset: Coin {
                        denom: "uusdy".to_string(),
                        amount: Uint128::new(10_000u128),
                    },
                    farm_identifier: None,
                },
            },
            vec![coin(10_000u128, "uusdy"), coin(1_000, "uom")],
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
                    preliminary_end_epoch: Some(11),
                    curve: None,
                    farm_asset: Coin {
                        denom: "uusdy".to_string(),
                        amount: Uint128::new(10_000u128),
                    },
                    farm_identifier: None,
                },
            },
            vec![coin(10_000u128, "uusdy"), coin(1_000, "uom")],
            |result| {
                result.unwrap();
            },
        );

    // bob fills three positions
    suite.manage_position(
        &bob,
        PositionAction::Create {
            identifier: Some("lp_denom_1".to_string()),
            unlocking_duration: 86_400,
            receiver: None,
        },
        vec![coin(1_000, lp_denom_1.clone())],
        |result| {
            result.unwrap();
        },
    );

    suite
        .manage_position(
            &bob,
            PositionAction::Create {
                identifier: Some("lp_denom_2_1".to_string()),
                unlocking_duration: 86_400,
                receiver: None,
            },
            vec![coin(500, lp_denom_2.clone())],
            |result| {
                result.unwrap();
            },
        )
        .manage_position(
            &bob,
            PositionAction::Create {
                identifier: Some("lp_denom_2_2".to_string()),
                unlocking_duration: 86_400,
                receiver: None,
            },
            vec![coin(500, lp_denom_2.clone())],
            |result| {
                result.unwrap();
            },
        )
        .manage_position(
            &alice,
            PositionAction::Create {
                identifier: Some("alice_lp_denom_2".to_string()),
                unlocking_duration: 86_400,
                receiver: None,
            },
            vec![coin(1_000, lp_denom_2.clone())],
            |result| {
                result.unwrap();
            },
        );

    suite.add_epochs(2).query_current_epoch(|result| {
        let epoch_response = result.unwrap();
        assert_eq!(epoch_response.epoch.id, 2);
    });

    suite.manage_position(
        &bob,
        PositionAction::Close {
            identifier: "u-lp_denom_1_1".to_string(),
            lp_asset: None,
        },
        vec![],
        |result| {
            let err = result.unwrap_err().downcast::<ContractError>().unwrap();
            match err {
                ContractError::PendingRewards => {}
                _ => panic!("Wrong error type, should return ContractError::PendingRewards"),
            }
        },
    );

    // claim some rewards, but not all
    suite
        .query_balance("uusdy".to_string(), &bob, |balance| {
            assert_eq!(balance, Uint128::new(1_000_000_000));
        })
        .claim(&bob, vec![], Some(1), |result| {
            result.unwrap();
        })
        .query_balance("uusdy".to_string(), &bob, |balance| {
            assert_eq!(balance, Uint128::new(1_000_000_000 + 1_000 + 500));
        });

    // still can't close the position as the current epoch is 2, but claimed until epoch 1
    suite.manage_position(
        &bob,
        PositionAction::Close {
            identifier: "u-lp_denom_1_1".to_string(),
            lp_asset: None,
        },
        vec![],
        |result| {
            let err = result.unwrap_err().downcast::<ContractError>().unwrap();
            match err {
                ContractError::PendingRewards => {}
                _ => panic!("Wrong error type, should return ContractError::PendingRewards"),
            }
        },
    );

    suite
        .query_balance("uusdy".to_string(), &bob, |balance| {
            assert_eq!(balance, Uint128::new(1_000_000_000 + 1_000 + 500));
        })
        .claim(&bob, vec![], None, |result| {
            result.unwrap();
        })
        .query_balance("uusdy".to_string(), &bob, |balance| {
            assert_eq!(balance, Uint128::new(1_000_001_500 + 1_000 + 500));
        });

    suite.manage_position(
        &bob,
        PositionAction::Close {
            identifier: "u-lp_denom_2_1".to_string(),
            lp_asset: None,
        },
        vec![],
        |result| {
            result.unwrap();
        },
    );

    let farm_manager = suite.farm_manager_addr.clone();

    suite
        .query_lp_weight(&farm_manager, &lp_denom_2, 3, |result| {
            let lp_weight = result.unwrap();
            assert_eq!(
                lp_weight,
                LpWeightResponse {
                    lp_weight: Uint128::new(1500),
                    epoch_id: 3,
                }
            );
        })
        .query_lp_weight(&bob, &lp_denom_2, 3, |result| {
            let lp_weight = result.unwrap();
            assert_eq!(
                lp_weight,
                LpWeightResponse {
                    lp_weight: Uint128::new(500),
                    epoch_id: 3,
                }
            );
        });

    suite.add_epochs(2).query_current_epoch(|result| {
        let epoch_response = result.unwrap();
        assert_eq!(epoch_response.epoch.id, 4);
    });

    suite.query_rewards(&bob, None, |result| {
        let rewards_response = result.unwrap();
        match rewards_response {
            RewardsResponse::RewardsResponse { total_rewards, .. } => {
                // 666 comes from 2 epochs of 333 rewards, 2 * 1000 * (500 / 1500)
                assert_eq!(total_rewards, coins(2000 + 666, "uusdy"));
            }
            _ => {
                panic!("Wrong response type, should return RewardsResponse::RewardsResponse")
            }
        }
    });

    suite
        .query_balance("uusdy".to_string(), &bob, |balance| {
            assert_eq!(balance, Uint128::new(1_000_001_500 + 1_000 + 500));
        })
        .claim(&bob, vec![], Some(1), |result| {
            // can't claim in the past
            result.unwrap_err();
        })
        .claim(&bob, vec![], None, |result| {
            // can't claim in the past
            result.unwrap();
        })
        .query_balance("uusdy".to_string(), &bob, |balance| {
            assert_eq!(balance, Uint128::new(1_000_003_000 + 2_000 + 666));
        })
        .claim(&bob, vec![], Some(3), |result| {
            // can't claim in the past
            result.unwrap_err();
        })
        .claim(&bob, vec![], Some(5), |result| {
            // can't claim in the future
            result.unwrap_err();
        })
        .claim(&bob, vec![], Some(4), |result| {
            // can "claim" the current epoch, but it will return nothing as it was already claimed.
            result.unwrap();
        })
        .query_balance("uusdy".to_string(), &bob, |balance| {
            assert_eq!(balance, Uint128::new(1_000_003_000 + 2_000 + 666));
        });
}

#[test]
fn test_claiming_while_expanding_farm() {
    let lp_denom_1 = format!("factory/{MOCK_CONTRACT_ADDR_1}/1.{LP_SYMBOL}").to_string();
    let lp_denom_2 = format!("factory/{MOCK_CONTRACT_ADDR_1}/2.{LP_SYMBOL}").to_string();

    let mut suite = TestingSuite::default_with_balances(vec![
        coin(1_000_000_000u128, "uom".to_string()),
        coin(1_000_000_000u128, "uusdy".to_string()),
        coin(1_000_000_000u128, "uosmo".to_string()),
        coin(1_000_000_000_000, lp_denom_1.clone()),
        coin(1_000_000_000_000, lp_denom_2.clone()),
    ]);

    let alice = suite.creator();
    let bob = suite.senders[1].clone();

    suite.instantiate_default();

    // create overlapping farms
    suite.manage_farm(
        &alice,
        FarmAction::Fill {
            params: FarmParams {
                lp_denom: lp_denom_1.clone(),
                start_epoch: Some(1),
                preliminary_end_epoch: Some(2),
                curve: None,
                farm_asset: Coin {
                    denom: "uusdy".to_string(),
                    amount: Uint128::new(10_000u128),
                },
                farm_identifier: Some("farm-1".to_string()),
            },
        },
        vec![coin(10_000u128, "uusdy"), coin(1_000, "uom")],
        |result| {
            result.unwrap();
        },
    );

    suite.add_one_epoch();

    suite.manage_position(
        &bob,
        PositionAction::Create {
            identifier: Some("lp_denom_1".to_string()),
            unlocking_duration: 86_400,
            receiver: None,
        },
        vec![coin(1_000, lp_denom_1.clone())],
        |result| {
            result.unwrap();
        },
    );

    suite.manage_farm(
        &alice,
        FarmAction::Fill {
            params: FarmParams {
                lp_denom: lp_denom_1.clone(),
                start_epoch: None,
                preliminary_end_epoch: Some(3),
                curve: None,
                farm_asset: Coin {
                    denom: "uusdy".to_string(),
                    amount: Uint128::new(10_000u128),
                },
                farm_identifier: Some("m-farm-1".to_string()),
            },
        },
        vec![coin(10_000u128, "uusdy")],
        |result| {
            result.unwrap();
        },
    );

    suite.add_epochs(2).query_current_epoch(|result| {
        let epoch_response = result.unwrap();
        assert_eq!(epoch_response.epoch.id, 3);
    });

    suite
        .query_balance("uusdy".to_string(), &bob, |balance| {
            assert_eq!(balance, Uint128::new(1_000_000_000));
        })
        .claim(&bob, vec![], None, |result| {
            result.unwrap();
        })
        .query_balance("uusdy".to_string(), &bob, |balance| {
            assert_eq!(balance, Uint128::new(1_000_000_000 + 10_000));
        });
}
