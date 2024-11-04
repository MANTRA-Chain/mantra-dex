extern crate core;

use std::cell::RefCell;

use amm::constants::{LP_SYMBOL, MONTH_IN_SECONDS};
use amm::farm_manager::{
    Config, Curve, Farm, FarmAction, FarmParams, FarmsBy, LpWeightResponse, Position,
    PositionAction, PositionsBy, PositionsResponse, RewardsResponse,
};
use cosmwasm_std::{coin, Addr, Coin, Decimal, StdResult, Timestamp, Uint128};
use cw_utils::PaymentError;
use farm_manager::state::MAX_ITEMS_LIMIT;
use farm_manager::ContractError;

use crate::common::suite::TestingSuite;
use crate::common::{MOCK_CONTRACT_ADDR_1, MOCK_CONTRACT_ADDR_2};

mod common;

#[test]
fn instantiate_farm_manager() {
    let mut suite =
        TestingSuite::default_with_balances(vec![coin(1_000_000_000u128, "uom".to_string())]);

    suite.instantiate_err(
        MOCK_CONTRACT_ADDR_1.to_string(),
        MOCK_CONTRACT_ADDR_1.to_string(),
        MOCK_CONTRACT_ADDR_1.to_string(),
        Coin {
            denom: "uom".to_string(),
            amount: Uint128::new(1_000u128),
        },
        0,
        14,
        86_400,
        31_536_000,
        MONTH_IN_SECONDS,
        Decimal::percent(10),
        |result| {
            let err = result.unwrap_err().downcast::<ContractError>().unwrap();

            match err {
                ContractError::UnspecifiedConcurrentFarms { .. } => {}
                _ => panic!("Wrong error type, should return ContractError::UnspecifiedConcurrentFarms"),
            }
        },
    ).instantiate_err(
        MOCK_CONTRACT_ADDR_1.to_string(),
        MOCK_CONTRACT_ADDR_1.to_string(),
        MOCK_CONTRACT_ADDR_1.to_string(),
        Coin {
            denom: "uom".to_string(),
            amount: Uint128::new(1_000u128),
        },
        1,
        14,
        86_400,
        86_399,
        MONTH_IN_SECONDS,
        Decimal::percent(10),
        |result| {
            let err = result.unwrap_err().downcast::<ContractError>().unwrap();

            match err {
                ContractError::InvalidUnlockingRange { .. } => {}
                _ => panic!("Wrong error type, should return ContractError::InvalidUnlockingRange"),
            }
        },
    ).instantiate_err(
        MOCK_CONTRACT_ADDR_1.to_string(),
        MOCK_CONTRACT_ADDR_1.to_string(),
        MOCK_CONTRACT_ADDR_1.to_string(),
        Coin {
            denom: "uom".to_string(),
            amount: Uint128::new(1_000u128),
        },
        1,
        14,
        86_400,
        86_500,
        MONTH_IN_SECONDS,
        Decimal::percent(101),
        |result| {
            let err = result.unwrap_err().downcast::<ContractError>().unwrap();

            match err {
                ContractError::InvalidEmergencyUnlockPenalty { .. } => {}
                _ => panic!("Wrong error type, should return ContractError::InvalidEmergencyUnlockPenalty"),
            }
        },
    ).instantiate_err(
        MOCK_CONTRACT_ADDR_1.to_string(),
        MOCK_CONTRACT_ADDR_1.to_string(),
        MOCK_CONTRACT_ADDR_1.to_string(),
        Coin {
            denom: "uom".to_string(),
            amount: Uint128::new(1_000u128),
        },
        1,
        14,
        86_400,
        86_500,
        MONTH_IN_SECONDS - 1,
        Decimal::percent(101),
        |result| {
            let err = result.unwrap_err().downcast::<ContractError>().unwrap();

            match err {
                ContractError::FarmExpirationTimeInvalid { .. } => {}
                _ => panic!("Wrong error type, should return ContractError::FarmExpirationTimeInvalid"),
            }
        },
    ).instantiate(
        MOCK_CONTRACT_ADDR_1.to_string(),
        MOCK_CONTRACT_ADDR_1.to_string(),
        MOCK_CONTRACT_ADDR_1.to_string(),
        Coin {
            denom: "uom".to_string(),
            amount: Uint128::new(1_000u128),
        },
        7,
        14,
        86_400,
        31_536_000,
        MONTH_IN_SECONDS,
        Decimal::percent(10), //10% penalty
    );
}

#[test]
fn create_farms() {
    let lp_denom = format!("factory/{MOCK_CONTRACT_ADDR_1}/{LP_SYMBOL}").to_string();
    let invalid_lp_denom = format!("factory/{MOCK_CONTRACT_ADDR_2}/{LP_SYMBOL}").to_string();

    let mut suite = TestingSuite::default_with_balances(vec![
        coin(1_000_000_000u128, "uom".to_string()),
        coin(1_000_000_000u128, "uusdy".to_string()),
        coin(1_000_000_000u128, "uosmo".to_string()),
        coin(1_000_000_000u128, lp_denom.clone()),
        coin(1_000_000_000u128, invalid_lp_denom.clone()),
    ]);
    suite.instantiate_default();

    let creator = suite.creator().clone();
    let other = suite.senders[1].clone();
    let fee_collector = suite.fee_collector_addr.clone();

    for _ in 0..10 {
        suite.add_one_epoch();
    }
    // current epoch is 10

    // try all misconfigurations when creating a farm
    suite
        .manage_farm(
            &creator,
            FarmAction::Fill {
                params: FarmParams {
                    lp_denom: lp_denom.clone(),
                    start_epoch: Some(25),
                    preliminary_end_epoch: None,
                    curve: None,
                    farm_asset: Coin {
                        denom: "uusdy".to_string(),
                        amount: Default::default(),
                    },
                    farm_identifier: None,
                },
            },
            vec![],
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();

                match err {
                    ContractError::InvalidFarmAmount { .. } => {}
                    _ => panic!("Wrong error type, should return ContractError::InvalidFarmAmount"),
                }
            },
        )
        .manage_farm(
            &other,
            FarmAction::Fill {
                params: FarmParams {
                    lp_denom: lp_denom.clone(),
                    start_epoch: Some(25),
                    preliminary_end_epoch: None,
                    curve: None,
                    farm_asset: Coin {
                        denom: "uusdy".to_string(),
                        amount: Uint128::new(2_000u128),
                    },
                    farm_identifier: None,
                },
            },
            vec![coin(2_000, "uusdy")],
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();
                match err {
                    ContractError::FarmFeeMissing { .. } => {}
                    _ => {
                        panic!("Wrong error type, should return ContractError::FarmFeeMissing")
                    }
                }
            },
        )
        .manage_farm(
            &other,
            FarmAction::Fill {
                params: FarmParams {
                    lp_denom: lp_denom.clone(),
                    start_epoch: Some(25),
                    preliminary_end_epoch: None,
                    curve: None,
                    farm_asset: Coin {
                        denom: "uom".to_string(),
                        amount: Uint128::new(5_000u128),
                    },
                    farm_identifier: None,
                },
            },
            vec![coin(8_000, "uom")],
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();
                match err {
                    ContractError::AssetMismatch { .. } => {}
                    _ => panic!("Wrong error type, should return ContractError::AssetMismatch"),
                }
            },
        )
        .manage_farm(
            &other,
            FarmAction::Fill {
                params: FarmParams {
                    lp_denom: lp_denom.clone(),
                    start_epoch: Some(25),
                    preliminary_end_epoch: None,
                    curve: None,
                    farm_asset: Coin {
                        denom: "uusdy".to_string(),
                        amount: Uint128::new(2_000u128),
                    },
                    farm_identifier: None,
                },
            },
            vec![coin(1_000, "uom")],
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();
                match err {
                    ContractError::AssetMismatch { .. } => {}
                    _ => panic!("Wrong error type, should return ContractError::AssetMismatch"),
                }
            },
        )
        .manage_farm(
            &other,
            FarmAction::Fill {
                params: FarmParams {
                    lp_denom: lp_denom.clone(),
                    start_epoch: Some(25),
                    preliminary_end_epoch: None,
                    curve: None,
                    farm_asset: Coin {
                        denom: "uusdy".to_string(),
                        amount: Uint128::new(2_000u128),
                    },
                    farm_identifier: None,
                },
            },
            vec![coin(5_000, "uusdy"), coin(1_000, "uom")],
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();
                match err {
                    ContractError::AssetMismatch { .. } => {}
                    _ => panic!("Wrong error type, should return ContractError::AssetMismatch"),
                }
            },
        )
        .manage_farm(
            &other,
            FarmAction::Fill {
                params: FarmParams {
                    lp_denom: lp_denom.clone(),
                    start_epoch: Some(25),
                    preliminary_end_epoch: None,
                    curve: None,
                    farm_asset: Coin {
                        denom: "uusdy".to_string(),
                        amount: Uint128::new(5_000u128),
                    },
                    farm_identifier: None,
                },
            },
            vec![coin(4_000, "uusdy"), coin(1_000, "uom")],
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();
                match err {
                    ContractError::AssetMismatch { .. } => {}
                    _ => panic!("Wrong error type, should return ContractError::AssetMismatch"),
                }
            },
        )
        .manage_farm(
            &other,
            FarmAction::Fill {
                params: FarmParams {
                    lp_denom: lp_denom.clone(),
                    start_epoch: Some(25),
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
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();

                match err {
                    ContractError::FarmStartTooFar { .. } => {}
                    _ => panic!("Wrong error type, should return ContractError::FarmStartTooFar"),
                }
            },
        )
        .manage_farm(
            &other,
            FarmAction::Fill {
                params: FarmParams {
                    lp_denom: lp_denom.clone(),
                    start_epoch: Some(20),
                    preliminary_end_epoch: Some(8),
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
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();

                match err {
                    ContractError::FarmStartTimeAfterEndTime { .. } => {}
                    _ => panic!(
                        "Wrong error type, should return ContractError::FarmStartTimeAfterEndTime"
                    ),
                }
            },
        )
        .manage_farm(
            &other,
            FarmAction::Fill {
                params: FarmParams {
                    lp_denom: lp_denom.clone(),
                    start_epoch: Some(20),
                    preliminary_end_epoch: Some(15),
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
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();

                match err {
                    ContractError::FarmStartTimeAfterEndTime { .. } => {}
                    _ => panic!(
                        "Wrong error type, should return ContractError::FarmStartTimeAfterEndTime"
                    ),
                }
            },
        )
        .manage_farm(
            &other,
            FarmAction::Fill {
                params: FarmParams {
                    lp_denom: lp_denom.clone(),
                    // current epoch is 10
                    start_epoch: Some(3),
                    preliminary_end_epoch: Some(5),
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
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();

                match err {
                    ContractError::FarmEndsInPast { .. } => {}
                    _ => panic!("Wrong error type, should return ContractError::FarmEndsInPast"),
                }
            },
        )
        .manage_farm(
            &other,
            FarmAction::Fill {
                params: FarmParams {
                    lp_denom: lp_denom.clone(),
                    start_epoch: Some(20),
                    preliminary_end_epoch: Some(20),
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
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();

                match err {
                    ContractError::FarmStartTimeAfterEndTime { .. } => {}
                    _ => panic!(
                        "Wrong error type, should return ContractError::FarmStartTimeAfterEndTime"
                    ),
                }
            },
        )
        .manage_farm(
            &other,
            FarmAction::Fill {
                params: FarmParams {
                    lp_denom: lp_denom.clone(),
                    start_epoch: Some(30),
                    preliminary_end_epoch: Some(35),
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
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();

                match err {
                    ContractError::FarmStartTooFar { .. } => {}
                    _ => panic!("Wrong error type, should return ContractError::FarmStartTooFar"),
                }
            },
        )
        .manage_farm(
            &other,
            FarmAction::Fill {
                params: FarmParams {
                    lp_denom: invalid_lp_denom.clone(),
                    start_epoch: Some(20),
                    preliminary_end_epoch: Some(28),
                    curve: None,
                    farm_asset: Coin {
                        denom: "uusdy".to_string(),
                        amount: Uint128::new(4_000u128),
                    },
                    farm_identifier: Some("farm_1".to_string()),
                },
            },
            vec![coin(4_000, "uusdy"), coin(1_000, "uom")],
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();
                // trying to create a farm for an invalid lp_denom, i.e. an lp_denom that wasn't created
                // by the pool manager, should fail
                match err {
                    ContractError::AssetMismatch { .. } => {}
                    _ => panic!("Wrong error type, should return ContractError::AssetMismatch"),
                }
            },
        );

    // create a farm properly
    suite
        .manage_farm(
            &other,
            FarmAction::Fill {
                params: FarmParams {
                    lp_denom: lp_denom.clone(),
                    start_epoch: Some(20),
                    preliminary_end_epoch: Some(28),
                    curve: None,
                    farm_asset: Coin {
                        denom: "uusdy".to_string(),
                        amount: Uint128::new(4_000u128),
                    },
                    farm_identifier: Some("farm_1".to_string()),
                },
            },
            vec![coin(4_000, "uusdy"), coin(1_000, "uom")],
            |result| {
                result.unwrap();
            },
        )
        .manage_farm(
            &other,
            FarmAction::Fill {
                params: FarmParams {
                    lp_denom: lp_denom.clone(),
                    start_epoch: Some(20),
                    preliminary_end_epoch: Some(28),
                    curve: None,
                    farm_asset: Coin {
                        denom: "uusdy".to_string(),
                        amount: Uint128::new(10_000u128),
                    },
                    farm_identifier: None,
                },
            },
            vec![coin(10_000, "uusdy"), coin(1_000, "uom")],
            |result| {
                result.unwrap();
            },
        )
        .manage_farm(
            &other,
            FarmAction::Fill {
                params: FarmParams {
                    lp_denom: lp_denom.clone(),
                    start_epoch: Some(20),
                    preliminary_end_epoch: Some(28),
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
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();
                // should fail, max farms per lp_denom was set to 2 in the instantiate_default
                // function
                match err {
                    ContractError::TooManyFarms { .. } => {}
                    _ => panic!("Wrong error type, should return ContractError::TooManyFarms"),
                }
            },
        )
        .query_farms(None, None, None, |result| {
            let farms_response = result.unwrap();
            assert_eq!(farms_response.farms.len(), 2);
        })
        .query_farms(
            Some(FarmsBy::Identifier("m-farm_1".to_string())),
            None,
            None,
            |result| {
                let farms_response = result.unwrap();
                assert_eq!(farms_response.farms.len(), 1);
                assert_eq!(
                    farms_response.farms[0].farm_asset,
                    Coin {
                        denom: "uusdy".to_string(),
                        amount: Uint128::new(4_000),
                    }
                );
            },
        )
        .query_farms(
            Some(FarmsBy::Identifier("f-1".to_string())),
            None,
            None,
            |result| {
                let farms_response = result.unwrap();
                assert_eq!(farms_response.farms.len(), 1);
                assert_eq!(
                    farms_response.farms[0].farm_asset,
                    Coin {
                        denom: "uusdy".to_string(),
                        amount: Uint128::new(10_000),
                    }
                );
            },
        )
        .query_farms(
            Some(FarmsBy::FarmAsset("uusdy".to_string())),
            None,
            None,
            |result| {
                let farms_response = result.unwrap();
                assert_eq!(farms_response.farms.len(), 2);
            },
        )
        .query_farms(
            Some(FarmsBy::LpDenom(lp_denom.clone())),
            None,
            None,
            |result| {
                let farms_response = result.unwrap();
                assert_eq!(farms_response.farms.len(), 2);
            },
        )
        // two farms were created, therefore the fee collector should have received 2_000 uom
        .query_balance("uom".to_string(), &fee_collector, |balance| {
            assert_eq!(balance, Uint128::new(2 * 1_000));
        });
}

#[test]
fn expand_farms() {
    let lp_denom = format!("factory/{MOCK_CONTRACT_ADDR_1}/{LP_SYMBOL}").to_string();

    let mut suite = TestingSuite::default_with_balances(vec![
        coin(1_000_000_000u128, "uom".to_string()),
        coin(1_000_000_000u128, "uusdy".to_string()),
        coin(1_000_000_000u128, "uosmo".to_string()),
        coin(1_000_000_000u128, lp_denom.clone()),
    ]);

    let creator = suite.creator();
    let other = suite.senders[1].clone();

    suite.instantiate_default();

    for _ in 0..10 {
        suite.add_one_epoch();
    }
    // current epoch is 10

    suite
        .manage_farm(
            &other,
            FarmAction::Fill {
                params: FarmParams {
                    lp_denom: lp_denom.clone(),
                    start_epoch: Some(20),
                    preliminary_end_epoch: Some(28),
                    curve: None,
                    farm_asset: Coin {
                        denom: "uusdy".to_string(),
                        amount: Uint128::new(4_000u128),
                    },
                    farm_identifier: Some("farm_1".to_string()),
                },
            },
            vec![coin(4_000, "uusdy"), coin(1_000, "uom")],
            |result| {
                result.unwrap();
            },
        )
        .manage_farm(
            &creator,
            FarmAction::Fill {
                params: FarmParams {
                    lp_denom: lp_denom.clone(),
                    start_epoch: Some(20),
                    preliminary_end_epoch: Some(28),
                    curve: None,
                    farm_asset: Coin {
                        denom: "uusdy".to_string(),
                        amount: Uint128::new(8_000u128),
                    },
                    farm_identifier: Some("m-farm_1".to_string()),
                },
            },
            vec![coin(4_000, "uusdy")],
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();

                match err {
                    ContractError::Unauthorized { .. } => {}
                    _ => panic!("Wrong error type, should return ContractError::Unauthorized"),
                }
            },
        )
        .manage_farm(
            &other,
            FarmAction::Fill {
                params: FarmParams {
                    lp_denom: lp_denom.clone(),
                    start_epoch: Some(20),
                    preliminary_end_epoch: Some(28),
                    curve: None,
                    farm_asset: Coin {
                        denom: "uom".to_string(),
                        amount: Uint128::new(8_000u128),
                    },
                    farm_identifier: Some("m-farm_1".to_string()),
                },
            },
            vec![coin(8_000, "uom")],
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();
                match err {
                    ContractError::AssetMismatch { .. } => {}
                    _ => panic!("Wrong error type, should return ContractError::AssetMismatch"),
                }
            },
        )
        .manage_farm(
            &other,
            FarmAction::Fill {
                params: FarmParams {
                    lp_denom: lp_denom.clone(),
                    start_epoch: Some(20),
                    preliminary_end_epoch: Some(28),
                    curve: None,
                    farm_asset: Coin {
                        denom: "uusdy".to_string(),
                        amount: Uint128::new(4_100u128),
                    },
                    farm_identifier: Some("m-farm_1".to_string()),
                },
            },
            vec![coin(4_100, "uusdy")],
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();

                match err {
                    ContractError::InvalidExpansionAmount { .. } => {}
                    _ => panic!(
                        "Wrong error type, should return ContractError::InvalidExpansionAmount"
                    ),
                }
            },
        )
        .manage_farm(
            &other,
            FarmAction::Fill {
                params: FarmParams {
                    lp_denom: lp_denom.clone(),
                    start_epoch: Some(20),
                    preliminary_end_epoch: Some(28),
                    curve: None,
                    farm_asset: Coin {
                        denom: "uusdy".to_string(),
                        amount: Uint128::new(4_100u128),
                    },
                    farm_identifier: Some("m-farm_1".to_string()),
                },
            },
            vec![], // sending no funds when expanding a farm should fail
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();

                match err {
                    ContractError::PaymentError(e) => {
                        assert_eq!(e, PaymentError::NoFunds {})
                    }
                    _ => panic!("Wrong error type, should return ContractError::PaymentError"),
                }
            },
        )
        .manage_farm(
            &other,
            FarmAction::Fill {
                params: FarmParams {
                    lp_denom: lp_denom.clone(),
                    start_epoch: Some(20),
                    preliminary_end_epoch: Some(28),
                    curve: None,
                    farm_asset: Coin {
                        denom: "uusdy".to_string(),
                        amount: Uint128::new(4_100u128),
                    },
                    farm_identifier: Some("m-farm_1".to_string()),
                },
            },
            vec![coin(4_100u128, "uom"), coin(4_100u128, "uusdy")], // sending different funds than the one provided in the params should fail
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();

                match err {
                    ContractError::PaymentError(e) => {
                        assert_eq!(e, PaymentError::MultipleDenoms {})
                    }
                    _ => panic!("Wrong error type, should return ContractError::PaymentError"),
                }
            },
        )
        .manage_farm(
            &other,
            FarmAction::Fill {
                params: FarmParams {
                    lp_denom: lp_denom.clone(),
                    start_epoch: Some(20),
                    preliminary_end_epoch: Some(28),
                    curve: None,
                    farm_asset: Coin {
                        denom: "uusdy".to_string(),
                        amount: Uint128::new(4_100u128),
                    },
                    farm_identifier: Some("m-farm_1".to_string()),
                },
            },
            vec![coin(4_100u128, "uom")], // sending different funds than the one provided in the params should fail
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();

                match err {
                    ContractError::AssetMismatch => {}
                    _ => panic!("Wrong error type, should return ContractError::AssetMismatch"),
                }
            },
        )
        .query_farms(
            Some(FarmsBy::Identifier("m-farm_1".to_string())),
            None,
            None,
            |result| {
                let farms_response = result.unwrap();
                let farm = farms_response.farms[0].clone();
                assert_eq!(
                    farm.farm_asset,
                    Coin {
                        denom: "uusdy".to_string(),
                        amount: Uint128::new(4_000),
                    }
                );

                assert_eq!(farm.preliminary_end_epoch, 28);
            },
        )
        .manage_farm(
            &other,
            FarmAction::Fill {
                params: FarmParams {
                    lp_denom: lp_denom.clone(),
                    start_epoch: Some(20),
                    preliminary_end_epoch: Some(28),
                    curve: None,
                    farm_asset: Coin {
                        denom: "uusdy".to_string(),
                        amount: Uint128::new(5_000u128),
                    },
                    farm_identifier: Some("m-farm_1".to_string()),
                },
            },
            vec![coin(5_000u128, "uusdy")],
            |result| {
                result.unwrap();
            },
        )
        .query_farms(
            Some(FarmsBy::Identifier("m-farm_1".to_string())),
            None,
            None,
            |result| {
                let farms_response = result.unwrap();
                let farm = farms_response.farms[0].clone();
                assert_eq!(
                    farm.farm_asset,
                    Coin {
                        denom: "uusdy".to_string(),
                        amount: Uint128::new(9_000),
                    }
                );

                assert_eq!(farm.preliminary_end_epoch, 38);
            },
        );
}

#[test]
#[allow(clippy::inconsistent_digit_grouping)]
fn close_farms() {
    let lp_denom = format!("factory/{MOCK_CONTRACT_ADDR_1}/{LP_SYMBOL}").to_string();
    let lp_denom_2 = format!("factory/{MOCK_CONTRACT_ADDR_1}/2.{LP_SYMBOL}").to_string();

    let mut suite = TestingSuite::default_with_balances(vec![
        coin(1_000_000_000u128, "uom".to_string()),
        coin(1_000_000_000u128, "uusdy".to_string()),
        coin(1_000_000_000u128, "uosmo".to_string()),
        coin(1_000_000_000u128, lp_denom.clone()),
        coin(1_000_000_000u128, lp_denom_2.clone()),
    ]);

    suite.instantiate_default();

    let other = suite.senders[1].clone();
    let another = suite.senders[2].clone();

    for _ in 0..10 {
        suite.add_one_epoch();
    }
    // current epoch is 10

    suite.manage_farm(
        &other,
        FarmAction::Fill {
            params: FarmParams {
                lp_denom: lp_denom.clone(),
                start_epoch: Some(20),
                preliminary_end_epoch: Some(28),
                curve: None,
                farm_asset: Coin {
                    denom: "uusdy".to_string(),
                    amount: Uint128::new(4_000u128),
                },
                farm_identifier: Some("farm_1".to_string()),
            },
        },
        vec![coin(4_000, "uusdy"), coin(1_000, "uom")],
        |result| {
            result.unwrap();
        },
    );
    suite
        .manage_farm(
            &other,
            FarmAction::Close {
                farm_identifier: "m-farm_1".to_string(),
            },
            vec![coin(1_000, "uom")],
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();
                match err {
                    ContractError::PaymentError { .. } => {}
                    _ => panic!("Wrong error type, should return ContractError::PaymentError"),
                }
            },
        )
        .manage_farm(
            &other,
            FarmAction::Close {
                farm_identifier: "m-farm_2".to_string(),
            },
            vec![],
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();

                match err {
                    ContractError::NonExistentFarm { .. } => {}
                    _ => panic!("Wrong error type, should return ContractError::NonExistentFarm"),
                }
            },
        )
        .manage_farm(
            &another,
            FarmAction::Close {
                farm_identifier: "m-farm_1".to_string(),
            },
            vec![],
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();

                match err {
                    ContractError::Unauthorized { .. } => {}
                    _ => panic!("Wrong error type, should return ContractError::Unauthorized"),
                }
            },
        )
        .query_balance("uusdy".to_string(), &other, |balance| {
            assert_eq!(balance, Uint128::new(999_996_000));
        })
        .manage_farm(
            &other,
            FarmAction::Close {
                farm_identifier: "m-farm_1".to_string(),
            },
            vec![],
            |result| {
                result.unwrap();
            },
        )
        .query_balance("uusdy".to_string(), &other, |balance| {
            assert_eq!(balance, Uint128::new(1000_000_000));
        });

    // open new farm
    suite
        .query_current_epoch(|result| {
            let epoch_response = result.unwrap();
            assert_eq!(epoch_response.epoch.id, 10);
        })
        .manage_position(
            &another,
            PositionAction::Create {
                identifier: None,
                unlocking_duration: 86_400,
                receiver: None,
            },
            vec![coin(1_000, lp_denom_2.clone())],
            |result| {
                result.unwrap();
            },
        )
        .manage_farm(
            &other,
            FarmAction::Fill {
                params: FarmParams {
                    lp_denom: lp_denom_2.clone(),
                    start_epoch: Some(12),
                    preliminary_end_epoch: Some(13),
                    curve: None,
                    farm_asset: Coin {
                        denom: "uusdy".to_string(),
                        amount: Uint128::new(4_000u128),
                    },
                    farm_identifier: Some("farm_x".to_string()),
                },
            },
            vec![coin(4_000, "uusdy"), coin(1_000, "uom")],
            |result| {
                result.unwrap();
            },
        );

    for _ in 0..=2 {
        suite.add_one_epoch();
    }

    suite.query_current_epoch(|result| {
        let epoch_response = result.unwrap();
        assert_eq!(epoch_response.epoch.id, 13);
    });

    suite
        .query_balance("uusdy".to_string(), &another, |balance| {
            assert_eq!(balance, Uint128::new(1_000_000_000));
        })
        .claim(&another, vec![], |result| {
            result.unwrap();
        })
        .query_farms(
            Some(FarmsBy::Identifier("m-farm_x".to_string())),
            None,
            None,
            |result| {
                let farms_response = result.unwrap();
                let farm = farms_response.farms[0].clone();
                assert_eq!(
                    farm.farm_asset,
                    Coin {
                        denom: "uusdy".to_string(),
                        amount: Uint128::new(4_000),
                    }
                );
                // the farm is empty
                assert_eq!(farm.claimed_amount, Uint128::new(4_000),);

                assert_eq!(farm.preliminary_end_epoch, 13);
                assert_eq!(farm.start_epoch, 12);
            },
        )
        .query_balance("uusdy".to_string(), &another, |balance| {
            assert_eq!(balance, Uint128::new(1_000_004_000));
        })
        .manage_farm(
            &other,
            FarmAction::Close {
                farm_identifier: "m-farm_x".to_string(),
            },
            vec![],
            |result| {
                result.unwrap();
            },
        );
}

#[test]
fn verify_ownership() {
    let mut suite = TestingSuite::default_with_balances(vec![]);
    let creator = suite.creator();
    let other = suite.senders[1].clone();
    let unauthorized = suite.senders[2].clone();

    suite
        .instantiate_default()
        .query_ownership(|result| {
            let ownership = result.unwrap();
            assert_eq!(Addr::unchecked(ownership.owner.unwrap()), creator);
        })
        .update_ownership(
            &unauthorized,
            cw_ownable::Action::TransferOwnership {
                new_owner: other.to_string(),
                expiry: None,
            },
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();

                match err {
                    ContractError::OwnershipError { .. } => {}
                    _ => panic!("Wrong error type, should return ContractError::OwnershipError"),
                }
            },
        )
        .update_ownership(
            &creator,
            cw_ownable::Action::TransferOwnership {
                new_owner: other.to_string(),
                expiry: None,
            },
            |result| {
                result.unwrap();
            },
        )
        .update_ownership(&other, cw_ownable::Action::AcceptOwnership, |result| {
            result.unwrap();
        })
        .query_ownership(|result| {
            let ownership = result.unwrap();
            assert_eq!(Addr::unchecked(ownership.owner.unwrap()), other);
        })
        .update_ownership(&other, cw_ownable::Action::RenounceOwnership, |result| {
            result.unwrap();
        })
        .query_ownership(|result| {
            let ownership = result.unwrap();
            assert!(ownership.owner.is_none());
        });
}

#[test]
pub fn update_config() {
    let lp_denom = format!("factory/{MOCK_CONTRACT_ADDR_1}/{LP_SYMBOL}").to_string();

    let mut suite = TestingSuite::default_with_balances(vec![
        coin(1_000_000_000u128, "uom".to_string()),
        coin(1_000_000_000u128, "uusdy".to_string()),
        coin(1_000_000_000u128, "uosmo".to_string()),
        coin(1_000_000_000u128, lp_denom.clone()),
    ]);

    let creator = suite.creator();
    let other = suite.senders[1].clone();

    suite.instantiate_default();

    let fee_collector = suite.fee_collector_addr.clone();
    let epoch_manager = suite.epoch_manager_addr.clone();
    let pool_manager = suite.pool_manager_addr.clone();

    let expected_config = Config {
        fee_collector_addr: fee_collector,
        epoch_manager_addr: epoch_manager,
        pool_manager_addr: pool_manager,
        create_farm_fee: Coin {
            denom: "uom".to_string(),
            amount: Uint128::new(1_000u128),
        },
        max_concurrent_farms: 2u32,
        max_farm_epoch_buffer: 14u32,
        min_unlocking_duration: 86_400u64,
        max_unlocking_duration: 31_556_926u64,
        farm_expiration_time: MONTH_IN_SECONDS,
        emergency_unlock_penalty: Decimal::percent(10),
    };

    suite.query_config(|result| {
        let config = result.unwrap();
        assert_eq!(config, expected_config);
    })
        .update_config(
            &other,
            Some(MOCK_CONTRACT_ADDR_1.to_string()),
            Some(MOCK_CONTRACT_ADDR_1.to_string()),
            Some(MOCK_CONTRACT_ADDR_1.to_string()),
            Some(Coin {
                denom: "uom".to_string(),
                amount: Uint128::new(2_000u128),
            }),
            Some(3u32),
            Some(15u32),
            Some(172_800u64),
            Some(864_000u64),
            Some(MONTH_IN_SECONDS * 2),
            Some(Decimal::percent(50)),
            vec![coin(1_000, "uom")],
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();
                match err {
                    ContractError::PaymentError { .. } => {}
                    _ => panic!("Wrong error type, should return ContractError::PaymentError"),
                }
            },
        ).update_config(
        &other,
        Some(MOCK_CONTRACT_ADDR_1.to_string()),
        Some(MOCK_CONTRACT_ADDR_1.to_string()),
        Some(MOCK_CONTRACT_ADDR_1.to_string()),
        Some(Coin {
            denom: "uom".to_string(),
            amount: Uint128::new(2_000u128),
        }),
        Some(0u32),
        Some(15u32),
        Some(172_800u64),
        Some(864_000u64),
        Some(MONTH_IN_SECONDS * 2),
        Some(Decimal::percent(50)),
        vec![],
        |result| {
            let err = result.unwrap_err().downcast::<ContractError>().unwrap();
            match err {
                ContractError::OwnershipError { .. } => {}
                _ => panic!("Wrong error type, should return ContractError::OwnershipError"),
            }
        },
    ).update_config(
        &creator,
        Some(MOCK_CONTRACT_ADDR_1.to_string()),
        Some(MOCK_CONTRACT_ADDR_1.to_string()),
        Some(MOCK_CONTRACT_ADDR_1.to_string()),
        Some(Coin {
            denom: "uom".to_string(),
            amount: Uint128::new(2_000u128),
        }),
        Some(0u32),
        Some(15u32),
        Some(172_800u64),
        Some(864_000u64),
        Some(MONTH_IN_SECONDS * 2),
        Some(Decimal::percent(50)),
        vec![],
        |result| {
            let err = result.unwrap_err().downcast::<ContractError>().unwrap();
            match err {
                ContractError::MaximumConcurrentFarmsDecreased => {}
                _ => panic!("Wrong error type, should return ContractError::MaximumConcurrentFarmsDecreased"),
            }
        },
    ).update_config(
        &creator,
        Some(MOCK_CONTRACT_ADDR_1.to_string()),
        Some(MOCK_CONTRACT_ADDR_1.to_string()),
        Some(MOCK_CONTRACT_ADDR_1.to_string()),
        Some(Coin {
            denom: "uom".to_string(),
            amount: Uint128::new(2_000u128),
        }),
        Some(5u32),
        Some(15u32),
        Some(80_800u64),
        Some(80_000u64),
        Some(MONTH_IN_SECONDS * 2),
        Some(Decimal::percent(50)),
        vec![],
        |result| {
            let err = result.unwrap_err().downcast::<ContractError>().unwrap();
            match err {
                ContractError::InvalidUnlockingRange { .. } => {}
                _ => panic!("Wrong error type, should return ContractError::InvalidUnlockingRange"),
            }
        },
    ).update_config(
        &creator,
        Some(MOCK_CONTRACT_ADDR_1.to_string()),
        Some(MOCK_CONTRACT_ADDR_1.to_string()),
        Some(MOCK_CONTRACT_ADDR_1.to_string()),
        Some(Coin {
            denom: "uom".to_string(),
            amount: Uint128::new(2_000u128),
        }),
        Some(5u32),
        Some(15u32),
        Some(300_000u64),
        Some(200_000u64),
        Some(MONTH_IN_SECONDS * 2),
        Some(Decimal::percent(50)),
        vec![],
        |result| {
            let err = result.unwrap_err().downcast::<ContractError>().unwrap();
            match err {
                ContractError::InvalidUnlockingRange { .. } => {}
                _ => panic!("Wrong error type, should return ContractError::InvalidUnlockingRange"),
            }
        },
    ).update_config(
        &creator,
        Some(MOCK_CONTRACT_ADDR_1.to_string()),
        Some(MOCK_CONTRACT_ADDR_1.to_string()),
        Some(MOCK_CONTRACT_ADDR_1.to_string()),
        Some(Coin {
            denom: "uom".to_string(),
            amount: Uint128::new(2_000u128),
        }),
        Some(5u32),
        Some(15u32),
        Some(100_000u64),
        Some(200_000u64),
        Some(MONTH_IN_SECONDS * 2),
        Some(Decimal::percent(105)),
        vec![],
        |result| {
            let err = result.unwrap_err().downcast::<ContractError>().unwrap();
            match err {
                ContractError::InvalidEmergencyUnlockPenalty { .. } => {}
                _ => panic!("Wrong error type, should return ContractError::InvalidEmergencyUnlockPenalty"),
            }
        },
    ).update_config(
        &creator,
        Some(MOCK_CONTRACT_ADDR_1.to_string()),
        Some(MOCK_CONTRACT_ADDR_1.to_string()),
        Some(MOCK_CONTRACT_ADDR_1.to_string()),
        Some(Coin {
            denom: "uom".to_string(),
            amount: Uint128::new(2_000u128),
        }),
        Some(5u32),
        Some(15u32),
        Some(100_000u64),
        Some(200_000u64),
        Some(MONTH_IN_SECONDS * 2),
        Some(Decimal::percent(20)),
        vec![],
        |result| {
            result.unwrap();
        },
    );

    let expected_config = Config {
        fee_collector_addr: Addr::unchecked(MOCK_CONTRACT_ADDR_1),
        epoch_manager_addr: Addr::unchecked(MOCK_CONTRACT_ADDR_1),
        pool_manager_addr: Addr::unchecked(MOCK_CONTRACT_ADDR_1),
        create_farm_fee: Coin {
            denom: "uom".to_string(),
            amount: Uint128::new(2_000u128),
        },
        max_concurrent_farms: 5u32,
        max_farm_epoch_buffer: 15u32,
        min_unlocking_duration: 100_000u64,
        max_unlocking_duration: 200_000u64,
        farm_expiration_time: MONTH_IN_SECONDS * 2,
        emergency_unlock_penalty: Decimal::percent(20),
    };

    suite.query_config(|result| {
        let config = result.unwrap();
        assert_eq!(config, expected_config);
    });

    suite.update_config(
        &creator,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some(MONTH_IN_SECONDS - 100),
        None,
        vec![],
        |result| {
            let err = result.unwrap_err().downcast::<ContractError>().unwrap();
            match err {
                ContractError::FarmExpirationTimeInvalid { .. } => {}
                _ => panic!(
                    "Wrong error type, should return ContractError::FarmExpirationTimeInvalid"
                ),
            }
        },
    );
}

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
                    ContractError::AssetMismatch { .. } => {}
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
                    ContractError::Unauthorized { .. } => {}
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
                    ContractError::PendingRewards { .. } => {}
                    _ => panic!("Wrong error type, should return ContractError::PendingRewards"),
                }
            },
        )
        .claim(&creator, vec![coin(4_000, lp_denom.clone())], |result| {
            let err = result.unwrap_err().downcast::<ContractError>().unwrap();
            match err {
                ContractError::PaymentError { .. } => {}
                _ => panic!("Wrong error type, should return ContractError::PaymentError"),
            }
        })
        .claim(&other, vec![], |result| {
            let err = result.unwrap_err().downcast::<ContractError>().unwrap();
            match err {
                ContractError::NoOpenPositions { .. } => {}
                _ => panic!("Wrong error type, should return ContractError::NoOpenPositions"),
            }
        })
        .query_balance("uusdy".to_string(), &creator, |balance| {
            assert_eq!(balance, Uint128::new(999_992_000));
        })
        .claim(&creator, vec![], |result| {
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
                    ContractError::Unauthorized { .. } => {}
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
                    ContractError::AssetMismatch { .. } => {}
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
                    ContractError::PositionNotExpired { .. } => {}
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
                    ContractError::Unauthorized { .. } => {}
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
        .query_rewards(&creator, |result| {
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
        .claim(&creator, vec![], |result| {
            result.unwrap();
        })
        .query_farms(None, None, None, |result| {
            let farms_response = result.unwrap();
            assert_eq!(
                farms_response.farms[0].farm_asset.amount,
                farms_response.farms[0].claimed_amount
            );
        })
        .query_rewards(&creator, |result| {
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
                    ContractError::Unauthorized { .. } => {}
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
                    ContractError::Unauthorized { .. } => {}
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
            assert_eq!(balance, Uint128::new(250));
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
        .query_rewards(&creator, |result| {
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
                    ContractError::Unauthorized { .. } => {}
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
        .add_one_epoch()
        .add_one_epoch()
        .add_one_epoch()
        .add_one_epoch()
        .query_current_epoch(|result| {
            let epoch_response = result.unwrap();
            assert_eq!(epoch_response.epoch.id, 14);
        })
        .query_balance("uusdy".to_string(), &other, |balance| {
            assert_eq!(balance, Uint128::new(1_000_000_000u128));
        })
        .claim(&other, vec![], |result| {
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
        .query_rewards(&creator, |result| {
            let rewards_response = result.unwrap();
            match rewards_response {
                RewardsResponse::RewardsResponse { total_rewards, .. } => {
                    assert!(total_rewards.is_empty());
                }
                _ => panic!("shouldn't return this but RewardsResponse"),
            }
        })
        .claim(&other, vec![], |result| {
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
        .add_one_epoch()
        .add_one_epoch()
        .add_one_epoch()
        .query_current_epoch(|result| {
            let epoch_response = result.unwrap();
            assert_eq!(epoch_response.epoch.id, 13);
        })
        .query_balance("uusdy".to_string(), &other, |balance| {
            assert_eq!(balance, Uint128::new(1_000_000_000u128));
        })
        .claim(&other, vec![], |result| {
            result.unwrap();
        })
        .query_balance("uusdy".to_string(), &other, |balance| {
            assert_eq!(balance, Uint128::new(1_000_004_000u128));
        })
        .query_balance("uusdy".to_string(), &another, |balance| {
            assert_eq!(balance, Uint128::new(1_000_000_000u128));
        })
        .claim(&another, vec![], |result| {
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
                        last_epoch_claimed: 13u64,
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
                        last_epoch_claimed: 14u64,
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
                        last_epoch_claimed: 13u64,
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
                        last_epoch_claimed: 15u64,
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

    suite
        .add_one_epoch()
        .add_one_epoch()
        .add_one_epoch()
        .query_current_epoch(|result| {
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
        .claim(&other, vec![], |result| {
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
        .claim(&another, vec![], |result| {
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
fn test_close_expired_farms() {
    let lp_denom = format!("factory/{MOCK_CONTRACT_ADDR_1}/{LP_SYMBOL}").to_string();

    let mut suite = TestingSuite::default_with_balances(vec![
        coin(2_000_000_000u128, "uom"),
        coin(2_000_000_000u128, "uusdy"),
        coin(2_000_000_000u128, "uosmo"),
        coin(1_000_000_000u128, lp_denom.clone()),
        coin(1_000_000_000u128, "invalid_lp"),
    ]);

    let creator = suite.creator();
    let other = suite.senders[1].clone();

    suite.instantiate_default();

    for _ in 0..10 {
        suite.add_one_epoch();
    }

    suite.manage_farm(
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
                farm_identifier: Some("farm".to_string()),
            },
        },
        vec![coin(8_000, "uusdy"), coin(1_000, "uom")],
        |result| {
            result.unwrap();
        },
    );

    // create enough epochs to make the farm expire
    for _ in 0..=37 {
        suite.add_one_epoch();
    }

    // try opening another farm for the same lp denom, the expired farm should get closed
    suite
        .query_current_epoch(|result| {
            let epoch_response = result.unwrap();
            assert_eq!(epoch_response.epoch.id, 48);
        })
        .query_farms(None, None, None, |result| {
            let farms_response = result.unwrap();
            assert_eq!(farms_response.farms.len(), 1);
            assert_eq!(farms_response.farms[0].identifier, "m-farm");
        })
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
                        amount: Uint128::new(10_000u128),
                    },
                    farm_identifier: Some("new_farm".to_string()),
                },
            },
            vec![coin(10_000, "uusdy"), coin(1_000, "uom")],
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
                    identifier: "m-new_farm".to_string(),
                    owner: other.clone(),
                    lp_denom: lp_denom.clone(),
                    farm_asset: Coin {
                        denom: "uusdy".to_string(),
                        amount: Uint128::new(10_000u128),
                    },
                    claimed_amount: Uint128::zero(),
                    emission_rate: Uint128::new(714),
                    curve: Curve::Linear,
                    start_epoch: 49u64,
                    preliminary_end_epoch: 63u64,
                    last_epoch_claimed: 48u64,
                }
            );
        });
}

#[test]
fn expand_expired_farm() {
    let lp_denom = format!("factory/{MOCK_CONTRACT_ADDR_1}/{LP_SYMBOL}").to_string();

    let mut suite = TestingSuite::default_with_balances(vec![
        coin(2_000_000_000u128, "uom".to_string()),
        coin(2_000_000_000u128, "uusdy".to_string()),
        coin(2_000_000_000u128, "uosmo".to_string()),
        coin(2_000_000_000u128, lp_denom.clone()),
    ]);

    let other = suite.senders[1].clone();

    suite.instantiate_default();

    suite
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
                    emission_rate: Uint128::new(285),
                    curve: Curve::Linear,
                    start_epoch: 1u64,
                    preliminary_end_epoch: 15u64,
                    last_epoch_claimed: 0u64,
                }
            );
        });

    // create enough epochs to make the farm expire
    // should expire at epoch 16 + config.farm_expiration_time, i.e. 16 + 30 = 46
    for _ in 0..=46 {
        suite.add_one_epoch();
    }

    suite
        .query_current_epoch(|result| {
            let epoch_response = result.unwrap();
            assert_eq!(epoch_response.epoch.id, 47);
        })
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
                    ContractError::FarmAlreadyExpired { .. } => {}
                    _ => {
                        panic!("Wrong error type, should return ContractError::FarmAlreadyExpired")
                    }
                }
            },
        );
}

#[test]
fn test_emergency_withdrawal() {
    let lp_denom = format!("factory/{MOCK_CONTRACT_ADDR_1}/{LP_SYMBOL}").to_string();

    let mut suite = TestingSuite::default_with_balances(vec![
        coin(1_000_000_000u128, "uom".to_string()),
        coin(1_000_000_000u128, "uusdy".to_string()),
        coin(1_000_000_000u128, "uosmo".to_string()),
        coin(1_000_000_000u128, lp_denom.clone()),
    ]);

    let other = suite.senders[1].clone();

    suite.instantiate_default();

    let fee_collector = suite.fee_collector_addr.clone();

    suite
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
        .manage_position(
            &other,
            PositionAction::Create {
                identifier: Some("other_position".to_string()),
                unlocking_duration: 86_400,
                receiver: None,
            },
            vec![coin(1_000, lp_denom.clone())],
            |result| {
                result.unwrap();
            },
        )
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
                        identifier: "u-other_position".to_string(),
                        lp_asset: Coin {
                            denom: format!("factory/{MOCK_CONTRACT_ADDR_1}/{LP_SYMBOL}")
                                .to_string(),
                            amount: Uint128::new(1_000),
                        },
                        unlocking_duration: 86400,
                        open: true,
                        expiring_at: None,
                        receiver: other.clone(),
                    }
                );
            },
        )
        .query_balance(lp_denom.clone().to_string(), &other, |balance| {
            assert_eq!(balance, Uint128::new(999_999_000));
        })
        .query_balance(lp_denom.clone().to_string(), &fee_collector, |balance| {
            assert_eq!(balance, Uint128::zero());
        })
        .manage_position(
            &other,
            PositionAction::Withdraw {
                identifier: "u-other_position".to_string(),
                emergency_unlock: Some(true),
            },
            vec![],
            |result| {
                result.unwrap();
            },
        )
        .query_balance(lp_denom.clone().to_string(), &other, |balance| {
            //emergency unlock penalty is 10% of the position amount, so the user gets 1000 - 100 = 900 + 50
            // (as he was the owner of the farm, he got 50% of the penalty fee`
            assert_eq!(balance, Uint128::new(999_999_950));
        })
        .query_balance(lp_denom.clone().to_string(), &fee_collector, |balance| {
            assert_eq!(balance, Uint128::new(50));
        });
}

#[test]
fn test_emergency_withdrawal_with_pending_rewards_are_lost() {
    let lp_denom = format!("factory/{MOCK_CONTRACT_ADDR_1}/{LP_SYMBOL}").to_string();

    let mut suite = TestingSuite::default_with_balances(vec![
        coin(1_000_000_000u128, "uom".to_string()),
        coin(1_000_000_000u128, "uusdy".to_string()),
        coin(1_000_000_000u128, "uosmo".to_string()),
        coin(1_000_000_000u128, lp_denom.clone()),
    ]);

    let other = suite.senders[1].clone();

    suite.instantiate_default();

    suite
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
        .manage_position(
            &other,
            PositionAction::Create {
                identifier: Some("other_position".to_string()),
                unlocking_duration: 86_400,
                receiver: None,
            },
            vec![coin(1_000, lp_denom.clone())],
            |result| {
                result.unwrap();
            },
        )
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
                        identifier: "u-other_position".to_string(),
                        lp_asset: Coin {
                            denom: format!("factory/{MOCK_CONTRACT_ADDR_1}/{LP_SYMBOL}")
                                .to_string(),
                            amount: Uint128::new(1_000),
                        },
                        unlocking_duration: 86400,
                        open: true,
                        expiring_at: None,
                        receiver: other.clone(),
                    }
                );
            },
        )
        .add_one_epoch()
        .add_one_epoch()
        .add_one_epoch()
        // rewards are pending to be claimed
        .query_rewards(&other, |result| {
            let response = result.unwrap();

            match response {
                RewardsResponse::RewardsResponse { total_rewards, .. } => {
                    assert_eq!(total_rewards.len(), 1);
                    assert_eq!(total_rewards[0], coin(855, "uusdy"));
                }
                _ => panic!("shouldn't return this but RewardsResponse"),
            }
        })
        .query_balance("uusdy".to_string(), &other, |balance| {
            assert_eq!(balance, Uint128::new(999_996_000));
        })
        // the user emergency withdraws the position
        .manage_position(
            &other,
            PositionAction::Withdraw {
                identifier: "u-other_position".to_string(),
                emergency_unlock: Some(true),
            },
            vec![],
            |result| {
                result.unwrap();
            },
        )
        // rewards were not claimed
        .query_balance("uusdy".to_string(), &other, |balance| {
            assert_eq!(balance, Uint128::new(999_996_000));
        });
}

#[test]
fn emergency_withdrawal_shares_penalty_with_farm_owners() {
    let lp_denom = format!("factory/{MOCK_CONTRACT_ADDR_1}/{LP_SYMBOL}").to_string();

    let mut suite = TestingSuite::default_with_balances(vec![
        coin(1_000_000_000u128, "uom".to_string()),
        coin(1_000_000_000u128, "uusdy".to_string()),
        coin(1_000_000_000u128, "uosmo".to_string()),
        coin(1_000_000_000u128, lp_denom.clone()),
    ]);

    let other = suite.senders[0].clone();
    let alice = suite.senders[1].clone();
    let bob = suite.senders[2].clone();

    suite.instantiate_default();

    let fee_collector = suite.fee_collector_addr.clone();
    let farm_manager = suite.farm_manager_addr.clone();

    suite
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
        .manage_farm(
            &alice,
            FarmAction::Fill {
                params: FarmParams {
                    lp_denom: lp_denom.clone(),
                    start_epoch: None,
                    preliminary_end_epoch: None,
                    curve: None,
                    farm_asset: Coin {
                        denom: "uusdy".to_string(),
                        amount: Uint128::new(4_000u128),
                    },
                    farm_identifier: Some("farm_2".to_string()),
                },
            },
            vec![coin(4_000, "uusdy"), coin(1_000, "uom")],
            |result| {
                result.unwrap();
            },
        )
        .manage_position(
            &bob,
            PositionAction::Create {
                identifier: Some("bob_position".to_string()),
                unlocking_duration: 86_400,
                receiver: None,
            },
            vec![coin(6_000_000, lp_denom.clone())],
            |result| {
                result.unwrap();
            },
        )
        .query_positions(
            Some(PositionsBy::Receiver(bob.to_string())),
            Some(true),
            None,
            None,
            |result| {
                let positions = result.unwrap();
                assert_eq!(positions.positions.len(), 1);
                assert_eq!(
                    positions.positions[0],
                    Position {
                        identifier: "u-bob_position".to_string(),
                        lp_asset: Coin {
                            denom: format!("factory/{MOCK_CONTRACT_ADDR_1}/{LP_SYMBOL}")
                                .to_string(),
                            amount: Uint128::new(6_000_000),
                        },
                        unlocking_duration: 86400,
                        open: true,
                        expiring_at: None,
                        receiver: bob.clone(),
                    }
                );
            },
        )
        .query_balance(lp_denom.clone().to_string(), &other, |balance| {
            assert_eq!(balance, Uint128::new(1_000_000_000));
        })
        .query_balance(lp_denom.clone().to_string(), &fee_collector, |balance| {
            assert_eq!(balance, Uint128::zero());
        })
        .query_balance(lp_denom.clone().to_string(), &alice, |balance| {
            assert_eq!(balance, Uint128::new(1_000_000_000));
        })
        .query_balance(lp_denom.clone().to_string(), &farm_manager, |balance| {
            assert_eq!(balance, Uint128::new(6_000_000));
        })
        .manage_position(
            &bob,
            PositionAction::Withdraw {
                identifier: "u-bob_position".to_string(),
                emergency_unlock: Some(true),
            },
            vec![],
            |result| {
                result.unwrap();
            },
        )
        .query_balance(lp_denom.clone().to_string(), &other, |balance| {
            assert_eq!(balance, Uint128::new(1_000_150_000));
        })
        .query_balance(lp_denom.clone().to_string(), &fee_collector, |balance| {
            assert_eq!(balance, Uint128::new(300_000));
        })
        .query_balance(lp_denom.clone().to_string(), &alice, |balance| {
            assert_eq!(balance, Uint128::new(1_000_150_000));
        })
        .query_balance(lp_denom.clone().to_string(), &farm_manager, |balance| {
            assert_eq!(balance, Uint128::zero());
        });
}

#[test]
fn test_farm_helper() {
    let lp_denom = format!("factory/{MOCK_CONTRACT_ADDR_1}/{LP_SYMBOL}").to_string();

    let mut suite = TestingSuite::default_with_balances(vec![
        coin(1_000_000_000u128, "uom".to_string()),
        coin(1_000_000_000u128, "uusdy".to_string()),
        coin(1_000_000_000u128, "uosmo".to_string()),
        coin(1_000_000_000u128, lp_denom.clone()),
    ]);

    let creator = suite.creator();
    let other = suite.senders[1].clone();

    suite.instantiate_default();

    let farm_manager = suite.farm_manager_addr.clone();
    let fee_collector = suite.fee_collector_addr.clone();

    suite
        .manage_farm(
            &creator,
            FarmAction::Fill {
                params: FarmParams {
                    lp_denom: lp_denom.clone(),
                    start_epoch: None,
                    preliminary_end_epoch: None,
                    curve: None,
                    farm_asset: Coin {
                        denom: "uom".to_string(),
                        amount: Uint128::new(4_000u128),
                    },
                    farm_identifier: Some("farm".to_string()),
                },
            },
            vec![coin(3_000, "uom")],
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();
                match err {
                    ContractError::AssetMismatch { .. } => {}
                    _ => {
                        panic!("Wrong error type, should return ContractError::AssetMismatch")
                    }
                }
            },
        )
        .query_balance("uom".to_string(), &creator, |balance| {
            assert_eq!(balance, Uint128::new(1_000_000_000));
        })
        .query_balance("uom".to_string(), &fee_collector, |balance| {
            assert_eq!(balance, Uint128::zero());
        })
        .query_balance("uom".to_string(), &farm_manager, |balance| {
            assert_eq!(balance, Uint128::zero());
        })
        .manage_farm(
            &creator,
            FarmAction::Fill {
                params: FarmParams {
                    lp_denom: lp_denom.clone(),
                    start_epoch: None,
                    preliminary_end_epoch: None,
                    curve: None,
                    farm_asset: Coin {
                        denom: "uusdy".to_string(),
                        amount: Uint128::new(2_000u128),
                    },
                    farm_identifier: Some("farm".to_string()),
                },
            },
            vec![coin(2_000, "uusdy"), coin(3_000, "uom")],
            |result| {
                result.unwrap();
            },
        )
        .query_balance("uom".to_string(), &fee_collector, |balance| {
            assert_eq!(balance, Uint128::new(1_000));
        })
        .query_balance("uom".to_string(), &farm_manager, |balance| {
            assert_eq!(balance, Uint128::zero());
        })
        .query_balance("uom".to_string(), &creator, |balance| {
            // got the excess of whale back
            assert_eq!(balance, Uint128::new(999_999_000));
        });

    suite.manage_farm(
        &other,
        FarmAction::Fill {
            params: FarmParams {
                lp_denom: lp_denom.clone(),
                start_epoch: None,
                preliminary_end_epoch: None,
                curve: None,
                farm_asset: Coin {
                    denom: "uusdy".to_string(),
                    amount: Uint128::new(2_000u128),
                },
                farm_identifier: Some("underpaid_farm".to_string()),
            },
        },
        vec![coin(2_000, "uusdy"), coin(500, "uom")],
        |result| {
            let err = result.unwrap_err().downcast::<ContractError>().unwrap();
            match err {
                ContractError::FarmFeeNotPaid { .. } => {}
                _ => {
                    panic!("Wrong error type, should return ContractError::FarmFeeNotPaid")
                }
            }
        },
    );
}

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
                        last_epoch_claimed: 11u64,
                    }
                );
            },
        )
        .query_balance("uusdy".to_string(), &creator, |balance| {
            assert_eq!(balance, Uint128::new(999_920_000));
        })
        .claim(&creator, vec![], |result| {
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
                    last_epoch_claimed: 15u64,
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
                    last_epoch_claimed: 15u64,
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
        .claim(&other, vec![], |result| {
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
                    last_epoch_claimed: 19u64,
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
                    last_epoch_claimed: 19u64,
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
                assert_eq!(balance, Uint128::new(4_000));
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
        .claim(&creator, vec![], |result| {
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
                    last_epoch_claimed: 19u64,
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
                    last_epoch_claimed: 30u64,
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
                    last_epoch_claimed: 30u64,
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
                    last_epoch_claimed: 30u64,
                }
            );
        })
        .claim(&another, vec![], |result| {
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
                    last_epoch_claimed: 19u64,
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
                    last_epoch_claimed: 30u64,
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
                    last_epoch_claimed: 30u64,
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
                    last_epoch_claimed: 30u64,
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
        .claim(&creator, vec![], |result| {
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
                    last_epoch_claimed: 35u64,
                }
            );
        })
        .claim(&another, vec![], |result| {
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
                    last_epoch_claimed: 35u64,
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

#[test]
fn test_rewards_query_overlapping_farms() {
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
                    start_epoch: Some(12),
                    preliminary_end_epoch: Some(16),
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
                    start_epoch: Some(12),
                    preliminary_end_epoch: Some(16),
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
                    start_epoch: Some(12),
                    preliminary_end_epoch: Some(16),
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

    suite.query_rewards(&creator, |result| {
        let rewards_response = result.unwrap();

        assert_eq!(
            rewards_response,
            RewardsResponse::RewardsResponse {
                total_rewards: vec![
                    coin(15000, "uom"),
                    coin(5000, "uosmo"),
                    coin(75000, "uusdy"),
                ],
                rewards_per_lp_denom: vec![
                    (
                        lp_denom_1.clone(),
                        vec![coin(5000, "uosmo"), coin(40000, "uusdy")]
                    ),
                    (
                        lp_denom_2.clone(),
                        vec![coin(15000, "uom"), coin(35000, "uusdy")]
                    ),
                ],
            }
        );
    });
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
                    ContractError::Unauthorized { .. } => {}
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
                    ContractError::Unauthorized { .. } => {}
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
                    ContractError::Unauthorized { .. } => {}
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
    for i in 1..MAX_ITEMS_LIMIT * 2 {
        let lp_denom = format!("factory/{MOCK_CONTRACT_ADDR_1}/{i}.{LP_SYMBOL}");
        balances.push(coin(1_000_000_000u128, lp_denom.clone()));
    }

    let mut suite = TestingSuite::default_with_balances(balances);

    let creator = suite.creator();
    let alice = suite.senders[1].clone();
    suite.instantiate_default();

    // prepare farms, create more than the user could participate on
    for i in 1..MAX_ITEMS_LIMIT * 2 {
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
    for i in 1..=MAX_ITEMS_LIMIT {
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
        Some(MAX_ITEMS_LIMIT),
        |result| {
            let response = result.unwrap();
            assert_eq!(response.positions.len(), MAX_ITEMS_LIMIT as usize);
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
        .claim(&alice, vec![], |result| {
            result.unwrap();
        })
        .query_balance("uusdy".to_string(), &alice, |balance| {
            // all the rewards were claimed, 1000 uusdy * 100
            assert_eq!(balance, Uint128::new(1_000_100_000u128));
        })
        .query_positions(
            Some(PositionsBy::Receiver(alice.to_string())),
            Some(true),
            None,
            Some(MAX_ITEMS_LIMIT),
            |result| {
                let response = result.unwrap();
                assert_eq!(response.positions.len(), MAX_ITEMS_LIMIT as usize);
            },
        );

    // now let's try closing positions
    for i in 1..=MAX_ITEMS_LIMIT {
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
            Some(MAX_ITEMS_LIMIT),
            |result| {
                let response = result.unwrap();
                assert!(response.positions.is_empty());
            },
        )
        .query_positions(
            Some(PositionsBy::Receiver(alice.to_string())),
            Some(false),
            None,
            Some(MAX_ITEMS_LIMIT),
            |result| {
                let response = result.unwrap();
                assert_eq!(response.positions.len(), MAX_ITEMS_LIMIT as usize);
            },
        );

    // try opening more positions
    for i in 1..=MAX_ITEMS_LIMIT {
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
            Some(MAX_ITEMS_LIMIT),
            |result| {
                let response = result.unwrap();
                assert_eq!(response.positions.len(), MAX_ITEMS_LIMIT as usize);
            },
        )
        .query_positions(
            Some(PositionsBy::Receiver(alice.to_string())),
            Some(false),
            None,
            Some(MAX_ITEMS_LIMIT),
            |result| {
                let response = result.unwrap();
                assert_eq!(response.positions.len(), MAX_ITEMS_LIMIT as usize);
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
            Some(MAX_ITEMS_LIMIT),
            |result| {
                let response = result.unwrap();
                assert_eq!(response.positions.len(), (MAX_ITEMS_LIMIT - 1) as usize);
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
            Some(MAX_ITEMS_LIMIT),
            |result| {
                let response = result.unwrap();
                assert_eq!(response.positions.len(), MAX_ITEMS_LIMIT as usize);
            },
        );
}

#[test]
fn test_positions_query_filters_and_pagination() {
    let mut balances = vec![
        coin(1_000_000_000u128, "uom"),
        coin(1_000_000_000u128, "uusdy"),
        coin(1_000_000_000u128, "uosmo"),
    ];

    // prepare lp denoms
    for i in 1..MAX_ITEMS_LIMIT * 2 {
        let lp_denom = format!("factory/{MOCK_CONTRACT_ADDR_1}/{i}.{LP_SYMBOL}");
        balances.push(coin(1_000_000_000u128, lp_denom.clone()));
    }

    let mut suite = TestingSuite::default_with_balances(balances);

    let alice = suite.senders[1].clone();
    suite.instantiate_default();

    // open positions
    for i in 1..=MAX_ITEMS_LIMIT {
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

    let position_a_id = RefCell::new("".to_string());
    let position_b_id = RefCell::new("".to_string());

    suite
        .query_positions(
            Some(PositionsBy::Receiver(alice.to_string())),
            Some(true),
            None,
            Some(MAX_ITEMS_LIMIT),
            |result| {
                let response = result.unwrap();
                assert_eq!(response.positions.len(), MAX_ITEMS_LIMIT as usize);
            },
        )
        .query_positions(
            Some(PositionsBy::Receiver(alice.to_string())),
            Some(true),
            None,
            Some(10),
            |result| {
                let response = result.unwrap();
                assert_eq!(response.positions.len(), 10usize);

                position_a_id.replace(response.positions[9].identifier.clone());
            },
        )
        .query_positions(
            Some(PositionsBy::Receiver(alice.to_string())),
            Some(true),
            Some(position_a_id.borrow().clone()),
            Some(10),
            |result| {
                let response = result.unwrap();
                assert_eq!(response.positions.len(), 10usize);
                position_b_id.replace(response.positions[9].identifier.clone());
            },
        )
        .query_positions(
            Some(PositionsBy::Receiver(alice.to_string())),
            Some(true),
            None,
            Some(20),
            |result| {
                let response = result.unwrap();
                assert_eq!(response.positions.len(), 20usize);
                assert_eq!(
                    response.positions[9].identifier,
                    position_a_id.borrow().clone()
                );
                assert_eq!(
                    response.positions[19].identifier,
                    position_b_id.borrow().clone()
                );
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

#[test]
fn test_farm_expired() {
    let lp_denom = format!("factory/{MOCK_CONTRACT_ADDR_1}/{LP_SYMBOL}").to_string();

    let mut suite = TestingSuite::default_with_balances(vec![
        coin(2_000_000_000u128, "uom"),
        coin(2_000_000_000u128, "uusdy"),
        coin(2_000_000_000u128, "uosmo"),
        coin(1_000_000_000u128, lp_denom.clone()),
        coin(1_000_000_000u128, "invalid_lp"),
    ]);

    let creator = suite.creator();

    suite.instantiate_default();

    for _ in 0..10 {
        suite.add_one_epoch();
    }

    suite
        .query_current_epoch(|result| {
            let epoch_response = result.unwrap();
            assert_eq!(epoch_response.epoch.id, 10);
        })
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
                    farm_identifier: Some("short_farm".to_string()),
                },
            },
            vec![coin(8_000, "uusdy"), coin(1_000, "uom")],
            |result| {
                result.unwrap();
            },
        )
        .manage_farm(
            &creator,
            FarmAction::Fill {
                params: FarmParams {
                    lp_denom: lp_denom.clone(),
                    start_epoch: Some(12),
                    preliminary_end_epoch: Some(100),
                    curve: None,
                    farm_asset: Coin {
                        denom: "uusdy".to_string(),
                        amount: Uint128::new(8_000u128),
                    },
                    farm_identifier: Some("long_farm".to_string()),
                },
            },
            vec![coin(8_000, "uusdy"), coin(1_000, "uom")],
            |result| {
                result.unwrap();
            },
        )
        .manage_farm(
            &creator,
            FarmAction::Fill {
                params: FarmParams {
                    lp_denom: lp_denom.clone(),
                    start_epoch: Some(12),
                    preliminary_end_epoch: Some(100),
                    curve: None,
                    farm_asset: Coin {
                        denom: "uusdy".to_string(),
                        amount: Uint128::new(8_000u128),
                    },
                    farm_identifier: Some("another_farm".to_string()),
                },
            },
            vec![coin(8_000, "uusdy"), coin(1_000, "uom")],
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();
                match err {
                    ContractError::TooManyFarms { .. } => {}
                    _ => panic!("Wrong error type, should return ContractError::TooManyFarms"),
                }
            },
        );

    // create a few epochs, but not enough for the farm to expire.
    // a farm expires after config.farm_expiration_time seconds from the epoch the farm ended
    // in this case, from the start of epoch 17 + config.farm_expiration_time
    for _ in 0..20 {
        suite.add_one_epoch();
    }

    let mut current_epoch_id = 0;

    // try opening another farm for the same lp denom, the expired farm should get closed
    suite
        .query_current_epoch(|result| {
            let epoch_response = result.unwrap();
            assert_eq!(epoch_response.epoch.id, 30);
            current_epoch_id = epoch_response.epoch.id;
        })
        .query_farms(None, None, None, |result| {
            let farms_response = result.unwrap();
            assert_eq!(farms_response.farms.len(), 2);
            // not expired due to the claimed criteria
            assert!(farms_response.farms[0].claimed_amount.is_zero());
            assert!(farms_response.farms[1].claimed_amount.is_zero());
        });

    // creating a new farm of the same LP should fail as the previous ones are technically not expired yet
    // otherwise the contract would close them automatically when someone tries to open a new farm of that
    // same lp denom
    suite.manage_farm(
        &creator,
        FarmAction::Fill {
            params: FarmParams {
                lp_denom: lp_denom.clone(),
                start_epoch: Some(12),
                preliminary_end_epoch: Some(100),
                curve: None,
                farm_asset: Coin {
                    denom: "uusdy".to_string(),
                    amount: Uint128::new(8_000u128),
                },
                farm_identifier: Some("another_farm".to_string()),
            },
        },
        vec![coin(8_000, "uusdy"), coin(1_000, "uom")],
        |result| {
            let err = result.unwrap_err().downcast::<ContractError>().unwrap();
            match err {
                ContractError::TooManyFarms { .. } => {}
                _ => panic!("Wrong error type, should return ContractError::TooManyFarms"),
            }
        },
    );

    // since the short epoch ended on epoch 16, and each epoch is 1 day, the farm should be expired
    // on epoch 17.start_time + config.farm_expiration_time, which is set to a month.
    // That is, epoch 48, let's move to that epoch

    for _ in 0..18 {
        suite.add_one_epoch();
    }

    suite
        .query_current_epoch(|result| {
            let epoch_response = result.unwrap();
            assert_eq!(epoch_response.epoch.id, 48);
            current_epoch_id = epoch_response.epoch.id;
        })
        .query_farms(None, None, None, |result| {
            let farms_response = result.unwrap();
            assert_eq!(farms_response.farms.len(), 2);

            assert!(farms_response.farms[0].claimed_amount.is_zero());
            assert!(farms_response.farms[1].claimed_amount.is_zero());
            assert_eq!(
                farms_response.farms[0].identifier,
                "m-long_farm".to_string()
            );
            assert_eq!(
                farms_response.farms[1].identifier,
                "m-short_farm".to_string()
            );
        });

    // the short farm should be expired by now, let's try creating a new farm
    suite
        .manage_farm(
            &creator,
            FarmAction::Fill {
                params: FarmParams {
                    lp_denom: lp_denom.clone(),
                    start_epoch: Some(50),
                    preliminary_end_epoch: Some(100),
                    curve: None,
                    farm_asset: Coin {
                        denom: "uusdy".to_string(),
                        amount: Uint128::new(8_000u128),
                    },
                    farm_identifier: Some("another_farm".to_string()),
                },
            },
            vec![coin(8_000, "uusdy"), coin(1_000, "uom")],
            |result| {
                result.unwrap();
            },
        )
        .query_farms(None, None, None, |result| {
            let farms_response = result.unwrap();
            assert_eq!(farms_response.farms.len(), 2);

            assert!(farms_response.farms[0].claimed_amount.is_zero());
            assert!(farms_response.farms[1].claimed_amount.is_zero());
            assert_eq!(
                farms_response.farms[0].identifier,
                "m-another_farm".to_string()
            );
            assert_eq!(
                farms_response.farms[1].identifier,
                "m-long_farm".to_string()
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
                    last_epoch_claimed: 9u64,
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
    for _ in 0..100 {
        suite.add_one_epoch();
    }

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
                    ContractError::FarmAlreadyExpired { .. } => {}
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
                    last_epoch_claimed: 9u64,
                }
            );
        })
        .query_balance("uusdy".to_string(), &alice, |balance| {
            assert_eq!(balance, Uint128::new(2_000_000_000));
        })
        .claim(&alice, vec![], |result| {
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
                    last_epoch_claimed: 100u64,
                }
            );
        });
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
            Some(MAX_ITEMS_LIMIT),
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
        Some(MAX_ITEMS_LIMIT),
        is_as_expected,
    );
}

#[test]
fn test_farm_and_position_id_validation() {
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
                    farm_identifier: Some("invalid!".to_string()),
                },
            },
            vec![coin(8_000, "uusdy"), coin(1_000, "uom")],
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();
                match err {
                    ContractError::InvalidIdentifier { .. } => {}
                    _ => {
                        panic!("Wrong error type, should return ContractError::InvalidIdentifier")
                    }
                }
            },
        )
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
                    farm_identifier: Some(
                        "7105920181635468364293788789264771059201816354683642937887892647a"
                            .to_string(),
                    ),
                },
            },
            vec![coin(8_000, "uusdy"), coin(1_000, "uom")],
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();
                match err {
                    ContractError::InvalidIdentifier { .. } => {}
                    _ => {
                        panic!("Wrong error type, should return ContractError::InvalidIdentifier")
                    }
                }
            },
        )
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
                    farm_identifier: Some("感".to_string()),
                },
            },
            vec![coin(8_000, "uusdy"), coin(1_000, "uom")],
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();
                match err {
                    ContractError::InvalidIdentifier { .. } => {}
                    _ => {
                        panic!("Wrong error type, should return ContractError::InvalidIdentifier")
                    }
                }
            },
        )
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
                    farm_identifier: Some(
                        "INSERT INTO my_table (my_string) VALUES (values)".to_string(),
                    ),
                },
            },
            vec![coin(8_000, "uusdy"), coin(1_000, "uom")],
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();
                match err {
                    ContractError::InvalidIdentifier { .. } => {}
                    _ => {
                        panic!("Wrong error type, should return ContractError::InvalidIdentifier")
                    }
                }
            },
        )
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
                    farm_identifier: Some(
                        "7105920181635468364293788789264771059201816354683642937887892647"
                            .to_string(),
                    ),
                },
            },
            vec![coin(8_000, "uusdy"), coin(1_000, "uom")],
            |result| {
                result.unwrap();
            },
        );

    suite
        .manage_position(
            &creator,
            PositionAction::Create {
                identifier: Some("invalid!".to_string()),
                unlocking_duration: 86_400,
                receiver: None,
            },
            vec![coin(5_000, lp_denom.clone())],
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();
                match err {
                    ContractError::InvalidIdentifier { .. } => {}
                    _ => {
                        panic!("Wrong error type, should return ContractError::InvalidIdentifier")
                    }
                }
            },
        )
        .manage_position(
            &creator,
            PositionAction::Create {
                identifier: Some(
                    "7105920181635468364293788789264771059201816354683642937887892647a".to_string(),
                ),
                unlocking_duration: 86_400,
                receiver: None,
            },
            vec![coin(5_000, lp_denom.clone())],
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();
                match err {
                    ContractError::InvalidIdentifier { .. } => {}
                    _ => {
                        panic!("Wrong error type, should return ContractError::InvalidIdentifier")
                    }
                }
            },
        )
        .manage_position(
            &creator,
            PositionAction::Create {
                identifier: Some("INSERT INTO my_table (my_string) VALUES (values)".to_string()),
                unlocking_duration: 86_400,
                receiver: None,
            },
            vec![coin(5_000, lp_denom.clone())],
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();
                match err {
                    ContractError::InvalidIdentifier { .. } => {}
                    _ => {
                        panic!("Wrong error type, should return ContractError::InvalidIdentifier")
                    }
                }
            },
        )
        .manage_position(
            &creator,
            PositionAction::Create {
                identifier: Some("感".to_string()),
                unlocking_duration: 86_400,
                receiver: None,
            },
            vec![coin(5_000, lp_denom.clone())],
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();
                match err {
                    ContractError::InvalidIdentifier { .. } => {}
                    _ => {
                        panic!("Wrong error type, should return ContractError::InvalidIdentifier")
                    }
                }
            },
        );
}

#[test]
fn fails_to_create_farm_if_more_tokens_than_needed_were_sent() {
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
        .manage_farm(
            &creator,
            FarmAction::Fill {
                params: FarmParams {
                    lp_denom: lp_denom.clone(),
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
            vec![
                coin(4_000, "uusdy"),
                coin(1_000, "uom"),
                coin(1_000, "uosmo"),
            ],
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();
                match err {
                    ContractError::AssetMismatch { .. } => {}
                    _ => {
                        panic!("Wrong error type, should return ContractError::AssetMismatch")
                    }
                }
            },
        )
        .manage_farm(
            &creator,
            FarmAction::Fill {
                params: FarmParams {
                    lp_denom: lp_denom.clone(),
                    start_epoch: None,
                    preliminary_end_epoch: None,
                    curve: None,
                    farm_asset: Coin {
                        denom: "uom".to_string(),
                        amount: Uint128::new(4_000u128),
                    },
                    farm_identifier: None,
                },
            },
            vec![coin(5_000, "uom"), coin(1_000, "uosmo")],
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();
                match err {
                    ContractError::AssetMismatch { .. } => {}
                    _ => {
                        panic!("Wrong error type, should return ContractError::AssetMismatch")
                    }
                }
            },
        )
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
        .manage_farm(
            &creator,
            FarmAction::Fill {
                params: FarmParams {
                    lp_denom: lp_denom.clone(),
                    start_epoch: Some(12),
                    preliminary_end_epoch: Some(16),
                    curve: None,
                    farm_asset: Coin {
                        denom: "uom".to_string(),
                        amount: Uint128::new(8_000u128),
                    },
                    farm_identifier: None,
                },
            },
            vec![coin(9_000, "uom")],
            |result| {
                result.unwrap();
            },
        );
}

#[test]
fn fails_to_create_farm_if_start_epoch_is_zero() {
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

    suite.manage_farm(
        &creator,
        FarmAction::Fill {
            params: FarmParams {
                lp_denom: lp_denom.clone(),
                start_epoch: Some(0),
                preliminary_end_epoch: Some(28),
                curve: None,
                farm_asset: Coin {
                    denom: "uusdy".to_string(),
                    amount: Uint128::new(4_000u128),
                },
                farm_identifier: Some("farm_1".to_string()),
            },
        },
        vec![coin(4_000, "uusdy"), coin(1_000, "uom")],
        |result| {
            let err = result.unwrap_err().downcast::<ContractError>().unwrap();
            match err {
                ContractError::InvalidEpoch { which } => {
                    assert_eq!(which, "start".to_string())
                }
                _ => {
                    panic!("Wrong error type, should return ContractError::InvalidEpoch")
                }
            }
        },
    );
}

#[test]
fn overriding_farm_with_bogus_id_not_possible() {
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
        .manage_farm(
            &creator,
            FarmAction::Fill {
                params: FarmParams {
                    lp_denom: lp_denom.clone(),
                    start_epoch: None,
                    preliminary_end_epoch: None,
                    curve: None,
                    farm_asset: Coin {
                        denom: "uusdy".to_string(),
                        amount: Uint128::new(4_000u128),
                    },
                    farm_identifier: Some("1".to_string()),
                },
            },
            vec![coin(4_000, "uusdy"), coin(1_000, "uom")],
            |result| {
                result.unwrap();
            },
        )
        .manage_farm(
            &creator,
            FarmAction::Fill {
                params: FarmParams {
                    lp_denom: lp_denom.clone(),
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
        .query_farms(None, None, None, |result| {
            let farms_response = result.unwrap();
            assert_eq!(farms_response.farms.len(), 2);
            assert_eq!(farms_response.farms[0].identifier, "f-1");
            assert_eq!(farms_response.farms[1].identifier, "m-1");
        });
}

#[test]
fn closing_expired_farm_wont_pay_penalty() {
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

    let fee_collector = suite.fee_collector_addr.clone();

    suite
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
            Some(MAX_ITEMS_LIMIT),
            |result| {
                let response = result.unwrap();
                assert_eq!(response.positions.len(), 1);
                assert_eq!(response.positions[0].identifier, "p-1");
            },
        )
        .manage_position(
            &creator,
            PositionAction::Close {
                identifier: "p-1".to_string(),
                lp_asset: None,
            },
            vec![],
            |result| {
                result.unwrap();
            },
        )
        .add_one_epoch()
        .query_balance(lp_denom.clone(), &creator, |balance| {
            assert_eq!(balance, Uint128::new(999_990_000));
        })
        .query_balance(lp_denom.clone(), &fee_collector, |balance| {
            assert_eq!(balance, Uint128::zero());
        })
        .manage_position(
            &creator,
            PositionAction::Withdraw {
                identifier: "p-1".to_string(),
                // shouldn't pay emergency fee
                emergency_unlock: Some(true),
            },
            vec![],
            |result| {
                result.unwrap();
            },
        )
        .query_balance(lp_denom.clone(), &creator, |balance| {
            assert_eq!(balance, Uint128::new(1_000_000_000));
        })
        .query_balance(lp_denom.clone(), &fee_collector, |balance| {
            assert_eq!(balance, Uint128::zero());
        });
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
            Some(MAX_ITEMS_LIMIT),
            |result| {
                let response = result.unwrap();
                assert_eq!(response.positions.len(), 3);
                assert_eq!(response.positions[0].identifier, "p-1");
                assert_eq!(response.positions[1].identifier, "u-custom_id_1");
                assert_eq!(response.positions[2].identifier, "u-custom_id_2");
            },
        );
}

#[test]
fn providing_custom_farm_id_doesnt_increment_farm_counter() {
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
        .manage_farm(
            &creator,
            FarmAction::Fill {
                params: FarmParams {
                    lp_denom: lp_denom.clone(),
                    start_epoch: Some(12),
                    preliminary_end_epoch: Some(16),
                    curve: None,
                    farm_asset: Coin {
                        denom: "uom".to_string(),
                        amount: Uint128::new(8_000u128),
                    },
                    farm_identifier: Some("custom_id_1".to_string()),
                },
            },
            vec![coin(9_000, "uom")],
            |result| {
                result.unwrap();
            },
        )
        .manage_farm(
            &creator,
            FarmAction::Fill {
                params: FarmParams {
                    lp_denom: lp_denom.clone(),
                    start_epoch: Some(12),
                    preliminary_end_epoch: Some(16),
                    curve: None,
                    farm_asset: Coin {
                        denom: "uom".to_string(),
                        amount: Uint128::new(8_000u128),
                    },
                    farm_identifier: None,
                },
            },
            vec![coin(9_000, "uom")],
            |result| {
                result.unwrap();
            },
        )
        .query_farms(None, None, None, |result| {
            let response = result.unwrap();
            assert_eq!(response.farms.len(), 2);
            assert_eq!(response.farms[0].identifier, "f-1");
            assert_eq!(response.farms[1].identifier, "m-custom_id_1");
        });
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
        coin(1_000_000_000u128, "uom".to_string()),
        coin(1_000_000_000u128, "uusdy".to_string()),
        coin(1_000_000_000u128, "uosmo".to_string()),
        coin(1_000_000_000_000, lp_denom_1.clone()),
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
                    denom: "uusdy".to_string(),
                    amount: Uint128::new(3333u128),
                },
                farm_identifier: None,
            },
        },
        vec![coin(3333u128, "uusdy"), coin(1_000, "uom")],
        |result| {
            result.unwrap();
        },
    );

    // creator and other fill a position
    suite.manage_position(
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
    );

    suite
        .add_one_epoch()
        .add_one_epoch()
        .add_one_epoch()
        .add_one_epoch()
        .add_one_epoch()
        .query_current_epoch(|result| {
            let epoch_response = result.unwrap();
            assert_eq!(epoch_response.epoch.id, 5);
        });

    suite.query_rewards(&creator, |result| {
        result.unwrap();
    });

    suite
        .claim(&creator, vec![], |result| {
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
        .query_rewards(&creator, |result| {
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

    suite
        .add_one_epoch()
        .add_one_epoch()
        .query_rewards(&creator, |result| {
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

    // this would normally fail as in some point of the reward calculation the total_lp_weight
    // would be zero.
    // This is a case that the contract shouldn't compute rewards for anyway, so the epoch is skipped.
    suite
        .query_rewards(&creator, |result| {
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
        .claim(&creator, vec![], |result| {
            result.unwrap();
        })
        .query_lp_weight(&creator, &lp_denom_1, 7, |result| {
            result.unwrap_err();
        })
        .query_lp_weight(&creator, &lp_denom_1, 8, |result| {
            let lp_weight_response = result.unwrap();
            assert_eq!(lp_weight_response.lp_weight, Uint128::new(2_000));
        })
        .query_lp_weight(&creator, &lp_denom_1, 9, |result| {
            result.unwrap_err();
        });

    suite
        .add_one_epoch()
        .add_one_epoch()
        .query_rewards(&creator, |result| {
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
        .query_rewards(&creator, |result| {
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

/// This test creates multiple farms, and multiple positions with different users. Users open and close
/// and withdraw positions in different fashion, and claim rewards. The test checks if the rewards
/// are calculated correctly, and if the positions are managed correctly.
#[test]
fn test_managing_positions_close_and_emergency_withdraw() {
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
    let carol = suite.senders[2].clone();

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
                    start_epoch: Some(10),
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

    // alice locks liquidity early
    suite.manage_position(
        &alice,
        PositionAction::Create {
            identifier: Some("alice_position_1".to_string()),
            unlocking_duration: 86_400,
            receiver: None,
        },
        vec![coin(333, lp_denom_1.clone())],
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
            assert_eq!(epoch_response.epoch.id, 5);
        });

    // then bob joins alice after a few epochs, having positions in both farms
    suite
        .manage_position(
            &bob,
            PositionAction::Create {
                identifier: Some("bob_position_1".to_string()),
                unlocking_duration: 86_400,
                receiver: None,
            },
            vec![coin(666, lp_denom_1.clone())],
            |result| {
                result.unwrap();
            },
        )
        .manage_position(
            &bob,
            PositionAction::Create {
                identifier: Some("bob_position_2".to_string()),
                unlocking_duration: 86_400,
                receiver: None,
            },
            vec![coin(666, lp_denom_2.clone())],
            |result| {
                result.unwrap();
            },
        );

    suite
        .query_rewards(&alice, |result| {
            let rewards_response = result.unwrap();
            match rewards_response {
                RewardsResponse::RewardsResponse { total_rewards, .. } => {
                    assert_eq!(total_rewards.len(), 1);
                    assert_eq!(total_rewards[0], coin(3_170u128, "uusdy"));
                }
                _ => {
                    panic!("Wrong response type, should return RewardsResponse::RewardsResponse")
                }
            }
        })
        .query_rewards(&bob, |result| {
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
        .query_rewards(&carol, |result| {
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

    suite
        .query_balance("uusdy".to_string(), &alice, |balance| {
            assert_eq!(
                balance,
                Uint128::new(1_000_000_000u128 - (8_888u128 + 666_666u128))
            );
        })
        .claim(&alice, vec![], |result| {
            result.unwrap();
        })
        .query_balance("uusdy".to_string(), &alice, |balance| {
            assert_eq!(
                balance,
                Uint128::new(1_000_000_000u128 - (8_888u128 + 666_666u128) + 3_170u128)
            );
        });

    // last claimed epoch for alice = 5
    suite
        .add_one_epoch()
        .add_one_epoch()
        .add_one_epoch()
        .query_current_epoch(|result| {
            let epoch_response = result.unwrap();
            assert_eq!(epoch_response.epoch.id, 8);
        });

    // then carol joins alice and bob after a few epochs
    suite.manage_position(
        &carol,
        PositionAction::Create {
            identifier: Some("carol_position_2".to_string()),
            unlocking_duration: 86_400,
            receiver: None,
        },
        vec![coin(1_000, lp_denom_2.clone())],
        |result| {
            result.unwrap();
        },
    );

    // create two more farms, one overlapping, the other one not.
    suite
        .manage_farm(
            &alice,
            FarmAction::Fill {
                params: FarmParams {
                    lp_denom: lp_denom_1.clone(),
                    start_epoch: Some(15),
                    preliminary_end_epoch: Some(20),
                    curve: None,
                    farm_asset: Coin {
                        denom: "uosmo".to_string(),
                        amount: Uint128::new(8_888u128),
                    },
                    farm_identifier: None,
                },
            },
            vec![coin(8_888u128, "uosmo"), coin(1_000, "uom")],
            |result| {
                result.unwrap();
            },
        )
        .manage_farm(
            &alice,
            FarmAction::Fill {
                params: FarmParams {
                    lp_denom: lp_denom_2.clone(),
                    start_epoch: Some(22),
                    preliminary_end_epoch: None,
                    curve: None,
                    farm_asset: Coin {
                        denom: "uom".to_string(),
                        amount: Uint128::new(1_000_000u128),
                    },
                    farm_identifier: None,
                },
            },
            vec![coin(1_001_000u128, "uom")],
            |result| {
                result.unwrap();
            },
        );

    suite
        .query_rewards(&alice, |result| {
            let rewards_response = result.unwrap();
            match rewards_response {
                RewardsResponse::RewardsResponse { total_rewards, .. } => {
                    assert_eq!(total_rewards.len(), 1);
                    assert_eq!(total_rewards[0], coin(633u128, "uusdy"));
                }
                _ => {
                    panic!("Wrong response type, should return RewardsResponse::RewardsResponse")
                }
            }
        })
        .query_rewards(&bob, |result| {
            let rewards_response = result.unwrap();
            match rewards_response {
                RewardsResponse::RewardsResponse { total_rewards, .. } => {
                    assert_eq!(total_rewards.len(), 1);
                    assert_eq!(total_rewards[0], coin(1_266u128, "uusdy"));
                }
                _ => {
                    panic!("Wrong response type, should return RewardsResponse::RewardsResponse")
                }
            }
        })
        .query_rewards(&carol, |result| {
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

    // now alice emergency withdraws her position, giving up her rewards
    suite
        .manage_position(
            &alice,
            PositionAction::Withdraw {
                identifier: "u-alice_position_1".to_string(),
                emergency_unlock: Some(true),
            },
            vec![],
            |result| {
                result.unwrap();
            },
        )
        .query_rewards(&alice, |result| {
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
        // Bob's rewards should remain the same for the current epoch
        .query_rewards(&bob, |result| {
            let rewards_response = result.unwrap();
            match rewards_response {
                RewardsResponse::RewardsResponse { total_rewards, .. } => {
                    assert_eq!(total_rewards.len(), 1);
                    assert_eq!(total_rewards[0], coin(1_266u128, "uusdy"));
                }
                _ => {
                    panic!("Wrong response type, should return RewardsResponse::RewardsResponse")
                }
            }
        });

    suite.add_one_epoch().query_current_epoch(|result| {
        let epoch_response = result.unwrap();
        assert_eq!(epoch_response.epoch.id, 9);
    });

    suite.query_rewards(&bob, |result| {
        let rewards_response = result.unwrap();
        match rewards_response {
            RewardsResponse::RewardsResponse { total_rewards, .. } => {
                assert_eq!(total_rewards.len(), 1);
                // 634 is the emission rate for farm 1
                assert_eq!(total_rewards[0], coin(1_266u128 + 634, "uusdy"));
            }
            _ => {
                panic!("Wrong response type, should return RewardsResponse::RewardsResponse")
            }
        }
    });

    // alice creates a new position with the same LP denom
    suite
        .manage_position(
            &alice,
            PositionAction::Create {
                identifier: Some("alice_second_position_1".to_string()),
                unlocking_duration: 86_400,
                receiver: None,
            },
            vec![coin(300, lp_denom_1.clone())],
            |result| {
                result.unwrap();
            },
        )
        .manage_position(
            &alice,
            PositionAction::Create {
                identifier: Some("alice_second_position_2".to_string()),
                unlocking_duration: 86_400,
                receiver: None,
            },
            vec![coin(700, lp_denom_1.clone())],
            |result| {
                result.unwrap();
            },
        );

    suite.add_one_epoch();

    suite
        .query_rewards(&alice, |result| {
            let rewards_response = result.unwrap();
            match rewards_response {
                RewardsResponse::RewardsResponse { total_rewards, .. } => {
                    assert_eq!(total_rewards.len(), 1);
                    assert_eq!(total_rewards[0], coin(380u128, "uusdy"));
                }
                _ => {
                    panic!("Wrong response type, should return RewardsResponse::RewardsResponse")
                }
            }
        })
        .claim(&alice, vec![], |result| {
            result.unwrap();
        });

    suite.add_one_epoch().add_one_epoch();

    suite
        .manage_position(
            &alice,
            PositionAction::Withdraw {
                identifier: "u-alice_second_position_1".to_string(),
                emergency_unlock: Some(true),
            },
            vec![],
            |result| {
                result.unwrap();
            },
        )
        .claim(&alice, vec![], |result| {
            result.unwrap();
        });

    suite
        .add_one_epoch()
        .query_rewards(&alice, |result| {
            let rewards_response = result.unwrap();
            match rewards_response {
                RewardsResponse::RewardsResponse { total_rewards, .. } => {
                    assert_eq!(total_rewards.len(), 1usize);
                    assert_eq!(total_rewards[0], coin(324u128, "uusdy"));
                }
                _ => {
                    panic!("Wrong response type, should return RewardsResponse::RewardsResponse")
                }
            }
        })
        .query_balance("uusdy".to_string(), &alice, |balance| {
            assert_eq!(balance, Uint128::new(999_328_756u128));
        })
        .claim(&alice, vec![], |result| {
            result.unwrap();
        })
        .query_balance("uusdy".to_string(), &alice, |balance| {
            assert_eq!(balance, Uint128::new(999_328_756u128 + 324u128));
        })
        .manage_position(
            &alice,
            PositionAction::Close {
                identifier: "u-alice_second_position_2".to_string(),
                lp_asset: Some(coin(500, lp_denom_1.clone())),
            },
            vec![],
            |result| {
                result.unwrap();
            },
        );

    suite.add_one_epoch();

    suite.query_current_epoch(|result| {
        let epoch_response = result.unwrap();
        assert_eq!(epoch_response.epoch.id, 14);
    });

    suite
        .manage_position(
            &alice,
            PositionAction::Withdraw {
                identifier: "p-1".to_string(),
                emergency_unlock: None,
            },
            vec![],
            |result| {
                result.unwrap();
            },
        )
        .manage_position(
            &alice,
            PositionAction::Withdraw {
                identifier: "u-alice_second_position_2".to_string(),
                emergency_unlock: Some(true),
            },
            vec![],
            |result| {
                result.unwrap();
            },
        )
        .query_lp_weight(&alice, &lp_denom_1, 15, |result| {
            result.unwrap_err();
        })
        .query_lp_weight(&alice, &lp_denom_1, 14, |result| {
            result.unwrap_err();
        })
        .query_lp_weight(&alice, &lp_denom_1, 13, |result| {
            result.unwrap_err();
        })
        .query_lp_weight(&alice, &lp_denom_1, 12, |result| {
            result.unwrap_err();
        });

    suite
        .query_rewards(&alice, |result| {
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
        .manage_position(
            &alice,
            PositionAction::Create {
                identifier: Some("final_alice_position".to_string()),
                unlocking_duration: 86_400,
                receiver: None,
            },
            vec![coin(3000, lp_denom_1.clone())],
            |result| {
                result.unwrap();
            },
        )
        .query_rewards(&alice, |result| {
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
        .query_lp_weight(&alice, &lp_denom_1, 14, |result| {
            result.unwrap_err();
        })
        .query_lp_weight(&alice, &lp_denom_1, 15, |result| {
            let lp_weight_response = result.unwrap();
            assert_eq!(lp_weight_response.lp_weight, Uint128::new(3000));
        })
        .add_one_epoch()
        .query_rewards(&alice, |result| {
            let rewards_response = result.unwrap();
            match rewards_response {
                RewardsResponse::RewardsResponse { total_rewards, .. } => {
                    assert_eq!(total_rewards.len(), 1usize);
                    assert_eq!(total_rewards[0], coin(1_454, "uosmo"));
                }
                _ => {
                    panic!("Wrong response type, should return RewardsResponse::RewardsResponse")
                }
            }
        });

    suite
        .query_rewards(&bob, |result| {
            let rewards_response = result.unwrap();
            match rewards_response {
                RewardsResponse::RewardsResponse { total_rewards, .. } => {
                    assert_eq!(total_rewards.len(), 2usize);
                    assert_eq!(
                        total_rewards,
                        vec![coin(322u128, "uosmo"), coin(163_355u128, "uusdy")]
                    );
                }
                _ => {
                    panic!("Wrong response type, should return RewardsResponse::RewardsResponse")
                }
            }
        })
        .query_balance("uusdy".to_string(), &bob, |balance| {
            assert_eq!(balance, Uint128::new(1_000_000_000u128));
        })
        .query_balance("uosmo".to_string(), &bob, |balance| {
            assert_eq!(balance, Uint128::new(1_000_000_000u128));
        })
        .claim(&bob, vec![], |result| {
            result.unwrap();
        })
        .query_balance("uusdy".to_string(), &bob, |balance| {
            assert_eq!(balance, Uint128::new(1_000_000_000u128 + 163_355u128));
        })
        .query_balance("uosmo".to_string(), &bob, |balance| {
            assert_eq!(balance, Uint128::new(1_000_000_000u128 + 322u128));
        });

    suite
        .add_one_epoch()
        .add_one_epoch()
        .add_one_epoch()
        .query_current_epoch(|result| {
            let epoch_response = result.unwrap();
            assert_eq!(epoch_response.epoch.id, 18);
        });

    suite
        .query_rewards(&bob, |result| {
            let rewards_response = result.unwrap();
            match rewards_response {
                RewardsResponse::RewardsResponse { total_rewards, .. } => {
                    assert_eq!(total_rewards.len(), 2usize);
                    assert_eq!(
                        total_rewards,
                        vec![coin(966u128, "uosmo"), coin(79_950u128, "uusdy")]
                    );
                }
                _ => {
                    panic!("Wrong response type, should return RewardsResponse::RewardsResponse")
                }
            }
        })
        // since bob didn't have more positions for lp1, the lp_weight_history gets wiped for that lp denom
        .manage_position(
            &bob,
            PositionAction::Withdraw {
                identifier: "u-bob_position_1".to_string(),
                emergency_unlock: Some(true),
            },
            vec![],
            |result| {
                result.unwrap();
            },
        )
        .query_rewards(&bob, |result| {
            let rewards_response = result.unwrap();
            match rewards_response {
                RewardsResponse::RewardsResponse { total_rewards, .. } => {
                    assert_eq!(total_rewards.len(), 1usize);
                    assert_eq!(total_rewards, vec![coin(79_950u128, "uusdy")]);
                }
                _ => {
                    panic!("Wrong response type, should return RewardsResponse::RewardsResponse")
                }
            }
        })
        // creating a new position for bob with the lp denom 1 won't give him the rewards in the past
        // epochs he had but gave up by emergency withdrawing
        .manage_position(
            &bob,
            PositionAction::Create {
                identifier: Some("new_bob_position_lp_1".to_string()),
                unlocking_duration: 86_400,
                receiver: None,
            },
            vec![coin(1_000, lp_denom_1.clone())],
            |result| {
                result.unwrap();
            },
        )
        .query_rewards(&bob, |result| {
            let rewards_response = result.unwrap();
            match rewards_response {
                RewardsResponse::RewardsResponse { total_rewards, .. } => {
                    assert_eq!(total_rewards.len(), 1usize);
                    assert_eq!(total_rewards, vec![coin(79_950u128, "uusdy")]);
                }
                _ => {
                    panic!("Wrong response type, should return RewardsResponse::RewardsResponse")
                }
            }
        })
        .claim(&bob, vec![], |result| {
            result.unwrap();
        })
        .add_one_epoch()
        .query_rewards(&bob, |result| {
            let rewards_response = result.unwrap();
            match rewards_response {
                RewardsResponse::RewardsResponse { total_rewards, .. } => {
                    assert_eq!(total_rewards.len(), 2usize);
                    assert_eq!(
                        total_rewards,
                        vec![coin(444, "uosmo"), coin(26_650u128, "uusdy")]
                    );
                }
                _ => {
                    panic!("Wrong response type, should return RewardsResponse::RewardsResponse")
                }
            }
        });
}

#[test]
#[allow(clippy::inconsistent_digit_grouping)]
pub fn can_emergency_withdraw_an_lp_without_farm() {
    let lp_denom = format!("factory/{MOCK_CONTRACT_ADDR_1}/{LP_SYMBOL}").to_string();
    let lp_without_farm = format!("factory/{MOCK_CONTRACT_ADDR_1}/2.{LP_SYMBOL}").to_string();

    let mut suite = TestingSuite::default_with_balances(vec![
        coin(1_000_000_000u128, "uom"),
        coin(1_000_000_000u128, "uusdy"),
        coin(1_000_000_000u128, "uosmo"),
        coin(1_000_000_000u128, lp_denom.clone()),
        coin(1_000_000_000u128, lp_without_farm.clone()),
    ]);

    let creator = suite.creator();

    suite.instantiate_default();

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
        .manage_position(
            &creator,
            PositionAction::Create {
                identifier: Some("creator_position".to_string()),
                unlocking_duration: 86_400,
                receiver: None,
            },
            vec![coin(2_000, lp_without_farm.clone())],
            |result| {
                result.unwrap();
            },
        );

    suite.add_one_epoch().add_one_epoch();

    // withdraw the position
    suite.manage_position(
        &creator,
        PositionAction::Withdraw {
            identifier: "u-creator_position".to_string(),
            emergency_unlock: Some(true),
        },
        vec![],
        |result| {
            result.unwrap();
        },
    );
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

    suite.query_rewards(&dan, |result| {
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
