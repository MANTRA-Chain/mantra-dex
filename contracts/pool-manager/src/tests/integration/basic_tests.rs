use super::super::suite::TestingSuite;
use super::common_constants::{DENOM_UOM, STARGATE_MOCK_UOM_AMOUNT};
use cosmwasm_std::coin;
use mantra_common_testing::multi_test::stargate_mock::StargateMock;

// Test token denominations
const DENOM_TEST: &str = "utest";

// Initial balances
const INITIAL_TEST_BALANCE: u128 = 1000;

#[test]
fn instantiate_normal() {
    let mut suite = TestingSuite::default_with_balances(
        vec![],
        StargateMock::new(vec![
            coin(STARGATE_MOCK_UOM_AMOUNT, DENOM_UOM),
            coin(INITIAL_TEST_BALANCE, DENOM_TEST),
        ]),
    );

    suite.instantiate(suite.senders[0].to_string(), suite.senders[1].to_string());
}
