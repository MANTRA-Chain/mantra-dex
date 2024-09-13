use amm::fee_collector::ExecuteMsg::UpdateOwnership;
use amm::fee_collector::{InstantiateMsg, QueryMsg};
use cosmwasm_std::{Empty, StdResult};
use cw_multi_test::{App, Contract, ContractWrapper, Executor, IntoBech32};

pub fn fee_collector_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        fee_collector::contract::execute,
        fee_collector::contract::instantiate,
        fee_collector::contract::query,
    )
    .with_migrate(fee_collector::contract::migrate);

    Box::new(contract)
}

#[test]
fn change_contract_ownership() {
    let mut app = App::default();
    let code_id = app.store_code(fee_collector_contract());
    let msg = InstantiateMsg {};

    let admin = "admin".into_bech32();
    let alice = "alice".into_bech32();

    let fee_collector = app
        .instantiate_contract(
            code_id,
            admin.clone(),
            &msg,
            &[],
            "Fee Collector",
            Some(admin.to_string()),
        )
        .unwrap();

    // Unauthorized attempt to change ownership
    let msg = UpdateOwnership(cw_ownable::Action::TransferOwnership {
        new_owner: alice.to_string(),
        expiry: None,
    });

    let result = app.execute_contract(alice.clone(), fee_collector.clone(), &msg, &[]);

    match result {
        Ok(_) => panic!("Unauthorized attempt to change ownership should fail"),
        Err(_) => {}
    }

    // Authorized attempt to change ownership
    app.execute_contract(admin.clone(), fee_collector.clone(), &msg, &[])
        .unwrap();

    let ownership_response: StdResult<cw_ownable::Ownership<String>> = app
        .wrap()
        .query_wasm_smart(&fee_collector, &QueryMsg::Ownership {});

    assert_eq!(ownership_response.unwrap().owner, Some(admin.to_string()));

    // accept ownership transfer
    let msg = UpdateOwnership(cw_ownable::Action::AcceptOwnership {});
    app.execute_contract(admin.clone(), fee_collector.clone(), &msg, &[])
        .unwrap_err();
    app.execute_contract(alice.clone(), fee_collector.clone(), &msg, &[])
        .unwrap();

    let ownership_response: StdResult<cw_ownable::Ownership<String>> = app
        .wrap()
        .query_wasm_smart(&fee_collector, &QueryMsg::Ownership {});

    assert_eq!(ownership_response.unwrap().owner, Some(alice.to_string()));
}
