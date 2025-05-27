extern crate core;

use cosmwasm_std::{coin, Coin, Uint128};
use farm_manager::state::MAX_FARMS_LIMIT;
use farm_manager::ContractError;
use mantra_dex_std::constants::LP_SYMBOL;
use mantra_dex_std::farm_manager::{
    Curve, Farm, FarmAction, FarmParams, PositionAction, PositionsBy,
};

use crate::common::suite::TestingSuite;
use crate::common::MOCK_CONTRACT_ADDR_1;
use test_utils::common_constants::{
    DEFAULT_UNLOCKING_DURATION_SECONDS, DENOM_UOM, DENOM_UOSMO, DENOM_UUSDY, INITIAL_BALANCE,
    ONE_THOUSAND,
};

// Denoms
const INVALID_LP_DENOM: &str = "invalid_lp";

// Amounts
const FARM_ASSET_AMOUNT_8000: u128 = 8_000u128;
const FARM_ASSET_AMOUNT_10000: u128 = 10_000u128;
const FARM_ASSET_AMOUNT_4000: u128 = 4_000u128;
const LP_DEPOSIT_AMOUNT_10000: u128 = 10_000u128;

// Farm Identifiers
const FARM_IDENTIFIER_FARM: &str = "farm";

const FARM_IDENTIFIER_ANOTHER_FARM: &str = "another_farm";

// Expected Prefixed Farm Identifiers
const EXPECTED_PREFIXED_FARM_ID_FARM: &str = "m-farm";

const EXPECTED_PREFIXED_FARM_ID_LONG_FARM: &str = "m-long_farm";

// Epochs
const START_EPOCH_12: u64 = 12;
const PRELIMINARY_END_EPOCH_16: u64 = 16;
const PRELIMINARY_END_EPOCH_100: u64 = 100;

// Position Identifiers
const POSITION_ID_P1: &str = "p-1";

// Emission Rates

#[test]
fn test_close_expired_farms() {
    let lp_denom = format!("factory/{MOCK_CONTRACT_ADDR_1}/{LP_SYMBOL}").to_string();

    let mut suite = TestingSuite::default_with_balances(vec![
        coin(INITIAL_BALANCE, DENOM_UOM),
        coin(INITIAL_BALANCE, DENOM_UUSDY),
        coin(INITIAL_BALANCE, DENOM_UOSMO),
        coin(INITIAL_BALANCE, lp_denom.clone()),
        coin(INITIAL_BALANCE, INVALID_LP_DENOM),
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
                start_epoch: Some(START_EPOCH_12),
                preliminary_end_epoch: Some(PRELIMINARY_END_EPOCH_16),
                curve: None,
                farm_asset: Coin {
                    denom: DENOM_UUSDY.to_string(),
                    amount: Uint128::new(FARM_ASSET_AMOUNT_8000),
                },
                farm_identifier: Some(FARM_IDENTIFIER_FARM.to_string()),
            },
        },
        vec![
            coin(FARM_ASSET_AMOUNT_8000, DENOM_UUSDY),
            coin(ONE_THOUSAND, DENOM_UOM),
        ],
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
            assert_eq!(
                farms_response.farms[0].identifier,
                EXPECTED_PREFIXED_FARM_ID_FARM
            );
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
                        denom: DENOM_UUSDY.to_string(),
                        amount: Uint128::new(FARM_ASSET_AMOUNT_10000),
                    },
                    farm_identifier: Some("new_farm".to_string()),
                },
            },
            vec![
                coin(FARM_ASSET_AMOUNT_10000, DENOM_UUSDY),
                coin(ONE_THOUSAND, DENOM_UOM),
            ],
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
                        denom: DENOM_UUSDY.to_string(),
                        amount: Uint128::new(FARM_ASSET_AMOUNT_10000),
                    },
                    claimed_amount: Uint128::zero(),
                    emission_rate: Uint128::new(714), // (10000 / (63 - 49 + 1)) * 1 = 10000 / 15 = 666 - check this logic based on current_epoch being 48
                    curve: Curve::Linear,
                    start_epoch: 48 + 1, // the default of (current epoch + 1u64) was used
                    preliminary_end_epoch: 63,
                }
            );
        });
}

#[test]
fn expand_expired_farm() {
    let lp_denom = format!("factory/{MOCK_CONTRACT_ADDR_1}/{LP_SYMBOL}").to_string();

    let mut suite = TestingSuite::default_with_balances(vec![
        coin(INITIAL_BALANCE, DENOM_UOM.to_string()),
        coin(INITIAL_BALANCE, DENOM_UUSDY.to_string()),
        coin(INITIAL_BALANCE, DENOM_UOSMO.to_string()),
        coin(INITIAL_BALANCE, lp_denom.clone()),
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
                        denom: DENOM_UUSDY.to_string(),
                        amount: Uint128::new(FARM_ASSET_AMOUNT_4000),
                    },
                    farm_identifier: Some(FARM_IDENTIFIER_FARM.to_string()),
                },
            },
            vec![
                coin(FARM_ASSET_AMOUNT_4000, DENOM_UUSDY),
                coin(ONE_THOUSAND, DENOM_UOM),
            ],
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
                    identifier: EXPECTED_PREFIXED_FARM_ID_FARM.to_string(),
                    owner: other.clone(),
                    lp_denom: lp_denom.clone(),
                    farm_asset: Coin {
                        denom: DENOM_UUSDY.to_string(),
                        amount: Uint128::new(FARM_ASSET_AMOUNT_4000),
                    },
                    claimed_amount: Uint128::zero(),
                    emission_rate: Uint128::new(285), // 4000 / 14 = 285.7...
                    curve: Curve::Linear,
                    start_epoch: 1u64,
                    preliminary_end_epoch: 15,
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
                        denom: DENOM_UUSDY.to_string(),
                        amount: Uint128::new(FARM_ASSET_AMOUNT_8000),
                    },
                    farm_identifier: Some(EXPECTED_PREFIXED_FARM_ID_FARM.to_string()),
                },
            },
            vec![coin(FARM_ASSET_AMOUNT_8000, DENOM_UUSDY)],
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
}

#[test]
fn test_farm_expired() {
    let lp_denom = format!("factory/{MOCK_CONTRACT_ADDR_1}/{LP_SYMBOL}").to_string();

    let mut suite = TestingSuite::default_with_balances(vec![
        coin(INITIAL_BALANCE, DENOM_UOM),
        coin(INITIAL_BALANCE, DENOM_UUSDY),
        coin(INITIAL_BALANCE, DENOM_UOSMO),
        coin(INITIAL_BALANCE, lp_denom.clone()),
        coin(INITIAL_BALANCE, INVALID_LP_DENOM),
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
                    start_epoch: Some(START_EPOCH_12),
                    preliminary_end_epoch: Some(PRELIMINARY_END_EPOCH_16),
                    curve: None,
                    farm_asset: Coin {
                        denom: DENOM_UUSDY.to_string(),
                        amount: Uint128::new(FARM_ASSET_AMOUNT_8000),
                    },
                    farm_identifier: Some("short_farm".to_string()),
                },
            },
            vec![
                coin(FARM_ASSET_AMOUNT_8000, DENOM_UUSDY),
                coin(ONE_THOUSAND, DENOM_UOM),
            ],
            |result| {
                result.unwrap();
            },
        )
        .manage_farm(
            &creator,
            FarmAction::Fill {
                params: FarmParams {
                    lp_denom: lp_denom.clone(),
                    start_epoch: Some(START_EPOCH_12),
                    preliminary_end_epoch: Some(PRELIMINARY_END_EPOCH_100),
                    curve: None,
                    farm_asset: Coin {
                        denom: DENOM_UUSDY.to_string(),
                        amount: Uint128::new(FARM_ASSET_AMOUNT_8000),
                    },
                    farm_identifier: Some("long_farm".to_string()),
                },
            },
            vec![
                coin(FARM_ASSET_AMOUNT_8000, DENOM_UUSDY),
                coin(ONE_THOUSAND, DENOM_UOM),
            ],
            |result| {
                result.unwrap();
            },
        )
        .manage_farm(
            &creator,
            FarmAction::Fill {
                params: FarmParams {
                    lp_denom: lp_denom.clone(),
                    start_epoch: Some(START_EPOCH_12),
                    preliminary_end_epoch: Some(PRELIMINARY_END_EPOCH_100),
                    curve: None,
                    farm_asset: Coin {
                        denom: DENOM_UUSDY.to_string(),
                        amount: Uint128::new(FARM_ASSET_AMOUNT_8000),
                    },
                    farm_identifier: Some(FARM_IDENTIFIER_ANOTHER_FARM.to_string()),
                },
            },
            vec![
                coin(FARM_ASSET_AMOUNT_8000, DENOM_UUSDY),
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

    // create a few epochs, but not enough for the farm to expire.
    // a farm expires after config.farm_expiration_time seconds from the epoch the farm ended
    // in this case, from the start of epoch 17 + config.farm_expiration_time
    for _ in 0..20 {
        suite.add_one_epoch();
    }

    let mut _current_epoch_id = 0;

    // try opening another farm for the same lp denom, the expired farm should get closed
    suite
        .query_current_epoch(|result| {
            let epoch_response = result.unwrap();
            assert_eq!(epoch_response.epoch.id, 30);
            _current_epoch_id = epoch_response.epoch.id;
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
                start_epoch: Some(START_EPOCH_12),
                preliminary_end_epoch: Some(PRELIMINARY_END_EPOCH_100),
                curve: None,
                farm_asset: Coin {
                    denom: DENOM_UUSDY.to_string(),
                    amount: Uint128::new(FARM_ASSET_AMOUNT_8000),
                },
                farm_identifier: Some(FARM_IDENTIFIER_ANOTHER_FARM.to_string()),
            },
        },
        vec![
            coin(FARM_ASSET_AMOUNT_8000, DENOM_UUSDY),
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
            _current_epoch_id = epoch_response.epoch.id;
        })
        .query_farms(None, None, None, |result| {
            let farms_response = result.unwrap();
            assert_eq!(farms_response.farms.len(), 2);

            assert!(farms_response.farms[0].claimed_amount.is_zero());
            assert!(farms_response.farms[1].claimed_amount.is_zero());
            assert_eq!(
                farms_response.farms[0].identifier,
                EXPECTED_PREFIXED_FARM_ID_LONG_FARM.to_string()
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
                    preliminary_end_epoch: Some(PRELIMINARY_END_EPOCH_100),
                    curve: None,
                    farm_asset: Coin {
                        denom: DENOM_UUSDY.to_string(),
                        amount: Uint128::new(FARM_ASSET_AMOUNT_8000),
                    },
                    farm_identifier: Some(FARM_IDENTIFIER_ANOTHER_FARM.to_string()),
                },
            },
            vec![
                coin(FARM_ASSET_AMOUNT_8000, DENOM_UUSDY),
                coin(ONE_THOUSAND, DENOM_UOM),
            ],
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
                EXPECTED_PREFIXED_FARM_ID_LONG_FARM.to_string()
            );
        });
}

#[test]
fn closing_expired_farm_wont_pay_penalty() {
    let lp_denom = format!("factory/{MOCK_CONTRACT_ADDR_1}/{LP_SYMBOL}").to_string();
    let mut suite = TestingSuite::default_with_balances(vec![
        coin(INITIAL_BALANCE, DENOM_UOM),
        coin(INITIAL_BALANCE, DENOM_UUSDY),
        coin(INITIAL_BALANCE, DENOM_UOSMO),
        coin(INITIAL_BALANCE, lp_denom.clone()),
        coin(INITIAL_BALANCE, INVALID_LP_DENOM),
    ]);
    let creator = suite.creator();

    suite.instantiate_default();

    let fee_collector = suite.fee_collector_addr.clone();

    suite
        .manage_position(
            &creator,
            PositionAction::Create {
                identifier: None,
                unlocking_duration: DEFAULT_UNLOCKING_DURATION_SECONDS,
                receiver: None,
            },
            vec![coin(LP_DEPOSIT_AMOUNT_10000, lp_denom.clone())],
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
                assert_eq!(response.positions.len(), 1);
                assert_eq!(response.positions[0].identifier, POSITION_ID_P1);
            },
        )
        .manage_position(
            &creator,
            PositionAction::Close {
                identifier: POSITION_ID_P1.to_string(),
                lp_asset: None,
            },
            vec![],
            |result| {
                result.unwrap();
            },
        )
        .add_one_epoch()
        .query_balance(lp_denom.clone(), &creator, |balance| {
            assert_eq!(
                balance,
                Uint128::new(INITIAL_BALANCE - LP_DEPOSIT_AMOUNT_10000)
            );
        })
        .query_balance(lp_denom.clone(), &fee_collector, |balance| {
            assert_eq!(balance, Uint128::zero());
        })
        .manage_position(
            &creator,
            PositionAction::Withdraw {
                identifier: POSITION_ID_P1.to_string(),
                // shouldn't pay emergency fee
                emergency_unlock: Some(true),
            },
            vec![],
            |result| {
                result.unwrap();
            },
        )
        .query_balance(lp_denom.clone(), &creator, |balance| {
            assert_eq!(balance, Uint128::new(INITIAL_BALANCE));
        })
        .query_balance(lp_denom.clone(), &fee_collector, |balance| {
            assert_eq!(balance, Uint128::zero());
        });
}
