use super::super::suite::TestingSuite;
use cosmwasm_std::coin;
use mantra_common_testing::multi_test::stargate_mock::StargateMock;
use test_utils::common_constants::{DENOM_UOM, STARGATE_MOCK_UOM_AMOUNT};

#[test]
fn instantiate_normal() {
    let mut suite = TestingSuite::default_with_balances(
        vec![],
        StargateMock::new(vec![
            coin(STARGATE_MOCK_UOM_AMOUNT, DENOM_UOM),
            coin(1000, "utest"),
        ]),
    );

    suite.instantiate(suite.senders[0].to_string(), suite.senders[1].to_string());
}
