use cosmwasm_std::testing::{message_info, mock_dependencies, mock_env};
use cosmwasm_std::{from_json, Timestamp, Uint64};
use cw_multi_test::IntoBech32;

use amm::epoch_manager::{
    ConfigResponse, Epoch, EpochConfig, EpochResponse, InstantiateMsg, QueryMsg,
};
use epoch_manager::contract::{instantiate, query};
use epoch_manager::ContractError;

use crate::common::mock_instantiation;

mod common;

#[test]
fn get_new_epoch_successfully() {
    let mut deps = mock_dependencies();
    let owner = "owner".into_bech32();

    let info = message_info(&owner, &[]);
    let mut env = mock_env();
    mock_instantiation(deps.as_mut(), &env, info.clone()).unwrap();

    let config_res = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
    let config_response: ConfigResponse = from_json(config_res).unwrap();

    let next_epoch_start_time = config_response
        .epoch_config
        .genesis_epoch
        .checked_add(config_response.epoch_config.duration)
        .unwrap();

    // move time ahead so we can get the new epoch
    env.block.time = Timestamp::from_seconds(next_epoch_start_time.u64());

    let query_res = query(deps.as_ref(), env.clone(), QueryMsg::CurrentEpoch {}).unwrap();
    let epoch_response: EpochResponse = from_json(query_res).unwrap();

    let current_epoch = Epoch {
        id: 1,
        start_time: Timestamp::from_seconds(next_epoch_start_time.u64()),
    };
    assert_eq!(epoch_response.epoch, current_epoch);

    let query_res = query(deps.as_ref(), env.clone(), QueryMsg::Epoch { id: 1 }).unwrap();
    let epoch_response: EpochResponse = from_json(query_res).unwrap();
    assert_eq!(epoch_response.epoch, current_epoch);

    let query_res = query(deps.as_ref(), env.clone(), QueryMsg::Epoch { id: 2 }).unwrap();
    let epoch_response: EpochResponse = from_json(query_res).unwrap();

    assert_eq!(
        epoch_response.epoch,
        Epoch {
            id: 2,
            start_time: Timestamp::from_seconds(next_epoch_start_time.u64())
                .plus_seconds(config_response.epoch_config.duration.u64()),
        }
    );

    // let's move to epoch 2
    env.block.time = env
        .block
        .time
        .plus_seconds(config_response.epoch_config.duration.u64());

    let third_epoch_time = current_epoch
        .start_time
        .plus_seconds(config_response.epoch_config.duration.u64() * 2);

    env.block.time = third_epoch_time;

    let query_res = query(deps.as_ref(), env.clone(), QueryMsg::CurrentEpoch {}).unwrap();
    let epoch_response: EpochResponse = from_json(query_res).unwrap();

    let current_epoch = Epoch {
        id: 3,
        start_time: third_epoch_time,
    };

    assert_eq!(epoch_response.epoch, current_epoch);

    // move time ahead but not enough to trigger the next epoch
    env.block.time = env
        .block
        .time
        .plus_seconds(config_response.epoch_config.duration.u64() - 1);

    let query_res = query(deps.as_ref(), env.clone(), QueryMsg::CurrentEpoch {}).unwrap();
    let epoch_response: EpochResponse = from_json(query_res).unwrap();
    // should still be the third epoch
    assert_eq!(epoch_response.epoch, current_epoch);

    // move time ahead but not enough to trigger the next epoch
    let fourth_epoch_time = current_epoch
        .start_time
        .plus_seconds(config_response.epoch_config.duration.u64());

    // move the time necessary to trigger next epoch
    env.block.time = env.block.time.plus_seconds(1);

    let query_res = query(deps.as_ref(), env.clone(), QueryMsg::CurrentEpoch {}).unwrap();
    let epoch_response: EpochResponse = from_json(query_res).unwrap();

    let current_epoch = Epoch {
        id: 4,
        start_time: fourth_epoch_time,
    };

    assert_eq!(epoch_response.epoch, current_epoch);
}

#[test]
fn get_new_epoch_unsuccessfully() {
    let mut deps = mock_dependencies();
    let owner = "owner".into_bech32();

    let info = message_info(&owner, &[]);
    let mut env = mock_env();

    let current_time = env.block.time;
    let msg = InstantiateMsg {
        owner: "owner".into_bech32().to_string(),
        epoch_config: EpochConfig {
            duration: Uint64::new(86400),
            // instantiate the epoch manager with the genesis epoch 1 day in the future
            genesis_epoch: Uint64::new(current_time.plus_days(1).seconds()),
        },
    };

    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    let config_res = query(deps.as_ref(), env.clone(), QueryMsg::Config {}).unwrap();
    let config_response: ConfigResponse = from_json(config_res).unwrap();

    // move time ahead but not enough to get above the genesis epoch a new epoch
    env.block.time = Timestamp::from_seconds(
        config_response
            .epoch_config
            .genesis_epoch
            .saturating_sub(Uint64::new(100))
            .u64(),
    );

    let query_res = query(deps.as_ref(), env.clone(), QueryMsg::CurrentEpoch {}).unwrap_err();

    match query_res {
        ContractError::GenesisEpochHasNotStarted => {}
        _ => panic!("should return ContractError::GenesisEpochHasNotStarted"),
    }

    // move time a bit ahead of the genesis epoch
    env.block.time = env.block.time.plus_seconds(100);

    let query_res = query(deps.as_ref(), env.clone(), QueryMsg::CurrentEpoch {}).unwrap();
    let epoch_response: EpochResponse = from_json(query_res).unwrap();

    assert_eq!(
        epoch_response.epoch,
        Epoch {
            id: 0,
            start_time: Timestamp::from_seconds(config_response.epoch_config.genesis_epoch.u64()),
        }
    );
}
