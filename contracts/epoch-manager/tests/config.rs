use cosmwasm_std::testing::{message_info, mock_dependencies, mock_env};
use cosmwasm_std::{from_json, Uint64};
use cw_multi_test::IntoBech32;

use epoch_manager::contract::{execute, query};
use epoch_manager::ContractError;
use mantra_dex_std::epoch_manager::{ConfigResponse, EpochConfig, ExecuteMsg, QueryMsg};

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

    let future_time = current_time.plus_seconds(3600); // 1 hour in the future

    let msg = ExecuteMsg::UpdateConfig {
        epoch_config: Some(EpochConfig {
            duration: Uint64::new(172800),
            genesis_epoch: Uint64::new(future_time.seconds()),
        }),
    };

    execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    let query_res = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
    let config_res: ConfigResponse = from_json(query_res).unwrap();
    assert_eq!(
        EpochConfig {
            duration: Uint64::new(172800),
            genesis_epoch: Uint64::new(future_time.seconds()),
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

    // Use a future time for genesis_epoch but invalid duration
    let future_time = current_time.plus_seconds(3600); // 1 hour in the future

    let msg = ExecuteMsg::UpdateConfig {
        epoch_config: Some(EpochConfig {
            duration: Uint64::new(600),
            genesis_epoch: Uint64::new(future_time.seconds()),
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
            genesis_epoch: Uint64::new(future_time.seconds()),
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

#[test]
fn update_config_genesis_epoch_in_past() {
    let mut deps = mock_dependencies();

    let owner = "owner".into_bech32();

    let info = message_info(&owner, &[]);
    let current_time = mock_env().block.time;
    mock_instantiation(deps.as_mut(), &mock_env(), info.clone()).unwrap();

    // Try to update with a genesis_epoch in the past
    let past_time = current_time.minus_seconds(100); // 100 seconds in the past

    let msg = ExecuteMsg::UpdateConfig {
        epoch_config: Some(EpochConfig {
            duration: Uint64::new(86400),
            genesis_epoch: Uint64::new(past_time.seconds()),
        }),
    };

    let err = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
    match err {
        ContractError::InvalidStartTime => {}
        _ => panic!("should return ContractError::InvalidStartTime"),
    }

    // Verify config was not updated
    let query_res = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
    let config_res: ConfigResponse = from_json(query_res).unwrap();
    assert_eq!(
        EpochConfig {
            duration: Uint64::new(86400),
            genesis_epoch: Uint64::new(current_time.seconds()),
        },
        config_res.epoch_config
    );
}
