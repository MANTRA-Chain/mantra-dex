use cosmwasm_std::Empty;
use cw_multi_test::{Contract, ContractWrapper};

/// Creates the farm manager contract
pub fn farm_manager_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        farm_manager::contract::execute,
        farm_manager::contract::instantiate,
        farm_manager::contract::query,
    )
    .with_migrate(farm_manager::contract::migrate);

    Box::new(contract)
}

/// Creates the epoch manager contract
pub fn epoch_manager_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        epoch_manager::contract::execute,
        epoch_manager::contract::instantiate,
        epoch_manager::contract::query,
    )
    .with_migrate(epoch_manager::contract::migrate);

    Box::new(contract)
}

/// Creates the fee collector contract
pub fn fee_collector_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        fee_collector::contract::execute,
        fee_collector::contract::instantiate,
        fee_collector::contract::query,
    )
    .with_migrate(fee_collector::contract::migrate);

    Box::new(contract)
}
