extern crate core;

use cosmwasm_std::{coin, coins, Coin, Uint128};
use cw_utils::PaymentError;
use farm_manager::ContractError;
use mantra_dex_std::constants::LP_SYMBOL;
use mantra_dex_std::farm_manager::{FarmAction, FarmParams, FarmsBy, PositionAction};

use crate::common::suite::TestingSuite;
use crate::common::{MOCK_CONTRACT_ADDR_1, MOCK_CONTRACT_ADDR_2};
use test_utils::common_constants::{
    DENOM_UOM as UOM_DENOM, DENOM_UOSMO as UOSMO_DENOM, DENOM_UUSDY as UUSDY_DENOM,
    INITIAL_BALANCE, UOM_FARM_CREATION_FEE,
};

const INVALID_LP: &str = "invalid_lp";

const FARM_AMOUNT_4K: u128 = 4_000u128;
const FARM_AMOUNT_10K: u128 = 10_000u128;
const FARM_AMOUNT_2K: u128 = 2_000u128;
const FARM_AMOUNT_5K: u128 = 5_000u128;
const FARM_AMOUNT_8K: u128 = 8_000u128;
const EXCESS_FARM_FEE: u128 = 3_000u128;

const START_EPOCH_0: Option<u64> = Some(0);
const START_EPOCH_1: Option<u64> = Some(1);
const START_EPOCH_3: Option<u64> = Some(3);
const START_EPOCH_12: Option<u64> = Some(12);
const START_EPOCH_20: Option<u64> = Some(20);
const START_EPOCH_25: Option<u64> = Some(25);
const START_EPOCH_30: Option<u64> = Some(30);

const END_EPOCH_3: Option<u64> = Some(3);
const END_EPOCH_4: Option<u64> = Some(4);
const END_EPOCH_5: Option<u64> = Some(5);
const END_EPOCH_8: Option<u64> = Some(8);
const END_EPOCH_13: Option<u64> = Some(13);
const END_EPOCH_15: Option<u64> = Some(15);
const END_EPOCH_16: Option<u64> = Some(16);
const END_EPOCH_20: Option<u64> = Some(20);
const END_EPOCH_28: Option<u64> = Some(28);
const END_EPOCH_35: Option<u64> = Some(35);

const FARM_ID_1: &str = "farm_1";
const FARM_ID_2: &str = "farm_2";
const FARM_ID_X: &str = "farm_x";
const CUSTOM_ID_1: &str = "custom_id_1";
const BOGUS_ID_1: &str = "1";

const M_FARM_ID_1: &str = "m-farm_1";
const M_FARM_ID_2: &str = "m-farm_2";
const M_FARM_ID_X: &str = "m-farm_x";
const M_CUSTOM_ID_1: &str = "m-custom_id_1";
const F_1_ID: &str = "f-1";

#[test]
fn create_farms() {
    let lp_denom = format!("factory/{MOCK_CONTRACT_ADDR_1}/{LP_SYMBOL}").to_string();
    let invalid_lp_denom = format!("factory/{MOCK_CONTRACT_ADDR_2}/{LP_SYMBOL}").to_string();

    let mut suite = TestingSuite::default_with_balances(vec![
        coin(INITIAL_BALANCE, UOM_DENOM.to_string()),
        coin(INITIAL_BALANCE, UUSDY_DENOM.to_string()),
        coin(INITIAL_BALANCE, UOSMO_DENOM.to_string()),
        coin(INITIAL_BALANCE, lp_denom.clone()),
        coin(INITIAL_BALANCE, invalid_lp_denom.clone()),
    ]);
    suite.instantiate_default();

    let creator = suite.creator().clone();
    let other = suite.senders[1].clone();
    let fee_collector = suite.fee_collector_addr.clone();

    suite
        .query_current_epoch(|result| {
            let epoch_response = result.unwrap();
            assert_eq!(epoch_response.epoch.id, 0u64);
        })
        .manage_farm(
            &other,
            FarmAction::Fill {
                params: FarmParams {
                    lp_denom: lp_denom.clone(),
                    // current epoch is 0
                    start_epoch: START_EPOCH_0,
                    preliminary_end_epoch: None,
                    curve: None,
                    farm_asset: Coin {
                        denom: UUSDY_DENOM.to_string(),
                        amount: Uint128::new(FARM_AMOUNT_4K),
                    },
                    farm_identifier: None,
                },
            },
            vec![
                coin(FARM_AMOUNT_4K, UUSDY_DENOM),
                coin(UOM_FARM_CREATION_FEE, UOM_DENOM),
            ],
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();
                match err {
                    ContractError::InvalidEpoch { which } => {
                        assert_eq!(which, "start")
                    }
                    _ => panic!("Wrong error type, should return ContractError::InvalidEpoch"),
                }
            },
        );

    for _ in 0..10 {
        suite.add_one_epoch();
    }
    // current epoch is 10

    // try all misconfigurations when creating a farm
    suite
        .manage_farm(
            &creator,
            FarmAction::Fill {
                params: FarmParams {
                    lp_denom: lp_denom.clone(),
                    start_epoch: START_EPOCH_25,
                    preliminary_end_epoch: None,
                    curve: None,
                    farm_asset: Coin {
                        denom: UUSDY_DENOM.to_string(),
                        amount: Default::default(),
                    },
                    farm_identifier: None,
                },
            },
            vec![],
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();

                match err {
                    ContractError::InvalidFarmAmount { .. } => {}
                    _ => panic!("Wrong error type, should return ContractError::InvalidFarmAmount"),
                }
            },
        )
        .manage_farm(
            &other,
            FarmAction::Fill {
                params: FarmParams {
                    lp_denom: lp_denom.clone(),
                    start_epoch: START_EPOCH_25,
                    preliminary_end_epoch: None,
                    curve: None,
                    farm_asset: Coin {
                        denom: UUSDY_DENOM.to_string(),
                        amount: Uint128::new(FARM_AMOUNT_2K),
                    },
                    farm_identifier: None,
                },
            },
            vec![coin(FARM_AMOUNT_2K, UUSDY_DENOM)],
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();
                match err {
                    ContractError::FarmFeeMissing => {}
                    _ => {
                        panic!("Wrong error type, should return ContractError::FarmFeeMissing")
                    }
                }
            },
        )
        .manage_farm(
            &other,
            FarmAction::Fill {
                params: FarmParams {
                    lp_denom: lp_denom.clone(),
                    start_epoch: START_EPOCH_25,
                    preliminary_end_epoch: None,
                    curve: None,
                    farm_asset: Coin {
                        denom: UOM_DENOM.to_string(),
                        amount: Uint128::new(FARM_AMOUNT_5K),
                    },
                    farm_identifier: None,
                },
            },
            vec![coin(FARM_AMOUNT_8K, UOM_DENOM)],
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();
                match err {
                    ContractError::AssetMismatch => {}
                    _ => panic!("Wrong error type, should return ContractError::AssetMismatch"),
                }
            },
        )
        .manage_farm(
            &other,
            FarmAction::Fill {
                params: FarmParams {
                    lp_denom: lp_denom.clone(),
                    start_epoch: START_EPOCH_25,
                    preliminary_end_epoch: None,
                    curve: None,
                    farm_asset: Coin {
                        denom: UUSDY_DENOM.to_string(),
                        amount: Uint128::new(FARM_AMOUNT_2K),
                    },
                    farm_identifier: None,
                },
            },
            vec![coin(UOM_FARM_CREATION_FEE, UOM_DENOM)],
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();
                match err {
                    ContractError::AssetMismatch => {}
                    _ => panic!("Wrong error type, should return ContractError::AssetMismatch"),
                }
            },
        )
        .manage_farm(
            &other,
            FarmAction::Fill {
                params: FarmParams {
                    lp_denom: lp_denom.clone(),
                    start_epoch: START_EPOCH_25,
                    preliminary_end_epoch: None,
                    curve: None,
                    farm_asset: Coin {
                        denom: UUSDY_DENOM.to_string(),
                        amount: Uint128::new(FARM_AMOUNT_2K),
                    },
                    farm_identifier: None,
                },
            },
            vec![
                coin(FARM_AMOUNT_5K, UUSDY_DENOM),
                coin(UOM_FARM_CREATION_FEE, UOM_DENOM),
            ],
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();
                match err {
                    ContractError::AssetMismatch => {}
                    _ => panic!("Wrong error type, should return ContractError::AssetMismatch"),
                }
            },
        )
        .manage_farm(
            &other,
            FarmAction::Fill {
                params: FarmParams {
                    lp_denom: lp_denom.clone(),
                    start_epoch: START_EPOCH_25,
                    preliminary_end_epoch: None,
                    curve: None,
                    farm_asset: Coin {
                        denom: UUSDY_DENOM.to_string(),
                        amount: Uint128::new(FARM_AMOUNT_5K),
                    },
                    farm_identifier: None,
                },
            },
            vec![
                coin(FARM_AMOUNT_4K, UUSDY_DENOM),
                coin(UOM_FARM_CREATION_FEE, UOM_DENOM),
            ],
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();
                match err {
                    ContractError::AssetMismatch => {}
                    _ => panic!("Wrong error type, should return ContractError::AssetMismatch"),
                }
            },
        )
        .manage_farm(
            &other,
            FarmAction::Fill {
                params: FarmParams {
                    lp_denom: lp_denom.clone(),
                    start_epoch: START_EPOCH_25,
                    preliminary_end_epoch: None,
                    curve: None,
                    farm_asset: Coin {
                        denom: UUSDY_DENOM.to_string(),
                        amount: Uint128::new(FARM_AMOUNT_4K),
                    },
                    farm_identifier: None,
                },
            },
            vec![
                coin(FARM_AMOUNT_4K, UUSDY_DENOM),
                coin(UOM_FARM_CREATION_FEE, UOM_DENOM),
            ],
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();

                match err {
                    ContractError::FarmStartTooFar => {}
                    _ => panic!("Wrong error type, should return ContractError::FarmStartTooFar"),
                }
            },
        )
        .manage_farm(
            &other,
            FarmAction::Fill {
                params: FarmParams {
                    lp_denom: lp_denom.clone(),
                    start_epoch: START_EPOCH_20,
                    preliminary_end_epoch: END_EPOCH_8,
                    curve: None,
                    farm_asset: Coin {
                        denom: UUSDY_DENOM.to_string(),
                        amount: Uint128::new(FARM_AMOUNT_4K),
                    },
                    farm_identifier: None,
                },
            },
            vec![
                coin(FARM_AMOUNT_4K, UUSDY_DENOM),
                coin(UOM_FARM_CREATION_FEE, UOM_DENOM),
            ],
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();

                match err {
                    ContractError::FarmStartTimeAfterEndTime => {}
                    _ => panic!(
                        "Wrong error type, should return ContractError::FarmStartTimeAfterEndTime"
                    ),
                }
            },
        )
        .manage_farm(
            &other,
            FarmAction::Fill {
                params: FarmParams {
                    lp_denom: lp_denom.clone(),
                    start_epoch: START_EPOCH_20,
                    preliminary_end_epoch: END_EPOCH_15,
                    curve: None,
                    farm_asset: Coin {
                        denom: UUSDY_DENOM.to_string(),
                        amount: Uint128::new(FARM_AMOUNT_4K),
                    },
                    farm_identifier: None,
                },
            },
            vec![
                coin(FARM_AMOUNT_4K, UUSDY_DENOM),
                coin(UOM_FARM_CREATION_FEE, UOM_DENOM),
            ],
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();

                match err {
                    ContractError::FarmStartTimeAfterEndTime => {}
                    _ => panic!(
                        "Wrong error type, should return ContractError::FarmStartTimeAfterEndTime"
                    ),
                }
            },
        )
        .manage_farm(
            &other,
            FarmAction::Fill {
                params: FarmParams {
                    lp_denom: lp_denom.clone(),
                    // current epoch is 10
                    start_epoch: START_EPOCH_3,
                    preliminary_end_epoch: END_EPOCH_5,
                    curve: None,
                    farm_asset: Coin {
                        denom: UUSDY_DENOM.to_string(),
                        amount: Uint128::new(FARM_AMOUNT_4K),
                    },
                    farm_identifier: None,
                },
            },
            vec![
                coin(FARM_AMOUNT_4K, UUSDY_DENOM),
                coin(UOM_FARM_CREATION_FEE, UOM_DENOM),
            ],
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();
                match err {
                    ContractError::InvalidEpoch { which } => {
                        assert_eq!(which, "start")
                    }
                    _ => panic!("Wrong error type, should return ContractError::InvalidEpoch"),
                }
            },
        )
        .manage_farm(
            &other,
            FarmAction::Fill {
                params: FarmParams {
                    lp_denom: lp_denom.clone(),
                    // current epoch is 10
                    start_epoch: None,
                    preliminary_end_epoch: END_EPOCH_5,
                    curve: None,
                    farm_asset: Coin {
                        denom: UUSDY_DENOM.to_string(),
                        amount: Uint128::new(FARM_AMOUNT_4K),
                    },
                    farm_identifier: None,
                },
            },
            vec![
                coin(FARM_AMOUNT_4K, UUSDY_DENOM),
                coin(UOM_FARM_CREATION_FEE, UOM_DENOM),
            ],
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();
                match err {
                    ContractError::FarmStartTimeAfterEndTime => {}
                    _ => panic!(
                        "Wrong error type, should return ContractError::FarmStartTimeAfterEndTime"
                    ),
                }
            },
        )
        .manage_farm(
            &other,
            FarmAction::Fill {
                params: FarmParams {
                    lp_denom: lp_denom.clone(),
                    start_epoch: START_EPOCH_20,
                    preliminary_end_epoch: END_EPOCH_20,
                    curve: None,
                    farm_asset: Coin {
                        denom: UUSDY_DENOM.to_string(),
                        amount: Uint128::new(FARM_AMOUNT_4K),
                    },
                    farm_identifier: None,
                },
            },
            vec![
                coin(FARM_AMOUNT_4K, UUSDY_DENOM),
                coin(UOM_FARM_CREATION_FEE, UOM_DENOM),
            ],
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();

                match err {
                    ContractError::FarmStartTimeAfterEndTime => {}
                    _ => panic!(
                        "Wrong error type, should return ContractError::FarmStartTimeAfterEndTime"
                    ),
                }
            },
        )
        .manage_farm(
            &other,
            FarmAction::Fill {
                params: FarmParams {
                    lp_denom: lp_denom.clone(),
                    start_epoch: START_EPOCH_30,
                    preliminary_end_epoch: END_EPOCH_35,
                    curve: None,
                    farm_asset: Coin {
                        denom: UUSDY_DENOM.to_string(),
                        amount: Uint128::new(FARM_AMOUNT_4K),
                    },
                    farm_identifier: None,
                },
            },
            vec![
                coin(FARM_AMOUNT_4K, UUSDY_DENOM),
                coin(UOM_FARM_CREATION_FEE, UOM_DENOM),
            ],
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();

                match err {
                    ContractError::FarmStartTooFar => {}
                    _ => panic!("Wrong error type, should return ContractError::FarmStartTooFar"),
                }
            },
        )
        .manage_farm(
            &other,
            FarmAction::Fill {
                params: FarmParams {
                    lp_denom: invalid_lp_denom.clone(),
                    start_epoch: START_EPOCH_20,
                    preliminary_end_epoch: END_EPOCH_28,
                    curve: None,
                    farm_asset: Coin {
                        denom: UUSDY_DENOM.to_string(),
                        amount: Uint128::new(FARM_AMOUNT_4K),
                    },
                    farm_identifier: Some(FARM_ID_1.to_string()),
                },
            },
            vec![
                coin(FARM_AMOUNT_4K, UUSDY_DENOM),
                coin(UOM_FARM_CREATION_FEE, UOM_DENOM),
            ],
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();
                // trying to create a farm for an invalid lp_denom, i.e. an lp_denom that wasn't created
                // by the pool manager, should fail
                match err {
                    ContractError::AssetMismatch => {}
                    _ => panic!("Wrong error type, should return ContractError::AssetMismatch"),
                }
            },
        );

    // create a farm properly
    suite
        .manage_farm(
            &other,
            FarmAction::Fill {
                params: FarmParams {
                    lp_denom: lp_denom.clone(),
                    start_epoch: START_EPOCH_20,
                    preliminary_end_epoch: END_EPOCH_28,
                    curve: None,
                    farm_asset: Coin {
                        denom: UUSDY_DENOM.to_string(),
                        amount: Uint128::new(FARM_AMOUNT_4K),
                    },
                    farm_identifier: Some(FARM_ID_1.to_string()),
                },
            },
            vec![
                coin(FARM_AMOUNT_4K, UUSDY_DENOM),
                coin(UOM_FARM_CREATION_FEE, UOM_DENOM),
            ],
            |result| {
                result.unwrap();
            },
        )
        .manage_farm(
            &other,
            FarmAction::Fill {
                params: FarmParams {
                    lp_denom: lp_denom.clone(),
                    start_epoch: START_EPOCH_20,
                    preliminary_end_epoch: END_EPOCH_28,
                    curve: None,
                    farm_asset: Coin {
                        denom: UUSDY_DENOM.to_string(),
                        amount: Uint128::new(FARM_AMOUNT_10K),
                    },
                    farm_identifier: None,
                },
            },
            vec![
                coin(FARM_AMOUNT_10K, UUSDY_DENOM),
                coin(UOM_FARM_CREATION_FEE, UOM_DENOM),
            ],
            |result| {
                result.unwrap();
            },
        )
        .manage_farm(
            &other,
            FarmAction::Fill {
                params: FarmParams {
                    lp_denom: lp_denom.clone(),
                    start_epoch: START_EPOCH_20,
                    preliminary_end_epoch: END_EPOCH_28,
                    curve: None,
                    farm_asset: Coin {
                        denom: UUSDY_DENOM.to_string(),
                        amount: Uint128::new(FARM_AMOUNT_4K),
                    },
                    farm_identifier: None,
                },
            },
            vec![
                coin(FARM_AMOUNT_4K, UUSDY_DENOM),
                coin(UOM_FARM_CREATION_FEE, UOM_DENOM),
            ],
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();
                // should fail, max farms per lp_denom was set to 2 in the instantiate_default
                // function
                match err {
                    ContractError::TooManyFarms { .. } => {}
                    _ => panic!("Wrong error type, should return ContractError::TooManyFarms"),
                }
            },
        )
        .query_farms(None, None, None, |result| {
            let farms_response = result.unwrap();
            assert_eq!(farms_response.farms.len(), 2);
        })
        .query_farms(
            Some(FarmsBy::Identifier(M_FARM_ID_1.to_string())),
            None,
            None,
            |result| {
                let farms_response = result.unwrap();
                assert_eq!(farms_response.farms.len(), 1);
                assert_eq!(
                    farms_response.farms[0].farm_asset,
                    Coin {
                        denom: UUSDY_DENOM.to_string(),
                        amount: Uint128::new(FARM_AMOUNT_4K),
                    }
                );
            },
        )
        .query_farms(
            Some(FarmsBy::Identifier(F_1_ID.to_string())),
            None,
            None,
            |result| {
                let farms_response = result.unwrap();
                assert_eq!(farms_response.farms.len(), 1);
                assert_eq!(
                    farms_response.farms[0].farm_asset,
                    Coin {
                        denom: UUSDY_DENOM.to_string(),
                        amount: Uint128::new(FARM_AMOUNT_10K),
                    }
                );
            },
        )
        .query_farms(
            Some(FarmsBy::FarmAsset(UUSDY_DENOM.to_string())),
            None,
            None,
            |result| {
                let farms_response = result.unwrap();
                assert_eq!(farms_response.farms.len(), 2);
            },
        )
        .query_farms(
            Some(FarmsBy::LpDenom(lp_denom.clone())),
            None,
            None,
            |result| {
                let farms_response = result.unwrap();
                assert_eq!(farms_response.farms.len(), 2);
            },
        )
        // two farms were created, therefore the fee collector should have received 2_000 uom
        .query_balance(UOM_DENOM.to_string(), &fee_collector, |balance| {
            assert_eq!(balance, Uint128::new(2 * UOM_FARM_CREATION_FEE));
        });
}

#[test]
fn expand_farms() {
    let lp_denom = format!("factory/{MOCK_CONTRACT_ADDR_1}/{LP_SYMBOL}").to_string();

    let mut suite = TestingSuite::default_with_balances(vec![
        coin(INITIAL_BALANCE, UOM_DENOM.to_string()),
        coin(INITIAL_BALANCE, UUSDY_DENOM.to_string()),
        coin(INITIAL_BALANCE, UOSMO_DENOM.to_string()),
        coin(INITIAL_BALANCE, lp_denom.clone()),
    ]);

    let creator = suite.creator();
    let other = suite.senders[1].clone();

    suite.instantiate_default();

    for _ in 0..10 {
        suite.add_one_epoch();
    }
    // current epoch is 10

    suite
        .manage_farm(
            &other,
            FarmAction::Fill {
                params: FarmParams {
                    lp_denom: lp_denom.clone(),
                    start_epoch: START_EPOCH_20,
                    preliminary_end_epoch: END_EPOCH_28,
                    curve: None,
                    farm_asset: Coin {
                        denom: UUSDY_DENOM.to_string(),
                        amount: Uint128::new(FARM_AMOUNT_4K),
                    },
                    farm_identifier: Some(FARM_ID_1.to_string()),
                },
            },
            vec![
                coin(FARM_AMOUNT_4K, UUSDY_DENOM),
                coin(UOM_FARM_CREATION_FEE, UOM_DENOM),
            ],
            |result| {
                result.unwrap();
            },
        )
        .manage_farm(
            &creator,
            FarmAction::Fill {
                params: FarmParams {
                    lp_denom: lp_denom.clone(),
                    start_epoch: START_EPOCH_20,
                    preliminary_end_epoch: END_EPOCH_28,
                    curve: None,
                    farm_asset: Coin {
                        denom: UUSDY_DENOM.to_string(),
                        amount: Uint128::new(FARM_AMOUNT_8K),
                    },
                    farm_identifier: Some(M_FARM_ID_1.to_string()),
                },
            },
            vec![coin(FARM_AMOUNT_4K, UUSDY_DENOM)],
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();

                match err {
                    ContractError::Unauthorized => {}
                    _ => panic!("Wrong error type, should return ContractError::Unauthorized"),
                }
            },
        )
        .manage_farm(
            &other,
            FarmAction::Fill {
                params: FarmParams {
                    lp_denom: lp_denom.clone(),
                    start_epoch: START_EPOCH_20,
                    preliminary_end_epoch: END_EPOCH_28,
                    curve: None,
                    farm_asset: Coin {
                        denom: UOM_DENOM.to_string(),
                        amount: Uint128::new(FARM_AMOUNT_8K),
                    },
                    farm_identifier: Some(M_FARM_ID_1.to_string()),
                },
            },
            vec![coin(FARM_AMOUNT_8K, UOM_DENOM)],
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();
                match err {
                    ContractError::AssetMismatch => {}
                    _ => panic!("Wrong error type, should return ContractError::AssetMismatch"),
                }
            },
        )
        .manage_farm(
            &other,
            FarmAction::Fill {
                params: FarmParams {
                    lp_denom: lp_denom.clone(),
                    start_epoch: START_EPOCH_20,
                    preliminary_end_epoch: END_EPOCH_28,
                    curve: None,
                    farm_asset: Coin {
                        denom: UUSDY_DENOM.to_string(),
                        amount: Uint128::new(4_100u128),
                    },
                    farm_identifier: Some(M_FARM_ID_1.to_string()),
                },
            },
            vec![coin(4_100, UUSDY_DENOM)],
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();

                match err {
                    ContractError::InvalidExpansionAmount { .. } => {}
                    _ => panic!(
                        "Wrong error type, should return ContractError::InvalidExpansionAmount"
                    ),
                }
            },
        )
        .manage_farm(
            &other,
            FarmAction::Fill {
                params: FarmParams {
                    lp_denom: lp_denom.clone(),
                    start_epoch: START_EPOCH_20,
                    preliminary_end_epoch: END_EPOCH_28,
                    curve: None,
                    farm_asset: Coin {
                        denom: UUSDY_DENOM.to_string(),
                        amount: Uint128::new(4_100u128),
                    },
                    farm_identifier: Some(M_FARM_ID_1.to_string()),
                },
            },
            vec![], // sending no funds when expanding a farm should fail
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();

                match err {
                    ContractError::PaymentError(e) => {
                        assert_eq!(e, PaymentError::NoFunds {})
                    }
                    _ => panic!("Wrong error type, should return ContractError::PaymentError"),
                }
            },
        )
        .manage_farm(
            &other,
            FarmAction::Fill {
                params: FarmParams {
                    lp_denom: lp_denom.clone(),
                    start_epoch: START_EPOCH_20,
                    preliminary_end_epoch: END_EPOCH_28,
                    curve: None,
                    farm_asset: Coin {
                        denom: UUSDY_DENOM.to_string(),
                        amount: Uint128::new(4_100u128),
                    },
                    farm_identifier: Some(M_FARM_ID_1.to_string()),
                },
            },
            vec![coin(4_100u128, UOM_DENOM)], // sending different funds than the one provided in the params should fail
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();

                match err {
                    ContractError::AssetMismatch => {}
                    _ => panic!("Wrong error type, should return ContractError::AssetMismatch"),
                }
            },
        )
        .manage_farm(
            &other,
            FarmAction::Fill {
                params: FarmParams {
                    lp_denom: lp_denom.clone(),
                    start_epoch: START_EPOCH_20,
                    preliminary_end_epoch: END_EPOCH_28,
                    curve: None,
                    farm_asset: Coin {
                        denom: UUSDY_DENOM.to_string(),
                        amount: Uint128::new(4_100u128),
                    },
                    farm_identifier: Some(M_FARM_ID_1.to_string()),
                },
            },
            vec![coin(4_100u128, UOM_DENOM)], // sending different funds than the one provided in the params should fail
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();

                match err {
                    ContractError::AssetMismatch => {}
                    _ => panic!("Wrong error type, should return ContractError::AssetMismatch"),
                }
            },
        )
        .query_farms(
            Some(FarmsBy::Identifier(M_FARM_ID_1.to_string())),
            None,
            None,
            |result| {
                let farms_response = result.unwrap();
                let farm = farms_response.farms[0].clone();
                assert_eq!(
                    farm.farm_asset,
                    Coin {
                        denom: UUSDY_DENOM.to_string(),
                        amount: Uint128::new(FARM_AMOUNT_4K),
                    }
                );

                assert_eq!(farm.preliminary_end_epoch, 28);
            },
        )
        .manage_farm(
            &other,
            FarmAction::Fill {
                params: FarmParams {
                    lp_denom: lp_denom.clone(),
                    start_epoch: START_EPOCH_20,
                    preliminary_end_epoch: END_EPOCH_28,
                    curve: None,
                    farm_asset: Coin {
                        denom: UUSDY_DENOM.to_string(),
                        amount: Uint128::new(FARM_AMOUNT_5K),
                    },
                    farm_identifier: Some(M_FARM_ID_1.to_string()),
                },
            },
            vec![coin(FARM_AMOUNT_5K, UUSDY_DENOM)],
            |result| {
                result.unwrap();
            },
        )
        .query_farms(
            Some(FarmsBy::Identifier(M_FARM_ID_1.to_string())),
            None,
            None,
            |result| {
                let farms_response = result.unwrap();
                let farm = farms_response.farms[0].clone();
                assert_eq!(
                    farm.farm_asset,
                    Coin {
                        denom: UUSDY_DENOM.to_string(),
                        amount: Uint128::new(9_000), // 4_000 + 5_000
                    }
                );

                assert_eq!(farm.preliminary_end_epoch, 38); // 28 + 10 ( (5000 / 4000) * 8 ) = 28 + 10 = 38 . 8 is the duration
            },
        );
}

#[test]
fn cant_expand_farm_too_late() {
    let lp_denom = format!("factory/{MOCK_CONTRACT_ADDR_1}/{LP_SYMBOL}").to_string();

    let mut suite = TestingSuite::default_with_balances(vec![
        coin(INITIAL_BALANCE, UOM_DENOM.to_string()),
        coin(INITIAL_BALANCE, UUSDY_DENOM.to_string()),
        coin(INITIAL_BALANCE, UOSMO_DENOM.to_string()),
        coin(INITIAL_BALANCE, lp_denom.clone()),
    ]);

    let creator = suite.creator();

    suite.instantiate_default();

    suite
        .manage_farm(
            &creator,
            FarmAction::Fill {
                params: FarmParams {
                    lp_denom: lp_denom.clone(),
                    start_epoch: START_EPOCH_1,
                    preliminary_end_epoch: Some(2),
                    curve: None,
                    farm_asset: Coin {
                        denom: UUSDY_DENOM.to_string(),
                        amount: Uint128::new(FARM_AMOUNT_4K),
                    },
                    farm_identifier: Some(FARM_ID_1.to_string()),
                },
            },
            vec![
                coin(FARM_AMOUNT_4K, UUSDY_DENOM),
                coin(UOM_FARM_CREATION_FEE, UOM_DENOM),
            ],
            |result| {
                result.unwrap();
            },
        )
        .manage_farm(
            &creator,
            FarmAction::Fill {
                params: FarmParams {
                    lp_denom: lp_denom.clone(),
                    start_epoch: START_EPOCH_1,
                    preliminary_end_epoch: END_EPOCH_3,
                    curve: None,
                    farm_asset: Coin {
                        denom: UUSDY_DENOM.to_string(),
                        amount: Uint128::new(FARM_AMOUNT_4K),
                    },
                    farm_identifier: Some(FARM_ID_2.to_string()),
                },
            },
            vec![
                coin(FARM_AMOUNT_4K, UUSDY_DENOM),
                coin(UOM_FARM_CREATION_FEE, UOM_DENOM),
            ],
            |result| {
                result.unwrap();
            },
        );

    suite.add_epochs(2).query_current_epoch(|result| {
        let epoch_response = result.unwrap();
        assert_eq!(epoch_response.epoch.id, 2);
    });

    suite
        .manage_farm(
            &creator,
            FarmAction::Fill {
                params: FarmParams {
                    lp_denom: lp_denom.clone(),
                    start_epoch: None,
                    preliminary_end_epoch: END_EPOCH_28,
                    curve: None,
                    farm_asset: Coin {
                        denom: UUSDY_DENOM.to_string(),
                        amount: Uint128::new(FARM_AMOUNT_8K),
                    },
                    farm_identifier: Some(M_FARM_ID_1.to_string()),
                },
            },
            vec![coin(FARM_AMOUNT_8K, UUSDY_DENOM)],
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();

                match err {
                    ContractError::FarmAlreadyExpired => {}
                    _ => {
                        panic!("Wrong error type, should return ContractError::FarmAlreadyExpired")
                    }
                }
            },
        )
        .manage_farm(
            &creator,
            FarmAction::Fill {
                params: FarmParams {
                    lp_denom: lp_denom.clone(),
                    start_epoch: None,
                    preliminary_end_epoch: END_EPOCH_4,
                    curve: None,
                    farm_asset: Coin {
                        denom: UUSDY_DENOM.to_string(),
                        amount: Uint128::new(FARM_AMOUNT_2K),
                    },
                    farm_identifier: Some(M_FARM_ID_2.to_string()),
                },
            },
            vec![coin(FARM_AMOUNT_2K, UUSDY_DENOM)],
            |result| {
                result.unwrap();
            },
        );

    suite.add_epochs(2).query_current_epoch(|result| {
        let epoch_response = result.unwrap();
        assert_eq!(epoch_response.epoch.id, 4);
    });

    suite.manage_farm(
        &creator,
        FarmAction::Fill {
            params: FarmParams {
                lp_denom: lp_denom.clone(),
                start_epoch: None,
                preliminary_end_epoch: None,
                curve: None,
                farm_asset: Coin {
                    denom: UUSDY_DENOM.to_string(),
                    amount: Uint128::new(FARM_AMOUNT_2K),
                },
                farm_identifier: Some(M_FARM_ID_2.to_string()),
            },
        },
        vec![coin(FARM_AMOUNT_2K, UUSDY_DENOM)],
        |result| {
            let err = result.unwrap_err().downcast::<ContractError>().unwrap();

            match err {
                ContractError::FarmAlreadyExpired => {}
                _ => panic!("Wrong error type, should return ContractError::FarmAlreadyExpired"),
            }
        },
    );
}

#[test]
#[allow(clippy::inconsistent_digit_grouping)]
fn close_farms() {
    let lp_denom = format!("factory/{MOCK_CONTRACT_ADDR_1}/{LP_SYMBOL}").to_string();
    let lp_denom_2 = format!("factory/{MOCK_CONTRACT_ADDR_1}/2.{LP_SYMBOL}").to_string();

    let mut suite = TestingSuite::default_with_balances(vec![
        coin(INITIAL_BALANCE, UOM_DENOM.to_string()),
        coin(INITIAL_BALANCE, UUSDY_DENOM.to_string()),
        coin(INITIAL_BALANCE, UOSMO_DENOM.to_string()),
        coin(INITIAL_BALANCE, lp_denom.clone()),
        coin(INITIAL_BALANCE, lp_denom_2.clone()),
    ]);

    suite.instantiate_default();

    let other = suite.senders[1].clone();
    let another = suite.senders[2].clone();

    for _ in 0..10 {
        suite.add_one_epoch();
    }
    // current epoch is 10

    suite.manage_farm(
        &other,
        FarmAction::Fill {
            params: FarmParams {
                lp_denom: lp_denom.clone(),
                start_epoch: START_EPOCH_20,
                preliminary_end_epoch: END_EPOCH_28,
                curve: None,
                farm_asset: Coin {
                    denom: UUSDY_DENOM.to_string(),
                    amount: Uint128::new(FARM_AMOUNT_4K),
                },
                farm_identifier: Some(FARM_ID_1.to_string()),
            },
        },
        vec![
            coin(FARM_AMOUNT_4K, UUSDY_DENOM),
            coin(UOM_FARM_CREATION_FEE, UOM_DENOM),
        ],
        |result| {
            result.unwrap();
        },
    );
    suite
        .manage_farm(
            &other,
            FarmAction::Close {
                farm_identifier: M_FARM_ID_1.to_string(),
            },
            vec![coin(UOM_FARM_CREATION_FEE, UOM_DENOM)],
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();
                match err {
                    ContractError::PaymentError { .. } => {}
                    _ => panic!("Wrong error type, should return ContractError::PaymentError"),
                }
            },
        )
        .manage_farm(
            &other,
            FarmAction::Close {
                farm_identifier: M_FARM_ID_2.to_string(),
            },
            vec![],
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();

                match err {
                    ContractError::NonExistentFarm => {}
                    _ => panic!("Wrong error type, should return ContractError::NonExistentFarm"),
                }
            },
        )
        .manage_farm(
            &another,
            FarmAction::Close {
                farm_identifier: M_FARM_ID_1.to_string(),
            },
            vec![],
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();

                match err {
                    ContractError::Unauthorized => {}
                    _ => panic!("Wrong error type, should return ContractError::Unauthorized"),
                }
            },
        )
        .query_balance(UUSDY_DENOM.to_string(), &other, |balance| {
            assert_eq!(balance, Uint128::new(INITIAL_BALANCE - FARM_AMOUNT_4K));
        })
        .manage_farm(
            &other,
            FarmAction::Close {
                farm_identifier: M_FARM_ID_1.to_string(),
            },
            vec![],
            |result| {
                result.unwrap();
            },
        )
        .query_balance(UUSDY_DENOM.to_string(), &other, |balance| {
            assert_eq!(balance, Uint128::new(INITIAL_BALANCE));
        });

    // open new farm
    suite
        .query_current_epoch(|result| {
            let epoch_response = result.unwrap();
            assert_eq!(epoch_response.epoch.id, 10);
        })
        .manage_position(
            &another,
            PositionAction::Create {
                identifier: None,
                unlocking_duration: 86_400,
                receiver: None,
            },
            vec![coin(UOM_FARM_CREATION_FEE, lp_denom_2.clone())],
            |result| {
                result.unwrap();
            },
        )
        .manage_farm(
            &other,
            FarmAction::Fill {
                params: FarmParams {
                    lp_denom: lp_denom_2.clone(),
                    start_epoch: START_EPOCH_12,
                    preliminary_end_epoch: END_EPOCH_13,
                    curve: None,
                    farm_asset: Coin {
                        denom: UUSDY_DENOM.to_string(),
                        amount: Uint128::new(FARM_AMOUNT_4K),
                    },
                    farm_identifier: Some(FARM_ID_X.to_string()),
                },
            },
            vec![
                coin(FARM_AMOUNT_4K, UUSDY_DENOM),
                coin(UOM_FARM_CREATION_FEE, UOM_DENOM),
            ],
            |result| {
                result.unwrap();
            },
        );

    for _ in 0..=2 {
        suite.add_one_epoch();
    }

    suite.query_current_epoch(|result| {
        let epoch_response = result.unwrap();
        assert_eq!(epoch_response.epoch.id, 13);
    });

    suite
        .query_balance(UUSDY_DENOM.to_string(), &another, |balance| {
            assert_eq!(balance, Uint128::new(INITIAL_BALANCE));
        })
        .claim(&another, vec![], None, |result| {
            result.unwrap();
        })
        .query_farms(
            Some(FarmsBy::Identifier(M_FARM_ID_X.to_string())),
            None,
            None,
            |result| {
                let farms_response = result.unwrap();
                let farm = farms_response.farms[0].clone();
                assert_eq!(
                    farm.farm_asset,
                    Coin {
                        denom: UUSDY_DENOM.to_string(),
                        amount: Uint128::new(FARM_AMOUNT_4K),
                    }
                );
                // the farm is empty
                assert_eq!(farm.claimed_amount, Uint128::new(FARM_AMOUNT_4K),);

                assert_eq!(farm.preliminary_end_epoch, 13);
                assert_eq!(farm.start_epoch, 12);
            },
        )
        .query_balance(UUSDY_DENOM.to_string(), &another, |balance| {
            assert_eq!(balance, Uint128::new(INITIAL_BALANCE + FARM_AMOUNT_4K));
        })
        .manage_farm(
            &other,
            FarmAction::Close {
                farm_identifier: M_FARM_ID_X.to_string(),
            },
            vec![],
            |result| {
                result.unwrap();
            },
        );
}

/// This test recreates the scenario where a malicious TF token freezes token transfers via hooks,
/// which would brick the rewards claiming mechanism and prevent closing the farm (in case the contract owner
/// would like to salvage the contract).
#[test]
#[allow(clippy::inconsistent_digit_grouping)]
fn close_farms_wont_fail_with_malicious_tf_token() {
    let lp_denom = format!("factory/{MOCK_CONTRACT_ADDR_1}/{LP_SYMBOL}").to_string();
    let lp_denom_2 = format!("factory/{MOCK_CONTRACT_ADDR_1}/2.{LP_SYMBOL}").to_string();

    let mut suite = TestingSuite::default_with_balances(vec![
        coin(INITIAL_BALANCE, UOM_DENOM.to_string()),
        coin(INITIAL_BALANCE, UUSDY_DENOM.to_string()),
        coin(INITIAL_BALANCE, UOSMO_DENOM.to_string()),
        coin(INITIAL_BALANCE, lp_denom.clone()),
        coin(INITIAL_BALANCE, lp_denom_2.clone()),
    ]);

    suite.instantiate_default();

    let other = suite.senders[1].clone();

    // create two farms
    suite
        .manage_farm(
            &other,
            FarmAction::Fill {
                params: FarmParams {
                    lp_denom: lp_denom.clone(),
                    start_epoch: None,
                    preliminary_end_epoch: None,
                    curve: None,
                    farm_asset: Coin {
                        denom: UUSDY_DENOM.to_string(),
                        amount: Uint128::new(FARM_AMOUNT_4K),
                    },
                    farm_identifier: Some(FARM_ID_1.to_string()),
                },
            },
            vec![
                coin(FARM_AMOUNT_4K, UUSDY_DENOM),
                coin(UOM_FARM_CREATION_FEE, UOM_DENOM),
            ],
            |result| {
                result.unwrap();
            },
        )
        .manage_farm(
            &other,
            FarmAction::Fill {
                params: FarmParams {
                    lp_denom: lp_denom_2.clone(),
                    start_epoch: None,
                    preliminary_end_epoch: None,
                    curve: None,
                    farm_asset: Coin {
                        denom: UOSMO_DENOM.to_string(),
                        amount: Uint128::new(FARM_AMOUNT_4K),
                    },
                    farm_identifier: Some(FARM_ID_2.to_string()),
                },
            },
            vec![
                coin(FARM_AMOUNT_4K, UOSMO_DENOM),
                coin(UOM_FARM_CREATION_FEE, UOM_DENOM),
            ],
            |result| {
                result.unwrap();
            },
        );

    let farm_manager = suite.farm_manager_addr.clone();

    suite
        .query_balance(UUSDY_DENOM.to_string(), &farm_manager, |balance| {
            assert_eq!(balance, Uint128::new(FARM_AMOUNT_4K));
        })
        .query_balance(UOSMO_DENOM.to_string(), &farm_manager, |balance| {
            assert_eq!(balance, Uint128::new(FARM_AMOUNT_4K));
        });

    // let's burn tokens from the contract to simulate the case where a malicious TF token freezes
    // token transfers
    suite
        .burn_tokens(
            &farm_manager,
            coins(UOM_FARM_CREATION_FEE, UOSMO_DENOM),
            |result| {
                result.unwrap();
            },
        )
        .query_balance(UOSMO_DENOM.to_string(), &farm_manager, |balance| {
            assert_eq!(
                balance,
                Uint128::new(FARM_AMOUNT_4K - UOM_FARM_CREATION_FEE)
            );
        });

    // closing the farm would have failed and bricked the rewards claiming, but not anymore
    suite.manage_farm(
        &other,
        FarmAction::Close {
            farm_identifier: M_FARM_ID_2.to_string(),
        },
        vec![],
        |result| {
            assert!(result.unwrap().events.iter().any(|event| {
                event
                    .attributes
                    .iter()
                    .any(|attr| attr.key == "reason" && !attr.value.is_empty())
            }));
        },
    );

    suite.query_farms(
        Some(FarmsBy::Identifier(M_FARM_ID_2.to_string())),
        None,
        None,
        |result| {
            let err = result.unwrap_err();
            assert!(err.to_string().contains("Farm doesn't exist"));
        },
    );

    suite.query_farms(None, None, None, |result| {
        let farms_response = result.unwrap();
        assert_eq!(farms_response.farms.len(), 1);
        assert_eq!(farms_response.farms[0].identifier, M_FARM_ID_1);
    });
}

#[test]
fn test_farm_helper() {
    let lp_denom = format!("factory/{MOCK_CONTRACT_ADDR_1}/{LP_SYMBOL}").to_string();

    let mut suite = TestingSuite::default_with_balances(vec![
        coin(INITIAL_BALANCE, UOM_DENOM.to_string()),
        coin(INITIAL_BALANCE, UUSDY_DENOM.to_string()),
        coin(INITIAL_BALANCE, UOSMO_DENOM.to_string()),
        coin(INITIAL_BALANCE, lp_denom.clone()),
    ]);

    let creator = suite.creator();
    let other = suite.senders[1].clone();

    suite.instantiate_default();

    let farm_manager = suite.farm_manager_addr.clone();
    let fee_collector = suite.fee_collector_addr.clone();

    suite
        .manage_farm(
            &creator,
            FarmAction::Fill {
                params: FarmParams {
                    lp_denom: lp_denom.clone(),
                    start_epoch: None,
                    preliminary_end_epoch: None,
                    curve: None,
                    farm_asset: Coin {
                        denom: UOM_DENOM.to_string(),
                        amount: Uint128::new(FARM_AMOUNT_4K),
                    },
                    farm_identifier: Some("farm".to_string()),
                },
            },
            vec![coin(EXCESS_FARM_FEE, UOM_DENOM)],
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();
                match err {
                    ContractError::AssetMismatch => {}
                    _ => {
                        panic!("Wrong error type, should return ContractError::AssetMismatch")
                    }
                }
            },
        )
        .query_balance(UOM_DENOM.to_string(), &creator, |balance| {
            assert_eq!(balance, Uint128::new(INITIAL_BALANCE));
        })
        .query_balance(UOM_DENOM.to_string(), &fee_collector, |balance| {
            assert_eq!(balance, Uint128::zero());
        })
        .query_balance(UOM_DENOM.to_string(), &farm_manager, |balance| {
            assert_eq!(balance, Uint128::zero());
        })
        .manage_farm(
            &creator,
            FarmAction::Fill {
                params: FarmParams {
                    lp_denom: lp_denom.clone(),
                    start_epoch: None,
                    preliminary_end_epoch: None,
                    curve: None,
                    farm_asset: Coin {
                        denom: UUSDY_DENOM.to_string(),
                        amount: Uint128::new(FARM_AMOUNT_2K),
                    },
                    farm_identifier: Some("farm".to_string()),
                },
            },
            vec![
                coin(FARM_AMOUNT_2K, UUSDY_DENOM),
                coin(EXCESS_FARM_FEE, UOM_DENOM),
            ],
            |result| {
                result.unwrap();
            },
        )
        .query_balance(UOM_DENOM.to_string(), &fee_collector, |balance| {
            assert_eq!(balance, Uint128::new(UOM_FARM_CREATION_FEE));
        })
        .query_balance(UOM_DENOM.to_string(), &farm_manager, |balance| {
            assert_eq!(balance, Uint128::zero());
        })
        .query_balance(UOM_DENOM.to_string(), &creator, |balance| {
            // got the excess of whale back
            assert_eq!(
                balance,
                Uint128::new(INITIAL_BALANCE - UOM_FARM_CREATION_FEE)
            );
        });

    suite.manage_farm(
        &other,
        FarmAction::Fill {
            params: FarmParams {
                lp_denom: lp_denom.clone(),
                start_epoch: None,
                preliminary_end_epoch: None,
                curve: None,
                farm_asset: Coin {
                    denom: UUSDY_DENOM.to_string(),
                    amount: Uint128::new(FARM_AMOUNT_2K),
                },
                farm_identifier: Some("underpaid_farm".to_string()),
            },
        },
        vec![coin(FARM_AMOUNT_2K, UUSDY_DENOM), coin(500u128, UOM_DENOM)],
        |result| {
            let err = result.unwrap_err().downcast::<ContractError>().unwrap();
            match err {
                ContractError::FarmFeeNotPaid { .. } => {}
                _ => {
                    panic!("Wrong error type, should return ContractError::FarmFeeNotPaid")
                }
            }
        },
    );
}

#[test]
fn fails_to_create_farm_if_more_tokens_than_needed_were_sent() {
    let lp_denom = format!("factory/{MOCK_CONTRACT_ADDR_1}/{LP_SYMBOL}").to_string();
    let mut suite = TestingSuite::default_with_balances(vec![
        coin(INITIAL_BALANCE, UOM_DENOM),
        coin(INITIAL_BALANCE, UUSDY_DENOM),
        coin(INITIAL_BALANCE, UOSMO_DENOM),
        coin(INITIAL_BALANCE, lp_denom.clone()),
        coin(INITIAL_BALANCE, INVALID_LP),
    ]);
    let creator = suite.creator();

    suite.instantiate_default();

    suite
        .manage_farm(
            &creator,
            FarmAction::Fill {
                params: FarmParams {
                    lp_denom: lp_denom.clone(),
                    start_epoch: None,
                    preliminary_end_epoch: None,
                    curve: None,
                    farm_asset: Coin {
                        denom: UUSDY_DENOM.to_string(),
                        amount: Uint128::new(FARM_AMOUNT_4K),
                    },
                    farm_identifier: None,
                },
            },
            vec![
                coin(FARM_AMOUNT_4K, UUSDY_DENOM),
                coin(UOM_FARM_CREATION_FEE, UOM_DENOM),
                coin(UOM_FARM_CREATION_FEE, UOSMO_DENOM),
            ],
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();
                match err {
                    ContractError::AssetMismatch => {}
                    _ => {
                        panic!("Wrong error type, should return ContractError::AssetMismatch")
                    }
                }
            },
        )
        .manage_farm(
            &creator,
            FarmAction::Fill {
                params: FarmParams {
                    lp_denom: lp_denom.clone(),
                    start_epoch: None,
                    preliminary_end_epoch: None,
                    curve: None,
                    farm_asset: Coin {
                        denom: UOM_DENOM.to_string(),
                        amount: Uint128::new(FARM_AMOUNT_4K),
                    },
                    farm_identifier: None,
                },
            },
            vec![
                coin(FARM_AMOUNT_5K, UOM_DENOM),
                coin(UOM_FARM_CREATION_FEE, UOSMO_DENOM),
            ],
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();
                match err {
                    ContractError::AssetMismatch => {}
                    _ => {
                        panic!("Wrong error type, should return ContractError::AssetMismatch")
                    }
                }
            },
        )
        .manage_farm(
            &creator,
            FarmAction::Fill {
                params: FarmParams {
                    lp_denom: lp_denom.clone(),
                    start_epoch: START_EPOCH_12,
                    preliminary_end_epoch: END_EPOCH_16,
                    curve: None,
                    farm_asset: Coin {
                        denom: UUSDY_DENOM.to_string(),
                        amount: Uint128::new(FARM_AMOUNT_8K),
                    },
                    farm_identifier: None,
                },
            },
            vec![
                coin(FARM_AMOUNT_8K, UUSDY_DENOM),
                coin(UOM_FARM_CREATION_FEE, UOM_DENOM),
            ],
            |result| {
                result.unwrap();
            },
        )
        .manage_farm(
            &creator,
            FarmAction::Fill {
                params: FarmParams {
                    lp_denom: lp_denom.clone(),
                    start_epoch: START_EPOCH_12,
                    preliminary_end_epoch: END_EPOCH_16,
                    curve: None,
                    farm_asset: Coin {
                        denom: UOM_DENOM.to_string(),
                        amount: Uint128::new(FARM_AMOUNT_8K),
                    },
                    farm_identifier: None,
                },
            },
            vec![coin(9_000, UOM_DENOM)], // 8000 farm asset + 1000 fee
            |result| {
                result.unwrap();
            },
        );
}

#[test]
fn fails_to_create_farm_if_start_epoch_is_zero() {
    let lp_denom = format!("factory/{MOCK_CONTRACT_ADDR_1}/{LP_SYMBOL}").to_string();
    let mut suite = TestingSuite::default_with_balances(vec![
        coin(INITIAL_BALANCE, UOM_DENOM),
        coin(INITIAL_BALANCE, UUSDY_DENOM),
        coin(INITIAL_BALANCE, UOSMO_DENOM),
        coin(INITIAL_BALANCE, lp_denom.clone()),
        coin(INITIAL_BALANCE, INVALID_LP),
    ]);
    let creator = suite.creator();

    suite.instantiate_default();

    suite.manage_farm(
        &creator,
        FarmAction::Fill {
            params: FarmParams {
                lp_denom: lp_denom.clone(),
                start_epoch: START_EPOCH_0,
                preliminary_end_epoch: END_EPOCH_28,
                curve: None,
                farm_asset: Coin {
                    denom: UUSDY_DENOM.to_string(),
                    amount: Uint128::new(FARM_AMOUNT_4K),
                },
                farm_identifier: Some("farm_1".to_string()),
            },
        },
        vec![
            coin(FARM_AMOUNT_4K, UUSDY_DENOM),
            coin(UOM_FARM_CREATION_FEE, UOM_DENOM),
        ],
        |result| {
            let err = result.unwrap_err().downcast::<ContractError>().unwrap();
            match err {
                ContractError::InvalidEpoch { which } => {
                    assert_eq!(which, "start".to_string())
                }
                _ => {
                    panic!("Wrong error type, should return ContractError::InvalidEpoch")
                }
            }
        },
    );
}

#[test]
fn overriding_farm_with_bogus_id_not_possible() {
    let lp_denom = format!("factory/{MOCK_CONTRACT_ADDR_1}/{LP_SYMBOL}").to_string();
    let mut suite = TestingSuite::default_with_balances(vec![
        coin(INITIAL_BALANCE, UOM_DENOM),
        coin(INITIAL_BALANCE, UUSDY_DENOM),
        coin(INITIAL_BALANCE, UOSMO_DENOM),
        coin(INITIAL_BALANCE, lp_denom.clone()),
        coin(INITIAL_BALANCE, INVALID_LP),
    ]);
    let creator = suite.creator();

    suite.instantiate_default();

    suite
        .manage_farm(
            &creator,
            FarmAction::Fill {
                params: FarmParams {
                    lp_denom: lp_denom.clone(),
                    start_epoch: None,
                    preliminary_end_epoch: None,
                    curve: None,
                    farm_asset: Coin {
                        denom: UUSDY_DENOM.to_string(),
                        amount: Uint128::new(FARM_AMOUNT_4K),
                    },
                    farm_identifier: Some(BOGUS_ID_1.to_string()),
                },
            },
            vec![
                coin(FARM_AMOUNT_4K, UUSDY_DENOM),
                coin(UOM_FARM_CREATION_FEE, UOM_DENOM),
            ],
            |result| {
                result.unwrap();
            },
        )
        .manage_farm(
            &creator,
            FarmAction::Fill {
                params: FarmParams {
                    lp_denom: lp_denom.clone(),
                    start_epoch: None,
                    preliminary_end_epoch: None,
                    curve: None,
                    farm_asset: Coin {
                        denom: UUSDY_DENOM.to_string(),
                        amount: Uint128::new(FARM_AMOUNT_4K),
                    },
                    farm_identifier: None,
                },
            },
            vec![
                coin(FARM_AMOUNT_4K, UUSDY_DENOM),
                coin(UOM_FARM_CREATION_FEE, UOM_DENOM),
            ],
            |result| {
                result.unwrap();
            },
        )
        .query_farms(None, None, None, |result| {
            let farms_response = result.unwrap();
            assert_eq!(farms_response.farms.len(), 2);
            assert_eq!(farms_response.farms[0].identifier, F_1_ID);
            assert_eq!(
                farms_response.farms[1].identifier,
                format!("m-{}", BOGUS_ID_1)
            );
        });
}

#[test]
fn providing_custom_farm_id_doesnt_increment_farm_counter() {
    let lp_denom = format!("factory/{MOCK_CONTRACT_ADDR_1}/{LP_SYMBOL}").to_string();
    let mut suite = TestingSuite::default_with_balances(vec![
        coin(INITIAL_BALANCE, UOM_DENOM),
        coin(INITIAL_BALANCE, UUSDY_DENOM),
        coin(INITIAL_BALANCE, UOSMO_DENOM),
        coin(INITIAL_BALANCE, lp_denom.clone()),
        coin(INITIAL_BALANCE, INVALID_LP),
    ]);
    let creator = suite.creator();

    suite.instantiate_default();

    suite
        .manage_farm(
            &creator,
            FarmAction::Fill {
                params: FarmParams {
                    lp_denom: lp_denom.clone(),
                    start_epoch: START_EPOCH_12,
                    preliminary_end_epoch: END_EPOCH_16,
                    curve: None,
                    farm_asset: Coin {
                        denom: UOM_DENOM.to_string(),
                        amount: Uint128::new(FARM_AMOUNT_8K),
                    },
                    farm_identifier: Some(CUSTOM_ID_1.to_string()),
                },
            },
            vec![coin(9_000, UOM_DENOM)], // 8000 farm asset + 1000 fee
            |result| {
                result.unwrap();
            },
        )
        .manage_farm(
            &creator,
            FarmAction::Fill {
                params: FarmParams {
                    lp_denom: lp_denom.clone(),
                    start_epoch: START_EPOCH_12,
                    preliminary_end_epoch: END_EPOCH_16,
                    curve: None,
                    farm_asset: Coin {
                        denom: UOM_DENOM.to_string(),
                        amount: Uint128::new(FARM_AMOUNT_8K),
                    },
                    farm_identifier: None,
                },
            },
            vec![coin(9_000, UOM_DENOM)], // 8000 farm asset + 1000 fee
            |result| {
                result.unwrap();
            },
        )
        .query_farms(None, None, None, |result| {
            let response = result.unwrap();
            assert_eq!(response.farms.len(), 2);
            assert_eq!(response.farms[0].identifier, F_1_ID);
            assert_eq!(response.farms[1].identifier, M_CUSTOM_ID_1);
        });
}

#[test]
fn farm_cant_be_created_in_the_past() {
    let lp_denom = format!("factory/{MOCK_CONTRACT_ADDR_1}/{LP_SYMBOL}").to_string();
    let invalid_lp_denom = format!("factory/{MOCK_CONTRACT_ADDR_2}/{LP_SYMBOL}").to_string();

    let mut suite = TestingSuite::default_with_balances(vec![
        coin(INITIAL_BALANCE, UOM_DENOM.to_string()),
        coin(INITIAL_BALANCE, UUSDY_DENOM.to_string()),
        coin(INITIAL_BALANCE, UOSMO_DENOM.to_string()),
        coin(INITIAL_BALANCE, lp_denom.clone()),
        coin(INITIAL_BALANCE, invalid_lp_denom.clone()),
    ]);
    suite.instantiate_default();

    let other = suite.senders[1].clone();

    for _ in 0..10 {
        suite.add_one_epoch();
    }
    // current epoch is 10

    // We can create a farm in a past epoch
    suite.manage_farm(
        &other,
        FarmAction::Fill {
            params: FarmParams {
                lp_denom: lp_denom.clone(),
                start_epoch: START_EPOCH_1, // start epoch in the past
                preliminary_end_epoch: END_EPOCH_28,
                curve: None,
                farm_asset: Coin {
                    denom: UUSDY_DENOM.to_string(),
                    amount: Uint128::new(FARM_AMOUNT_4K),
                },
                farm_identifier: Some("farm_1".to_string()),
            },
        },
        vec![
            coin(FARM_AMOUNT_4K, UUSDY_DENOM),
            coin(UOM_FARM_CREATION_FEE, UOM_DENOM),
        ],
        |result| {
            let err = result.unwrap_err().downcast::<ContractError>().unwrap();

            match err {
                ContractError::InvalidEpoch { which } => {
                    assert_eq!(which, "start")
                }
                _ => panic!("Wrong error type, should return ContractError::InvalidEpoch"),
            }
        },
    );
}
