use cosmwasm_std::testing::{message_info, mock_dependencies, mock_env};
use cosmwasm_std::{from_json, Uint64};
use cw_multi_test::IntoBech32;

use amm::epoch_manager::{ConfigResponse, EpochConfig, ExecuteMsg, QueryMsg};
use epoch_manager::contract::{execute, query};
use epoch_manager::ContractError;

use crate::common::mock_instantiation;

mod common;

#[test]
fn update_config_successfully() {
    let mut deps = mock_dependencies();

    let owner = "owner".into_bech32();

    let info = message_info(&owner, &[]);
    let current_time = mock_env().block.time;
    mock_instantiation(deps.as_mut(), &mock_env(), info.clone()).unwrap();

    let query_res = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
    let config_res: ConfigResponse = from_json(query_res).unwrap();
    assert_eq!(
        EpochConfig {
            duration: Uint64::new(86400),
            genesis_epoch: Uint64::new(current_time.seconds()),
        },
        config_res.epoch_config
    );

    let msg = ExecuteMsg::UpdateConfig {
        epoch_config: Some(EpochConfig {
            duration: Uint64::new(172800),
            genesis_epoch: Uint64::new(current_time.seconds()),
        }),
    };

    execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    let query_res = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
    let config_res: ConfigResponse = from_json(query_res).unwrap();
    assert_eq!(
        EpochConfig {
            duration: Uint64::new(172800),
            genesis_epoch: Uint64::new(current_time.seconds()),
        },
        config_res.epoch_config
    );
}

#[test]
fn update_config_unsuccessfully() {
    let mut deps = mock_dependencies();

    let owner = "owner".into_bech32();

    let info = message_info(&owner, &[]);
    let current_time = mock_env().block.time;
    mock_instantiation(deps.as_mut(), &mock_env(), info.clone()).unwrap();

    let query_res = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
    let config_res: ConfigResponse = from_json(query_res).unwrap();
    assert_eq!(
        EpochConfig {
            duration: Uint64::new(86400),
            genesis_epoch: Uint64::new(current_time.seconds()),
        },
        config_res.epoch_config
    );

    let msg = ExecuteMsg::UpdateConfig {
        epoch_config: Some(EpochConfig {
            duration: Uint64::new(600),
            genesis_epoch: Uint64::new(current_time.seconds()),
        }),
    };

    let err = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
    match err {
        ContractError::InvalidEpochDuration { .. } => {}
        _ => panic!("should return ContractError::InvalidEpochDuration"),
    }

    let msg = ExecuteMsg::UpdateConfig {
        epoch_config: Some(EpochConfig {
            duration: Uint64::new(172800),
            genesis_epoch: Uint64::new(current_time.seconds()),
        }),
    };

    let unauthorized = "unauthorized".into_bech32();

    let info = message_info(&unauthorized, &[]);
    let err = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();

    match err {
        ContractError::OwnershipError(error) => {
            assert_eq!(error, cw_ownable::OwnershipError::NotOwner)
        }
        _ => panic!("should return OwnershipError::NotOwner"),
    }

    let query_res = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
    let config_res: ConfigResponse = from_json(query_res).unwrap();

    // has not changed
    assert_eq!(
        EpochConfig {
            duration: Uint64::new(86400),
            genesis_epoch: Uint64::new(current_time.seconds()),
        },
        config_res.epoch_config
    );
}
