extern crate core;

use cosmwasm_std::{coin, Coin, Uint128};
use mantra_dex_std::constants::LP_SYMBOL;
use mantra_dex_std::farm_manager::{Curve, Farm, FarmAction, FarmParams, FarmsBy, PositionAction};

use super::common_constants::{
    DEFAULT_UNLOCKING_DURATION_SECONDS, INITIAL_USER_BALANCE, UOM_DENOM, UOM_FARM_CREATION_FEE,
    UOSMO_DENOM, UUSDY_DENOM,
};
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
    // Farm Raw Identifiers (used in FarmParams)
    const RAW_FARM_1_ID: &str = "farm_1";
    const RAW_FARM_2_ID: &str = "farm_2";
    const RAW_FARM_3_ID: &str = "farm_3";
    const RAW_FARM_4_ID: &str = "farm_4";

    // Prefixed Farm Identifiers (used in queries and assertions)
    const M_FARM_1_ID: &str = "m-farm_1";
    const M_FARM_2_ID: &str = "m-farm_2";
    const M_FARM_3_ID: &str = "m-farm_3";
    const M_FARM_4_ID: &str = "m-farm_4";

    // Position Raw Identifiers (used in PositionAction::Create)
    const CREATOR_POS_1_RAW_ID: &str = "creator_pos_1";
    const CREATOR_POS_2_RAW_ID: &str = "creator_pos_2";
    const OTHER_POS_1_RAW_ID: &str = "other_pos_1";
    const OTHER_POS_2_RAW_ID: &str = "other_pos_2";
    const ANOTHER_POS_1_RAW_ID: &str = "another_pos_1";

    // Prefixed Position Identifiers (used in PositionAction::Withdraw, Close)
    const U_OTHER_POS_1_ID: &str = "u-other_pos_1";
    const U_OTHER_POS_2_ID: &str = "u-other_pos_2";
    const U_ANOTHER_POS_1_ID: &str = "u-another_pos_1";

    // Unlocking Durations
    const SIX_MONTHS_UNLOCKING_DURATION_SECONDS: u64 = 15_778_476; // ~6 months

    // Farm Asset Amounts
    const FARM_1_UUSDY_ASSET_AMOUNT: u128 = 80_000u128;
    const FARM_2_UOSMO_ASSET_AMOUNT: u128 = 10_000u128;
    const FARM_3_UOM_ASSET_AMOUNT: u128 = 30_000u128;
    const FARM_4_UUSDY_ASSET_AMOUNT: u128 = 70_000u128;

    // Farm Creation Fee (uom based)
    const UOM_FARM_CREATION_FEE: u128 = 1_000u128;

    // Position Lock Amounts
    const CREATOR_LP1_LOCK_AMOUNT: u128 = 35_000;
    const CREATOR_LP2_LOCK_AMOUNT: u128 = 70_000;
    const OTHER_LP1_LOCK_AMOUNT: u128 = 40_000;
    const OTHER_LP2_LOCK_AMOUNT: u128 = 80_000;
    const ANOTHER_LP2_LOCK_AMOUNT: u128 = 6_000;

    // Partial Unlock Amount
    const ANOTHER_LP2_PARTIAL_UNLOCK_AMOUNT: u128 = 3_000;

    // Epoch IDs for various stages in the test
    const INITIAL_EPOCH_ID: u64 = 10;
    const FARM_1_START_EPOCH: u64 = 12;
    const FARM_1_END_EPOCH: u64 = 16;
    const EPOCH_ID_13: u64 = 13;
    const FARM_2_START_EPOCH: u64 = 14;
    const FARM_2_END_EPOCH: u64 = 24;
    const EPOCH_ID_15: u64 = 15;
    const EPOCH_ID_19: u64 = 19;
    const FARM_3_START_EPOCH: u64 = 20;
    const FARM_3_END_EPOCH: u64 = 23;
    const EPOCH_ID_20: u64 = 20;
    const FARM_4_START_EPOCH: u64 = 23;
    const FARM_4_ASSERTED_END_EPOCH: u64 = 37;

    const EPOCH_ID_30: u64 = 30;
    const EPOCH_ID_35: u64 = 35;
    const FINAL_EPOCH_ID_FOR_TEST: u64 = 40;

    // Expected emergency unlock fees collected by fee_collector
    const EXPECTED_FEE_COLLECTED_LP1_EMERGENCY_UNLOCK: u128 = 2_000;
    const EXPECTED_FEE_COLLECTED_LP2_EMERGENCY_UNLOCK: u128 = 8_000;

    // Emission rates (used in assertions of Farm struct)
    const FARM_1_EMISSION_RATE: u128 = 20_000;
    const FARM_2_EMISSION_RATE: u128 = 1_000;
    const FARM_3_EMISSION_RATE: u128 = 10_000;
    const FARM_4_EMISSION_RATE: u128 = 5_000;

    let lp_denom_1 = format!("factory/{MOCK_CONTRACT_ADDR_1}/1.{LP_SYMBOL}").to_string();
    let lp_denom_2 = format!("factory/{MOCK_CONTRACT_ADDR_1}/2.{LP_SYMBOL}").to_string();

    let mut suite = TestingSuite::default_with_balances(vec![
        coin(INITIAL_USER_BALANCE, UOM_DENOM.to_string()),
        coin(INITIAL_USER_BALANCE, UUSDY_DENOM.to_string()),
        coin(INITIAL_USER_BALANCE, UOSMO_DENOM.to_string()),
        coin(INITIAL_USER_BALANCE, lp_denom_1.clone()),
        coin(INITIAL_USER_BALANCE, lp_denom_2.clone()),
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
            FarmAction::Fill {
                params: FarmParams {
                    lp_denom: lp_denom_1.clone(),
                    start_epoch: Some(FARM_1_START_EPOCH),
                    preliminary_end_epoch: Some(FARM_1_END_EPOCH),
                    curve: None,
                    farm_asset: Coin {
                        denom: UUSDY_DENOM.to_string(),
                        amount: Uint128::new(FARM_1_UUSDY_ASSET_AMOUNT),
                    },
                    farm_identifier: Some(RAW_FARM_1_ID.to_string()),
                },
            },
            vec![
                coin(FARM_1_UUSDY_ASSET_AMOUNT, UUSDY_DENOM),
                coin(UOM_FARM_CREATION_FEE, UOM_DENOM),
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
                    start_epoch: Some(FARM_2_START_EPOCH),
                    preliminary_end_epoch: Some(FARM_2_END_EPOCH),
                    curve: None,
                    farm_asset: Coin {
                        denom: UOSMO_DENOM.to_string(),
                        amount: Uint128::new(FARM_2_UOSMO_ASSET_AMOUNT),
                    },
                    farm_identifier: Some(RAW_FARM_2_ID.to_string()),
                },
            },
            vec![
                coin(FARM_2_UOSMO_ASSET_AMOUNT, UOSMO_DENOM),
                coin(UOM_FARM_CREATION_FEE, UOM_DENOM),
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
                    start_epoch: Some(FARM_3_START_EPOCH),
                    preliminary_end_epoch: Some(FARM_3_END_EPOCH),
                    curve: None,
                    farm_asset: Coin {
                        denom: UOM_DENOM.to_string(),
                        amount: Uint128::new(FARM_3_UOM_ASSET_AMOUNT),
                    },
                    farm_identifier: Some(RAW_FARM_3_ID.to_string()),
                },
            },
            vec![coin(
                FARM_3_UOM_ASSET_AMOUNT + UOM_FARM_CREATION_FEE,
                UOM_DENOM,
            )], // Combined fee and asset
            |result| {
                result.unwrap();
            },
        )
        .manage_farm(
            &other,
            FarmAction::Fill {
                params: FarmParams {
                    lp_denom: lp_denom_2.clone(),
                    start_epoch: Some(FARM_4_START_EPOCH),
                    preliminary_end_epoch: None, // Farm 4 has open end initially
                    curve: None,
                    farm_asset: Coin {
                        denom: UUSDY_DENOM.to_string(),
                        amount: Uint128::new(FARM_4_UUSDY_ASSET_AMOUNT),
                    },
                    farm_identifier: Some(RAW_FARM_4_ID.to_string()),
                },
            },
            vec![
                coin(FARM_4_UUSDY_ASSET_AMOUNT, UUSDY_DENOM),
                coin(UOM_FARM_CREATION_FEE, UOM_DENOM),
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
                identifier: Some(CREATOR_POS_1_RAW_ID.to_string()),
                unlocking_duration: DEFAULT_UNLOCKING_DURATION_SECONDS,
                receiver: None,
            },
            vec![coin(CREATOR_LP1_LOCK_AMOUNT, lp_denom_1.clone())],
            |result| {
                result.unwrap();
            },
        )
        .manage_position(
            &creator,
            PositionAction::Create {
                identifier: Some(CREATOR_POS_2_RAW_ID.to_string()),
                unlocking_duration: DEFAULT_UNLOCKING_DURATION_SECONDS,
                receiver: None,
            },
            vec![coin(CREATOR_LP2_LOCK_AMOUNT, lp_denom_2.clone())],
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
            assert_eq!(epoch_response.epoch.id, EPOCH_ID_13);
        });

    // other fills a position
    suite
        .manage_position(
            &other,
            PositionAction::Create {
                identifier: Some(OTHER_POS_1_RAW_ID.to_string()),
                unlocking_duration: DEFAULT_UNLOCKING_DURATION_SECONDS,
                receiver: None,
            },
            vec![coin(OTHER_LP1_LOCK_AMOUNT, lp_denom_1.clone())],
            |result| {
                result.unwrap();
            },
        )
        .manage_position(
            &other,
            PositionAction::Create {
                identifier: Some(OTHER_POS_2_RAW_ID.to_string()),
                unlocking_duration: DEFAULT_UNLOCKING_DURATION_SECONDS,
                receiver: None,
            },
            vec![coin(OTHER_LP2_LOCK_AMOUNT, lp_denom_2.clone())],
            |result| {
                result.unwrap();
            },
        );

    suite
        .add_one_epoch()
        .add_one_epoch()
        .query_current_epoch(|result| {
            let epoch_response = result.unwrap();
            assert_eq!(epoch_response.epoch.id, EPOCH_ID_15);
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
                            denom: UUSDY_DENOM.to_string(),
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
        .query_balance(UUSDY_DENOM.to_string(), &creator, |balance| {
            assert_eq!(
                balance,
                Uint128::new(INITIAL_USER_BALANCE - FARM_1_UUSDY_ASSET_AMOUNT)
            );
        })
        .claim(&creator, vec![], None, |result| {
            result.unwrap();
        })
        .query_balance(UUSDY_DENOM.to_string(), &creator, |balance| {
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
                        denom: UUSDY_DENOM.to_string(),
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
                        denom: UOSMO_DENOM.to_string(),
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

    suite
        .add_one_epoch()
        .add_one_epoch()
        .add_one_epoch()
        .add_one_epoch()
        .query_current_epoch(|result| {
            let epoch_response = result.unwrap();
            assert_eq!(epoch_response.epoch.id, EPOCH_ID_19);
        });

    // other emergency unlocks mid-way farm 2
    suite
        .query_balance(UUSDY_DENOM.to_string(), &other, |balance| {
            // Initial - farm 4 asset amount (other created farm 4)
            assert_eq!(
                balance,
                Uint128::new(INITIAL_USER_BALANCE - FARM_4_UUSDY_ASSET_AMOUNT)
            );
        })
        .query_balance(UOSMO_DENOM.to_string(), &other, |balance| {
            assert_eq!(balance, Uint128::new(INITIAL_USER_BALANCE));
        })
        .claim(&other, vec![], None, |result| {
            result.unwrap();
        })
        .query_balance(UUSDY_DENOM.to_string(), &other, |balance| {
            assert_eq!(balance, Uint128::new(999_951_332)); // Derived
        })
        .query_balance(UOSMO_DENOM.to_string(), &other, |balance| {
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
                        denom: UUSDY_DENOM.to_string(),
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
                        denom: UOSMO_DENOM.to_string(),
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
                identifier: U_OTHER_POS_1_ID.to_string(),
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
                identifier: U_OTHER_POS_2_ID.to_string(),
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
                assert_eq!(
                    balance,
                    Uint128::new(EXPECTED_FEE_COLLECTED_LP1_EMERGENCY_UNLOCK)
                );
            },
        )
        .query_balance(
            lp_denom_2.clone().to_string(),
            &fee_collector_addr,
            |balance| {
                assert_eq!(
                    balance,
                    Uint128::new(EXPECTED_FEE_COLLECTED_LP2_EMERGENCY_UNLOCK)
                );
            },
        );

    // at this point, other doesn't have any positions, and creator owns 100% of the weight

    suite.add_one_epoch().query_current_epoch(|result| {
        let epoch_response = result.unwrap();
        assert_eq!(epoch_response.epoch.id, EPOCH_ID_20);
    });

    // another fills a position
    suite.manage_position(
        &another,
        PositionAction::Create {
            identifier: Some(ANOTHER_POS_1_RAW_ID.to_string()),
            unlocking_duration: SIX_MONTHS_UNLOCKING_DURATION_SECONDS,
            receiver: None,
        },
        vec![coin(ANOTHER_LP2_LOCK_AMOUNT, lp_denom_2.clone())],
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
            assert_eq!(epoch_response.epoch.id, EPOCH_ID_30);
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
                        denom: UUSDY_DENOM.to_string(),
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
                        denom: UOSMO_DENOM.to_string(),
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
                        denom: UOM_DENOM.to_string(),
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
                        denom: UUSDY_DENOM.to_string(),
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
                        denom: UUSDY_DENOM.to_string(),
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
                        denom: UOSMO_DENOM.to_string(),
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
                        denom: UOM_DENOM.to_string(),
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
                        denom: UUSDY_DENOM.to_string(),
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
            identifier: U_ANOTHER_POS_1_ID.to_string(),
            lp_asset: Some(coin(ANOTHER_LP2_PARTIAL_UNLOCK_AMOUNT, lp_denom_2.clone())),
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
            assert_eq!(epoch_response.epoch.id, EPOCH_ID_35);
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
                        denom: UUSDY_DENOM.to_string(),
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
                        denom: UUSDY_DENOM.to_string(),
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

    suite
        .add_one_epoch()
        .add_one_epoch()
        .add_one_epoch()
        .add_one_epoch()
        .add_one_epoch()
        .query_current_epoch(|result| {
            let epoch_response = result.unwrap();
            assert_eq!(epoch_response.epoch.id, FINAL_EPOCH_ID_FOR_TEST);
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
