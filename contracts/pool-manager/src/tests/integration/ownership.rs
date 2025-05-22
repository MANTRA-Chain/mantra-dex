use cosmwasm_std::{coin, Addr, Uint128};
use mantra_common_testing::multi_test::stargate_mock::StargateMock;

use crate::{tests::suite::TestingSuite, ContractError};
use super::common_constants::{DENOM_UOM, STARGATE_MOCK_UOM_AMOUNT};

// Test constants
// Common token amounts
const POOL_CREATION_FEE_INCREMENT: u32 = 1;

#[test]
fn verify_ownership() {
    let mut suite = TestingSuite::default_with_balances(
        vec![],
        StargateMock::new(vec![coin(STARGATE_MOCK_UOM_AMOUNT, DENOM_UOM)]),
    );
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
                    _ => {
                        panic!("Wrong error type, should return ContractError::OwnershipError")
                    }
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
fn checks_ownership_when_updating_config() {
    let mut suite = TestingSuite::default_with_balances(
        vec![],
        StargateMock::new(vec![coin(STARGATE_MOCK_UOM_AMOUNT, DENOM_UOM)]),
    );
    let unauthorized = suite.senders[2].clone();

    suite
        .instantiate_default()
        .update_config(&unauthorized, None, None, None, None, |result| {
            let err = result.unwrap_err().downcast::<ContractError>().unwrap();

            match err {
                ContractError::OwnershipError { .. } => {}
                _ => {
                    panic!("Wrong error type, should return ContractError::OwnershipError")
                }
            }
        });
}

#[test]
fn updates_config_fields() {
    let mut suite = TestingSuite::default_with_balances(
        vec![],
        StargateMock::new(vec![coin(STARGATE_MOCK_UOM_AMOUNT, DENOM_UOM)]),
    );
    let creator = suite.creator();
    let other = suite.senders[1].clone();
    let another = suite.senders[2].clone();

    suite.instantiate_default();
    let current_pool_creation_fee = suite.query_config().pool_creation_fee;
    let initial_config = suite.query_config();

    suite.update_config(
        &creator,
        Some(other),
        Some(another),
        Some(coin(
            current_pool_creation_fee
                .amount
                .checked_add(Uint128::from(POOL_CREATION_FEE_INCREMENT))
                .unwrap()
                .u128(),
            current_pool_creation_fee.denom,
        )),
        None,
        |res| {
            res.unwrap();
        },
    );

    let config = suite.query_config();
    assert_ne!(config.fee_collector_addr, initial_config.fee_collector_addr);
    assert_ne!(config.pool_creation_fee, initial_config.pool_creation_fee);
    assert_ne!(config.farm_manager_addr, initial_config.farm_manager_addr);
}
