extern crate core;

use cosmwasm_std::{coin, Coin, Uint128};
use farm_manager::ContractError;
use mantra_dex_std::constants::LP_SYMBOL;
use mantra_dex_std::farm_manager::{FarmAction, FarmParams, PositionAction};

use crate::common::suite::TestingSuite;
use crate::common::MOCK_CONTRACT_ADDR_1;

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
