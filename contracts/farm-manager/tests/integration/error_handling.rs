extern crate core;

use cosmwasm_std::{coin, Coin, Uint128};
use farm_manager::ContractError;
use mantra_dex_std::constants::LP_SYMBOL;
use mantra_dex_std::farm_manager::{FarmAction, FarmParams, PositionAction};

use crate::common::suite::TestingSuite;
use crate::common::MOCK_CONTRACT_ADDR_1;

const UUSDY_DENOM: &str = "uusdy";
const FARM_ASSET_AMOUNT: u128 = 8_000u128;
const START_EPOCH: u64 = 12;
const PRELIMINARY_END_EPOCH: u64 = 16;
const UNLOCKING_DURATION: u64 = 86_400;

const INVALID_ID_SPECIAL_CHARS: &str = "invalid!";
const INVALID_ID_TOO_LONG: &str =
    "7105920181635468364293788789264771059201816354683642937887892647a";
const INVALID_ID_NON_ASCII: &str = "感";
const INVALID_ID_SQL_INJECTION_LIKE: &str = "INSERT INTO my_table (my_string) VALUES (values)";
const VALID_ID: &str = "7105920181635468364293788789264771059201816354683642937887892647";
const LP_STAKE_AMOUNT: u128 = 5_000;

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
                    start_epoch: Some(START_EPOCH),
                    preliminary_end_epoch: Some(PRELIMINARY_END_EPOCH),
                    curve: None,
                    farm_asset: Coin {
                        denom: UUSDY_DENOM.to_string(),
                        amount: Uint128::new(FARM_ASSET_AMOUNT),
                    },
                    farm_identifier: Some(INVALID_ID_SPECIAL_CHARS.to_string()),
                },
            },
            vec![coin(FARM_ASSET_AMOUNT, UUSDY_DENOM), coin(1_000, "uom")],
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
                    start_epoch: Some(START_EPOCH),
                    preliminary_end_epoch: Some(PRELIMINARY_END_EPOCH),
                    curve: None,
                    farm_asset: Coin {
                        denom: UUSDY_DENOM.to_string(),
                        amount: Uint128::new(FARM_ASSET_AMOUNT),
                    },
                    farm_identifier: Some(INVALID_ID_TOO_LONG.to_string()),
                },
            },
            vec![coin(FARM_ASSET_AMOUNT, UUSDY_DENOM), coin(1_000, "uom")],
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
                    start_epoch: Some(START_EPOCH),
                    preliminary_end_epoch: Some(PRELIMINARY_END_EPOCH),
                    curve: None,
                    farm_asset: Coin {
                        denom: UUSDY_DENOM.to_string(),
                        amount: Uint128::new(FARM_ASSET_AMOUNT),
                    },
                    farm_identifier: Some(INVALID_ID_NON_ASCII.to_string()),
                },
            },
            vec![coin(FARM_ASSET_AMOUNT, UUSDY_DENOM), coin(1_000, "uom")],
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
                    start_epoch: Some(START_EPOCH),
                    preliminary_end_epoch: Some(PRELIMINARY_END_EPOCH),
                    curve: None,
                    farm_asset: Coin {
                        denom: UUSDY_DENOM.to_string(),
                        amount: Uint128::new(FARM_ASSET_AMOUNT),
                    },
                    farm_identifier: Some(INVALID_ID_SQL_INJECTION_LIKE.to_string()),
                },
            },
            vec![coin(FARM_ASSET_AMOUNT, UUSDY_DENOM), coin(1_000, "uom")],
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
                    start_epoch: Some(START_EPOCH),
                    preliminary_end_epoch: Some(PRELIMINARY_END_EPOCH),
                    curve: None,
                    farm_asset: Coin {
                        denom: UUSDY_DENOM.to_string(),
                        amount: Uint128::new(FARM_ASSET_AMOUNT),
                    },
                    farm_identifier: Some(VALID_ID.to_string()),
                },
            },
            vec![coin(FARM_ASSET_AMOUNT, UUSDY_DENOM), coin(1_000, "uom")],
            |result| {
                result.unwrap();
            },
        );

    suite
        .manage_position(
            &creator,
            PositionAction::Create {
                identifier: Some(INVALID_ID_SPECIAL_CHARS.to_string()),
                unlocking_duration: UNLOCKING_DURATION,
                receiver: None,
            },
            vec![coin(LP_STAKE_AMOUNT, lp_denom.clone())],
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
                identifier: Some(INVALID_ID_TOO_LONG.to_string()),
                unlocking_duration: UNLOCKING_DURATION,
                receiver: None,
            },
            vec![coin(LP_STAKE_AMOUNT, lp_denom.clone())],
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
                identifier: Some(INVALID_ID_SQL_INJECTION_LIKE.to_string()),
                unlocking_duration: UNLOCKING_DURATION,
                receiver: None,
            },
            vec![coin(LP_STAKE_AMOUNT, lp_denom.clone())],
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
                identifier: Some(INVALID_ID_NON_ASCII.to_string()),
                unlocking_duration: UNLOCKING_DURATION,
                receiver: None,
            },
            vec![coin(LP_STAKE_AMOUNT, lp_denom.clone())],
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
