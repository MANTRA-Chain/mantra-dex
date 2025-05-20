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
                    start_epoch: 49u64, // the default of (current epoch + 1u64) was used
                    preliminary_end_epoch: 63u64,
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
            Some(MAX_FARMS_LIMIT),
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
