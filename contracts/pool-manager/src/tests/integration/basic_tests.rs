use super::super::suite::TestingSuite;
use cosmwasm_std::coin;
use mantra_common_testing::multi_test::stargate_mock::StargateMock;

// Test token denominations
const DENOM_OM: &str = "uom";
const DENOM_TEST: &str = "utest";

// Initial balances
const INITIAL_OM_BALANCE: u128 = 8888;
const INITIAL_TEST_BALANCE: u128 = 1000;

#[test]
fn instantiate_normal() {
    let mut suite = TestingSuite::default_with_balances(
        vec![],
        StargateMock::new(vec![
            coin(INITIAL_OM_BALANCE, DENOM_OM),
            coin(INITIAL_TEST_BALANCE, DENOM_TEST),
        ]),
    );

    suite.instantiate(suite.senders[0].to_string(), suite.senders[1].to_string());
}
