extern crate core;

use cosmwasm_std::{coin, Coin, Uint128};
use farm_manager::ContractError;
use mantra_dex_std::constants::LP_SYMBOL;
use mantra_dex_std::farm_manager::{Curve, Farm, FarmAction, FarmParams, FarmsBy};

use crate::common::suite::TestingSuite;
use crate::common::{MOCK_CONTRACT_ADDR_1, MOCK_CONTRACT_ADDR_2};
use test_utils::common_constants::{
    DENOM_UOM, DENOM_UOSMO, DENOM_UUSDY, INITIAL_BALANCE, ONE_THOUSAND,
};

const FARM_ASSET_AMOUNT_4K: u128 = 4_000u128;
const FARM_ASSET_AMOUNT_8K: u128 = 8_000u128;
const FARM_ASSET_AMOUNT_10K: u128 = 10_000u128;

const START_EPOCH_12: u64 = 12;
const START_EPOCH_20: u64 = 20;
const END_EPOCH_16: u64 = 16;
const END_EPOCH_28: u64 = 28;

const FARM_ID_1: &str = "farm_1";
const FARM_ID_2: &str = "farm_2";
const FARM_ID_DUPLICATE: &str = "duplicate_farm";

const EXPECTED_FARM_ID_1: &str = "m-farm_1";
const EXPECTED_FARM_ID_DUPLICATE: &str = "m-duplicate_farm";
const EXPECTED_AUTO_FARM_ID_1: &str = "f-1";

#[test]
fn test_create_farm_success() {
    let lp_denom = format!("factory/{MOCK_CONTRACT_ADDR_1}/{LP_SYMBOL}").to_string();

    let mut suite = TestingSuite::default_with_balances(vec![
        coin(INITIAL_BALANCE, DENOM_UOM.to_string()),
        coin(INITIAL_BALANCE, DENOM_UUSDY.to_string()),
        coin(INITIAL_BALANCE, DENOM_UOSMO.to_string()),
        coin(INITIAL_BALANCE, lp_denom.clone()),
    ]);

    let creator = suite.creator();
    let other = suite.senders[1].clone();

    suite.instantiate_default();

    for _ in 0..10 {
        suite.add_one_epoch();
    }

    // Test creating farm with explicit identifier
    suite
        .manage_farm(
            &creator,
            FarmAction::Create {
                params: FarmParams {
                    lp_denom: lp_denom.clone(),
                    start_epoch: Some(START_EPOCH_20),
                    preliminary_end_epoch: Some(END_EPOCH_28),
                    curve: None,
                    farm_asset: Coin {
                        denom: DENOM_UUSDY.to_string(),
                        amount: Uint128::new(FARM_ASSET_AMOUNT_4K),
                    },
                    farm_identifier: Some(FARM_ID_1.to_string()),
                },
            },
            vec![
                coin(FARM_ASSET_AMOUNT_4K, DENOM_UUSDY),
                coin(ONE_THOUSAND, DENOM_UOM),
            ],
            |result| {
                result.unwrap();
            },
        )
        .query_farms(
            Some(FarmsBy::Identifier(EXPECTED_FARM_ID_1.to_string())),
            None,
            None,
            |result| {
                let farms_response = result.unwrap();
                assert_eq!(farms_response.farms.len(), 1);
                assert_eq!(
                    farms_response.farms[0],
                    Farm {
                        identifier: EXPECTED_FARM_ID_1.to_string(),
                        owner: creator.clone(),
                        lp_denom: lp_denom.clone(),
                        farm_asset: Coin {
                            denom: DENOM_UUSDY.to_string(),
                            amount: Uint128::new(FARM_ASSET_AMOUNT_4K),
                        },
                        claimed_amount: Uint128::zero(),
                        emission_rate: Uint128::new(500), // 4000 / (28-20) = 500
                        curve: Curve::Linear,
                        start_epoch: START_EPOCH_20,
                        preliminary_end_epoch: END_EPOCH_28,
                    }
                );
            },
        );

    // Test creating farm with auto-generated identifier
    suite
        .manage_farm(
            &other,
            FarmAction::Create {
                params: FarmParams {
                    lp_denom: lp_denom.clone(),
                    start_epoch: Some(START_EPOCH_20),
                    preliminary_end_epoch: Some(END_EPOCH_28),
                    curve: None,
                    farm_asset: Coin {
                        denom: DENOM_UUSDY.to_string(),
                        amount: Uint128::new(FARM_ASSET_AMOUNT_8K),
                    },
                    farm_identifier: None,
                },
            },
            vec![
                coin(FARM_ASSET_AMOUNT_8K, DENOM_UUSDY),
                coin(ONE_THOUSAND, DENOM_UOM),
            ],
            |result| {
                result.unwrap();
            },
        )
        .query_farms(
            Some(FarmsBy::Identifier(EXPECTED_AUTO_FARM_ID_1.to_string())),
            None,
            None,
            |result| {
                let farms_response = result.unwrap();
                assert_eq!(farms_response.farms.len(), 1);
                assert_eq!(
                    farms_response.farms[0],
                    Farm {
                        identifier: EXPECTED_AUTO_FARM_ID_1.to_string(),
                        owner: other.clone(),
                        lp_denom: lp_denom.clone(),
                        farm_asset: Coin {
                            denom: DENOM_UUSDY.to_string(),
                            amount: Uint128::new(FARM_ASSET_AMOUNT_8K),
                        },
                        claimed_amount: Uint128::zero(),
                        emission_rate: Uint128::new(1000), // 8000 / (28-20) = 1000
                        curve: Curve::Linear,
                        start_epoch: START_EPOCH_20,
                        preliminary_end_epoch: END_EPOCH_28,
                    }
                );
            },
        );

    // Verify we have two farms total
    suite.query_farms(None, None, None, |result| {
        let farms_response = result.unwrap();
        assert_eq!(farms_response.farms.len(), 2);
    });
}

#[test]
fn test_create_farm_duplicate_identifier_fails() {
    let lp_denom = format!("factory/{MOCK_CONTRACT_ADDR_1}/{LP_SYMBOL}").to_string();

    let mut suite = TestingSuite::default_with_balances(vec![
        coin(INITIAL_BALANCE, DENOM_UOM.to_string()),
        coin(INITIAL_BALANCE, DENOM_UUSDY.to_string()),
        coin(INITIAL_BALANCE, lp_denom.clone()),
    ]);

    let creator = suite.creator();

    suite.instantiate_default();

    for _ in 0..10 {
        suite.add_one_epoch();
    }

    // Create first farm
    suite.manage_farm(
        &creator,
        FarmAction::Create {
            params: FarmParams {
                lp_denom: lp_denom.clone(),
                start_epoch: Some(START_EPOCH_20),
                preliminary_end_epoch: Some(END_EPOCH_28),
                curve: None,
                farm_asset: Coin {
                    denom: DENOM_UUSDY.to_string(),
                    amount: Uint128::new(FARM_ASSET_AMOUNT_4K),
                },
                farm_identifier: Some(FARM_ID_DUPLICATE.to_string()),
            },
        },
        vec![
            coin(FARM_ASSET_AMOUNT_4K, DENOM_UUSDY),
            coin(ONE_THOUSAND, DENOM_UOM),
        ],
        |result| {
            result.unwrap();
        },
    );

    // Try to create second farm with same identifier - should fail
    suite.manage_farm(
        &creator,
        FarmAction::Create {
            params: FarmParams {
                lp_denom: lp_denom.clone(),
                start_epoch: Some(START_EPOCH_20),
                preliminary_end_epoch: Some(END_EPOCH_28),
                curve: None,
                farm_asset: Coin {
                    denom: DENOM_UUSDY.to_string(),
                    amount: Uint128::new(FARM_ASSET_AMOUNT_4K),
                },
                farm_identifier: Some(FARM_ID_DUPLICATE.to_string()),
            },
        },
        vec![
            coin(FARM_ASSET_AMOUNT_4K, DENOM_UUSDY),
            coin(ONE_THOUSAND, DENOM_UOM),
        ],
        |result| {
            let err = result.unwrap_err().downcast::<ContractError>().unwrap();
            match err {
                ContractError::FarmAlreadyExists => {}
                _ => panic!("Wrong error type, should return ContractError::FarmAlreadyExists"),
            }
        },
    );

    // Verify only one farm exists
    suite.query_farms(None, None, None, |result| {
        let farms_response = result.unwrap();
        assert_eq!(farms_response.farms.len(), 1);
        assert_eq!(
            farms_response.farms[0].identifier,
            EXPECTED_FARM_ID_DUPLICATE
        );
    });
}

#[test]
fn test_create_farm_validates_parameters() {
    let lp_denom = format!("factory/{MOCK_CONTRACT_ADDR_1}/{LP_SYMBOL}").to_string();
    let invalid_lp_denom = format!("factory/{MOCK_CONTRACT_ADDR_2}/{LP_SYMBOL}").to_string();

    let mut suite = TestingSuite::default_with_balances(vec![
        coin(INITIAL_BALANCE, DENOM_UOM.to_string()),
        coin(INITIAL_BALANCE, DENOM_UUSDY.to_string()),
        coin(INITIAL_BALANCE, lp_denom.clone()),
        coin(INITIAL_BALANCE, invalid_lp_denom.clone()),
    ]);

    let creator = suite.creator();

    suite.instantiate_default();

    for _ in 0..10 {
        suite.add_one_epoch();
    }

    // Test invalid LP denom
    suite
        .manage_farm(
            &creator,
            FarmAction::Create {
                params: FarmParams {
                    lp_denom: invalid_lp_denom.clone(),
                    start_epoch: Some(START_EPOCH_20),
                    preliminary_end_epoch: Some(END_EPOCH_28),
                    curve: None,
                    farm_asset: Coin {
                        denom: DENOM_UUSDY.to_string(),
                        amount: Uint128::new(FARM_ASSET_AMOUNT_4K),
                    },
                    farm_identifier: Some(FARM_ID_1.to_string()),
                },
            },
            vec![
                coin(FARM_ASSET_AMOUNT_4K, DENOM_UUSDY),
                coin(ONE_THOUSAND, DENOM_UOM),
            ],
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();
                match err {
                    ContractError::AssetMismatch => {}
                    _ => panic!("Wrong error type, should return ContractError::AssetMismatch"),
                }
            },
        )
        // Test insufficient farm amount
        .manage_farm(
            &creator,
            FarmAction::Create {
                params: FarmParams {
                    lp_denom: lp_denom.clone(),
                    start_epoch: Some(START_EPOCH_20),
                    preliminary_end_epoch: Some(END_EPOCH_28),
                    curve: None,
                    farm_asset: Coin {
                        denom: DENOM_UUSDY.to_string(),
                        amount: Uint128::new(500), // Less than MIN_FARM_AMOUNT (1000)
                    },
                    farm_identifier: Some(FARM_ID_1.to_string()),
                },
            },
            vec![coin(500, DENOM_UUSDY), coin(ONE_THOUSAND, DENOM_UOM)],
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();
                match err {
                    ContractError::InvalidFarmAmount { .. } => {}
                    _ => panic!("Wrong error type, should return ContractError::InvalidFarmAmount"),
                }
            },
        )
        // Test missing farm creation fee
        .manage_farm(
            &creator,
            FarmAction::Create {
                params: FarmParams {
                    lp_denom: lp_denom.clone(),
                    start_epoch: Some(START_EPOCH_20),
                    preliminary_end_epoch: Some(END_EPOCH_28),
                    curve: None,
                    farm_asset: Coin {
                        denom: DENOM_UUSDY.to_string(),
                        amount: Uint128::new(FARM_ASSET_AMOUNT_4K),
                    },
                    farm_identifier: Some(FARM_ID_1.to_string()),
                },
            },
            vec![coin(FARM_ASSET_AMOUNT_4K, DENOM_UUSDY)], // Missing fee
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();
                match err {
                    ContractError::FarmFeeMissing => {}
                    _ => panic!("Wrong error type, should return ContractError::FarmFeeMissing"),
                }
            },
        )
        // Test asset mismatch
        .manage_farm(
            &creator,
            FarmAction::Create {
                params: FarmParams {
                    lp_denom: lp_denom.clone(),
                    start_epoch: Some(START_EPOCH_20),
                    preliminary_end_epoch: Some(END_EPOCH_28),
                    curve: None,
                    farm_asset: Coin {
                        denom: DENOM_UUSDY.to_string(),
                        amount: Uint128::new(FARM_ASSET_AMOUNT_4K),
                    },
                    farm_identifier: Some(FARM_ID_1.to_string()),
                },
            },
            vec![
                coin(FARM_ASSET_AMOUNT_8K, DENOM_UUSDY), // Wrong amount
                coin(ONE_THOUSAND, DENOM_UOM),
            ],
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();
                match err {
                    ContractError::AssetMismatch => {}
                    _ => panic!("Wrong error type, should return ContractError::AssetMismatch"),
                }
            },
        )
        // Test invalid epoch range
        .manage_farm(
            &creator,
            FarmAction::Create {
                params: FarmParams {
                    lp_denom: lp_denom.clone(),
                    start_epoch: Some(START_EPOCH_20),
                    preliminary_end_epoch: Some(START_EPOCH_12), // End before start
                    curve: None,
                    farm_asset: Coin {
                        denom: DENOM_UUSDY.to_string(),
                        amount: Uint128::new(FARM_ASSET_AMOUNT_4K),
                    },
                    farm_identifier: Some(FARM_ID_1.to_string()),
                },
            },
            vec![
                coin(FARM_ASSET_AMOUNT_4K, DENOM_UUSDY),
                coin(ONE_THOUSAND, DENOM_UOM),
            ],
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();
                match err {
                    ContractError::FarmStartTimeAfterEndTime => {}
                    _ => panic!(
                        "Wrong error type, should return ContractError::FarmStartTimeAfterEndTime"
                    ),
                }
            },
        );
}

#[test]
fn test_create_farm_respects_max_concurrent_farms() {
    let lp_denom = format!("factory/{MOCK_CONTRACT_ADDR_1}/{LP_SYMBOL}").to_string();

    let mut suite = TestingSuite::default_with_balances(vec![
        coin(INITIAL_BALANCE, DENOM_UOM.to_string()),
        coin(INITIAL_BALANCE, DENOM_UUSDY.to_string()),
        coin(INITIAL_BALANCE, lp_denom.clone()),
    ]);

    let creator = suite.creator();

    suite.instantiate_default(); // Max concurrent farms is 2

    for _ in 0..10 {
        suite.add_one_epoch();
    }

    // Create first farm
    suite
        .manage_farm(
            &creator,
            FarmAction::Create {
                params: FarmParams {
                    lp_denom: lp_denom.clone(),
                    start_epoch: Some(START_EPOCH_20),
                    preliminary_end_epoch: Some(END_EPOCH_28),
                    curve: None,
                    farm_asset: Coin {
                        denom: DENOM_UUSDY.to_string(),
                        amount: Uint128::new(FARM_ASSET_AMOUNT_4K),
                    },
                    farm_identifier: Some(FARM_ID_1.to_string()),
                },
            },
            vec![
                coin(FARM_ASSET_AMOUNT_4K, DENOM_UUSDY),
                coin(ONE_THOUSAND, DENOM_UOM),
            ],
            |result| {
                result.unwrap();
            },
        )
        // Create second farm
        .manage_farm(
            &creator,
            FarmAction::Create {
                params: FarmParams {
                    lp_denom: lp_denom.clone(),
                    start_epoch: Some(START_EPOCH_20),
                    preliminary_end_epoch: Some(END_EPOCH_28),
                    curve: None,
                    farm_asset: Coin {
                        denom: DENOM_UUSDY.to_string(),
                        amount: Uint128::new(FARM_ASSET_AMOUNT_4K),
                    },
                    farm_identifier: Some(FARM_ID_2.to_string()),
                },
            },
            vec![
                coin(FARM_ASSET_AMOUNT_4K, DENOM_UUSDY),
                coin(ONE_THOUSAND, DENOM_UOM),
            ],
            |result| {
                result.unwrap();
            },
        )
        // Try to create third farm - should fail due to max concurrent farms limit
        .manage_farm(
            &creator,
            FarmAction::Create {
                params: FarmParams {
                    lp_denom: lp_denom.clone(),
                    start_epoch: Some(START_EPOCH_20),
                    preliminary_end_epoch: Some(END_EPOCH_28),
                    curve: None,
                    farm_asset: Coin {
                        denom: DENOM_UUSDY.to_string(),
                        amount: Uint128::new(FARM_ASSET_AMOUNT_4K),
                    },
                    farm_identifier: Some("farm_3".to_string()),
                },
            },
            vec![
                coin(FARM_ASSET_AMOUNT_4K, DENOM_UUSDY),
                coin(ONE_THOUSAND, DENOM_UOM),
            ],
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();
                match err {
                    ContractError::TooManyFarms { .. } => {}
                    _ => panic!("Wrong error type, should return ContractError::TooManyFarms"),
                }
            },
        );

    // Verify we have exactly 2 farms
    suite.query_farms(None, None, None, |result| {
        let farms_response = result.unwrap();
        assert_eq!(farms_response.farms.len(), 2);
    });
}

#[test]
fn test_create_farm_closes_expired_farms() {
    let lp_denom = format!("factory/{MOCK_CONTRACT_ADDR_1}/{LP_SYMBOL}").to_string();

    let mut suite = TestingSuite::default_with_balances(vec![
        coin(INITIAL_BALANCE, DENOM_UOM.to_string()),
        coin(INITIAL_BALANCE, DENOM_UUSDY.to_string()),
        coin(INITIAL_BALANCE, lp_denom.clone()),
    ]);

    let creator = suite.creator();

    suite.instantiate_default();

    for _ in 0..10 {
        suite.add_one_epoch();
    }

    // Create first farm that will expire
    suite.manage_farm(
        &creator,
        FarmAction::Create {
            params: FarmParams {
                lp_denom: lp_denom.clone(),
                start_epoch: Some(START_EPOCH_12),
                preliminary_end_epoch: Some(END_EPOCH_16),
                curve: None,
                farm_asset: Coin {
                    denom: DENOM_UUSDY.to_string(),
                    amount: Uint128::new(FARM_ASSET_AMOUNT_4K),
                },
                farm_identifier: Some(FARM_ID_1.to_string()),
            },
        },
        vec![
            coin(FARM_ASSET_AMOUNT_4K, DENOM_UUSDY),
            coin(ONE_THOUSAND, DENOM_UOM),
        ],
        |result| {
            result.unwrap();
        },
    );

    // Create enough epochs to make the farm expire
    for _ in 0..=37 {
        suite.add_one_epoch();
    }

    suite.query_current_epoch(|result| {
        let epoch_response = result.unwrap();
        assert_eq!(epoch_response.epoch.id, 48);
    });

    // Create new farm - should close the expired one automatically
    suite
        .query_farms(None, None, None, |result| {
            let farms_response = result.unwrap();
            assert_eq!(farms_response.farms.len(), 1); // Old farm still exists
            assert_eq!(farms_response.farms[0].identifier, EXPECTED_FARM_ID_1);
        })
        .manage_farm(
            &creator,
            FarmAction::Create {
                params: FarmParams {
                    lp_denom: lp_denom.clone(),
                    start_epoch: None,
                    preliminary_end_epoch: None,
                    curve: None,
                    farm_asset: Coin {
                        denom: DENOM_UUSDY.to_string(),
                        amount: Uint128::new(FARM_ASSET_AMOUNT_10K),
                    },
                    farm_identifier: Some("new_farm".to_string()),
                },
            },
            vec![
                coin(FARM_ASSET_AMOUNT_10K, DENOM_UUSDY),
                coin(ONE_THOUSAND, DENOM_UOM),
            ],
            |result| {
                result.unwrap();
            },
        )
        .query_farms(None, None, None, |result| {
            let farms_response = result.unwrap();
            assert_eq!(farms_response.farms.len(), 1); // Only new farm exists
            assert_eq!(farms_response.farms[0].identifier, "m-new_farm");
        });
}

#[test]
fn test_create_vs_fill_different_behavior() {
    let lp_denom = format!("factory/{MOCK_CONTRACT_ADDR_1}/{LP_SYMBOL}").to_string();

    let mut suite = TestingSuite::default_with_balances(vec![
        coin(INITIAL_BALANCE, DENOM_UOM.to_string()),
        coin(INITIAL_BALANCE, DENOM_UUSDY.to_string()),
        coin(INITIAL_BALANCE, lp_denom.clone()),
    ]);

    let creator = suite.creator();

    suite.instantiate_default();

    for _ in 0..10 {
        suite.add_one_epoch();
    }

    // Create a farm using Create action
    suite.manage_farm(
        &creator,
        FarmAction::Create {
            params: FarmParams {
                lp_denom: lp_denom.clone(),
                start_epoch: Some(START_EPOCH_20),
                preliminary_end_epoch: Some(END_EPOCH_28),
                curve: None,
                farm_asset: Coin {
                    denom: DENOM_UUSDY.to_string(),
                    amount: Uint128::new(FARM_ASSET_AMOUNT_4K),
                },
                farm_identifier: Some(FARM_ID_1.to_string()),
            },
        },
        vec![
            coin(FARM_ASSET_AMOUNT_4K, DENOM_UUSDY),
            coin(ONE_THOUSAND, DENOM_UOM),
        ],
        |result| {
            result.unwrap();
        },
    );

    // Try to use Fill action with same identifier - should expand existing farm
    suite.manage_farm(
        &creator,
        FarmAction::Expand {
            params: FarmParams {
                lp_denom: lp_denom.clone(),
                start_epoch: Some(START_EPOCH_20),
                preliminary_end_epoch: Some(END_EPOCH_28),
                curve: None,
                farm_asset: Coin {
                    denom: DENOM_UUSDY.to_string(),
                    amount: Uint128::new(FARM_ASSET_AMOUNT_4K),
                },
                farm_identifier: Some(EXPECTED_FARM_ID_1.to_string()),
            },
        },
        vec![coin(FARM_ASSET_AMOUNT_4K, DENOM_UUSDY)],
        |result| {
            result.unwrap();
        },
    );

    // Verify farm was expanded, not recreated
    suite.query_farms(
        Some(FarmsBy::Identifier(EXPECTED_FARM_ID_1.to_string())),
        None,
        None,
        |result| {
            let farms_response = result.unwrap();
            assert_eq!(farms_response.farms.len(), 1);
            assert_eq!(
                farms_response.farms[0].farm_asset.amount,
                Uint128::new(FARM_ASSET_AMOUNT_4K * 2) // Expanded to 8k
            );
        },
    );

    // Try to use Fill action without identifier - should fail
    suite.manage_farm(
        &creator,
        FarmAction::Expand {
            params: FarmParams {
                lp_denom: lp_denom.clone(),
                start_epoch: Some(START_EPOCH_20),
                preliminary_end_epoch: Some(END_EPOCH_28),
                curve: None,
                farm_asset: Coin {
                    denom: DENOM_UUSDY.to_string(),
                    amount: Uint128::new(FARM_ASSET_AMOUNT_4K),
                },
                farm_identifier: None, // No identifier
            },
        },
        vec![coin(FARM_ASSET_AMOUNT_4K, DENOM_UUSDY)],
        |result| {
            let err = result.unwrap_err().downcast::<ContractError>().unwrap();
            match err {
                ContractError::NonExistentFarm => {}
                _ => panic!("Wrong error type, should return ContractError::NonExistentFarm"),
            }
        },
    );

    // Try to use Create action with same identifier again - should fail
    suite.manage_farm(
        &creator,
        FarmAction::Create {
            params: FarmParams {
                lp_denom: lp_denom.clone(),
                start_epoch: Some(START_EPOCH_20),
                preliminary_end_epoch: Some(END_EPOCH_28),
                curve: None,
                farm_asset: Coin {
                    denom: DENOM_UUSDY.to_string(),
                    amount: Uint128::new(FARM_ASSET_AMOUNT_4K),
                },
                farm_identifier: Some(FARM_ID_1.to_string()),
            },
        },
        vec![
            coin(FARM_ASSET_AMOUNT_4K, DENOM_UUSDY),
            coin(ONE_THOUSAND, DENOM_UOM),
        ],
        |result| {
            let err = result.unwrap_err().downcast::<ContractError>().unwrap();
            match err {
                ContractError::FarmAlreadyExists => {}
                _ => panic!("Wrong error type, should return ContractError::FarmAlreadyExists"),
            }
        },
    );
}

#[test]
fn test_create_farm_resolves_dos_issue() {
    let lp_denom = format!("factory/{MOCK_CONTRACT_ADDR_1}/{LP_SYMBOL}").to_string();

    let mut suite = TestingSuite::default_with_balances(vec![
        coin(INITIAL_BALANCE, DENOM_UOM.to_string()),
        coin(INITIAL_BALANCE, DENOM_UUSDY.to_string()),
        coin(INITIAL_BALANCE, lp_denom.clone()),
    ]);

    let user1 = suite.creator();
    let user2 = suite.senders[1].clone();

    suite.instantiate_default();

    for _ in 0..10 {
        suite.add_one_epoch();
    }

    // User1 creates a farm with identifier "raj" (which becomes "m-raj")
    suite.manage_farm(
        &user1,
        FarmAction::Create {
            params: FarmParams {
                lp_denom: lp_denom.clone(),
                start_epoch: Some(START_EPOCH_20),
                preliminary_end_epoch: Some(END_EPOCH_28),
                curve: None,
                farm_asset: Coin {
                    denom: DENOM_UUSDY.to_string(),
                    amount: Uint128::new(FARM_ASSET_AMOUNT_4K),
                },
                farm_identifier: Some("raj".to_string()),
            },
        },
        vec![
            coin(FARM_ASSET_AMOUNT_4K, DENOM_UUSDY),
            coin(ONE_THOUSAND, DENOM_UOM),
        ],
        |result| {
            result.unwrap();
        },
    );

    // User2 wants to create a farm with identifier "m-raj" (which should become "m-m-raj")
    // In the old system, this would fail because "raj" exists and the system would try to expand it
    // In the new system with separate Create action, this should succeed
    suite.manage_farm(
        &user2,
        FarmAction::Create {
            params: FarmParams {
                lp_denom: lp_denom.clone(),
                start_epoch: Some(START_EPOCH_20),
                preliminary_end_epoch: Some(END_EPOCH_28),
                curve: None,
                farm_asset: Coin {
                    denom: DENOM_UUSDY.to_string(),
                    amount: Uint128::new(FARM_ASSET_AMOUNT_4K),
                },
                farm_identifier: Some("m-raj".to_string()), // This becomes "m-m-raj"
            },
        },
        vec![
            coin(FARM_ASSET_AMOUNT_4K, DENOM_UUSDY),
            coin(ONE_THOUSAND, DENOM_UOM),
        ],
        |result| {
            result.unwrap(); // Should succeed - no DoS
        },
    );

    // Verify both farms exist with different identifiers
    suite
        .query_farms(None, None, None, |result| {
            let farms_response = result.unwrap();
            assert_eq!(farms_response.farms.len(), 2);

            let farm_ids: Vec<String> = farms_response
                .farms
                .iter()
                .map(|f| f.identifier.clone())
                .collect();
            assert!(farm_ids.contains(&"m-raj".to_string()));
            assert!(farm_ids.contains(&"m-m-raj".to_string()));
        })
        .query_farms(
            Some(FarmsBy::Identifier("m-raj".to_string())),
            None,
            None,
            |result| {
                let farms_response = result.unwrap();
                assert_eq!(farms_response.farms.len(), 1);
                assert_eq!(farms_response.farms[0].owner, user1);
            },
        )
        .query_farms(
            Some(FarmsBy::Identifier("m-m-raj".to_string())),
            None,
            None,
            |result| {
                let farms_response = result.unwrap();
                assert_eq!(farms_response.farms.len(), 1);
                assert_eq!(farms_response.farms[0].owner, user2);
            },
        );

    // The key test: user2 was able to create a farm with identifier "m-raj"
    // even though "raj" already exists. This proves the DoS issue is resolved.
    // In the old system, this would have failed because it would try to expand
    // the existing "raj" farm instead of creating a new "m-raj" farm.

    // Verify that both unique farms exist with their correct identifiers
    suite.query_farms(None, None, None, |result| {
        let farms_response = result.unwrap();
        assert_eq!(farms_response.farms.len(), 2);

        // Check that we have farms with the expected identifiers
        let farm_ids: Vec<String> = farms_response
            .farms
            .iter()
            .map(|f| f.identifier.clone())
            .collect();
        assert!(farm_ids.contains(&"m-raj".to_string()));
        assert!(farm_ids.contains(&"m-m-raj".to_string()));

        // Verify they have different owners
        let raj_farm = farms_response
            .farms
            .iter()
            .find(|f| f.identifier == "m-raj")
            .unwrap();
        let m_raj_farm = farms_response
            .farms
            .iter()
            .find(|f| f.identifier == "m-m-raj")
            .unwrap();
        assert_eq!(raj_farm.owner, user1);
        assert_eq!(m_raj_farm.owner, user2);
    });
}
