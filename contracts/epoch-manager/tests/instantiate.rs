use amm::constants::DAY_IN_SECONDS;
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
        owner: "owner".into_bech32().to_string(),
        epoch_config: EpochConfig {
            duration: Uint64::new(86_400),
            genesis_epoch: Uint64::new(current_time.seconds()),
        },
    };

    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    let query_res = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
    let config_res: ConfigResponse = from_json(query_res).unwrap();
    assert_eq!(
        EpochConfig {
            duration: Uint64::new(86_400),
            genesis_epoch: Uint64::new(current_time.seconds()),
        },
        config_res.epoch_config
    );
}

#[test]
fn instantiation_unsuccessful() {
    let mut deps = mock_dependencies();

    let current_time = mock_env().block.time;
    let owner = "owner".into_bech32();
    let info = message_info(&owner, &[]);
    let msg = InstantiateMsg {
        owner: "owner".into_bech32().to_string(),
        epoch_config: EpochConfig {
            duration: Uint64::new(86_400),
            genesis_epoch: Uint64::new(current_time.minus_days(1).seconds()),
        },
    };

    let err = instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap_err();
    match err {
        ContractError::InvalidStartTime => {}
        _ => panic!("should return ContractError::InvalidStartTime"),
    }

    let msg = InstantiateMsg {
        owner: "owner".into_bech32().to_string(),
        epoch_config: EpochConfig {
            duration: Uint64::zero(),
            genesis_epoch: Uint64::new(current_time.seconds()),
        },
    };

    let err = instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap_err();
    match err {
        ContractError::InvalidEpochDuration { .. } => {}
        _ => panic!("should return ContractError::InvalidEpochDuration"),
    }

    let msg = InstantiateMsg {
        owner: "owner".into_bech32().to_string(),
        epoch_config: EpochConfig {
            duration: Uint64::new(DAY_IN_SECONDS - 1u64),
            genesis_epoch: Uint64::new(current_time.seconds()),
        },
    };

    let err = instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap_err();
    match err {
        ContractError::InvalidEpochDuration { .. } => {}
        _ => panic!("should return ContractError::InvalidEpochDuration"),
    }
}
