extern crate core;

use cosmwasm_std::{coin, Coin, Decimal, Uint128};
use farm_manager::ContractError;
use mantra_dex_std::constants::MONTH_IN_SECONDS;

use crate::common::suite::TestingSuite;
use crate::common::MOCK_CONTRACT_ADDR_1;
use test_utils::common_constants::{DEFAULT_UNLOCKING_DURATION_SECONDS, DENOM_UOM, ONE_BILLION};

const INITIAL_AMOUNT_UOM_RAW: u128 = 1_000u128;

const MAX_CONCURRENT_FARMS_1: u32 = 1;
const DEFAULT_UNLOCKING_PERIODS: u32 = 14;
const VALID_MAX_UNLOCKING_DURATION_SECONDS: u64 = 31_536_000;

const VALID_MAX_UNLOCKING_DURATION_SLIGHTLY_LONGER_SECONDS: u64 = 86_500;
const DEFAULT_EMERGENCY_UNLOCK_PENALTY: Decimal = Decimal::percent(10);
const INVALID_EMERGENCY_UNLOCK_PENALTY: Decimal = Decimal::percent(101);

#[test]
fn instantiate_farm_manager() {
    let mut suite =
        TestingSuite::default_with_balances(vec![coin(ONE_BILLION, DENOM_UOM.to_string())]);

    suite.instantiate_err(
        MOCK_CONTRACT_ADDR_1.to_string(),
        MOCK_CONTRACT_ADDR_1.to_string(),
        MOCK_CONTRACT_ADDR_1.to_string(),
        Coin {
            denom: DENOM_UOM.to_string(),
            amount: Uint128::new(INITIAL_AMOUNT_UOM_RAW),
        },
        0,
        DEFAULT_UNLOCKING_PERIODS,
        DEFAULT_UNLOCKING_DURATION_SECONDS,
        VALID_MAX_UNLOCKING_DURATION_SECONDS,
        MONTH_IN_SECONDS,
        DEFAULT_EMERGENCY_UNLOCK_PENALTY,
        |result| {
            let err = result.unwrap_err().downcast::<ContractError>().unwrap();

            match err {
                ContractError::UnspecifiedConcurrentFarms => {}
                _ => panic!("Wrong error type, should return ContractError::UnspecifiedConcurrentFarms"),
            }
        },
    ).instantiate_err(
        MOCK_CONTRACT_ADDR_1.to_string(),
        MOCK_CONTRACT_ADDR_1.to_string(),
        MOCK_CONTRACT_ADDR_1.to_string(),
        Coin {
            denom: DENOM_UOM.to_string(),
            amount: Uint128::new(INITIAL_AMOUNT_UOM_RAW),
        },
        MAX_CONCURRENT_FARMS_1,
        DEFAULT_UNLOCKING_PERIODS,
        DEFAULT_UNLOCKING_DURATION_SECONDS,
        86_399,
        MONTH_IN_SECONDS,
        DEFAULT_EMERGENCY_UNLOCK_PENALTY,
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
            denom: DENOM_UOM.to_string(),
            amount: Uint128::new(INITIAL_AMOUNT_UOM_RAW),
        },
        MAX_CONCURRENT_FARMS_1,
        DEFAULT_UNLOCKING_PERIODS,
        DEFAULT_UNLOCKING_DURATION_SECONDS,
        VALID_MAX_UNLOCKING_DURATION_SLIGHTLY_LONGER_SECONDS,
        MONTH_IN_SECONDS,
        INVALID_EMERGENCY_UNLOCK_PENALTY,
        |result| {
            let err = result.unwrap_err().downcast::<ContractError>().unwrap();

            match err {
                ContractError::InvalidEmergencyUnlockPenalty => {}
                _ => panic!("Wrong error type, should return ContractError::InvalidEmergencyUnlockPenalty"),
            }
        },
    ).instantiate_err(
        MOCK_CONTRACT_ADDR_1.to_string(),
        MOCK_CONTRACT_ADDR_1.to_string(),
        MOCK_CONTRACT_ADDR_1.to_string(),
        Coin {
            denom: DENOM_UOM.to_string(),
            amount: Uint128::new(INITIAL_AMOUNT_UOM_RAW),
        },
        MAX_CONCURRENT_FARMS_1,
        DEFAULT_UNLOCKING_PERIODS,
        DEFAULT_UNLOCKING_DURATION_SECONDS,
        VALID_MAX_UNLOCKING_DURATION_SLIGHTLY_LONGER_SECONDS,
        MONTH_IN_SECONDS - 1,
        INVALID_EMERGENCY_UNLOCK_PENALTY, // Note: This penalty is invalid, but the test prioritizes FarmExpirationTimeInvalid
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
            denom: DENOM_UOM.to_string(),
            amount: Uint128::new(INITIAL_AMOUNT_UOM_RAW),
        },
        7,
        DEFAULT_UNLOCKING_PERIODS,
        DEFAULT_UNLOCKING_DURATION_SECONDS,
        VALID_MAX_UNLOCKING_DURATION_SECONDS,
        MONTH_IN_SECONDS,
        DEFAULT_EMERGENCY_UNLOCK_PENALTY,
    );
}
