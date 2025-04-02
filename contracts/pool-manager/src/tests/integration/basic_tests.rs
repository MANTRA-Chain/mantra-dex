use super::super::suite::TestingSuite;
use cosmwasm_std::coin;
use mantra_common_testing::multi_test::stargate_mock::StargateMock;

#[test]
fn instantiate_normal() {
    let mut suite = TestingSuite::default_with_balances(
        vec![],
        StargateMock::new(vec![
            coin(8888u128, "uom".to_string()),
            coin(1000u128, "utest".to_string()),
        ]),
    );

    suite.instantiate(suite.senders[0].to_string(), suite.senders[1].to_string());
}
