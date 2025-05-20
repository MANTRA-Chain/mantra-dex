extern crate core;

use cosmwasm_std::{coin, Coin, Decimal, Uint128};
use farm_manager::ContractError;
use mantra_dex_std::constants::MONTH_IN_SECONDS;

use crate::common::suite::TestingSuite;
use crate::common::MOCK_CONTRACT_ADDR_1;

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
                ContractError::UnspecifiedConcurrentFarms => {}
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
                ContractError::InvalidEmergencyUnlockPenalty => {}
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
