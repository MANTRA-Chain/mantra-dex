use cosmwasm_std::testing::{message_info, mock_dependencies, mock_env};
use cosmwasm_std::{from_json, Uint64};
use cw_multi_test::IntoBech32;

use amm::epoch_manager::{ConfigResponse, EpochConfig, InstantiateMsg, QueryMsg};
use epoch_manager::contract::{instantiate, query};
use epoch_manager::ContractError;

mod common;

#[test]
fn instantiation_successful() {
    let mut deps = mock_dependencies();

    let current_time = mock_env().block.time;
    let owner = "owner".into_bech32();
    let info = message_info(&owner, &[]);
    let msg = InstantiateMsg {
        epoch_config: EpochConfig {
            duration: Uint64::new(86400),
            genesis_epoch: Uint64::new(current_time.nanos()),
        },
    };

    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    let query_res = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
    let config_res: ConfigResponse = from_json(query_res).unwrap();
    assert_eq!(
        EpochConfig {
            duration: Uint64::new(86400),
            genesis_epoch: Uint64::new(current_time.nanos()),
        },
        config_res.epoch_config
    );
    assert_eq!(owner, config_res.owner);
}

#[test]
fn instantiation_unsuccessful() {
    let mut deps = mock_dependencies();

    let current_time = mock_env().block.time;
    let owner = "owner".into_bech32();
    let info = message_info(&owner, &[]);
    let msg = InstantiateMsg {
        epoch_config: EpochConfig {
            duration: Uint64::new(86400),
            genesis_epoch: Uint64::new(current_time.minus_days(1).nanos()),
        },
    };

    let err = instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap_err();
    match err {
        ContractError::InvalidStartTime => {}
        _ => panic!("should return ContractError::InvalidStartTime"),
    }
}
