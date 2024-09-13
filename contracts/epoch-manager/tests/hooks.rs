use cosmwasm_std::from_json;
use cosmwasm_std::testing::{message_info, mock_dependencies, mock_env};
use cw_controllers::{AdminError, HookError, HooksResponse};
use cw_multi_test::IntoBech32;

use crate::common::{mock_add_hook, mock_instantiation};
use amm::epoch_manager::{ExecuteMsg, QueryMsg};
use epoch_manager::contract::{execute, query};
use epoch_manager::ContractError;

mod common;
#[test]
fn add_hook_successfully() {
    let mut deps = mock_dependencies();
    let owner = "owner".into_bech32();
    let hook = "hook".into_bech32();

    let info = message_info(&owner, &[]);
    mock_instantiation(deps.as_mut(), info.clone()).unwrap();

    let msg = ExecuteMsg::AddHook {
        contract_addr: hook.to_string(),
    };

    let query_res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::Hook {
            hook: hook.to_string(),
        },
    )
    .unwrap();
    let hook_registered: bool = from_json(query_res).unwrap();
    assert!(!hook_registered);

    execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

    let query_res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::Hook {
            hook: hook.to_string(),
        },
    )
    .unwrap();
    let hook_registered: bool = from_json(query_res).unwrap();
    assert!(hook_registered);

    for i in 2..10 {
        let msg = ExecuteMsg::AddHook {
            contract_addr: format!("hook_contract_{}", i).into_bech32().to_string(),
        };
        execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
    }

    let query_res = query(deps.as_ref(), mock_env(), QueryMsg::Hooks {}).unwrap();
    let hooks_response: HooksResponse = from_json(query_res).unwrap();
    assert_eq!(
        hooks_response,
        HooksResponse {
            hooks: vec![
                "cosmwasm1qeyznz6ghcp3n9380tj8yy26gmnevnf2cwyzucdcgdgl8slc54rssydn7s".to_string(),
                "cosmwasm1kzpr2c4w36qvtv6ekpkx35h3ge0p6w0kcsnywskj0m9vz3yrjadqt09yz6".to_string(),
                "cosmwasm1gxdm64dccqfkr523xjn76k6n5ajtx3s7xpk6x3u50lcrplrqv8nqzfzpx9".to_string(),
                "cosmwasm1ac2zsy73law4htkwtf2sskqxadhmasvm7k9m2579ddu7ek0rn80qg3pqj0".to_string(),
                "cosmwasm15y95duqr4vgtqq2vgqd7tcu07yzy6tgp8u0uv27jd9prgxp4ht4syg7xjd".to_string(),
                "cosmwasm16r3ekpxptpy6ve4vlyp72h4qrlheqxgap8t0vh5hq857w3f24vjs73errt".to_string(),
                "cosmwasm1w82sdkcqadlvu8p8clt5nmrg9vw8d9kjhs6q52mqnahksysx3enqt2g74r".to_string(),
                "cosmwasm1042g6xl3gjlnycv5mp87vaajl2naxgkc388ufczvwwczf0hzrdyscc5kmu".to_string(),
                "cosmwasm1qpjn05aazecj4qqpwt7px407jtxq7yryvxh3y74qwxvy9e9qmu6s36j94c".to_string(),
            ]
        }
    );
}

#[test]
fn add_hook_unsuccessfully() {
    let mut deps = mock_dependencies();
    let owner = "owner".into_bech32();
    let hook = "hook".into_bech32();

    let info = message_info(&owner, &[]);
    mock_instantiation(deps.as_mut(), info.clone()).unwrap();

    let msg = ExecuteMsg::AddHook {
        contract_addr: hook.to_string(),
    };

    let unauthorized = "unauthorized".into_bech32();
    let info = message_info(&unauthorized, &[]);
    let err = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
    match err {
        ContractError::HookError(error) => {
            assert_eq!(error, HookError::Admin(AdminError::NotAdmin {}))
        }
        _ => panic!("should return ContractError::HookError::Admin(AdminError::NotAdmin)"),
    }
}

#[test]
fn remove_hook_successfully() {
    let mut deps = mock_dependencies();
    let owner = "owner".into_bech32();
    let hook = "hook".into_bech32();

    let info = message_info(&owner, &[]);
    mock_instantiation(deps.as_mut(), info.clone()).unwrap();
    mock_add_hook(deps.as_mut(), info.clone(), &hook).unwrap();

    let msg = ExecuteMsg::RemoveHook {
        contract_addr: hook.to_string(),
    };

    let query_res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::Hook {
            hook: hook.to_string(),
        },
    )
    .unwrap();
    let hook_registered: bool = from_json(query_res).unwrap();
    assert!(hook_registered);

    execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    let query_res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::Hook {
            hook: hook.to_string(),
        },
    )
    .unwrap();
    let hook_registered: bool = from_json(query_res).unwrap();
    assert!(!hook_registered);
}

#[test]
fn remove_hook_unsuccessfully() {
    let mut deps = mock_dependencies();
    let owner = "owner".into_bech32();
    let hook = "hook".into_bech32();

    let info = message_info(&owner, &[]);
    mock_instantiation(deps.as_mut(), info.clone()).unwrap();
    mock_add_hook(deps.as_mut(), info, &hook).unwrap();

    let msg = ExecuteMsg::RemoveHook {
        contract_addr: hook.to_string(),
    };

    let query_res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::Hook {
            hook: hook.to_string(),
        },
    )
    .unwrap();
    let hook_registered: bool = from_json(query_res).unwrap();
    assert!(hook_registered);

    let unauthorized = "unauthorized".into_bech32();

    let info = message_info(&unauthorized, &[]);

    let err = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
    match err {
        ContractError::HookError(error) => {
            assert_eq!(error, HookError::Admin(AdminError::NotAdmin {}))
        }
        _ => panic!("should return ContractError::HookError::Admin(AdminError::NotAdmin)"),
    }
}
