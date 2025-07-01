extern crate core;

use cosmwasm_std::{coin, Coin, Uint128};
use mantra_dex_std::constants::LP_SYMBOL;
use mantra_dex_std::farm_manager::{Curve, Farm, FarmAction, FarmParams, FarmsBy, PositionAction};

use crate::common::suite::TestingSuite;
use crate::common::MOCK_CONTRACT_ADDR_1;
use test_utils::common_constants::*;

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
    // Prefixed Farm Identifiers (used in queries and assertions)
    const M_FARM_1_ID: &str = "m-farm_1";
    const M_FARM_2_ID: &str = "m-farm_2";
    const M_FARM_3_ID: &str = "m-farm_3";
    const M_FARM_4_ID: &str = "m-farm_4";

    // Prefixed Position Identifiers - removed constants, using literal values directly

    // Farm Asset Amounts
    const FARM_1_UUSDY_ASSET_AMOUNT: u128 = 80_000u128;
    const FARM_2_UOSMO_ASSET_AMOUNT: u128 = 10_000u128;
    const FARM_3_UOM_ASSET_AMOUNT: u128 = 30_000u128;
    const FARM_4_UUSDY_ASSET_AMOUNT: u128 = 70_000u128;

    // Farm Creation Fee (uom based)
    const UOM_FARM_CREATION_FEE: u128 = 1_000u128;

    // Position Lock Amounts - removed constants, using literal values directly

    // Epoch IDs for various stages in the test
    const INITIAL_EPOCH_ID: u64 = 10;
    const FARM_1_START_EPOCH: u64 = 12;
    const FARM_1_END_EPOCH: u64 = 16;
    const FARM_2_START_EPOCH: u64 = 14;
    const FARM_2_END_EPOCH: u64 = 24;
    const FARM_3_START_EPOCH: u64 = 20;
    const FARM_3_END_EPOCH: u64 = 23;
    const FARM_4_START_EPOCH: u64 = 23;
    const FARM_4_ASSERTED_END_EPOCH: u64 = 37;

    // Emission rates (used in assertions of Farm struct)
    const FARM_1_EMISSION_RATE: u128 = 20_000;
    const FARM_2_EMISSION_RATE: u128 = 1_000;
    const FARM_3_EMISSION_RATE: u128 = 10_000;
    const FARM_4_EMISSION_RATE: u128 = 5_000;

    let lp_denom_1 = format!("factory/{MOCK_CONTRACT_ADDR_1}/1.{LP_SYMBOL}").to_string();
    let lp_denom_2 = format!("factory/{MOCK_CONTRACT_ADDR_1}/2.{LP_SYMBOL}").to_string();

    let mut suite = TestingSuite::default_with_balances(vec![
        coin(INITIAL_BALANCE, DENOM_UOM.to_string()),
        coin(INITIAL_BALANCE, DENOM_UUSDY.to_string()),
        coin(INITIAL_BALANCE, DENOM_UOSMO.to_string()),
        coin(INITIAL_BALANCE, lp_denom_1.clone()),
        coin(INITIAL_BALANCE, lp_denom_2.clone()),
    ]);

    let creator = suite.creator();
    let other = suite.senders[1].clone();
    let another = suite.senders[2].clone();

    suite.instantiate_default();

    for _ in 0..INITIAL_EPOCH_ID {
        suite.add_one_epoch();
    }

    let fee_collector_addr = suite.fee_collector_addr.clone();

    // create 4 farms with 2 different LPs
    suite
        .query_current_epoch(|result| {
            let epoch_response = result.unwrap();
            assert_eq!(epoch_response.epoch.id, INITIAL_EPOCH_ID);
        })
        .manage_farm(
            &creator,
            FarmAction::Create {
                params: FarmParams {
                    lp_denom: lp_denom_1.clone(),
                    start_epoch: Some(FARM_1_START_EPOCH),
                    preliminary_end_epoch: Some(FARM_1_END_EPOCH),
                    curve: None,
                    farm_asset: Coin {
                        denom: DENOM_UUSDY.to_string(),
                        amount: Uint128::new(FARM_1_UUSDY_ASSET_AMOUNT),
                    },
                    farm_identifier: Some("farm_1".to_string()),
                },
            },
            vec![
                coin(FARM_1_UUSDY_ASSET_AMOUNT, DENOM_UUSDY),
                coin(UOM_FARM_CREATION_FEE, DENOM_UOM),
            ],
            |result| {
                result.unwrap();
            },
        )
        .manage_farm(
            &creator,
            FarmAction::Create {
                params: FarmParams {
                    lp_denom: lp_denom_1.clone(),
                    start_epoch: Some(FARM_2_START_EPOCH),
                    preliminary_end_epoch: Some(FARM_2_END_EPOCH),
                    curve: None,
                    farm_asset: Coin {
                        denom: DENOM_UOSMO.to_string(),
                        amount: Uint128::new(FARM_2_UOSMO_ASSET_AMOUNT),
                    },
                    farm_identifier: Some("farm_2".to_string()),
                },
            },
            vec![
                coin(FARM_2_UOSMO_ASSET_AMOUNT, DENOM_UOSMO),
                coin(UOM_FARM_CREATION_FEE, DENOM_UOM),
            ],
            |result| {
                result.unwrap();
            },
        )
        .manage_farm(
            &other,
            FarmAction::Create {
                params: FarmParams {
                    lp_denom: lp_denom_2.clone(),
                    start_epoch: Some(FARM_3_START_EPOCH),
                    preliminary_end_epoch: Some(FARM_3_END_EPOCH),
                    curve: None,
                    farm_asset: Coin {
                        denom: DENOM_UOM.to_string(),
                        amount: Uint128::new(FARM_3_UOM_ASSET_AMOUNT),
                    },
                    farm_identifier: Some("farm_3".to_string()),
                },
            },
            vec![coin(
                FARM_3_UOM_ASSET_AMOUNT + UOM_FARM_CREATION_FEE,
                DENOM_UOM,
            )], // Combined fee and asset
            |result| {
                result.unwrap();
            },
        )
        .manage_farm(
            &other,
            FarmAction::Create {
                params: FarmParams {
                    lp_denom: lp_denom_2.clone(),
                    start_epoch: Some(FARM_4_START_EPOCH),
                    preliminary_end_epoch: None, // Farm 4 has open end initially
                    curve: None,
                    farm_asset: Coin {
                        denom: DENOM_UUSDY.to_string(),
                        amount: Uint128::new(FARM_4_UUSDY_ASSET_AMOUNT),
                    },
                    farm_identifier: Some("farm_4".to_string()),
                },
            },
            vec![
                coin(FARM_4_UUSDY_ASSET_AMOUNT, DENOM_UUSDY),
                coin(UOM_FARM_CREATION_FEE, DENOM_UOM),
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

    // other fills a position
    suite
        .manage_position(
            &other,
            PositionAction::Create {
                identifier: Some("other_pos_1".to_string()),
                unlocking_duration: DEFAULT_UNLOCKING_DURATION_SECONDS,
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
                unlocking_duration: DEFAULT_UNLOCKING_DURATION_SECONDS,
                receiver: None,
            },
            vec![coin(80_000, lp_denom_2.clone())],
            |result| {
                result.unwrap();
            },
        );

    suite.add_epochs(2).query_current_epoch(|result| {
        let epoch_response = result.unwrap();
        assert_eq!(epoch_response.epoch.id, 15);
    });

    suite
        .query_farms(
            Some(FarmsBy::Identifier(M_FARM_1_ID.to_string())),
            None,
            None,
            |result| {
                let farms_response = result.unwrap();
                assert_eq!(
                    farms_response.farms[0],
                    Farm {
                        identifier: M_FARM_1_ID.to_string(),
                        owner: creator.clone(),
                        lp_denom: lp_denom_1.clone(),
                        farm_asset: Coin {
                            denom: DENOM_UUSDY.to_string(),
                            amount: Uint128::new(FARM_1_UUSDY_ASSET_AMOUNT),
                        },
                        claimed_amount: Uint128::zero(),
                        emission_rate: Uint128::new(FARM_1_EMISSION_RATE),
                        curve: Curve::Linear,
                        start_epoch: FARM_1_START_EPOCH,
                        preliminary_end_epoch: FARM_1_END_EPOCH,
                    }
                );
            },
        )
        .query_balance(DENOM_UUSDY.to_string(), &creator, |balance| {
            assert_eq!(
                balance,
                Uint128::new(INITIAL_BALANCE - FARM_1_UUSDY_ASSET_AMOUNT)
            );
        })
        .claim(&creator, vec![], None, |result| {
            result.unwrap();
        })
        .query_balance(DENOM_UUSDY.to_string(), &creator, |balance| {
            assert_eq!(balance, Uint128::new(999_978_666)); // This value is derived, keep as is or calculate based on previous if simple.
        })
        .query_farms(None, None, None, |result| {
            let farms_response = result.unwrap();
            assert_eq!(
                farms_response.farms[0],
                Farm {
                    identifier: M_FARM_1_ID.to_string(),
                    owner: creator.clone(),
                    lp_denom: lp_denom_1.clone(),
                    farm_asset: Coin {
                        denom: DENOM_UUSDY.to_string(),
                        amount: Uint128::new(FARM_1_UUSDY_ASSET_AMOUNT),
                    },
                    claimed_amount: Uint128::new(58_666), // Derived
                    emission_rate: Uint128::new(FARM_1_EMISSION_RATE),
                    curve: Curve::Linear,
                    start_epoch: FARM_1_START_EPOCH,
                    preliminary_end_epoch: FARM_1_END_EPOCH,
                }
            );
            assert_eq!(
                farms_response.farms[1],
                Farm {
                    identifier: M_FARM_2_ID.to_string(),
                    owner: creator.clone(),
                    lp_denom: lp_denom_1.clone(),
                    farm_asset: Coin {
                        denom: DENOM_UOSMO.to_string(),
                        amount: Uint128::new(FARM_2_UOSMO_ASSET_AMOUNT),
                    },
                    claimed_amount: Uint128::new(932), // Derived
                    emission_rate: Uint128::new(FARM_2_EMISSION_RATE),
                    curve: Curve::Linear,
                    start_epoch: FARM_2_START_EPOCH,
                    preliminary_end_epoch: FARM_2_END_EPOCH,
                }
            );
        });

    suite.add_epochs(4).query_current_epoch(|result| {
        let epoch_response = result.unwrap();
        assert_eq!(epoch_response.epoch.id, 19);
    });

    // other emergency unlocks mid-way farm 2
    suite
        .query_balance(DENOM_UUSDY.to_string(), &other, |balance| {
            // Initial - farm 4 asset amount (other created farm 4)
            assert_eq!(
                balance,
                Uint128::new(INITIAL_BALANCE - FARM_4_UUSDY_ASSET_AMOUNT)
            );
        })
        .query_balance(DENOM_UOSMO.to_string(), &other, |balance| {
            assert_eq!(balance, Uint128::new(INITIAL_BALANCE));
        })
        .claim(&other, vec![], None, |result| {
            result.unwrap();
        })
        .query_balance(DENOM_UUSDY.to_string(), &other, |balance| {
            assert_eq!(balance, Uint128::new(999_951_332)); // Derived
        })
        .query_balance(DENOM_UOSMO.to_string(), &other, |balance| {
            assert_eq!(balance, Uint128::new(1_000_003_198)); // Derived
        })
        .query_farms(None, None, None, |result| {
            let farms_response = result.unwrap();
            assert_eq!(
                farms_response.farms[0],
                Farm {
                    identifier: M_FARM_1_ID.to_string(),
                    owner: creator.clone(),
                    lp_denom: lp_denom_1.clone(),
                    farm_asset: Coin {
                        denom: DENOM_UUSDY.to_string(),
                        amount: Uint128::new(FARM_1_UUSDY_ASSET_AMOUNT),
                    },
                    claimed_amount: Uint128::new(79_998u128), // exhausted, derived
                    emission_rate: Uint128::new(FARM_1_EMISSION_RATE),
                    curve: Curve::Linear,
                    start_epoch: FARM_1_START_EPOCH,
                    preliminary_end_epoch: FARM_1_END_EPOCH,
                }
            );
            assert_eq!(
                farms_response.farms[1],
                Farm {
                    identifier: M_FARM_2_ID.to_string(),
                    owner: creator.clone(),
                    lp_denom: lp_denom_1.clone(),
                    farm_asset: Coin {
                        denom: DENOM_UOSMO.to_string(),
                        amount: Uint128::new(FARM_2_UOSMO_ASSET_AMOUNT),
                    },
                    claimed_amount: Uint128::new(4_130), // Derived
                    emission_rate: Uint128::new(FARM_2_EMISSION_RATE),
                    curve: Curve::Linear,
                    start_epoch: FARM_2_START_EPOCH,
                    preliminary_end_epoch: FARM_2_END_EPOCH,
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
                assert_eq!(balance, Uint128::new(2_000));
            },
        )
        .query_balance(
            lp_denom_2.clone().to_string(),
            &fee_collector_addr,
            |balance| {
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
            unlocking_duration: 15_778_476, // ~6 months
            receiver: None,
        },
        vec![coin(6_000, lp_denom_2.clone())],
        |result| {
            result.unwrap();
        },
    );

    // creator that had 100% now has ~70% of the weight, while another has ~30%
    suite.add_epochs(10).query_current_epoch(|result| {
        let epoch_response = result.unwrap();
        assert_eq!(epoch_response.epoch.id, 30);
    });

    suite
        .claim(&creator, vec![], None, |result| {
            result.unwrap();
        })
        .query_farms(None, None, None, |result| {
            let farms_response = result.unwrap();
            assert_eq!(
                farms_response.farms[0],
                Farm {
                    identifier: M_FARM_1_ID.to_string(),
                    owner: creator.clone(),
                    lp_denom: lp_denom_1.clone(),
                    farm_asset: Coin {
                        denom: DENOM_UUSDY.to_string(),
                        amount: Uint128::new(FARM_1_UUSDY_ASSET_AMOUNT),
                    },
                    claimed_amount: Uint128::new(79_998u128), // exhausted
                    emission_rate: Uint128::new(FARM_1_EMISSION_RATE),
                    curve: Curve::Linear,
                    start_epoch: FARM_1_START_EPOCH,
                    preliminary_end_epoch: FARM_1_END_EPOCH,
                }
            );
            assert_eq!(
                farms_response.farms[1],
                Farm {
                    identifier: M_FARM_2_ID.to_string(),
                    owner: creator.clone(),
                    lp_denom: lp_denom_1.clone(),
                    farm_asset: Coin {
                        denom: DENOM_UOSMO.to_string(),
                        amount: Uint128::new(FARM_2_UOSMO_ASSET_AMOUNT),
                    },
                    claimed_amount: Uint128::new(9_994), // exhausted
                    emission_rate: Uint128::new(FARM_2_EMISSION_RATE),
                    curve: Curve::Linear,
                    start_epoch: FARM_2_START_EPOCH,
                    preliminary_end_epoch: FARM_2_END_EPOCH,
                }
            );
            assert_eq!(
                farms_response.farms[2],
                Farm {
                    identifier: M_FARM_3_ID.to_string(),
                    owner: other.clone(),
                    lp_denom: lp_denom_2.clone(),
                    farm_asset: Coin {
                        denom: DENOM_UOM.to_string(),
                        amount: Uint128::new(FARM_3_UOM_ASSET_AMOUNT),
                    },
                    claimed_amount: Uint128::new(24_000), // Derived
                    emission_rate: Uint128::new(FARM_3_EMISSION_RATE),
                    curve: Curve::Linear,
                    start_epoch: FARM_3_START_EPOCH,
                    preliminary_end_epoch: FARM_3_END_EPOCH,
                }
            );
            assert_eq!(
                farms_response.farms[3],
                Farm {
                    identifier: M_FARM_4_ID.to_string(),
                    owner: other.clone(),
                    lp_denom: lp_denom_2.clone(),
                    farm_asset: Coin {
                        denom: DENOM_UUSDY.to_string(),
                        amount: Uint128::new(FARM_4_UUSDY_ASSET_AMOUNT),
                    },
                    claimed_amount: Uint128::new(28_000), // Derived
                    emission_rate: Uint128::new(FARM_4_EMISSION_RATE),
                    curve: Curve::Linear,
                    start_epoch: FARM_4_START_EPOCH,
                    preliminary_end_epoch: FARM_4_ASSERTED_END_EPOCH,
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
                    identifier: M_FARM_1_ID.to_string(),
                    owner: creator.clone(),
                    lp_denom: lp_denom_1.clone(),
                    farm_asset: Coin {
                        denom: DENOM_UUSDY.to_string(),
                        amount: Uint128::new(FARM_1_UUSDY_ASSET_AMOUNT),
                    },
                    claimed_amount: Uint128::new(79_998u128), // exhausted
                    emission_rate: Uint128::new(FARM_1_EMISSION_RATE),
                    curve: Curve::Linear,
                    start_epoch: FARM_1_START_EPOCH,
                    preliminary_end_epoch: FARM_1_END_EPOCH,
                }
            );
            assert_eq!(
                farms_response.farms[1],
                Farm {
                    identifier: M_FARM_2_ID.to_string(),
                    owner: creator.clone(),
                    lp_denom: lp_denom_1.clone(),
                    farm_asset: Coin {
                        denom: DENOM_UOSMO.to_string(),
                        amount: Uint128::new(FARM_2_UOSMO_ASSET_AMOUNT),
                    },
                    claimed_amount: Uint128::new(9_994), // exhausted
                    emission_rate: Uint128::new(FARM_2_EMISSION_RATE),
                    curve: Curve::Linear,
                    start_epoch: FARM_2_START_EPOCH,
                    preliminary_end_epoch: FARM_2_END_EPOCH,
                }
            );
            assert_eq!(
                farms_response.farms[2],
                Farm {
                    identifier: M_FARM_3_ID.to_string(),
                    owner: other.clone(),
                    lp_denom: lp_denom_2.clone(),
                    farm_asset: Coin {
                        denom: DENOM_UOM.to_string(),
                        amount: Uint128::new(FARM_3_UOM_ASSET_AMOUNT),
                    },
                    claimed_amount: Uint128::new(30_000), // exhausted
                    emission_rate: Uint128::new(FARM_3_EMISSION_RATE),
                    curve: Curve::Linear,
                    start_epoch: FARM_3_START_EPOCH,
                    preliminary_end_epoch: FARM_3_END_EPOCH,
                }
            );
            assert_eq!(
                farms_response.farms[3],
                Farm {
                    identifier: M_FARM_4_ID.to_string(),
                    owner: other.clone(),
                    lp_denom: lp_denom_2.clone(),
                    farm_asset: Coin {
                        denom: DENOM_UUSDY.to_string(),
                        amount: Uint128::new(FARM_4_UUSDY_ASSET_AMOUNT),
                    },
                    claimed_amount: Uint128::new(40_000), // Derived
                    emission_rate: Uint128::new(FARM_4_EMISSION_RATE),
                    curve: Curve::Linear,
                    start_epoch: FARM_4_START_EPOCH,
                    preliminary_end_epoch: FARM_4_ASSERTED_END_EPOCH,
                }
            );
        });

    // another closes part of his position mid-way through farm 4.
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

    suite.add_epochs(5).query_current_epoch(|result| {
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
                    identifier: M_FARM_4_ID.to_string(),
                    owner: other.clone(),
                    lp_denom: lp_denom_2.clone(),
                    farm_asset: Coin {
                        denom: DENOM_UUSDY.to_string(),
                        amount: Uint128::new(FARM_4_UUSDY_ASSET_AMOUNT),
                    },
                    claimed_amount: Uint128::new(60_585), // Derived
                    emission_rate: Uint128::new(FARM_4_EMISSION_RATE),
                    curve: Curve::Linear,
                    start_epoch: FARM_4_START_EPOCH,
                    preliminary_end_epoch: FARM_4_ASSERTED_END_EPOCH,
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
                    identifier: M_FARM_4_ID.to_string(),
                    owner: other.clone(),
                    lp_denom: lp_denom_2.clone(),
                    farm_asset: Coin {
                        denom: DENOM_UUSDY.to_string(),
                        amount: Uint128::new(FARM_4_UUSDY_ASSET_AMOUNT),
                    },
                    claimed_amount: Uint128::new(64_995), // Derived
                    emission_rate: Uint128::new(FARM_4_EMISSION_RATE),
                    curve: Curve::Linear,
                    start_epoch: FARM_4_START_EPOCH,
                    preliminary_end_epoch: FARM_4_ASSERTED_END_EPOCH,
                }
            );
        });

    // now the epochs go by, the farm expires and the creator withdraws the rest of the rewards

    suite.add_epochs(5).query_current_epoch(|result| {
        let epoch_response = result.unwrap();
        assert_eq!(epoch_response.epoch.id, 40);
    });

    suite.manage_farm(
        &creator,
        FarmAction::Close {
            farm_identifier: M_FARM_4_ID.to_string(),
        },
        vec![],
        |result| {
            result.unwrap();
        },
    );
}
