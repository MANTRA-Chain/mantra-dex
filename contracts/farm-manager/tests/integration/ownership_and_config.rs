extern crate core;

use cosmwasm_std::{coin, Addr, Coin, Decimal, Uint128};
use farm_manager::ContractError;
use mantra_dex_std::constants::{LP_SYMBOL, MONTH_IN_SECONDS};
use mantra_dex_std::farm_manager::Config;

use crate::common::suite::TestingSuite;
use crate::common::MOCK_CONTRACT_ADDR_1;
use test_utils::common_constants::{DENOM_UOM, DENOM_UOSMO, DENOM_UUSDY, ONE_BILLION};

const NEW_CREATE_FARM_FEE_AMOUNT: u128 = 2_000u128;
const NEW_MAX_CONCURRENT_FARMS: u32 = 5u32;
const NEW_MAX_FARM_EPOCH_BUFFER: u32 = 15u32;
const NEW_MIN_UNLOCKING_DURATION: u64 = 100_000u64;
const NEW_MAX_UNLOCKING_DURATION: u64 = 200_000u64;
const NEW_FARM_EXPIRATION_TIME: u64 = MONTH_IN_SECONDS * 2;
const NEW_EMERGENCY_UNLOCK_PENALTY_PERCENT: u64 = 20;

// Constants for invalid values
const INVALID_MAX_CONCURRENT_FARMS_DECREASED: u32 = 0u32;

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
        coin(ONE_BILLION, DENOM_UOM.to_string()),
        coin(ONE_BILLION, DENOM_UUSDY.to_string()),
        coin(ONE_BILLION, DENOM_UOSMO.to_string()),
        coin(ONE_BILLION, lp_denom.clone()),
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
            denom: DENOM_UOM.to_string(),
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
                denom: DENOM_UOM.to_string(),
                amount: Uint128::new(NEW_CREATE_FARM_FEE_AMOUNT),
            }),
            Some(3u32),
            Some(NEW_MAX_FARM_EPOCH_BUFFER),
            Some(172_800u64),
            Some(864_000u64),
            Some(NEW_FARM_EXPIRATION_TIME),
            Some(Decimal::percent(50)),
            vec![coin(1_000, DENOM_UOM)],
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
            denom: DENOM_UOM.to_string(),
            amount: Uint128::new(NEW_CREATE_FARM_FEE_AMOUNT),
        }),
        Some(INVALID_MAX_CONCURRENT_FARMS_DECREASED),
        Some(NEW_MAX_FARM_EPOCH_BUFFER),
        Some(172_800u64),
        Some(864_000u64),
        Some(NEW_FARM_EXPIRATION_TIME),
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
            denom: DENOM_UOM.to_string(),
            amount: Uint128::new(NEW_CREATE_FARM_FEE_AMOUNT),
        }),
        Some(INVALID_MAX_CONCURRENT_FARMS_DECREASED),
        Some(NEW_MAX_FARM_EPOCH_BUFFER),
        Some(172_800u64),
        Some(864_000u64),
        Some(NEW_FARM_EXPIRATION_TIME),
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
            denom: DENOM_UOM.to_string(),
            amount: Uint128::new(NEW_CREATE_FARM_FEE_AMOUNT),
        }),
        Some(NEW_MAX_CONCURRENT_FARMS),
        Some(NEW_MAX_FARM_EPOCH_BUFFER),
        Some(80_800u64),
        Some(80_000u64),
        Some(NEW_FARM_EXPIRATION_TIME),
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
            denom: DENOM_UOM.to_string(),
            amount: Uint128::new(NEW_CREATE_FARM_FEE_AMOUNT),
        }),
        Some(NEW_MAX_CONCURRENT_FARMS),
        Some(NEW_MAX_FARM_EPOCH_BUFFER),
        Some(300_000u64),
        Some(200_000u64),
        Some(NEW_FARM_EXPIRATION_TIME),
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
            denom: DENOM_UOM.to_string(),
            amount: Uint128::new(NEW_CREATE_FARM_FEE_AMOUNT),
        }),
        Some(NEW_MAX_CONCURRENT_FARMS),
        Some(NEW_MAX_FARM_EPOCH_BUFFER),
        Some(NEW_MIN_UNLOCKING_DURATION),
        Some(NEW_MAX_UNLOCKING_DURATION),
        Some(NEW_FARM_EXPIRATION_TIME),
        Some(Decimal::percent(105)),
        vec![],
        |result| {
            let err = result.unwrap_err().downcast::<ContractError>().unwrap();
            match err {
                ContractError::InvalidEmergencyUnlockPenalty => {}
                _ => panic!("Wrong error type, should return ContractError::InvalidEmergencyUnlockPenalty"),
            }
        },
    ).update_config(
        &creator,
        Some(MOCK_CONTRACT_ADDR_1.to_string()),
        Some(MOCK_CONTRACT_ADDR_1.to_string()),
        Some(MOCK_CONTRACT_ADDR_1.to_string()),
        Some(Coin {
            denom: DENOM_UOM.to_string(),
            amount: Uint128::new(NEW_CREATE_FARM_FEE_AMOUNT),
        }),
        Some(NEW_MAX_CONCURRENT_FARMS),
        Some(NEW_MAX_FARM_EPOCH_BUFFER),
        Some(NEW_MIN_UNLOCKING_DURATION),
        Some(NEW_MAX_UNLOCKING_DURATION),
        Some(NEW_FARM_EXPIRATION_TIME),
        Some(Decimal::percent(NEW_EMERGENCY_UNLOCK_PENALTY_PERCENT)),
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
            denom: DENOM_UOM.to_string(),
            amount: Uint128::new(NEW_CREATE_FARM_FEE_AMOUNT),
        },
        max_concurrent_farms: NEW_MAX_CONCURRENT_FARMS,
        max_farm_epoch_buffer: NEW_MAX_FARM_EPOCH_BUFFER,
        min_unlocking_duration: NEW_MIN_UNLOCKING_DURATION,
        max_unlocking_duration: NEW_MAX_UNLOCKING_DURATION,
        farm_expiration_time: NEW_FARM_EXPIRATION_TIME,
        emergency_unlock_penalty: Decimal::percent(NEW_EMERGENCY_UNLOCK_PENALTY_PERCENT),
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
