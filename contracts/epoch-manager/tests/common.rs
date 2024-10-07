use cosmwasm_std::testing::mock_env;
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response, Uint64};

use amm::epoch_manager::{EpochConfig, InstantiateMsg};
use epoch_manager::contract::instantiate;
use epoch_manager::ContractError;

/// Mocks contract instantiation.
#[allow(dead_code)]
pub fn mock_instantiation(
    deps: DepsMut,
    env: &Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let current_time = env.block.time;
    let msg = InstantiateMsg {
        epoch_config: EpochConfig {
            duration: Uint64::new(86400),
            genesis_epoch: Uint64::new(current_time.nanos()),
        },
    };

    instantiate(deps, mock_env(), info, msg)
}
