extern crate core;

use amm::constants::LP_SYMBOL;
use cosmwasm_std::{coin, Addr, Coin, Decimal, Uint128};
use cw_utils::PaymentError;

use amm::farm_manager::{
    Config, Curve, Farm, FarmAction, FarmParams, FarmsBy, LpWeightResponse, Position,
    PositionAction, RewardsResponse,
};
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
        Decimal::percent(101),
        |result| {
            let err = result.unwrap_err().downcast::<ContractError>().unwrap();

            match err {
                ContractError::InvalidEmergencyUnlockPenalty { .. } => {}
                _ => panic!("Wrong error type, should return ContractError::InvalidEmergencyUnlockPenalty"),
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
            Some(FarmsBy::Identifier("farm_1".to_string())),
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
            Some(FarmsBy::Identifier("2".to_string())),
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
                    farm_identifier: Some("farm_1".to_string()),
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
                    farm_identifier: Some("farm_1".to_string()),
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
                    farm_identifier: Some("farm_1".to_string()),
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
                    farm_identifier: Some("farm_1".to_string()),
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
                    farm_identifier: Some("farm_1".to_string()),
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
                    farm_identifier: Some("farm_1".to_string()),
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
            Some(FarmsBy::Identifier("farm_1".to_string())),
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
                    farm_identifier: Some("farm_1".to_string()),
                },
            },
            vec![coin(5_000u128, "uusdy")],
            |result| {
                result.unwrap();
            },
        )
        .query_farms(
            Some(FarmsBy::Identifier("farm_1".to_string())),
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

    let mut suite = TestingSuite::default_with_balances(vec![
        coin(1_000_000_000u128, "uom".to_string()),
        coin(1_000_000_000u128, "uusdy".to_string()),
        coin(1_000_000_000u128, "uosmo".to_string()),
        coin(1_000_000_000u128, lp_denom.clone()),
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
                farm_identifier: "farm_1".to_string(),
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
                farm_identifier: "farm_2".to_string(),
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
                farm_identifier: "farm_1".to_string(),
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
                farm_identifier: "farm_1".to_string(),
            },
            vec![],
            |result| {
                result.unwrap();
            },
        )
        .query_balance("uusdy".to_string(), &other, |balance| {
            assert_eq!(balance, Uint128::new(1000_000_000));
        });
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
        max_unlocking_duration: 31_536_000u64,
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
        Some(Decimal::percent(50)),
        vec![],
        |result| {
            let err = result.unwrap_err().downcast::<ContractError>().unwrap();
            match err {
                ContractError::UnspecifiedConcurrentFarms { .. } => {}
                _ => panic!("Wrong error type, should return ContractError::UnspecifiedConcurrentFarms"),
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
        emergency_unlock_penalty: Decimal::percent(20),
    };

    suite.query_config(|result| {
        let config = result.unwrap();
        assert_eq!(config, expected_config);
    });
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
        .query_lp_weight(&lp_denom, 0, |result| {
            let err = result.unwrap_err().to_string();

            assert_eq!(
                err,
                "Generic error: Querier contract error: There's no snapshot of the LP \
           weight in the contract for the epoch 0"
            );
        })
        .manage_position(
            &creator,
            PositionAction::Fill {
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
            PositionAction::Fill {
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
            PositionAction::Fill {
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
            PositionAction::Fill {
                identifier: Some("creator_position".to_string()),
                unlocking_duration: 86_400,
                receiver: None,
            },
            vec![coin(1_000, lp_denom.clone())],
            |result| {
                result.unwrap();
            },
        )
        .query_lp_weight(&lp_denom, 1, |result| {
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
            PositionAction::Fill {
                identifier: Some("creator_position".to_string()),
                unlocking_duration: 86_400,
                receiver: None,
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
        .query_positions(&creator, Some(true), |result| {
            let positions = result.unwrap();
            assert_eq!(positions.positions.len(), 1);
            assert_eq!(
                positions.positions[0],
                Position {
                    identifier: "creator_position".to_string(),
                    lp_asset: Coin {
                        denom: format!("factory/{MOCK_CONTRACT_ADDR_1}/{LP_SYMBOL}").to_string(),
                        amount: Uint128::new(1_000),
                    },
                    unlocking_duration: 86400,
                    open: true,
                    expiring_at: None,
                    receiver: creator.clone(),
                }
            );
        })
        .manage_position(
            &creator,
            PositionAction::Fill {
                identifier: Some("creator_position".to_string()),
                unlocking_duration: 86_400,
                receiver: None,
            },
            vec![coin(5_000, lp_denom.clone())],
            |result| {
                result.unwrap();
            },
        )
        .manage_position(
            &creator,
            PositionAction::Withdraw {
                identifier: "creator_position".to_string(),
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
        .query_lp_weight(&lp_denom, 1, |result| {
            let lp_weight = result.unwrap();
            assert_eq!(
                lp_weight,
                LpWeightResponse {
                    lp_weight: Uint128::new(6_000),
                    epoch_id: 1,
                }
            );
        })
        .query_positions(&creator, Some(true), |result| {
            let positions = result.unwrap();
            assert_eq!(positions.positions.len(), 1);
            assert_eq!(
                positions.positions[0],
                Position {
                    identifier: "creator_position".to_string(),
                    lp_asset: Coin {
                        denom: format!("factory/{MOCK_CONTRACT_ADDR_1}/{LP_SYMBOL}").to_string(),
                        amount: Uint128::new(6_000),
                    },
                    unlocking_duration: 86400,
                    open: true,
                    expiring_at: None,
                    receiver: creator.clone(),
                }
            );
        })
        .query_lp_weight(&lp_denom, 1, |result| {
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
            PositionAction::Fill {
                //refill position
                identifier: Some("creator_position".to_string()),
                unlocking_duration: 86_400,
                receiver: None,
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
                identifier: "creator_position".to_string(),
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
                identifier: "creator_position".to_string(),
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
            assert_eq!(farms_response.farms[0].claimed_amount, Uint128::new(2_000),);
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
                identifier: "creator_position".to_string(),
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
                identifier: "creator_position".to_string(),
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
                identifier: "creator_position".to_string(),
                lp_asset: Some(Coin {
                    denom: lp_denom.to_string(),
                    amount: Uint128::new(10_000),
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
            &creator,
            PositionAction::Close {
                // remove 5_000 from the 7_000 position
                identifier: "creator_position".to_string(),
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
                identifier: "2".to_string(),
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
        .query_lp_weight(&lp_denom, 3, |result| {
            let lp_weight = result.unwrap();
            assert_eq!(
                lp_weight,
                LpWeightResponse {
                    // should be the same for epoch 2, as the weight for new positions is added
                    // to the next epoch
                    lp_weight: Uint128::new(5_000),
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
                identifier: "creator_position".to_string(),
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
                identifier: "2".to_string(),
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
                RewardsResponse::RewardsResponse { rewards } => {
                    assert_eq!(rewards.len(), 1);
                    assert_eq!(
                        rewards[0],
                        Coin {
                            denom: "uusdy".to_string(),
                            amount: Uint128::new(6_000),
                        }
                    );
                }
                RewardsResponse::ClaimRewards { .. } => {
                    panic!("shouldn't return this but RewardsResponse")
                }
            }
        })
        .query_farms(None, None, None, |result| {
            let farms_response = result.unwrap();
            assert_eq!(farms_response.farms[0].claimed_amount, Uint128::new(2_000));
        })
        .manage_position(
            &creator,
            PositionAction::Withdraw {
                identifier: "2".to_string(),
                emergency_unlock: None,
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
        .claim(&creator, vec![], |result| {
            result.unwrap();
        })
        .query_balance("uusdy".to_string(), &creator, |balance| {
            assert_eq!(balance, Uint128::new(1000_000_000));
        })
        .query_farms(None, None, None, |result| {
            let farms_response = result.unwrap();
            assert_eq!(
                farms_response.farms[0].farm_asset.amount,
                farms_response.farms[0].claimed_amount
            );
            assert!(farms_response.farms[0].is_expired(5));
        })
        .query_rewards(&creator, |result| {
            let rewards_response = result.unwrap();
            match rewards_response {
                RewardsResponse::RewardsResponse { rewards } => {
                    assert!(rewards.is_empty());
                }
                RewardsResponse::ClaimRewards { .. } => {
                    panic!("shouldn't return this but RewardsResponse")
                }
            }
        })
        .claim(&creator, vec![], |result| {
            result.unwrap();
        })
        .query_balance("uusdy".to_string(), &creator, |balance| {
            assert_eq!(balance, Uint128::new(1000_000_000));
        })
        .manage_position(
            &creator,
            PositionAction::Withdraw {
                identifier: "2".to_string(),
                emergency_unlock: None,
            },
            vec![],
            |result| {
                result.unwrap();
            },
        )
        .query_positions(&other, Some(false), |result| {
            let positions = result.unwrap();
            assert!(positions.positions.is_empty());
        })
        .manage_position(
            &creator,
            PositionAction::Fill {
                identifier: None,
                unlocking_duration: 86_400,
                receiver: Some(another.to_string()),
            },
            vec![coin(5_000, lp_denom.clone())],
            |result| {
                result.unwrap();
            },
        )
        .query_positions(&another, Some(true), |result| {
            let positions = result.unwrap();
            assert_eq!(positions.positions.len(), 1);
            assert_eq!(
                positions.positions[0],
                Position {
                    identifier: "3".to_string(),
                    lp_asset: Coin {
                        denom: format!("factory/{MOCK_CONTRACT_ADDR_1}/{LP_SYMBOL}").to_string(),
                        amount: Uint128::new(5_000),
                    },
                    unlocking_duration: 86400,
                    open: true,
                    expiring_at: None,
                    receiver: another.clone(),
                }
            );
        })
        .manage_position(
            &creator,
            PositionAction::Close {
                identifier: "3".to_string(),
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
                identifier: "3".to_string(),
                lp_asset: None, //close in full
            },
            vec![],
            |result| {
                result.unwrap();
            },
        )
        .query_positions(&another, Some(true), |result| {
            let positions = result.unwrap();
            assert!(positions.positions.is_empty());
        })
        .query_positions(&another, Some(false), |result| {
            let positions = result.unwrap();
            assert_eq!(positions.positions.len(), 1);
            assert_eq!(
                positions.positions[0],
                Position {
                    identifier: "3".to_string(),
                    lp_asset: Coin {
                        denom: format!("factory/{MOCK_CONTRACT_ADDR_1}/{LP_SYMBOL}").to_string(),
                        amount: Uint128::new(5_000),
                    },
                    unlocking_duration: 86400,
                    open: false,
                    expiring_at: Some(1712847600),
                    receiver: another.clone(),
                }
            );
        });

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
            PositionAction::Fill {
                identifier: Some("special_position".to_string()),
                unlocking_duration: 100_000,
                receiver: None,
            },
            vec![coin(5_000, lp_denom.clone())],
            |result| {
                result.unwrap();
            },
        )
        .query_lp_weight(&lp_denom, 9, |result| {
            let lp_weight = result.unwrap();
            assert_eq!(
                lp_weight,
                LpWeightResponse {
                    lp_weight: Uint128::new(10_002),
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
                identifier: "special_position".to_string(),
                lp_asset: None,
            },
            vec![],
            |result| {
                result.unwrap();
            },
        )
        .query_lp_weight(&lp_denom, 10, |result| {
            let lp_weight = result.unwrap();
            assert_eq!(
                lp_weight,
                LpWeightResponse {
                    // the weight went back to what it was before the position was opened
                    lp_weight: Uint128::new(5_000),
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
                identifier: "special_position".to_string(),
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
                identifier: "special_position".to_string(),
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
        PositionAction::Fill {
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
            PositionAction::Fill {
                identifier: Some("creator_position".to_string()),
                unlocking_duration: 86_400,
                receiver: None,
            },
            vec![coin(10_000, &lp_denom)],
            |result| {
                result.unwrap();
            },
        )
        .query_positions(&creator, None, |result| {
            let positions = result.unwrap();
            assert_eq!(positions.positions.len(), 1);
            assert_eq!(
                positions.positions[0],
                Position {
                    identifier: "creator_position".to_string(),
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
        })
        .add_one_epoch()
        // close position
        .manage_position(
            &creator,
            PositionAction::Close {
                identifier: "creator_position".to_string(),
                lp_asset: None,
            },
            vec![],
            |result| {
                result.unwrap();
            },
        )
        .query_positions(&creator, None, |result| {
            let positions = result.unwrap();
            assert_eq!(positions.positions.len(), 1);
            assert_eq!(
                positions.positions[0],
                Position {
                    identifier: "creator_position".to_string(),
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
        })
        // try refilling the closed position should err
        .manage_position(
            &creator,
            PositionAction::Fill {
                identifier: Some("creator_position".to_string()),
                unlocking_duration: 86_400,
                receiver: None,
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
            PositionAction::Fill {
                identifier: Some("creator_position".to_string()),
                unlocking_duration: 86_400,
                receiver: None,
            },
            vec![coin(5_000, lp_denom.clone())],
            |result| {
                result.unwrap();
            },
        )
        .query_lp_weight(&lp_denom, 11, |result| {
            let lp_weight = result.unwrap();
            assert_eq!(
                lp_weight,
                LpWeightResponse {
                    lp_weight: Uint128::new(5_000),
                    epoch_id: 11,
                }
            );
        })
        .query_positions(&other, Some(true), |result| {
            let positions = result.unwrap();
            assert_eq!(positions.positions.len(), 1);
            assert_eq!(
                positions.positions[0],
                Position {
                    identifier: "creator_position".to_string(),
                    lp_asset: Coin {
                        denom: format!("factory/{MOCK_CONTRACT_ADDR_1}/{LP_SYMBOL}").to_string(),
                        amount: Uint128::new(5_000),
                    },
                    unlocking_duration: 86400,
                    open: true,
                    expiring_at: None,
                    receiver: other.clone(),
                }
            );
        });

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
                RewardsResponse::RewardsResponse { rewards } => {
                    assert!(rewards.is_empty());
                }
                RewardsResponse::ClaimRewards { .. } => {
                    panic!("shouldn't return this but RewardsResponse")
                }
            }
        })
        .claim(&other, vec![], |result| {
            result.unwrap();
        })
        .query_balance("uusdy".to_string(), &other, |balance| {
            // the balance hasn't changed
            assert_eq!(balance, Uint128::new(1_000_006_000u128));
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
                farm_identifier: None,
            },
        },
        vec![coin(8_000, "uusdy"), coin(1_000, "uom")],
        |result| {
            result.unwrap();
        },
    );

    // create a bunch of epochs to make the farm expire
    for _ in 0..20 {
        suite.add_one_epoch();
    }

    let mut current_id = 0;

    // try opening another farm for the same lp denom, the expired farm should get closed
    suite
        .query_current_epoch(|result| {
            let epoch_response = result.unwrap();
            current_id = epoch_response.epoch.id;
        })
        .query_farms(None, None, None, |result| {
            let farms_response = result.unwrap();
            assert_eq!(farms_response.farms.len(), 1);
            assert!(farms_response.farms[0].is_expired(current_id));
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
                    identifier: "new_farm".to_string(),
                    owner: other.clone(),
                    lp_denom: lp_denom.clone(),
                    farm_asset: Coin {
                        denom: "uusdy".to_string(),
                        amount: Uint128::new(10_000u128),
                    },
                    claimed_amount: Uint128::zero(),
                    emission_rate: Uint128::new(714),
                    curve: Curve::Linear,
                    start_epoch: 31u64,
                    preliminary_end_epoch: 45u64,
                    last_epoch_claimed: 30u64,
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
                    amount: Uint128::new(4_000u128),
                },
                farm_identifier: Some("farm".to_string()),
            },
        },
        vec![coin(4_000, "uusdy"), coin(1_000, "uom")],
        |result| {
            result.unwrap();
        },
    );

    // create a bunch of epochs to make the farm expire
    for _ in 0..15 {
        suite.add_one_epoch();
    }

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
                    amount: Uint128::new(8_000u128),
                },
                farm_identifier: Some("farm".to_string()),
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
            PositionAction::Fill {
                identifier: Some("other_position".to_string()),
                unlocking_duration: 86_400,
                receiver: None,
            },
            vec![coin(1_000, lp_denom.clone())],
            |result| {
                result.unwrap();
            },
        )
        .query_positions(&other, Some(true), |result| {
            let positions = result.unwrap();
            assert_eq!(positions.positions.len(), 1);
            assert_eq!(
                positions.positions[0],
                Position {
                    identifier: "other_position".to_string(),
                    lp_asset: Coin {
                        denom: format!("factory/{MOCK_CONTRACT_ADDR_1}/{LP_SYMBOL}").to_string(),
                        amount: Uint128::new(1_000),
                    },
                    unlocking_duration: 86400,
                    open: true,
                    expiring_at: None,
                    receiver: other.clone(),
                }
            );
        })
        .query_balance(lp_denom.clone().to_string(), &other, |balance| {
            assert_eq!(balance, Uint128::new(999_999_000));
        })
        .query_balance(lp_denom.clone().to_string(), &fee_collector, |balance| {
            assert_eq!(balance, Uint128::zero());
        })
        .manage_position(
            &other,
            PositionAction::Withdraw {
                identifier: "other_position".to_string(),
                emergency_unlock: Some(true),
            },
            vec![],
            |result| {
                result.unwrap();
            },
        )
        .query_balance(lp_denom.clone().to_string(), &other, |balance| {
            //emergency unlock penalty is 10% of the position amount, so the user gets 1000 - 100 = 900
            assert_eq!(balance, Uint128::new(999_999_900));
        })
        .query_balance(lp_denom.clone().to_string(), &fee_collector, |balance| {
            assert_eq!(balance, Uint128::new(100));
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
            PositionAction::Fill {
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
            PositionAction::Fill {
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
            PositionAction::Fill {
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
            PositionAction::Fill {
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
            Some(FarmsBy::Identifier("farm_1".to_string())),
            None,
            None,
            |result| {
                let farms_response = result.unwrap();
                assert_eq!(
                    farms_response.farms[0],
                    Farm {
                        identifier: "farm_1".to_string(),
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
                    identifier: "farm_1".to_string(),
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
                    identifier: "farm_2".to_string(),
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
                    identifier: "farm_1".to_string(),
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
                    identifier: "farm_2".to_string(),
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
                identifier: "other_pos_1".to_string(),
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
                identifier: "other_pos_2".to_string(),
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
                assert_eq!(balance, Uint128::new(4_000));
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
        PositionAction::Fill {
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
                    identifier: "farm_1".to_string(),
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
                    identifier: "farm_2".to_string(),
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
                    identifier: "farm_3".to_string(),
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
                    identifier: "farm_4".to_string(),
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
                    identifier: "farm_1".to_string(),
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
                    identifier: "farm_2".to_string(),
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
                    identifier: "farm_3".to_string(),
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
                    identifier: "farm_4".to_string(),
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
            identifier: "another_pos_1".to_string(),
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
                    identifier: "farm_4".to_string(),
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
                    identifier: "farm_4".to_string(),
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
            farm_identifier: "farm_4".to_string(),
        },
        vec![],
        |result| {
            result.unwrap();
        },
    );
}
