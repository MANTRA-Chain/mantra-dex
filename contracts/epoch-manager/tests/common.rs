use cosmwasm_std::testing::mock_env;
use cosmwasm_std::{Addr, DepsMut, MessageInfo, Response, Uint64};

use amm::epoch_manager::{Epoch, EpochConfig, ExecuteMsg, InstantiateMsg};
use epoch_manager::contract::{execute, instantiate};
use epoch_manager::ContractError;

/// Mocks contract instantiation.
#[allow(dead_code)]
pub fn mock_instantiation(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
    let current_time = mock_env().block.time;
    let msg = InstantiateMsg {
        start_epoch: Epoch {
            id: 123,
            start_time: current_time,
        },
        epoch_config: EpochConfig {
            duration: Uint64::new(86400),
            genesis_epoch: Uint64::new(current_time.nanos()),
        },
    };

    instantiate(deps, mock_env(), info, msg)
}

/// Mocks hook addition.
#[allow(dead_code)]
pub fn mock_add_hook(
    deps: DepsMut,
    info: MessageInfo,
    hook: &Addr,
) -> Result<Response, ContractError> {
    let msg = ExecuteMsg::AddHook {
        contract_addr: hook.to_string(),
    };

    execute(deps, mock_env(), info, msg)
}
