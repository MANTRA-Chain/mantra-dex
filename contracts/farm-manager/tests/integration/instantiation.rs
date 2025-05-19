extern crate core;

use std::cell::RefCell;

use cosmwasm_std::{coin, coins, Addr, Coin, Decimal, StdResult, Timestamp, Uint128};
use cw_utils::PaymentError;
use farm_manager::state::{MAX_FARMS_LIMIT, MAX_POSITIONS_LIMIT};
use farm_manager::ContractError;
use mantra_dex_std::constants::{LP_SYMBOL, MONTH_IN_SECONDS};
use mantra_dex_std::farm_manager::{
    Config, Curve, Farm, FarmAction, FarmParams, FarmsBy, LpWeightResponse, Position,
    PositionAction, PositionsBy, PositionsResponse, RewardsResponse,
};

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
