extern crate core;

use cosmwasm_std::{coin, Coin, Uint128};
use mantra_dex_std::constants::LP_SYMBOL;
use mantra_dex_std::farm_manager::{Curve, Farm, FarmAction, FarmParams, FarmsBy, PositionAction};

use crate::common::suite::TestingSuite;
use crate::common::MOCK_CONTRACT_ADDR_1;

/// Complex test case with 4 farms for 2 different LPs somewhat overlapping in time
/// Farm 1 -> runs from epoch 12 to 16
/// Farm 2 -> run from epoch 14 to 25
/// Farm 3 -> runs from epoch 20 to 23
/// Farm 4 -> runs from epoch 23 to 37
///
/// There are 3 users, creator, other and another
///
/// Locking tokens:
/// creator locks 35% of the LP tokens before farm 1 starts
/// other locks 40% of the LP tokens before after farm 1 starts and before farm 2 starts
/// another locks 25% of the LP tokens after farm 3 starts, before farm 3 ends
///
/// Unlocking tokens:
/// creator never unlocks
/// other emergency unlocks mid-way through farm 2
/// another partially unlocks mid-way through farm 4
///
/// Verify users got rewards pro rata to their locked tokens
#[test]
fn test_multiple_farms_and_positions() {
    let lp_denom_1 = format!("factory/{MOCK_CONTRACT_ADDR_1}/1.{LP_SYMBOL}").to_string();
    let lp_denom_2 = format!("factory/{MOCK_CONTRACT_ADDR_1}/2.{LP_SYMBOL}").to_string();

    let mut suite = TestingSuite::default_with_balances(vec![
        coin(1_000_000_000u128, "uom".to_string()),
        coin(1_000_000_000u128, "uusdy".to_string()),
        coin(1_000_000_000u128, "uosmo".to_string()),
        coin(1_000_000_000u128, lp_denom_1.clone()),
        coin(1_000_000_000u128, lp_denom_2.clone()),
    ]);

    let creator = suite.creator();
    let other = suite.senders[1].clone();
    let another = suite.senders[2].clone();

    suite.instantiate_default();

    for _ in 0..10 {
        suite.add_one_epoch();
    }

    let fee_collector_addr = suite.fee_collector_addr.clone();

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
                    start_epoch: Some(12),
                    preliminary_end_epoch: Some(16),
                    curve: None,
                    farm_asset: Coin {
                        denom: "uusdy".to_string(),
                        amount: Uint128::new(80_000u128),
                    },
                    farm_identifier: Some("farm_1".to_string()),
                },
            },
            vec![coin(80_000u128, "uusdy"), coin(1_000, "uom")],
            |result| {
                result.unwrap();
            },
        )
        .manage_farm(
            &creator,
            FarmAction::Fill {
                params: FarmParams {
                    lp_denom: lp_denom_1.clone(),
                    start_epoch: Some(14),
                    preliminary_end_epoch: Some(24),
                    curve: None,
                    farm_asset: Coin {
                        denom: "uosmo".to_string(),
                        amount: Uint128::new(10_000u128),
                    },
                    farm_identifier: Some("farm_2".to_string()),
                },
            },
            vec![coin(10_000u128, "uosmo"), coin(1_000, "uom")],
            |result| {
                result.unwrap();
            },
        )
        .manage_farm(
            &other,
            FarmAction::Fill {
                params: FarmParams {
                    lp_denom: lp_denom_2.clone(),
                    start_epoch: Some(20),
                    preliminary_end_epoch: Some(23),
                    curve: None,
                    farm_asset: Coin {
                        denom: "uom".to_string(),
                        amount: Uint128::new(30_000u128),
                    },
                    farm_identifier: Some("farm_3".to_string()),
                },
            },
            vec![coin(31_000u128, "uom")],
            |result| {
                result.unwrap();
            },
        )
        .manage_farm(
            &other,
            FarmAction::Fill {
                params: FarmParams {
                    lp_denom: lp_denom_2.clone(),
                    start_epoch: Some(23),
                    preliminary_end_epoch: None,
                    curve: None,
                    farm_asset: Coin {
                        denom: "uusdy".to_string(),
                        amount: Uint128::new(70_000u128),
                    },
                    farm_identifier: Some("farm_4".to_string()),
                },
            },
            vec![coin(70_000u128, "uusdy"), coin(1_000, "uom")],
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
                unlocking_duration: 86_400,
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
                unlocking_duration: 86_400,
                receiver: None,
            },
            vec![coin(70_000, lp_denom_2.clone())],
            |result| {
                result.unwrap();
            },
        );

    suite
        .add_one_epoch()
        .add_one_epoch()
        .add_one_epoch()
        .query_current_epoch(|result| {
            let epoch_response = result.unwrap();
            assert_eq!(epoch_response.epoch.id, 13);
        });

    // other fills a position
    suite
        .manage_position(
            &other,
            PositionAction::Create {
                identifier: Some("other_pos_1".to_string()),
                unlocking_duration: 86_400,
                receiver: None,
            },
            vec![coin(40_000, lp_denom_1.clone())],
            |result| {
                result.unwrap();
            },
        )
        .manage_position(
            &other,
            PositionAction::Create {
                identifier: Some("other_pos_2".to_string()),
                unlocking_duration: 86_400,
                receiver: None,
            },
            vec![coin(80_000, lp_denom_2.clone())],
            |result| {
                result.unwrap();
            },
        );

    suite
        .add_one_epoch()
        .add_one_epoch()
        .query_current_epoch(|result| {
            let epoch_response = result.unwrap();
            assert_eq!(epoch_response.epoch.id, 15);
        });

    suite
        .query_farms(
            Some(FarmsBy::Identifier("m-farm_1".to_string())),
            None,
            None,
            |result| {
                let farms_response = result.unwrap();
                assert_eq!(
                    farms_response.farms[0],
                    Farm {
                        identifier: "m-farm_1".to_string(),
                        owner: creator.clone(),
                        lp_denom: lp_denom_1.clone(),
                        farm_asset: Coin {
                            denom: "uusdy".to_string(),
                            amount: Uint128::new(80_000u128),
                        },
                        claimed_amount: Uint128::zero(),
                        emission_rate: Uint128::new(20_000),
                        curve: Curve::Linear,
                        start_epoch: 12u64,
                        preliminary_end_epoch: 16u64,
                    }
                );
            },
        )
        .query_balance("uusdy".to_string(), &creator, |balance| {
            assert_eq!(balance, Uint128::new(999_920_000));
        })
        .claim(&creator, vec![], None, |result| {
            result.unwrap();
        })
        .query_balance("uusdy".to_string(), &creator, |balance| {
            assert_eq!(balance, Uint128::new(999_978_666));
        })
        .query_farms(None, None, None, |result| {
            let farms_response = result.unwrap();
            assert_eq!(
                farms_response.farms[0],
                Farm {
                    identifier: "m-farm_1".to_string(),
                    owner: creator.clone(),
                    lp_denom: lp_denom_1.clone(),
                    farm_asset: Coin {
                        denom: "uusdy".to_string(),
                        amount: Uint128::new(80_000u128),
                    },
                    claimed_amount: Uint128::new(58_666),
                    emission_rate: Uint128::new(20_000),
                    curve: Curve::Linear,
                    start_epoch: 12u64,
                    preliminary_end_epoch: 16u64,
                }
            );
            assert_eq!(
                farms_response.farms[1],
                Farm {
                    identifier: "m-farm_2".to_string(),
                    owner: creator.clone(),
                    lp_denom: lp_denom_1.clone(),
                    farm_asset: Coin {
                        denom: "uosmo".to_string(),
                        amount: Uint128::new(10_000u128),
                    },
                    claimed_amount: Uint128::new(932),
                    emission_rate: Uint128::new(1_000),
                    curve: Curve::Linear,
                    start_epoch: 14u64,
                    preliminary_end_epoch: 24u64,
                }
            );
        });

    suite
        .add_one_epoch()
        .add_one_epoch()
        .add_one_epoch()
        .add_one_epoch()
        .query_current_epoch(|result| {
            let epoch_response = result.unwrap();
            assert_eq!(epoch_response.epoch.id, 19);
        });

    // other emergency unlocks mid-way farm 2
    suite
        .query_balance("uusdy".to_string(), &other, |balance| {
            assert_eq!(balance, Uint128::new(999_930_000));
        })
        .query_balance("uosmo".to_string(), &other, |balance| {
            assert_eq!(balance, Uint128::new(1_000_000_000));
        })
        .claim(&other, vec![], None, |result| {
            result.unwrap();
        })
        .query_balance("uusdy".to_string(), &other, |balance| {
            assert_eq!(balance, Uint128::new(999_951_332));
        })
        .query_balance("uosmo".to_string(), &other, |balance| {
            assert_eq!(balance, Uint128::new(1_000_003_198));
        })
        .query_farms(None, None, None, |result| {
            let farms_response = result.unwrap();
            assert_eq!(
                farms_response.farms[0],
                Farm {
                    identifier: "m-farm_1".to_string(),
                    owner: creator.clone(),
                    lp_denom: lp_denom_1.clone(),
                    farm_asset: Coin {
                        denom: "uusdy".to_string(),
                        amount: Uint128::new(80_000u128),
                    },
                    claimed_amount: Uint128::new(79_998u128), // exhausted
                    emission_rate: Uint128::new(20_000),
                    curve: Curve::Linear,
                    start_epoch: 12u64,
                    preliminary_end_epoch: 16u64,
                }
            );
            assert_eq!(
                farms_response.farms[1],
                Farm {
                    identifier: "m-farm_2".to_string(),
                    owner: creator.clone(),
                    lp_denom: lp_denom_1.clone(),
                    farm_asset: Coin {
                        denom: "uosmo".to_string(),
                        amount: Uint128::new(10_000u128),
                    },
                    claimed_amount: Uint128::new(4_130),
                    emission_rate: Uint128::new(1_000),
                    curve: Curve::Linear,
                    start_epoch: 14u64,
                    preliminary_end_epoch: 24u64,
                }
            );
        })
        .manage_position(
            &other,
            PositionAction::Withdraw {
                identifier: "u-other_pos_1".to_string(),
                emergency_unlock: Some(true),
            },
            vec![],
            |result| {
                result.unwrap();
            },
        )
        .manage_position(
            &other,
            PositionAction::Withdraw {
                identifier: "u-other_pos_2".to_string(),
                emergency_unlock: Some(true),
            },
            vec![],
            |result| {
                result.unwrap();
            },
        )
        .query_balance(
            lp_denom_1.clone().to_string(),
            &fee_collector_addr,
            |balance| {
                // 10% of the lp the user input initially
                assert_eq!(balance, Uint128::new(2_000));
            },
        )
        .query_balance(
            lp_denom_2.clone().to_string(),
            &fee_collector_addr,
            |balance| {
                // 10% of the lp the user input initially
                assert_eq!(balance, Uint128::new(8_000));
            },
        );

    // at this point, other doesn't have any positions, and creator owns 100% of the weight

    suite.add_one_epoch().query_current_epoch(|result| {
        let epoch_response = result.unwrap();
        assert_eq!(epoch_response.epoch.id, 20);
    });

    // another fills a position
    suite.manage_position(
        &another,
        PositionAction::Create {
            identifier: Some("another_pos_1".to_string()),
            unlocking_duration: 15_778_476, // 6 months, should give him 5x multiplier
            receiver: None,
        },
        vec![coin(6_000, lp_denom_2.clone())],
        |result| {
            result.unwrap();
        },
    );

    // creator that had 100% now has ~70% of the weight, while another has ~30%
    suite
        .add_one_epoch()
        .add_one_epoch()
        .add_one_epoch()
        .add_one_epoch()
        .add_one_epoch()
        .add_one_epoch()
        .add_one_epoch()
        .add_one_epoch()
        .add_one_epoch()
        .add_one_epoch()
        .query_current_epoch(|result| {
            let epoch_response = result.unwrap();
            assert_eq!(epoch_response.epoch.id, 30);
        });

    suite
        .claim(&creator, vec![], None, |result| {
            // creator claims from epoch 16 to 30
            // There's nothing to claim on farm 1
            // On farm 2, creator has a portion of the total weight until the epoch where other
            // triggered the emergency withdrawal. From that point (epoch 20) it has 100% of the weight
            // for lp_denom_1.
            // another never locked for lp_denom_1, so creator gets all the rewards for the farm 2
            // from epoch 20 till it finishes at epoch 23
            result.unwrap();
        })
        .query_farms(None, None, None, |result| {
            let farms_response = result.unwrap();
            assert_eq!(
                farms_response.farms[0],
                Farm {
                    identifier: "m-farm_1".to_string(),
                    owner: creator.clone(),
                    lp_denom: lp_denom_1.clone(),
                    farm_asset: Coin {
                        denom: "uusdy".to_string(),
                        amount: Uint128::new(80_000u128),
                    },
                    claimed_amount: Uint128::new(79_998u128), // exhausted
                    emission_rate: Uint128::new(20_000),
                    curve: Curve::Linear,
                    start_epoch: 12u64,
                    preliminary_end_epoch: 16u64,
                }
            );
            assert_eq!(
                farms_response.farms[1],
                Farm {
                    identifier: "m-farm_2".to_string(),
                    owner: creator.clone(),
                    lp_denom: lp_denom_1.clone(),
                    farm_asset: Coin {
                        denom: "uosmo".to_string(),
                        amount: Uint128::new(10_000u128),
                    },
                    claimed_amount: Uint128::new(9_994), // exhausted
                    emission_rate: Uint128::new(1_000),
                    curve: Curve::Linear,
                    start_epoch: 14u64,
                    preliminary_end_epoch: 24u64,
                }
            );
            assert_eq!(
                farms_response.farms[2],
                Farm {
                    identifier: "m-farm_3".to_string(),
                    owner: other.clone(),
                    lp_denom: lp_denom_2.clone(),
                    farm_asset: Coin {
                        denom: "uom".to_string(),
                        amount: Uint128::new(30_000u128),
                    },
                    claimed_amount: Uint128::new(24_000),
                    emission_rate: Uint128::new(10_000),
                    curve: Curve::Linear,
                    start_epoch: 20u64,
                    preliminary_end_epoch: 23u64,
                }
            );
            assert_eq!(
                farms_response.farms[3],
                Farm {
                    identifier: "m-farm_4".to_string(),
                    owner: other.clone(),
                    lp_denom: lp_denom_2.clone(),
                    farm_asset: Coin {
                        denom: "uusdy".to_string(),
                        amount: Uint128::new(70_000u128),
                    },
                    claimed_amount: Uint128::new(28_000),
                    emission_rate: Uint128::new(5_000),
                    curve: Curve::Linear,
                    start_epoch: 23u64,
                    preliminary_end_epoch: 37u64,
                }
            );
        })
        .claim(&another, vec![], None, |result| {
            result.unwrap();
        })
        .query_farms(None, None, None, |result| {
            let farms_response = result.unwrap();
            assert_eq!(
                farms_response.farms[0],
                Farm {
                    identifier: "m-farm_1".to_string(),
                    owner: creator.clone(),
                    lp_denom: lp_denom_1.clone(),
                    farm_asset: Coin {
                        denom: "uusdy".to_string(),
                        amount: Uint128::new(80_000u128),
                    },
                    claimed_amount: Uint128::new(79_998u128), // exhausted
                    emission_rate: Uint128::new(20_000),
                    curve: Curve::Linear,
                    start_epoch: 12u64,
                    preliminary_end_epoch: 16u64,
                }
            );
            assert_eq!(
                farms_response.farms[1],
                Farm {
                    identifier: "m-farm_2".to_string(),
                    owner: creator.clone(),
                    lp_denom: lp_denom_1.clone(),
                    farm_asset: Coin {
                        denom: "uosmo".to_string(),
                        amount: Uint128::new(10_000u128),
                    },
                    claimed_amount: Uint128::new(9_994), // exhausted
                    emission_rate: Uint128::new(1_000),
                    curve: Curve::Linear,
                    start_epoch: 14u64,
                    preliminary_end_epoch: 24u64,
                }
            );
            assert_eq!(
                farms_response.farms[2],
                Farm {
                    identifier: "m-farm_3".to_string(),
                    owner: other.clone(),
                    lp_denom: lp_denom_2.clone(),
                    farm_asset: Coin {
                        denom: "uom".to_string(),
                        amount: Uint128::new(30_000u128),
                    },
                    claimed_amount: Uint128::new(30_000), // exhausted
                    emission_rate: Uint128::new(10_000),
                    curve: Curve::Linear,
                    start_epoch: 20u64,
                    preliminary_end_epoch: 23u64,
                }
            );
            assert_eq!(
                farms_response.farms[3],
                Farm {
                    identifier: "m-farm_4".to_string(),
                    owner: other.clone(),
                    lp_denom: lp_denom_2.clone(),
                    farm_asset: Coin {
                        denom: "uusdy".to_string(),
                        amount: Uint128::new(70_000u128),
                    },
                    claimed_amount: Uint128::new(40_000),
                    emission_rate: Uint128::new(5_000),
                    curve: Curve::Linear,
                    start_epoch: 23u64,
                    preliminary_end_epoch: 37u64,
                }
            );
        });

    // another closes part of his position mid-way through farm 4.
    // since the total weight was 100k and he unlocked 50% of his position,
    // the new total weight is 85k, so he gets 15k/85k of the rewards while creator gets the rest
    suite.manage_position(
        &another,
        PositionAction::Close {
            identifier: "u-another_pos_1".to_string(),
            lp_asset: Some(coin(3_000, lp_denom_2.clone())),
        },
        vec![],
        |result| {
            result.unwrap();
        },
    );

    suite
        .add_one_epoch()
        .add_one_epoch()
        .add_one_epoch()
        .add_one_epoch()
        .add_one_epoch()
        .query_current_epoch(|result| {
            let epoch_response = result.unwrap();
            assert_eq!(epoch_response.epoch.id, 35);
        });

    suite
        .claim(&creator, vec![], None, |result| {
            result.unwrap();
        })
        .query_farms(None, None, None, |result| {
            let farms_response = result.unwrap();
            assert_eq!(
                farms_response.farms[3],
                Farm {
                    identifier: "m-farm_4".to_string(),
                    owner: other.clone(),
                    lp_denom: lp_denom_2.clone(),
                    farm_asset: Coin {
                        denom: "uusdy".to_string(),
                        amount: Uint128::new(70_000u128),
                    },
                    claimed_amount: Uint128::new(60_585),
                    emission_rate: Uint128::new(5_000),
                    curve: Curve::Linear,
                    start_epoch: 23u64,
                    preliminary_end_epoch: 37u64,
                }
            );
        })
        .claim(&another, vec![], None, |result| {
            result.unwrap();
        })
        .query_farms(None, None, None, |result| {
            let farms_response = result.unwrap();
            assert_eq!(
                farms_response.farms[3],
                Farm {
                    identifier: "m-farm_4".to_string(),
                    owner: other.clone(),
                    lp_denom: lp_denom_2.clone(),
                    farm_asset: Coin {
                        denom: "uusdy".to_string(),
                        amount: Uint128::new(70_000u128),
                    },
                    claimed_amount: Uint128::new(64_995),
                    emission_rate: Uint128::new(5_000),
                    curve: Curve::Linear,
                    start_epoch: 23u64,
                    preliminary_end_epoch: 37u64,
                }
            );
        });

    // now the epochs go by, the farm expires and the creator withdraws the rest of the rewards

    suite
        .add_one_epoch()
        .add_one_epoch()
        .add_one_epoch()
        .add_one_epoch()
        .add_one_epoch()
        .query_current_epoch(|result| {
            let epoch_response = result.unwrap();
            assert_eq!(epoch_response.epoch.id, 40);
        });

    suite.manage_farm(
        &creator,
        FarmAction::Close {
            farm_identifier: "m-farm_4".to_string(),
        },
        vec![],
        |result| {
            result.unwrap();
        },
    );
}
