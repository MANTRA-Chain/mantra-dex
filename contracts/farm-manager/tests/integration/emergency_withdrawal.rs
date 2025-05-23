use cosmwasm_std::{coin, Coin, Uint128};
use mantra_dex_std::constants::LP_SYMBOL;
use mantra_dex_std::farm_manager::{
    FarmAction, FarmParams, Position, PositionAction, PositionsBy, RewardsResponse,
};
use test_utils::common_constants::{
    DEFAULT_UNLOCKING_DURATION_SECONDS, DENOM_UOM as UOM_DENOM, DENOM_UOSMO as UOSMO_DENOM,
    DENOM_UUSDY as UUSDY_DENOM, INITIAL_BALANCE, UOM_FARM_CREATION_FEE,
};

use crate::common::suite::TestingSuite;
use crate::common::MOCK_CONTRACT_ADDR_1;

// Global constants for the test file
const INITIAL_BALANCE_LARGE: u128 = 1_000_000_000_000u128; // Used in test_managing_positions_close_and_emergency_withdraw

const YEAR_APPROX_UNLOCKING_DURATION_SECONDS: u64 = 31_556_926; // Used in test_emergency_withdrawal_with_proportional_penalty

// Constants for test_emergency_withdrawal & test_emergency_withdrawal_with_pending_rewards_are_lost
const FARM_ID_EW: &str = "farm";
const FARM_ASSET_UUSDY_EW: u128 = 4_000u128;
const LP_LOCK_AMOUNT_EW: u128 = 1_000u128;
const OTHER_POS_RAW_ID_EW: &str = "other_position";
const OTHER_POS_PREFIXED_ID_EW: &str = "u-other_position";

// Constants for emergency_withdrawal_shares_penalty_with_active_farm_owners
const FARM_ID_2_EWSP: &str = "farm_2";
const BOB_POS_RAW_ID_EWSP: &str = "bob_position";
const BOB_POS_PREFIXED_ID_EWSP: &str = "u-bob_position";
const LP_LOCK_AMOUNT_BOB_EWSP: u128 = 6_000_000u128;

// Constants for test_emergency_withdrawal_with_proportional_penalty
const FARM_ID_2_EWPP: &str = "farm2"; // Note: "farm2" not "farm_2"
const OTHER_POS_MAX_RAW_ID_EWPP: &str = "other_position_max";
const OTHER_POS_MAX_PREFIXED_ID_EWPP: &str = "u-other_position_max";

// Constants for test_emergency_withdrawal_penalty_only_to_active_farms
const EPOCH_ID_1: u64 = 1;
const EPOCH_ID_2: u64 = 2;
const EPOCH_ID_34: u64 = 34;
const EPOCH_ID_35: u64 = 35;
const EPOCH_ID_36: u64 = 36;
const FARM_1_ID_EWPOAF: &str = "farm-1";
const FARM_2_ID_EWPOAF: &str = "farm-2";
const FARM_3_ID_EWPOAF: &str = "farm-3";
const FARM_4_ID_EWPOAF: &str = "farm-4";
const POS_1_RAW_ID_EWPOAF: &str = "position-1";
const POS_1_PREFIXED_ID_EWPOAF: &str = "u-position-1";
const POS_2_RAW_ID_EWPOAF: &str = "position-2";
const POS_2_PREFIXED_ID_EWPOAF: &str = "u-position-2";
const LP_LOCK_AMOUNT_EWPOAF: u128 = 1_000u128;
const FARM_ASSET_UUSDY_EWPOAF: u128 = 4_000u128;

// Constants for can_emergency_withdraw_an_lp_without_farm
const FARM_ASSET_UUSDY_CEW: u128 = 8_000u128;
const CREATOR_POS_RAW_ID_CEW: &str = "creator_position";
const CREATOR_POS_PREFIXED_ID_CEW: &str = "u-creator_position";
const LP_LOCK_AMOUNT_CREATOR_CEW: u128 = 2_000u128;
const EPOCH_ID_6: u64 = 6;

// Constants for test_managing_positions_close_and_emergency_withdraw (MPCEW)
const FARM_ASSET_UUSDY_ALICE_FARM1_MPCEW: u128 = 8_888u128;
const FARM_ASSET_UUSDY_ALICE_FARM2_MPCEW: u128 = 666_666u128;
const FARM_ASSET_UOSMO_ALICE_FARM3_MPCEW: u128 = 8_888u128;
const FARM_ASSET_UOM_ALICE_FARM4_MPCEW: u128 = 1_000_000u128;
const UOM_FARM4_TOTAL_FUNDS_MPCEW: u128 = FARM_ASSET_UOM_ALICE_FARM4_MPCEW + UOM_FARM_CREATION_FEE;

const ALICE_POS_1_RAW_ID_MPCEW: &str = "alice_position_1";
const ALICE_POS_1_PREFIXED_ID_MPCEW: &str = "u-alice_position_1";
const LP_LOCK_ALICE_POS_1_MPCEW: u128 = 333u128;

const BOB_POS_1_RAW_ID_MPCEW: &str = "bob_position_1";
const BOB_POS_1_PREFIXED_ID_MPCEW: &str = "u-bob_position_1";
const BOB_POS_2_RAW_ID_MPCEW: &str = "bob_position_2";
// const BOB_POS_2_PREFIXED_ID_MPCEW: &str = "u-bob_position_2"; // Not used explicitly with this constant name
const LP_LOCK_BOB_POS_1_AND_2_MPCEW: u128 = 666u128;

const CAROL_POS_2_RAW_ID_MPCEW: &str = "carol_position_2";
const LP_LOCK_CAROL_POS_2_MPCEW: u128 = 1_000u128;

const ALICE_SECOND_POS_1_RAW_ID_MPCEW: &str = "alice_second_position_1";
const ALICE_SECOND_POS_1_PREFIXED_ID_MPCEW: &str = "u-alice_second_position_1";
const ALICE_SECOND_POS_2_RAW_ID_MPCEW: &str = "alice_second_position_2";
const ALICE_SECOND_POS_2_PREFIXED_ID_MPCEW: &str = "u-alice_second_position_2";
const LP_LOCK_ALICE_SECOND_POS_1_MPCEW: u128 = 300u128;
const LP_LOCK_ALICE_SECOND_POS_2_MPCEW: u128 = 700u128;

const PENDING_WITHDRAW_POS_ID_MPCEW: &str = "p-1";
const FINAL_ALICE_POS_RAW_ID_MPCEW: &str = "final_alice_position";
const LP_LOCK_FINAL_ALICE_POS_MPCEW: u128 = 3_000u128;

const NEW_BOB_POS_LP1_RAW_ID_MPCEW: &str = "new_bob_position_lp_1";
const LP_LOCK_NEW_BOB_POS_LP1_MPCEW: u128 = 1_000u128;

const EPOCH_ID_5_MPCEW: u64 = 5;
const EPOCH_ID_8_MPCEW: u64 = 8;
const EPOCH_ID_9_MPCEW: u64 = 9;
const EPOCH_ID_10_MPCEW: u64 = 10;
const EPOCH_ID_12_MPCEW: u64 = 12;
const EPOCH_ID_13_MPCEW: u64 = 13;
const EPOCH_ID_14_MPCEW: u64 = 14;
const EPOCH_ID_15_MPCEW: u64 = 15;
const EPOCH_ID_18_MPCEW: u64 = 18;
const EPOCH_ID_20_MPCEW: u64 = 20;
const EPOCH_ID_22_MPCEW: u64 = 22;

#[test]
fn test_emergency_withdrawal() {
    let lp_denom = format!("factory/{MOCK_CONTRACT_ADDR_1}/{LP_SYMBOL}").to_string();

    let mut suite = TestingSuite::default_with_balances(vec![
        coin(INITIAL_BALANCE, UOM_DENOM.to_string()),
        coin(INITIAL_BALANCE, UUSDY_DENOM.to_string()),
        coin(INITIAL_BALANCE, UOSMO_DENOM.to_string()),
        coin(INITIAL_BALANCE, lp_denom.clone()),
    ]);

    let other = suite.senders[1].clone();

    suite.instantiate_default();

    let fee_collector = suite.fee_collector_addr.clone();

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
                        amount: Uint128::new(FARM_ASSET_UUSDY_EW),
                    },
                    farm_identifier: Some(FARM_ID_EW.to_string()),
                },
            },
            vec![
                coin(FARM_ASSET_UUSDY_EW, UUSDY_DENOM),
                coin(UOM_FARM_CREATION_FEE, UOM_DENOM),
            ],
            |result| {
                result.unwrap();
            },
        )
        .manage_position(
            &other,
            PositionAction::Create {
                identifier: Some(OTHER_POS_RAW_ID_EW.to_string()),
                unlocking_duration: DEFAULT_UNLOCKING_DURATION_SECONDS,
                receiver: None,
            },
            vec![coin(LP_LOCK_AMOUNT_EW, lp_denom.clone())],
            |result| {
                result.unwrap();
            },
        )
        .query_positions(
            Some(PositionsBy::Receiver(other.to_string())),
            Some(true),
            None,
            None,
            |result| {
                let positions = result.unwrap();
                assert_eq!(positions.positions.len(), 1);
                assert_eq!(
                    positions.positions[0],
                    Position {
                        identifier: OTHER_POS_PREFIXED_ID_EW.to_string(),
                        lp_asset: Coin {
                            denom: format!("factory/{MOCK_CONTRACT_ADDR_1}/{LP_SYMBOL}")
                                .to_string(),
                            amount: Uint128::new(LP_LOCK_AMOUNT_EW),
                        },
                        unlocking_duration: DEFAULT_UNLOCKING_DURATION_SECONDS,
                        open: true,
                        expiring_at: None,
                        receiver: other.clone(),
                    }
                );
            },
        );

    suite.add_one_epoch();

    suite
        .query_balance(lp_denom.clone().to_string(), &other, |balance| {
            assert_eq!(balance, Uint128::new(INITIAL_BALANCE - LP_LOCK_AMOUNT_EW));
        })
        .query_balance(lp_denom.clone().to_string(), &fee_collector, |balance| {
            assert_eq!(balance, Uint128::zero());
        })
        .manage_position(
            &other,
            PositionAction::Withdraw {
                identifier: OTHER_POS_PREFIXED_ID_EW.to_string(),
                emergency_unlock: Some(true),
            },
            vec![],
            |result| {
                result.unwrap();
            },
        )
        .query_balance(lp_denom.clone().to_string(), &other, |balance| {
            //emergency unlock penalty is 10% of the position amount, so the user gets 1000 - 100 = 900 + 50
            // (as he was the owner of the farm, he got 50% of the penalty fee`
            assert_eq!(balance, Uint128::new(999_999_950)); // Derived value
        })
        .query_balance(lp_denom.clone().to_string(), &fee_collector, |balance| {
            assert_eq!(balance, Uint128::new(50)); // Derived value
        });
}

#[test]
fn test_emergency_withdrawal_with_pending_rewards_are_lost() {
    let lp_denom = format!("factory/{MOCK_CONTRACT_ADDR_1}/{LP_SYMBOL}").to_string();

    let mut suite = TestingSuite::default_with_balances(vec![
        coin(INITIAL_BALANCE, UOM_DENOM.to_string()),
        coin(INITIAL_BALANCE, UUSDY_DENOM.to_string()),
        coin(INITIAL_BALANCE, UOSMO_DENOM.to_string()),
        coin(INITIAL_BALANCE, lp_denom.clone()),
    ]);

    let other = suite.senders[1].clone();

    suite.instantiate_default();

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
                        amount: Uint128::new(FARM_ASSET_UUSDY_EW),
                    },
                    farm_identifier: Some(FARM_ID_EW.to_string()),
                },
            },
            vec![
                coin(FARM_ASSET_UUSDY_EW, UUSDY_DENOM),
                coin(UOM_FARM_CREATION_FEE, UOM_DENOM),
            ],
            |result| {
                result.unwrap();
            },
        )
        .manage_position(
            &other,
            PositionAction::Create {
                identifier: Some(OTHER_POS_RAW_ID_EW.to_string()),
                unlocking_duration: DEFAULT_UNLOCKING_DURATION_SECONDS,
                receiver: None,
            },
            vec![coin(LP_LOCK_AMOUNT_EW, lp_denom.clone())],
            |result| {
                result.unwrap();
            },
        )
        .query_positions(
            Some(PositionsBy::Receiver(other.to_string())),
            Some(true),
            None,
            None,
            |result| {
                let positions = result.unwrap();
                assert_eq!(positions.positions.len(), 1);
                assert_eq!(
                    positions.positions[0],
                    Position {
                        identifier: OTHER_POS_PREFIXED_ID_EW.to_string(),
                        lp_asset: Coin {
                            denom: format!("factory/{MOCK_CONTRACT_ADDR_1}/{LP_SYMBOL}")
                                .to_string(),
                            amount: Uint128::new(LP_LOCK_AMOUNT_EW),
                        },
                        unlocking_duration: DEFAULT_UNLOCKING_DURATION_SECONDS,
                        open: true,
                        expiring_at: None,
                        receiver: other.clone(),
                    }
                );
            },
        )
        .add_epochs(3)
        // rewards are pending to be claimed
        .query_rewards(&other, None, |result| {
            let response = result.unwrap();

            match response {
                RewardsResponse::RewardsResponse { total_rewards, .. } => {
                    assert_eq!(total_rewards.len(), 1);
                    assert_eq!(total_rewards[0], coin(855, UUSDY_DENOM)); // Derived value
                }
                _ => panic!("shouldn't return this but RewardsResponse"),
            }
        })
        .query_balance(UUSDY_DENOM.to_string(), &other, |balance| {
            assert_eq!(balance, Uint128::new(INITIAL_BALANCE - FARM_ASSET_UUSDY_EW));
        })
        // the user emergency withdraws the position
        .manage_position(
            &other,
            PositionAction::Withdraw {
                identifier: OTHER_POS_PREFIXED_ID_EW.to_string(),
                emergency_unlock: Some(true),
            },
            vec![],
            |result| {
                result.unwrap();
            },
        )
        // rewards were not claimed
        .query_balance(UUSDY_DENOM.to_string(), &other, |balance| {
            assert_eq!(balance, Uint128::new(INITIAL_BALANCE - FARM_ASSET_UUSDY_EW));
        });
}

#[test]
fn emergency_withdrawal_shares_penalty_with_active_farm_owners() {
    let lp_denom = format!("factory/{MOCK_CONTRACT_ADDR_1}/{LP_SYMBOL}").to_string();

    let mut suite = TestingSuite::default_with_balances(vec![
        coin(INITIAL_BALANCE, UOM_DENOM.to_string()),
        coin(INITIAL_BALANCE, UUSDY_DENOM.to_string()),
        coin(INITIAL_BALANCE, UOSMO_DENOM.to_string()),
        coin(INITIAL_BALANCE, lp_denom.clone()),
    ]);

    let other = suite.senders[0].clone();
    let alice = suite.senders[1].clone();
    let bob = suite.senders[2].clone();

    suite.instantiate_default();

    let fee_collector = suite.fee_collector_addr.clone();
    let farm_manager = suite.farm_manager_addr.clone();

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
                        amount: Uint128::new(FARM_ASSET_UUSDY_EW),
                    },
                    farm_identifier: Some(FARM_ID_EW.to_string()),
                },
            },
            vec![
                coin(FARM_ASSET_UUSDY_EW, UUSDY_DENOM),
                coin(UOM_FARM_CREATION_FEE, UOM_DENOM),
            ],
            |result| {
                result.unwrap();
            },
        )
        .manage_farm(
            &alice,
            FarmAction::Fill {
                params: FarmParams {
                    lp_denom: lp_denom.clone(),
                    start_epoch: None,
                    preliminary_end_epoch: None,
                    curve: None,
                    farm_asset: Coin {
                        denom: UUSDY_DENOM.to_string(),
                        amount: Uint128::new(FARM_ASSET_UUSDY_EW),
                    },
                    farm_identifier: Some(FARM_ID_2_EWSP.to_string()),
                },
            },
            vec![
                coin(FARM_ASSET_UUSDY_EW, UUSDY_DENOM),
                coin(UOM_FARM_CREATION_FEE, UOM_DENOM),
            ],
            |result| {
                result.unwrap();
            },
        )
        .manage_position(
            &bob,
            PositionAction::Create {
                identifier: Some(BOB_POS_RAW_ID_EWSP.to_string()),
                unlocking_duration: DEFAULT_UNLOCKING_DURATION_SECONDS,
                receiver: None,
            },
            vec![coin(LP_LOCK_AMOUNT_BOB_EWSP, lp_denom.clone())],
            |result| {
                result.unwrap();
            },
        )
        .query_positions(
            Some(PositionsBy::Receiver(bob.to_string())),
            Some(true),
            None,
            None,
            |result| {
                let positions = result.unwrap();
                assert_eq!(positions.positions.len(), 1);
                assert_eq!(
                    positions.positions[0],
                    Position {
                        identifier: BOB_POS_PREFIXED_ID_EWSP.to_string(),
                        lp_asset: Coin {
                            denom: format!("factory/{MOCK_CONTRACT_ADDR_1}/{LP_SYMBOL}")
                                .to_string(),
                            amount: Uint128::new(LP_LOCK_AMOUNT_BOB_EWSP),
                        },
                        unlocking_duration: DEFAULT_UNLOCKING_DURATION_SECONDS,
                        open: true,
                        expiring_at: None,
                        receiver: bob.clone(),
                    }
                );
            },
        )
        .add_one_epoch()
        .query_balance(lp_denom.clone().to_string(), &other, |balance| {
            assert_eq!(balance, Uint128::new(INITIAL_BALANCE));
        })
        .query_balance(lp_denom.clone().to_string(), &fee_collector, |balance| {
            assert_eq!(balance, Uint128::zero());
        })
        .query_balance(lp_denom.clone().to_string(), &alice, |balance| {
            assert_eq!(balance, Uint128::new(INITIAL_BALANCE));
        })
        .query_balance(lp_denom.clone().to_string(), &farm_manager, |balance| {
            assert_eq!(balance, Uint128::new(LP_LOCK_AMOUNT_BOB_EWSP));
        })
        .manage_position(
            &bob,
            PositionAction::Withdraw {
                identifier: BOB_POS_PREFIXED_ID_EWSP.to_string(),
                emergency_unlock: Some(true),
            },
            vec![],
            |result| {
                result.unwrap();
            },
        )
        .query_balance(lp_denom.clone().to_string(), &other, |balance| {
            assert_eq!(balance, Uint128::new(1_000_150_000)); // Derived
        })
        .query_balance(lp_denom.clone().to_string(), &fee_collector, |balance| {
            assert_eq!(balance, Uint128::new(300_000)); // Derived
        })
        .query_balance(lp_denom.clone().to_string(), &alice, |balance| {
            assert_eq!(balance, Uint128::new(1_000_150_000)); // Derived
        })
        .query_balance(lp_denom.clone().to_string(), &farm_manager, |balance| {
            assert_eq!(balance, Uint128::zero());
        });
}

#[test]
fn test_emergency_withdrawal_with_proportional_penalty() {
    let lp_denom = format!("factory/{MOCK_CONTRACT_ADDR_1}/{LP_SYMBOL}").to_string();
    let lp_denom_2 = format!("factory/{MOCK_CONTRACT_ADDR_1}/2.{LP_SYMBOL}").to_string();

    let mut suite = TestingSuite::default_with_balances(vec![
        coin(INITIAL_BALANCE, UOM_DENOM.to_string()),
        coin(INITIAL_BALANCE, UUSDY_DENOM.to_string()),
        coin(INITIAL_BALANCE, UOSMO_DENOM.to_string()),
        coin(INITIAL_BALANCE, lp_denom.clone()),
        coin(INITIAL_BALANCE, lp_denom_2.clone()),
    ]);

    let creator = suite.senders[0].clone();
    let other = suite.senders[1].clone();

    suite.instantiate_default();

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
                        denom: UUSDY_DENOM.to_string(),
                        amount: Uint128::new(FARM_ASSET_UUSDY_EW),
                    },
                    farm_identifier: Some(FARM_ID_EW.to_string()),
                },
            },
            vec![
                coin(FARM_ASSET_UUSDY_EW, UUSDY_DENOM),
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
                    lp_denom: lp_denom_2.clone(),
                    start_epoch: None,
                    preliminary_end_epoch: None,
                    curve: None,
                    farm_asset: Coin {
                        denom: UUSDY_DENOM.to_string(),
                        amount: Uint128::new(FARM_ASSET_UUSDY_EW),
                    },
                    farm_identifier: Some(FARM_ID_2_EWPP.to_string()),
                },
            },
            vec![
                coin(FARM_ASSET_UUSDY_EW, UUSDY_DENOM),
                coin(UOM_FARM_CREATION_FEE, UOM_DENOM),
            ],
            |result| {
                result.unwrap();
            },
        )
        .manage_position(
            &other,
            PositionAction::Create {
                identifier: Some(OTHER_POS_RAW_ID_EW.to_string()),
                unlocking_duration: DEFAULT_UNLOCKING_DURATION_SECONDS,
                receiver: None,
            },
            vec![coin(LP_LOCK_AMOUNT_EW, lp_denom.clone())],
            |result| {
                result.unwrap();
            },
        )
        .manage_position(
            &other,
            PositionAction::Create {
                identifier: Some(OTHER_POS_MAX_RAW_ID_EWPP.to_string()),
                unlocking_duration: YEAR_APPROX_UNLOCKING_DURATION_SECONDS,
                receiver: None,
            },
            vec![coin(LP_LOCK_AMOUNT_EW, lp_denom_2.clone())],
            |result| {
                result.unwrap();
            },
        );

    suite.manage_position(
        &other,
        PositionAction::Close {
            identifier: OTHER_POS_PREFIXED_ID_EW.to_string(),
            lp_asset: None,
        },
        vec![],
        |result| {
            result.unwrap();
        },
    );

    // move half of the unlocking period
    suite
        .add_hours(12)
        .query_balance(lp_denom.clone().to_string(), &other, |balance| {
            assert_eq!(balance, Uint128::new(INITIAL_BALANCE - LP_LOCK_AMOUNT_EW));
        })
        .query_balance(lp_denom.clone().to_string(), &fee_collector, |balance| {
            assert_eq!(balance, Uint128::zero());
        })
        .query_balance(lp_denom.clone().to_string(), &creator, |balance| {
            // The creator of the farm gets a cut
            assert_eq!(balance, Uint128::new(INITIAL_BALANCE));
        })
        .manage_position(
            &other,
            PositionAction::Withdraw {
                identifier: OTHER_POS_PREFIXED_ID_EW.to_string(),
                emergency_unlock: Some(true),
            },
            vec![],
            |result| {
                result.unwrap();
            },
        )
        .query_balance(lp_denom.clone().to_string(), &other, |balance| {
            assert_eq!(balance, Uint128::new(999_999_950)); // Derived
        })
        .query_balance(lp_denom.clone().to_string(), &creator, |balance| {
            // Since the farm is not active, the creator of the farm does not gets a cut
            assert_eq!(balance, Uint128::new(INITIAL_BALANCE));
        })
        .query_balance(lp_denom.clone().to_string(), &fee_collector, |balance| {
            assert_eq!(balance, Uint128::new(50)); // Derived
        });

    suite
        .query_balance(lp_denom_2.clone().to_string(), &other, |balance| {
            assert_eq!(balance, Uint128::new(INITIAL_BALANCE - LP_LOCK_AMOUNT_EW));
        })
        .query_balance(lp_denom_2.clone().to_string(), &fee_collector, |balance| {
            assert_eq!(balance, Uint128::zero());
        })
        .query_balance(lp_denom_2.clone().to_string(), &creator, |balance| {
            // The creator of the farm gets a cut
            assert_eq!(balance, Uint128::new(INITIAL_BALANCE));
        })
        // withdraw all without closing the position, highest penalty
        .manage_position(
            &other,
            PositionAction::Withdraw {
                identifier: OTHER_POS_MAX_PREFIXED_ID_EWPP.to_string(),
                emergency_unlock: Some(true),
            },
            vec![],
            |result| {
                result.unwrap();
            },
        )
        // penalty would be 0.1 * ~1 * ~16 = 1.6. Since that would exceed the max cap, the penalty would be 90%
        .query_balance(lp_denom_2.clone().to_string(), &other, |balance| {
            assert_eq!(balance, Uint128::new(999_999_100)); // Derived
        })
        .query_balance(lp_denom_2.clone().to_string(), &fee_collector, |balance| {
            assert_eq!(balance, Uint128::new(900)); // Derived
        })
        .query_balance(lp_denom_2.clone().to_string(), &creator, |balance| {
            // The creator of the farm does not gets a cut since the farm is not active
            assert_eq!(balance, Uint128::new(INITIAL_BALANCE));
        });
}

/// creates 4 farms:
/// Farm-1 and Farm-2 for LP.1: epoch 1 to 2 and 35 to 36
/// Farm-3 and Farm-4 for LP.2: epoch 1 to 2 and 35 to 36
/// Positions are closed and emergency withdrawal penalty distributed only to active farms
#[test]
fn test_emergency_withdrawal_penalty_only_to_active_farms() {
    let lp_denom_1 = format!("factory/{MOCK_CONTRACT_ADDR_1}/1.{LP_SYMBOL}").to_string();
    let lp_denom_2 = format!("factory/{MOCK_CONTRACT_ADDR_1}/2.{LP_SYMBOL}").to_string();

    let mut suite = TestingSuite::default_with_balances(vec![
        coin(INITIAL_BALANCE, UOM_DENOM.to_string()),
        coin(INITIAL_BALANCE, UUSDY_DENOM.to_string()),
        coin(INITIAL_BALANCE, UOSMO_DENOM.to_string()),
        coin(INITIAL_BALANCE, lp_denom_1.clone()),
        coin(INITIAL_BALANCE, lp_denom_2.clone()),
    ]);

    let creator_1 = suite.senders[0].clone();
    let creator_2 = suite.senders[1].clone();
    let other = suite.senders[2].clone();

    suite.instantiate_long_epoch_buffer();

    let fee_collector = suite.fee_collector_addr.clone();

    suite
        .manage_farm(
            &creator_1,
            FarmAction::Fill {
                params: FarmParams {
                    lp_denom: lp_denom_1.clone(),
                    start_epoch: None,
                    preliminary_end_epoch: Some(EPOCH_ID_2),
                    curve: None,
                    farm_asset: Coin {
                        denom: UUSDY_DENOM.to_string(),
                        amount: Uint128::new(FARM_ASSET_UUSDY_EWPOAF),
                    },
                    farm_identifier: Some(FARM_1_ID_EWPOAF.to_string()),
                },
            },
            vec![
                coin(FARM_ASSET_UUSDY_EWPOAF, UUSDY_DENOM),
                coin(UOM_FARM_CREATION_FEE, UOM_DENOM),
            ],
            |result| {
                result.unwrap();
            },
        )
        .manage_farm(
            &creator_2,
            FarmAction::Fill {
                params: FarmParams {
                    lp_denom: lp_denom_1.clone(),
                    start_epoch: Some(EPOCH_ID_35),
                    preliminary_end_epoch: Some(EPOCH_ID_36),
                    curve: None,
                    farm_asset: Coin {
                        denom: UUSDY_DENOM.to_string(),
                        amount: Uint128::new(FARM_ASSET_UUSDY_EWPOAF),
                    },
                    farm_identifier: Some(FARM_2_ID_EWPOAF.to_string()),
                },
            },
            vec![
                coin(FARM_ASSET_UUSDY_EWPOAF, UUSDY_DENOM),
                coin(UOM_FARM_CREATION_FEE, UOM_DENOM),
            ],
            |result| {
                result.unwrap();
            },
        )
        .manage_farm(
            &creator_1,
            FarmAction::Fill {
                params: FarmParams {
                    lp_denom: lp_denom_2.clone(),
                    start_epoch: None,
                    preliminary_end_epoch: Some(EPOCH_ID_2),
                    curve: None,
                    farm_asset: Coin {
                        denom: UUSDY_DENOM.to_string(),
                        amount: Uint128::new(FARM_ASSET_UUSDY_EWPOAF),
                    },
                    farm_identifier: Some(FARM_3_ID_EWPOAF.to_string()),
                },
            },
            vec![
                coin(FARM_ASSET_UUSDY_EWPOAF, UUSDY_DENOM),
                coin(UOM_FARM_CREATION_FEE, UOM_DENOM),
            ],
            |result| {
                result.unwrap();
            },
        )
        .manage_farm(
            &creator_2,
            FarmAction::Fill {
                params: FarmParams {
                    lp_denom: lp_denom_2.clone(),
                    start_epoch: Some(EPOCH_ID_35),
                    preliminary_end_epoch: Some(EPOCH_ID_36),
                    curve: None,
                    farm_asset: Coin {
                        denom: UUSDY_DENOM.to_string(),
                        amount: Uint128::new(FARM_ASSET_UUSDY_EWPOAF),
                    },
                    farm_identifier: Some(FARM_4_ID_EWPOAF.to_string()),
                },
            },
            vec![
                coin(FARM_ASSET_UUSDY_EWPOAF, UUSDY_DENOM),
                coin(UOM_FARM_CREATION_FEE, UOM_DENOM),
            ],
            |result| {
                result.unwrap();
            },
        )
        .manage_position(
            &other,
            PositionAction::Create {
                identifier: Some(POS_1_RAW_ID_EWPOAF.to_string()),
                unlocking_duration: DEFAULT_UNLOCKING_DURATION_SECONDS,
                receiver: None,
            },
            vec![coin(LP_LOCK_AMOUNT_EWPOAF, lp_denom_1.clone())],
            |result| {
                result.unwrap();
            },
        )
        .manage_position(
            &other,
            PositionAction::Create {
                identifier: Some(POS_2_RAW_ID_EWPOAF.to_string()),
                unlocking_duration: DEFAULT_UNLOCKING_DURATION_SECONDS,
                receiver: None,
            },
            vec![coin(LP_LOCK_AMOUNT_EWPOAF, lp_denom_2.clone())],
            |result| {
                result.unwrap();
            },
        )
        .query_positions(
            Some(PositionsBy::Receiver(other.to_string())),
            Some(true),
            None,
            None,
            |result| {
                let positions = result.unwrap();
                assert_eq!(positions.positions.len(), 2);
                assert_eq!(
                    positions.positions,
                    vec![
                        Position {
                            identifier: POS_1_PREFIXED_ID_EWPOAF.to_string(),
                            lp_asset: Coin {
                                denom: format!("factory/{MOCK_CONTRACT_ADDR_1}/1.{LP_SYMBOL}")
                                    .to_string(),
                                amount: Uint128::new(LP_LOCK_AMOUNT_EWPOAF),
                            },
                            unlocking_duration: DEFAULT_UNLOCKING_DURATION_SECONDS,
                            open: true,
                            expiring_at: None,
                            receiver: other.clone(),
                        },
                        Position {
                            identifier: POS_2_PREFIXED_ID_EWPOAF.to_string(),
                            lp_asset: Coin {
                                denom: format!("factory/{MOCK_CONTRACT_ADDR_1}/2.{LP_SYMBOL}")
                                    .to_string(),
                                amount: Uint128::new(LP_LOCK_AMOUNT_EWPOAF),
                            },
                            unlocking_duration: DEFAULT_UNLOCKING_DURATION_SECONDS,
                            open: true,
                            expiring_at: None,
                            receiver: other.clone(),
                        }
                    ]
                );
            },
        );

    suite.add_one_epoch().query_current_epoch(|result| {
        let epoch_response = result.unwrap();
        assert_eq!(epoch_response.epoch.id, EPOCH_ID_1);
    });

    suite
        .query_balance(lp_denom_1.clone().to_string(), &other, |balance| {
            assert_eq!(
                balance,
                Uint128::new(INITIAL_BALANCE - LP_LOCK_AMOUNT_EWPOAF)
            );
        })
        .query_balance(lp_denom_1.clone().to_string(), &creator_1, |balance| {
            assert_eq!(balance, Uint128::new(INITIAL_BALANCE));
        })
        .query_balance(lp_denom_1.clone().to_string(), &creator_2, |balance| {
            assert_eq!(balance, Uint128::new(INITIAL_BALANCE));
        })
        .query_balance(lp_denom_1.clone().to_string(), &fee_collector, |balance| {
            assert_eq!(balance, Uint128::zero());
        })
        .manage_position(
            &other,
            PositionAction::Withdraw {
                identifier: POS_1_PREFIXED_ID_EWPOAF.to_string(),
                emergency_unlock: Some(true),
            },
            vec![],
            |result| {
                result.unwrap();
            },
        )
        .query_balance(lp_denom_1.clone().to_string(), &other, |balance| {
            //emergency unlock penalty is 10% of the position amount, so the user gets 1000 - 100 = 900
            assert_eq!(balance, Uint128::new(999_999_900)); // Derived
        })
        .query_balance(lp_denom_1.clone().to_string(), &fee_collector, |balance| {
            //50% of the penalty goes to the fee collector
            assert_eq!(balance, Uint128::new(50)); // Derived
        })
        .query_balance(lp_denom_1.clone().to_string(), &creator_1, |balance| {
            //50% of the penalty goes to the active farm creator
            assert_eq!(balance, Uint128::new(1_000_000_050u128)); // Derived
        })
        .query_balance(lp_denom_1.clone().to_string(), &creator_2, |balance| {
            // creator_2 didn't get anything of the penalty since its farm starts in the future
            assert_eq!(balance, Uint128::new(INITIAL_BALANCE));
        });

    for _ in 0..33 {
        suite.add_one_epoch();
    }

    suite.query_current_epoch(|result| {
        let epoch_response = result.unwrap();
        assert_eq!(epoch_response.epoch.id, EPOCH_ID_34);
    });

    // at this time no farm is active. Farm-3 expired on epoch 32, and farm-4 has not started
    // withdrawing now won't give any penalty fees to the farm creators as they are inactive
    suite
        .query_balance(lp_denom_2.clone().to_string(), &other, |balance| {
            assert_eq!(
                balance,
                Uint128::new(INITIAL_BALANCE - LP_LOCK_AMOUNT_EWPOAF)
            );
        })
        .query_balance(lp_denom_2.clone().to_string(), &creator_1, |balance| {
            assert_eq!(balance, Uint128::new(INITIAL_BALANCE));
        })
        .query_balance(lp_denom_2.clone().to_string(), &creator_2, |balance| {
            assert_eq!(balance, Uint128::new(INITIAL_BALANCE));
        })
        .query_balance(lp_denom_2.clone().to_string(), &fee_collector, |balance| {
            assert_eq!(balance, Uint128::zero());
        })
        .manage_position(
            &other,
            PositionAction::Withdraw {
                identifier: POS_2_PREFIXED_ID_EWPOAF.to_string(),
                emergency_unlock: Some(true),
            },
            vec![],
            |result| {
                result.unwrap();
            },
        )
        .query_balance(lp_denom_2.clone().to_string(), &other, |balance| {
            //emergency unlock penalty is 10% of the position amount, so the user gets 1000 - 100 = 900
            assert_eq!(balance, Uint128::new(999_999_900)); // Derived
        })
        .query_balance(lp_denom_2.clone().to_string(), &fee_collector, |balance| {
            //100% of the penalty goes to the fee collector
            assert_eq!(balance, Uint128::new(100)); // Derived
        })
        .query_balance(lp_denom_2.clone().to_string(), &creator_1, |balance| {
            // creator_2 didn't get anything of the penalty since its farm ended in the past
            assert_eq!(balance, Uint128::new(INITIAL_BALANCE));
        })
        .query_balance(lp_denom_2.clone().to_string(), &creator_2, |balance| {
            // creator_2 didn't get anything of the penalty since its farm starts in the future
            assert_eq!(balance, Uint128::new(INITIAL_BALANCE));
        });
}

#[test]
#[allow(clippy::inconsistent_digit_grouping)]
pub fn can_emergency_withdraw_an_lp_without_farm() {
    let lp_denom = format!("factory/{MOCK_CONTRACT_ADDR_1}/{LP_SYMBOL}").to_string();
    let lp_without_farm = format!("factory/{MOCK_CONTRACT_ADDR_1}/2.{LP_SYMBOL}").to_string();

    let mut suite = TestingSuite::default_with_balances(vec![
        coin(INITIAL_BALANCE, UOM_DENOM.to_string()),
        coin(INITIAL_BALANCE, UUSDY_DENOM.to_string()),
        coin(INITIAL_BALANCE, UOSMO_DENOM.to_string()),
        coin(INITIAL_BALANCE, lp_denom.clone()),
        coin(INITIAL_BALANCE, lp_without_farm.clone()),
    ]);

    let creator = suite.creator();

    suite.instantiate_default();

    suite
        .manage_farm(
            &creator,
            FarmAction::Fill {
                params: FarmParams {
                    lp_denom: lp_denom.clone(),
                    start_epoch: Some(EPOCH_ID_2),
                    preliminary_end_epoch: Some(EPOCH_ID_6),
                    curve: None,
                    farm_asset: Coin {
                        denom: UUSDY_DENOM.to_string(),
                        amount: Uint128::new(FARM_ASSET_UUSDY_CEW),
                    },
                    farm_identifier: None,
                },
            },
            vec![
                coin(FARM_ASSET_UUSDY_CEW, UUSDY_DENOM),
                coin(UOM_FARM_CREATION_FEE, UOM_DENOM),
            ],
            |result| {
                result.unwrap();
            },
        )
        .manage_position(
            &creator,
            PositionAction::Create {
                identifier: Some(CREATOR_POS_RAW_ID_CEW.to_string()),
                unlocking_duration: DEFAULT_UNLOCKING_DURATION_SECONDS,
                receiver: None,
            },
            vec![coin(LP_LOCK_AMOUNT_CREATOR_CEW, lp_without_farm.clone())],
            |result| {
                result.unwrap();
            },
        );

    suite.add_epochs(2);

    // withdraw the position
    suite.manage_position(
        &creator,
        PositionAction::Withdraw {
            identifier: CREATOR_POS_PREFIXED_ID_CEW.to_string(),
            emergency_unlock: Some(true),
        },
        vec![],
        |result| {
            result.unwrap();
        },
    );
}

/// This test creates multiple farms, and multiple positions with different users. Users open and close
/// and withdraw positions in different fashion, and claim rewards. The test checks if the rewards
/// are calculated correctly, and if the positions are managed correctly.
#[test]
fn test_managing_positions_close_and_emergency_withdraw() {
    let lp_denom_1 = format!("factory/{MOCK_CONTRACT_ADDR_1}/1.{LP_SYMBOL}").to_string();
    let lp_denom_2 = format!("factory/{MOCK_CONTRACT_ADDR_1}/2.{LP_SYMBOL}").to_string();

    let mut suite = TestingSuite::default_with_balances(vec![
        coin(INITIAL_BALANCE, UOM_DENOM.to_string()),
        coin(INITIAL_BALANCE, UUSDY_DENOM.to_string()),
        coin(INITIAL_BALANCE, UOSMO_DENOM.to_string()),
        coin(INITIAL_BALANCE_LARGE, lp_denom_1.clone()),
        coin(INITIAL_BALANCE_LARGE, lp_denom_2.clone()),
    ]);

    let alice = suite.creator();
    let bob = suite.senders[1].clone();
    let carol = suite.senders[2].clone();

    suite.instantiate_default();

    // create overlapping farms
    suite
        .manage_farm(
            &alice,
            FarmAction::Fill {
                params: FarmParams {
                    lp_denom: lp_denom_1.clone(),
                    start_epoch: None,
                    preliminary_end_epoch: None,
                    curve: None,
                    farm_asset: Coin {
                        denom: UUSDY_DENOM.to_string(),
                        amount: Uint128::new(FARM_ASSET_UUSDY_ALICE_FARM1_MPCEW),
                    },
                    farm_identifier: None,
                },
            },
            vec![
                coin(FARM_ASSET_UUSDY_ALICE_FARM1_MPCEW, UUSDY_DENOM),
                coin(UOM_FARM_CREATION_FEE, UOM_DENOM),
            ],
            |result| {
                result.unwrap();
            },
        )
        .manage_farm(
            &alice,
            FarmAction::Fill {
                params: FarmParams {
                    lp_denom: lp_denom_2.clone(),
                    start_epoch: Some(EPOCH_ID_10_MPCEW),
                    preliminary_end_epoch: Some(EPOCH_ID_20_MPCEW),
                    curve: None,
                    farm_asset: Coin {
                        denom: UUSDY_DENOM.to_string(),
                        amount: Uint128::new(FARM_ASSET_UUSDY_ALICE_FARM2_MPCEW),
                    },
                    farm_identifier: None,
                },
            },
            vec![
                coin(FARM_ASSET_UUSDY_ALICE_FARM2_MPCEW, UUSDY_DENOM),
                coin(UOM_FARM_CREATION_FEE, UOM_DENOM),
            ],
            |result| {
                result.unwrap();
            },
        );

    // alice locks liquidity early
    suite.manage_position(
        &alice,
        PositionAction::Create {
            identifier: Some(ALICE_POS_1_RAW_ID_MPCEW.to_string()),
            unlocking_duration: DEFAULT_UNLOCKING_DURATION_SECONDS,
            receiver: None,
        },
        vec![coin(LP_LOCK_ALICE_POS_1_MPCEW, lp_denom_1.clone())],
        |result| {
            result.unwrap();
        },
    );

    suite.add_epochs(5).query_current_epoch(|result| {
        let epoch_response = result.unwrap();
        assert_eq!(epoch_response.epoch.id, EPOCH_ID_5_MPCEW);
    });

    // then bob joins alice after a few epochs, having positions in both farms
    suite
        .manage_position(
            &bob,
            PositionAction::Create {
                identifier: Some(BOB_POS_1_RAW_ID_MPCEW.to_string()),
                unlocking_duration: DEFAULT_UNLOCKING_DURATION_SECONDS,
                receiver: None,
            },
            vec![coin(LP_LOCK_BOB_POS_1_AND_2_MPCEW, lp_denom_1.clone())],
            |result| {
                result.unwrap();
            },
        )
        .manage_position(
            &bob,
            PositionAction::Create {
                identifier: Some(BOB_POS_2_RAW_ID_MPCEW.to_string()),
                unlocking_duration: DEFAULT_UNLOCKING_DURATION_SECONDS,
                receiver: None,
            },
            vec![coin(LP_LOCK_BOB_POS_1_AND_2_MPCEW, lp_denom_2.clone())],
            |result| {
                result.unwrap();
            },
        );

    suite
        .query_rewards(&alice, None, |result| {
            let rewards_response = result.unwrap();
            match rewards_response {
                RewardsResponse::RewardsResponse { total_rewards, .. } => {
                    assert_eq!(total_rewards.len(), 1);
                    assert_eq!(total_rewards[0], coin(3_170u128, UUSDY_DENOM)); // Derived
                }
                _ => {
                    panic!("Wrong response type, should return RewardsResponse::RewardsResponse")
                }
            }
        })
        .query_rewards(&bob, None, |result| {
            let rewards_response = result.unwrap();
            match rewards_response {
                RewardsResponse::RewardsResponse { total_rewards, .. } => {
                    assert!(total_rewards.is_empty());
                }
                _ => {
                    panic!("Wrong response type, should return RewardsResponse::RewardsResponse")
                }
            }
        })
        .query_rewards(&carol, None, |result| {
            let rewards_response = result.unwrap();
            match rewards_response {
                RewardsResponse::RewardsResponse { total_rewards, .. } => {
                    assert!(total_rewards.is_empty());
                }
                _ => {
                    panic!("Wrong response type, should return RewardsResponse::RewardsResponse")
                }
            }
        });

    suite
        .query_balance(UUSDY_DENOM.to_string(), &alice, |balance| {
            assert_eq!(
                balance,
                Uint128::new(
                    INITIAL_BALANCE
                        - (FARM_ASSET_UUSDY_ALICE_FARM1_MPCEW + FARM_ASSET_UUSDY_ALICE_FARM2_MPCEW)
                )
            );
        })
        .claim(&alice, vec![], None, |result| {
            result.unwrap();
        })
        .query_balance(UUSDY_DENOM.to_string(), &alice, |balance| {
            assert_eq!(
                balance,
                Uint128::new(
                    INITIAL_BALANCE
                        - (FARM_ASSET_UUSDY_ALICE_FARM1_MPCEW + FARM_ASSET_UUSDY_ALICE_FARM2_MPCEW)
                        + 3_170u128
                )
            );
        });

    // last claimed epoch for alice = 5
    suite.add_epochs(3).query_current_epoch(|result| {
        let epoch_response = result.unwrap();
        assert_eq!(epoch_response.epoch.id, EPOCH_ID_8_MPCEW);
    });

    // then carol joins alice and bob after a few epochs
    suite.manage_position(
        &carol,
        PositionAction::Create {
            identifier: Some(CAROL_POS_2_RAW_ID_MPCEW.to_string()),
            unlocking_duration: DEFAULT_UNLOCKING_DURATION_SECONDS,
            receiver: None,
        },
        vec![coin(LP_LOCK_CAROL_POS_2_MPCEW, lp_denom_2.clone())],
        |result| {
            result.unwrap();
        },
    );

    // create two more farms, one overlapping, the other one not.
    suite
        .manage_farm(
            &alice,
            FarmAction::Fill {
                params: FarmParams {
                    lp_denom: lp_denom_1.clone(),
                    start_epoch: Some(EPOCH_ID_15_MPCEW),
                    preliminary_end_epoch: Some(EPOCH_ID_20_MPCEW),
                    curve: None,
                    farm_asset: Coin {
                        denom: UOSMO_DENOM.to_string(),
                        amount: Uint128::new(FARM_ASSET_UOSMO_ALICE_FARM3_MPCEW),
                    },
                    farm_identifier: None,
                },
            },
            vec![
                coin(FARM_ASSET_UOSMO_ALICE_FARM3_MPCEW, UOSMO_DENOM),
                coin(UOM_FARM_CREATION_FEE, UOM_DENOM),
            ],
            |result| {
                result.unwrap();
            },
        )
        .manage_farm(
            &alice,
            FarmAction::Fill {
                params: FarmParams {
                    lp_denom: lp_denom_2.clone(),
                    start_epoch: Some(EPOCH_ID_22_MPCEW),
                    preliminary_end_epoch: None,
                    curve: None,
                    farm_asset: Coin {
                        denom: UOM_DENOM.to_string(),
                        amount: Uint128::new(FARM_ASSET_UOM_ALICE_FARM4_MPCEW),
                    },
                    farm_identifier: None,
                },
            },
            vec![coin(UOM_FARM4_TOTAL_FUNDS_MPCEW, UOM_DENOM)],
            |result| {
                result.unwrap();
            },
        );

    suite
        .query_rewards(&alice, None, |result| {
            let rewards_response = result.unwrap();
            match rewards_response {
                RewardsResponse::RewardsResponse { total_rewards, .. } => {
                    assert_eq!(total_rewards.len(), 1);
                    assert_eq!(total_rewards[0], coin(633u128, UUSDY_DENOM)); // Derived
                }
                _ => {
                    panic!("Wrong response type, should return RewardsResponse::RewardsResponse")
                }
            }
        })
        .query_rewards(&bob, None, |result| {
            let rewards_response = result.unwrap();
            match rewards_response {
                RewardsResponse::RewardsResponse { total_rewards, .. } => {
                    assert_eq!(total_rewards.len(), 1);
                    assert_eq!(total_rewards[0], coin(1_266u128, UUSDY_DENOM)); // Derived
                }
                _ => {
                    panic!("Wrong response type, should return RewardsResponse::RewardsResponse")
                }
            }
        })
        .query_rewards(&carol, None, |result| {
            let rewards_response = result.unwrap();
            match rewards_response {
                RewardsResponse::RewardsResponse { total_rewards, .. } => {
                    assert!(total_rewards.is_empty());
                }
                _ => {
                    panic!("Wrong response type, should return RewardsResponse::RewardsResponse")
                }
            }
        });

    // now alice emergency withdraws her position, giving up her rewards
    suite
        .manage_position(
            &alice,
            PositionAction::Withdraw {
                identifier: ALICE_POS_1_PREFIXED_ID_MPCEW.to_string(),
                emergency_unlock: Some(true),
            },
            vec![],
            |result| {
                result.unwrap();
            },
        )
        .query_rewards(&alice, None, |result| {
            let rewards_response = result.unwrap();
            match rewards_response {
                RewardsResponse::RewardsResponse { total_rewards, .. } => {
                    assert!(total_rewards.is_empty());
                }
                _ => {
                    panic!("Wrong response type, should return RewardsResponse::RewardsResponse")
                }
            }
        })
        // Bob's rewards should remain the same for the current epoch
        .query_rewards(&bob, None, |result| {
            let rewards_response = result.unwrap();
            match rewards_response {
                RewardsResponse::RewardsResponse { total_rewards, .. } => {
                    assert_eq!(total_rewards.len(), 1);
                    assert_eq!(total_rewards[0], coin(1_266u128, UUSDY_DENOM)); // Derived
                }
                _ => {
                    panic!("Wrong response type, should return RewardsResponse::RewardsResponse")
                }
            }
        });

    suite.add_one_epoch().query_current_epoch(|result| {
        let epoch_response = result.unwrap();
        assert_eq!(epoch_response.epoch.id, EPOCH_ID_9_MPCEW);
    });

    suite.query_rewards(&bob, None, |result| {
        let rewards_response = result.unwrap();
        match rewards_response {
            RewardsResponse::RewardsResponse { total_rewards, .. } => {
                assert_eq!(total_rewards.len(), 1);
                // 634 is the emission rate for farm 1
                assert_eq!(total_rewards[0], coin(1_266u128 + 634, UUSDY_DENOM));
                // Derived
            }
            _ => {
                panic!("Wrong response type, should return RewardsResponse::RewardsResponse")
            }
        }
    });

    // alice creates a new position with the same LP denom
    suite
        .manage_position(
            &alice,
            PositionAction::Create {
                identifier: Some(ALICE_SECOND_POS_1_RAW_ID_MPCEW.to_string()),
                unlocking_duration: DEFAULT_UNLOCKING_DURATION_SECONDS,
                receiver: None,
            },
            vec![coin(LP_LOCK_ALICE_SECOND_POS_1_MPCEW, lp_denom_1.clone())],
            |result| {
                result.unwrap();
            },
        )
        .manage_position(
            &alice,
            PositionAction::Create {
                identifier: Some(ALICE_SECOND_POS_2_RAW_ID_MPCEW.to_string()),
                unlocking_duration: DEFAULT_UNLOCKING_DURATION_SECONDS,
                receiver: None,
            },
            vec![coin(LP_LOCK_ALICE_SECOND_POS_2_MPCEW, lp_denom_1.clone())],
            |result| {
                result.unwrap();
            },
        );

    suite.add_one_epoch();

    suite
        .query_rewards(&alice, None, |result| {
            let rewards_response = result.unwrap();
            match rewards_response {
                RewardsResponse::RewardsResponse { total_rewards, .. } => {
                    assert_eq!(total_rewards.len(), 1);
                    assert_eq!(total_rewards[0], coin(380u128, UUSDY_DENOM)); // Derived
                }
                _ => {
                    panic!("Wrong response type, should return RewardsResponse::RewardsResponse")
                }
            }
        })
        .claim(&alice, vec![], None, |result| {
            result.unwrap();
        });

    suite.add_epochs(2);

    suite
        .manage_position(
            &alice,
            PositionAction::Withdraw {
                identifier: ALICE_SECOND_POS_1_PREFIXED_ID_MPCEW.to_string(),
                emergency_unlock: Some(true),
            },
            vec![],
            |result| {
                result.unwrap();
            },
        )
        .claim(&alice, vec![], None, |result| {
            result.unwrap();
        });

    suite
        .add_one_epoch()
        .query_rewards(&alice, None, |result| {
            let rewards_response = result.unwrap();
            match rewards_response {
                RewardsResponse::RewardsResponse { total_rewards, .. } => {
                    assert_eq!(total_rewards.len(), 1usize);
                    assert_eq!(total_rewards[0], coin(324u128, UUSDY_DENOM)); // Derived
                }
                _ => {
                    panic!("Wrong response type, should return RewardsResponse::RewardsResponse")
                }
            }
        })
        .query_balance(UUSDY_DENOM.to_string(), &alice, |balance| {
            assert_eq!(balance, Uint128::new(999_328_756u128)); // Derived
        })
        .claim(&alice, vec![], None, |result| {
            result.unwrap();
        })
        .query_balance(UUSDY_DENOM.to_string(), &alice, |balance| {
            assert_eq!(balance, Uint128::new(999_328_756u128 + 324u128)); // Derived
        })
        .manage_position(
            &alice,
            PositionAction::Close {
                identifier: ALICE_SECOND_POS_2_PREFIXED_ID_MPCEW.to_string(),
                lp_asset: Some(coin(500, lp_denom_1.clone())),
            },
            vec![],
            |result| {
                result.unwrap();
            },
        );

    suite.add_one_epoch();

    suite.query_current_epoch(|result| {
        let epoch_response = result.unwrap();
        assert_eq!(epoch_response.epoch.id, EPOCH_ID_14_MPCEW);
    });

    suite
        .manage_position(
            &alice,
            PositionAction::Withdraw {
                identifier: PENDING_WITHDRAW_POS_ID_MPCEW.to_string(),
                emergency_unlock: None,
            },
            vec![],
            |result| {
                result.unwrap();
            },
        )
        .manage_position(
            &alice,
            PositionAction::Withdraw {
                identifier: ALICE_SECOND_POS_2_PREFIXED_ID_MPCEW.to_string(),
                emergency_unlock: Some(true),
            },
            vec![],
            |result| {
                result.unwrap();
            },
        )
        .query_lp_weight(&alice, &lp_denom_1, EPOCH_ID_15_MPCEW, |result| {
            result.unwrap_err();
        })
        .query_lp_weight(&alice, &lp_denom_1, EPOCH_ID_14_MPCEW, |result| {
            result.unwrap_err();
        })
        .query_lp_weight(&alice, &lp_denom_1, EPOCH_ID_13_MPCEW, |result| {
            result.unwrap_err();
        })
        .query_lp_weight(&alice, &lp_denom_1, EPOCH_ID_12_MPCEW, |result| {
            result.unwrap_err();
        });

    suite
        .query_rewards(&alice, None, |result| {
            let rewards_response = result.unwrap();
            match rewards_response {
                RewardsResponse::RewardsResponse { total_rewards, .. } => {
                    assert!(total_rewards.is_empty());
                }
                _ => {
                    panic!("Wrong response type, should return RewardsResponse::RewardsResponse")
                }
            }
        })
        .manage_position(
            &alice,
            PositionAction::Create {
                identifier: Some(FINAL_ALICE_POS_RAW_ID_MPCEW.to_string()),
                unlocking_duration: DEFAULT_UNLOCKING_DURATION_SECONDS,
                receiver: None,
            },
            vec![coin(LP_LOCK_FINAL_ALICE_POS_MPCEW, lp_denom_1.clone())],
            |result| {
                result.unwrap();
            },
        )
        .query_rewards(&alice, None, |result| {
            let rewards_response = result.unwrap();
            match rewards_response {
                RewardsResponse::RewardsResponse { total_rewards, .. } => {
                    assert!(total_rewards.is_empty());
                }
                _ => {
                    panic!("Wrong response type, should return RewardsResponse::RewardsResponse")
                }
            }
        })
        .query_lp_weight(&alice, &lp_denom_1, EPOCH_ID_14_MPCEW, |result| {
            result.unwrap_err();
        })
        .query_lp_weight(&alice, &lp_denom_1, EPOCH_ID_15_MPCEW, |result| {
            let lp_weight_response = result.unwrap();
            assert_eq!(
                lp_weight_response.lp_weight,
                Uint128::new(LP_LOCK_FINAL_ALICE_POS_MPCEW)
            );
        })
        .add_one_epoch()
        .query_rewards(&alice, None, |result| {
            let rewards_response = result.unwrap();
            match rewards_response {
                RewardsResponse::RewardsResponse { total_rewards, .. } => {
                    assert_eq!(total_rewards.len(), 1usize);
                    assert_eq!(total_rewards[0], coin(1_454, UOSMO_DENOM)); // Derived
                }
                _ => {
                    panic!("Wrong response type, should return RewardsResponse::RewardsResponse")
                }
            }
        });

    suite
        .query_rewards(&bob, None, |result| {
            let rewards_response = result.unwrap();
            match rewards_response {
                RewardsResponse::RewardsResponse { total_rewards, .. } => {
                    assert_eq!(total_rewards.len(), 2usize);
                    assert_eq!(
                        total_rewards,
                        vec![coin(322u128, UOSMO_DENOM), coin(163_355u128, UUSDY_DENOM)]
                    );
                }
                _ => {
                    panic!("Wrong response type, should return RewardsResponse::RewardsResponse")
                }
            }
        })
        .query_balance(UUSDY_DENOM.to_string(), &bob, |balance| {
            assert_eq!(balance, Uint128::new(INITIAL_BALANCE));
        })
        .query_balance(UOSMO_DENOM.to_string(), &bob, |balance| {
            assert_eq!(balance, Uint128::new(INITIAL_BALANCE));
        })
        .claim(&bob, vec![], None, |result| {
            result.unwrap();
        })
        .query_balance(UUSDY_DENOM.to_string(), &bob, |balance| {
            assert_eq!(balance, Uint128::new(INITIAL_BALANCE + 163_355u128));
            // Derived
        })
        .query_balance(UOSMO_DENOM.to_string(), &bob, |balance| {
            assert_eq!(balance, Uint128::new(INITIAL_BALANCE + 322u128)); // Derived
        });

    suite.add_epochs(3).query_current_epoch(|result| {
        let epoch_response = result.unwrap();
        assert_eq!(epoch_response.epoch.id, EPOCH_ID_18_MPCEW);
    });

    suite
        .query_rewards(&bob, None, |result| {
            let rewards_response = result.unwrap();
            match rewards_response {
                RewardsResponse::RewardsResponse { total_rewards, .. } => {
                    assert_eq!(total_rewards.len(), 2usize);
                    assert_eq!(
                        total_rewards,
                        vec![coin(966u128, UOSMO_DENOM), coin(79_950u128, UUSDY_DENOM)]
                    );
                }
                _ => {
                    panic!("Wrong response type, should return RewardsResponse::RewardsResponse")
                }
            }
        })
        // since bob didn't have more positions for lp1, the lp_weight_history gets wiped for that lp denom
        .manage_position(
            &bob,
            PositionAction::Withdraw {
                identifier: BOB_POS_1_PREFIXED_ID_MPCEW.to_string(),
                emergency_unlock: Some(true),
            },
            vec![],
            |result| {
                result.unwrap();
            },
        )
        .query_rewards(&bob, None, |result| {
            let rewards_response = result.unwrap();
            match rewards_response {
                RewardsResponse::RewardsResponse { total_rewards, .. } => {
                    assert_eq!(total_rewards.len(), 1usize);
                    assert_eq!(total_rewards, vec![coin(79_950u128, UUSDY_DENOM)]);
                    // Derived
                }
                _ => {
                    panic!("Wrong response type, should return RewardsResponse::RewardsResponse")
                }
            }
        })
        // creating a new position for bob with the lp denom 1 won't give him the rewards in the past
        // epochs he had but gave up by emergency withdrawing
        .manage_position(
            &bob,
            PositionAction::Create {
                identifier: Some(NEW_BOB_POS_LP1_RAW_ID_MPCEW.to_string()),
                unlocking_duration: DEFAULT_UNLOCKING_DURATION_SECONDS,
                receiver: None,
            },
            vec![coin(LP_LOCK_NEW_BOB_POS_LP1_MPCEW, lp_denom_1.clone())],
            |result| {
                result.unwrap();
            },
        )
        .query_rewards(&bob, None, |result| {
            let rewards_response = result.unwrap();
            match rewards_response {
                RewardsResponse::RewardsResponse { total_rewards, .. } => {
                    assert_eq!(total_rewards.len(), 1usize);
                    assert_eq!(total_rewards, vec![coin(79_950u128, UUSDY_DENOM)]);
                    // Derived
                }
                _ => {
                    panic!("Wrong response type, should return RewardsResponse::RewardsResponse")
                }
            }
        })
        .claim(&bob, vec![], None, |result| {
            result.unwrap();
        })
        .add_one_epoch()
        .query_rewards(&bob, None, |result| {
            let rewards_response = result.unwrap();
            match rewards_response {
                RewardsResponse::RewardsResponse { total_rewards, .. } => {
                    assert_eq!(total_rewards.len(), 2usize);
                    assert_eq!(
                        total_rewards,
                        vec![coin(444, UOSMO_DENOM), coin(26_650u128, UUSDY_DENOM)]
                    );
                }
                _ => {
                    panic!("Wrong response type, should return RewardsResponse::RewardsResponse")
                }
            }
        });
}
