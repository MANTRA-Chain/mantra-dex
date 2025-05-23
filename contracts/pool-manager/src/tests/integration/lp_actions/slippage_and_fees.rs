use cosmwasm_std::{coin, Coin, Decimal, Uint128};
use mantra_common_testing::multi_test::stargate_mock::StargateMock;
use mantra_dex_std::fee::{Fee, PoolFee};
use mantra_dex_std::pool_manager::{PoolInfo, PoolStatus, PoolType, SwapOperation};
use test_utils::common_constants::*;

use crate::tests::suite::TestingSuite;
use crate::ContractError;

// ========== Initial Balances ==========
const INITIAL_BALANCE_1T: u128 = 1_000_000_000_000u128;

// ========== Pool Creation & Liquidity ==========
const LIQUIDITY_500K: u128 = 500_000u128;

// ========== Swap Amounts ==========
const SWAP_AMOUNT_2K: u128 = 2_000u128;

// ========== Fee Ratios & Percentages ==========

const SLIPPAGE_PERCENT_21: u64 = 21;

// ========== Pool Identifiers & LP Denoms ==========
const WHALE_ULUNA_POOL_LABEL: &str = "whale.uluna";
const O_WHALE_ULUNA_ID: &str = "o.whale.uluna";
const WHALE_ULUNA_POOL_1_LABEL: &str = "whale.uluna.pool.1";
const O_WHALE_ULUNA_POOL_1_ID: &str = "o.whale.uluna.pool.1";
const WHALE_ULUNA_POOL_1_LP: &str = "factory/mantra1zwv6feuzhy6a9wekh96cd57lsarmqlwxdypdsplw6zhfncqw6ftqlydlr9/o.whale.uluna.pool.1.LP";
const WHALE_ULUNA_POOL_2_LABEL: &str = "whale.uluna.pool.2";
const O_WHALE_ULUNA_POOL_2_ID: &str = "o.whale.uluna.pool.2";
const O_WHALE_ULUNA_POOL_2_LP: &str = "factory/mantra1zwv6feuzhy6a9wekh96cd57lsarmqlwxdypdsplw6zhfncqw6ftqlydlr9/o.whale.uluna.pool.2.LP";
const ULUNA_UUSD_POOL_1_LABEL: &str = "uluna.uusd.pool.1";
const O_ULUNA_UUSD_POOL_1_ID: &str = "o.uluna.uusd.pool.1";
const O_ULUNA_UUSD_POOL_1_LP: &str = "factory/mantra1zwv6feuzhy6a9wekh96cd57lsarmqlwxdypdsplw6zhfncqw6ftqlydlr9/o.uluna.uusd.pool.1.LP";

// ========== Expected Balances & Test Values ==========
const CONTRACT_LP_BALANCE_1K: u128 = 1_000u128;
const POOL_MANAGER_UWHALE_BALANCE_2M: u128 = 2_000_000u128;
const POOL_MANAGER_ULUNA_BALANCE_3M: u128 = 3_000_000u128;
const POOL_MANAGER_UUSD_BALANCE_1M: u128 = 1_000_000u128;

const FEE_COLLECTOR_LUNA_AFTER_SWAP_1: u128 = 99u128;
const POOL_1_WHALE_AFTER_SWAP_2: u128 = 999_140u128;
const POOL_1_LUNA_AFTER_SWAP_2: u128 = 1_001_070u128;
const FEE_COLLECTOR_WHALE_AFTER_SWAP_2: u128 = 199u128;
const POOL_2_WHALE_AFTER_SWAP_1: u128 = 1_001_000u128;
const POOL_2_LUNA_AFTER_SWAP_1: u128 = 999_150u128;
const POOL_2_WHALE_AFTER_SWAP_2: u128 = 999_300u128;
const POOL_2_LUNA_AFTER_SWAP_2: u128 = 1_001_150u128;
const POOL_3_LUNA_AFTER_SWAP_1: u128 = 1_003_000u128;
const POOL_3_UUSD_AFTER_SWAP_1: u128 = 997_218u128;
const FEE_COLLECTOR_UUSD_AFTER_SWAP_1: u128 = 3_000u128 + 299u128; // Pool creation fees + swap fee
const POOL_3_LUNA_AFTER_SWAP_2: u128 = 1_001_599u128;
const POOL_3_UUSD_AFTER_SWAP_2: u128 = 998_718u128;
const FEE_COLLECTOR_LUNA_AFTER_SWAP_2_POOL_3: u128 = FEE_COLLECTOR_LUNA_AFTER_SWAP_1 + 150u128;
const POOL_MANAGER_ULUNA_FINAL: u128 = 3_003_819u128;
const POOL_MANAGER_UUSD_FINAL: u128 = 998_718u128;
const POOL_MANAGER_UWHALE_FINAL: u128 = 1_998_440u128;
const POOL_2_WHALE_AFTER_ROUTER_SWAP: u128 = 1_004_300u128;
const POOL_2_LUNA_AFTER_ROUTER_SWAP: u128 = 996_913u128;
const POOL_3_LUNA_AFTER_ROUTER_SWAP: u128 = 1_005_587u128;
const POOL_3_UUSD_AFTER_ROUTER_SWAP: u128 = 995_035u128;

// Expected Pool Manager balances AFTER the final router swap in provide_liquidity_to_multiple_pools_check_fees

#[test]
fn assert_slippage_tolerance_correct_xyk() {
    let mut suite = TestingSuite::default_with_balances(
        vec![
            coin(INITIAL_BALANCE_PLUS_ONE, DENOM_UWHALE.to_string()),
            coin(INITIAL_BALANCE, DENOM_ULUNA.to_string()),
            coin(INITIAL_BALANCE_PLUS_ONE, DENOM_UUSD.to_string()),
            coin(INITIAL_BALANCE_PLUS_ONE, DENOM_UOM.to_string()),
        ],
        StargateMock::new(vec![coin(STARGATE_MOCK_UOM_AMOUNT, DENOM_UOM.to_string())]),
    );
    let creator = suite.creator();
    // Asset infos with uwhale and uluna

    let asset_infos = vec![DENOM_UWHALE.to_string(), DENOM_ULUNA.to_string()];

    // Protocol fee is 0.01% and swap fee is 0.02% and burn fee is 0%
    let pool_fees = PoolFee {
        protocol_fee: Fee {
            share: Decimal::from_ratio(PROTOCOL_FEE_RATIO_1_1000.0, PROTOCOL_FEE_RATIO_1_1000.1),
        },
        swap_fee: Fee {
            share: Decimal::from_ratio(SWAP_FEE_RATIO_1_1000.0, SWAP_FEE_RATIO_1_1000.1),
        },
        burn_fee: Fee {
            share: Decimal::zero(),
        },
        extra_fees: vec![],
    };

    // Create a 2 token pool
    suite.instantiate_default().create_pool(
        &creator,
        asset_infos,
        vec![DECIMAL_PLACES, DECIMAL_PLACES],
        pool_fees,
        PoolType::ConstantProduct {},
        Some(WHALE_ULUNA_POOL_LABEL.to_string()),
        vec![
            coin(POOL_CREATION_FEE, DENOM_UUSD),
            coin(STARGATE_MOCK_UOM_AMOUNT, DENOM_UOM),
        ],
        |result| {
            result.unwrap();
        },
    );

    // first liquidity provided as uwhale,uluna
    suite.provide_liquidity(
        &creator,
        O_WHALE_ULUNA_ID.to_string(),
        None,
        None,
        None,
        None,
        None,
        vec![
            Coin {
                denom: DENOM_UWHALE.to_string(),
                amount: Uint128::from(LIQUIDITY_500K),
            },
            Coin {
                denom: DENOM_ULUNA.to_string(),
                amount: Uint128::from(LIQUIDITY_AMOUNT),
            },
        ],
        |result| {
            result.unwrap();
        },
    );

    // second liquidity provided as uluna,uwhale
    suite.provide_liquidity(
        &creator,
        O_WHALE_ULUNA_ID.to_string(),
        None,
        None,
        Some(Decimal::percent(60)),
        None,
        None,
        vec![
            Coin {
                denom: DENOM_ULUNA.to_string(),
                amount: Uint128::from(LIQUIDITY_AMOUNT),
            },
            Coin {
                denom: DENOM_UWHALE.to_string(),
                amount: Uint128::from(200_000u128),
            },
        ],
        |result| {
            result.unwrap();
        },
    );
}

#[test]
fn provide_liquidity_to_multiple_pools_check_fees() {
    let mut suite = TestingSuite::default_with_balances(
        vec![
            coin(INITIAL_BALANCE_1T, DENOM_UWHALE.to_string()),
            coin(INITIAL_BALANCE_1T, DENOM_ULUNA.to_string()),
            coin(INITIAL_BALANCE_1T, DENOM_UOSMO.to_string()),
            coin(INITIAL_BALANCE_1T, DENOM_UUSD.to_string()),
            coin(INITIAL_BALANCE_1T, DENOM_UOM.to_string()),
        ],
        StargateMock::new(vec![coin(STARGATE_MOCK_UOM_AMOUNT, DENOM_UOM.to_string())]),
    );
    let creator = suite.creator();
    let other = suite.senders[1].clone();

    // Asset denoms with uwhale and uluna
    let asset_denoms_1 = vec![DENOM_UWHALE.to_string(), DENOM_ULUNA.to_string()];
    let asset_denoms_2 = vec![DENOM_ULUNA.to_string(), DENOM_UUSD.to_string()];

    let pool_fees_1 = PoolFee {
        protocol_fee: Fee {
            share: Decimal::percent(10),
        },
        swap_fee: Fee {
            share: Decimal::percent(7),
        },
        burn_fee: Fee {
            share: Decimal::percent(3),
        },
        extra_fees: vec![],
    };

    let pool_fees_2 = PoolFee {
        protocol_fee: Fee {
            share: Decimal::zero(),
        },
        swap_fee: Fee {
            share: Decimal::percent(15),
        },
        burn_fee: Fee {
            share: Decimal::percent(5),
        },
        extra_fees: vec![],
    };

    // Create pools
    suite
        .instantiate_default()
        .add_one_epoch()
        .create_pool(
            &creator,
            asset_denoms_1.clone(),
            vec![DECIMAL_PLACES, DECIMAL_PLACES],
            pool_fees_1.clone(),
            PoolType::ConstantProduct,
            Some(WHALE_ULUNA_POOL_1_LABEL.to_string()),
            vec![
                coin(POOL_CREATION_FEE, DENOM_UUSD),
                coin(STARGATE_MOCK_UOM_AMOUNT, DENOM_UOM),
            ],
            |result| {
                result.unwrap();
            },
        )
        .create_pool(
            &creator,
            asset_denoms_1.clone(), // Changed from asset_denoms_1 to asset_denoms_1.clone() to avoid move error
            vec![DECIMAL_PLACES, DECIMAL_PLACES],
            pool_fees_2.clone(),
            PoolType::ConstantProduct,
            Some(WHALE_ULUNA_POOL_2_LABEL.to_string()),
            vec![
                coin(POOL_CREATION_FEE, DENOM_UUSD),
                coin(STARGATE_MOCK_UOM_AMOUNT, DENOM_UOM),
            ],
            |result| {
                result.unwrap();
            },
        )
        .create_pool(
            &creator,
            asset_denoms_2,
            vec![DECIMAL_PLACES, DECIMAL_PLACES],
            pool_fees_1.clone(),
            PoolType::ConstantProduct,
            Some(ULUNA_UUSD_POOL_1_LABEL.to_string()),
            vec![
                coin(POOL_CREATION_FEE, DENOM_UUSD),
                coin(STARGATE_MOCK_UOM_AMOUNT, DENOM_UOM),
            ],
            |result| {
                result.unwrap();
            },
        );

    let pool_manager_addr = suite.pool_manager_addr.clone();
    let fee_collector_addr = suite.fee_collector_addr.clone();

    // after creating 3 pools, the fee collector should have 3_000 uusd in fees
    suite.query_balance(
        &fee_collector_addr.to_string(),
        DENOM_UUSD.to_string(),
        |result| {
            assert_eq!(result.unwrap().amount, Uint128::new(3 * POOL_CREATION_FEE));
        },
    );

    // let\'s provide liquidity with two assets
    suite
        .provide_liquidity(
            &creator,
            O_WHALE_ULUNA_ID.to_string(), // This ID does not match any created pool, expect error
            None,
            None,
            None,
            None,
            None,
            vec![
                Coin {
                    denom: DENOM_UWHALE.to_string(),
                    amount: Uint128::from(LIQUIDITY_AMOUNT),
                },
                Coin {
                    denom: DENOM_ULUNA.to_string(),
                    amount: Uint128::from(LIQUIDITY_AMOUNT),
                },
            ],
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();
                match err {
                    ContractError::UnExistingPool => {}
                    _ => panic!("Wrong error type, should return ContractError::UnExistingPool"),
                }
            },
        )
        .provide_liquidity(
            &creator,
            O_WHALE_ULUNA_POOL_1_ID.to_string(),
            None,
            None,
            None,
            None,
            None,
            vec![
                Coin {
                    denom: DENOM_UWHALE.to_string(),
                    amount: Uint128::from(LIQUIDITY_AMOUNT),
                },
                Coin {
                    denom: DENOM_ULUNA.to_string(),
                    amount: Uint128::from(LIQUIDITY_AMOUNT),
                },
            ],
            |result| {
                result.unwrap();
            },
        )
        .provide_liquidity(
            &other,
            O_WHALE_ULUNA_POOL_2_ID.to_string(),
            None,
            None,
            None,
            None,
            None,
            vec![
                Coin {
                    denom: DENOM_UWHALE.to_string(),
                    amount: Uint128::from(LIQUIDITY_AMOUNT),
                },
                Coin {
                    denom: DENOM_ULUNA.to_string(),
                    amount: Uint128::from(LIQUIDITY_AMOUNT),
                },
            ],
            |result| {
                result.unwrap();
            },
        )
        .provide_liquidity(
            &other,
            O_ULUNA_UUSD_POOL_1_ID.to_string(),
            None,
            None,
            None,
            None,
            None,
            vec![
                Coin {
                    denom: DENOM_ULUNA.to_string(),
                    amount: Uint128::from(LIQUIDITY_AMOUNT),
                },
                Coin {
                    denom: DENOM_UUSD.to_string(),
                    amount: Uint128::from(LIQUIDITY_AMOUNT),
                },
            ],
            |result| {
                result.unwrap();
            },
        )
        .query_all_balances(&pool_manager_addr.to_string(), |result| {
            let balances = result.unwrap();
            assert_eq!(
                balances,
                vec![
                    Coin {
                        denom: O_ULUNA_UUSD_POOL_1_LP.to_string(),
                        amount: Uint128::from(CONTRACT_LP_BALANCE_1K),
                    },
                    Coin {
                        denom: WHALE_ULUNA_POOL_1_LP.to_string(),
                        amount: Uint128::from(CONTRACT_LP_BALANCE_1K),
                    },
                    Coin {
                        denom: O_WHALE_ULUNA_POOL_2_LP.to_string(),
                        amount: Uint128::from(CONTRACT_LP_BALANCE_1K),
                    },
                    Coin {
                        denom: DENOM_ULUNA.to_string(),
                        amount: Uint128::from(POOL_MANAGER_ULUNA_BALANCE_3M),
                    },
                    Coin {
                        denom: DENOM_UUSD.to_string(),
                        amount: Uint128::from(POOL_MANAGER_UUSD_BALANCE_1M),
                    },
                    Coin {
                        denom: DENOM_UWHALE.to_string(),
                        amount: Uint128::from(POOL_MANAGER_UWHALE_BALANCE_2M),
                    },
                ]
            );
        });

    // let\'s do swaps in o.whale.uluna.pool.1 and verify the fees are channeled correctly
    suite
        .swap(
            &creator,
            DENOM_ULUNA.to_string(),
            None,
            Some(Decimal::percent(20)),
            None,
            O_WHALE_ULUNA_POOL_1_ID.to_string(),
            vec![coin(SWAP_AMOUNT, DENOM_UWHALE.to_string())],
            |result| {
                result.unwrap();
            },
        )
        .query_pools(
            Some(O_WHALE_ULUNA_POOL_1_ID.to_string()),
            None,
            None,
            |result| {
                let response = result.unwrap();
                let pool_info = response.pools[0].pool_info.clone();

                // swapped 1000 uwhale
                // fees:
                // swap -> 69 (~7%)
                // protocol -> 99 (~10%)
                // burn ->  29 (~3%)
                // total_fees = 197, of which 69 stay in the pool (for LPs).
                // Going out of the pool is 99 (fee collector) + 29 (burned)

                assert_eq!(
                    pool_info,
                    PoolInfo {
                        pool_identifier: O_WHALE_ULUNA_POOL_1_ID.to_string(),
                        asset_denoms: vec![DENOM_UWHALE.to_string(), DENOM_ULUNA.to_string()],
                        lp_denom: WHALE_ULUNA_POOL_1_LP.to_string(),
                        asset_decimals: vec![DECIMAL_PLACES, DECIMAL_PLACES],
                        assets: vec![
                            coin(1_001_000u128, DENOM_UWHALE),
                            coin(999_070u128, DENOM_ULUNA)
                        ],
                        pool_type: PoolType::ConstantProduct,
                        pool_fees: pool_fees_1.clone(),
                        status: PoolStatus::default(),
                    }
                );
            },
        );

    // verify the fees went to the fee collector
    suite
        .query_balance(&fee_collector_addr.to_string(), DENOM_ULUNA, |result| {
            assert_eq!(
                result.unwrap(),
                coin(FEE_COLLECTOR_LUNA_AFTER_SWAP_1, DENOM_ULUNA)
            );
        })
        .swap(
            &creator,
            DENOM_UWHALE.to_string(),
            None,
            Some(Decimal::percent(SLIPPAGE_PERCENT_21)),
            None,
            O_WHALE_ULUNA_POOL_1_ID.to_string(),
            vec![coin(SWAP_AMOUNT_2K, DENOM_ULUNA.to_string())],
            |result| {
                result.unwrap();
            },
        )
        .query_pools(
            Some(O_WHALE_ULUNA_POOL_1_ID.to_string()),
            None,
            None,
            |result| {
                let response = result.unwrap();
                let pool_info = response.pools[0].pool_info.clone();

                // swapped 2000 uluna
                // fees:
                // swap -> 139 (~7%)
                // protocol -> 199 (~10%)
                // burn ->  59 (~3%)
                // total_fees = 397, of which 139 stay in the pool (for LPs).
                // Going out of the pool is 199 (fee collector) + 59 (burned)

                assert_eq!(
                    pool_info,
                    PoolInfo {
                        pool_identifier: O_WHALE_ULUNA_POOL_1_ID.to_string(),
                        asset_denoms: vec![DENOM_UWHALE.to_string(), DENOM_ULUNA.to_string()],
                        lp_denom: WHALE_ULUNA_POOL_1_LP.to_string(),
                        asset_decimals: vec![DECIMAL_PLACES, DECIMAL_PLACES],
                        assets: vec![
                            coin(POOL_1_WHALE_AFTER_SWAP_2, DENOM_UWHALE),
                            coin(POOL_1_LUNA_AFTER_SWAP_2, DENOM_ULUNA)
                        ],
                        pool_type: PoolType::ConstantProduct,
                        pool_fees: pool_fees_1.clone(),
                        status: PoolStatus::default(),
                    }
                );
            },
        );

    suite
        .query_balance(&fee_collector_addr.to_string(), DENOM_UWHALE, |result| {
            assert_eq!(
                result.unwrap(),
                coin(FEE_COLLECTOR_WHALE_AFTER_SWAP_2, DENOM_UWHALE)
            );
        })
        .query_balance(&fee_collector_addr.to_string(), DENOM_ULUNA, |result| {
            assert_eq!(
                result.unwrap(),
                coin(FEE_COLLECTOR_LUNA_AFTER_SWAP_1, DENOM_ULUNA)
            );
        });

    // let\'s do swaps in o.whale.uluna.pool.2 and verify the fees are channeled correctly
    suite
        .swap(
            &creator,
            DENOM_ULUNA.to_string(),
            None,
            Some(Decimal::percent(SLIPPAGE_PERCENT_21)),
            None,
            O_WHALE_ULUNA_POOL_2_ID.to_string(),
            vec![coin(SWAP_AMOUNT, DENOM_UWHALE.to_string())],
            |result| {
                result.unwrap();
            },
        )
        .query_pools(
            Some(O_WHALE_ULUNA_POOL_2_ID.to_string()),
            None,
            None,
            |result| {
                let response = result.unwrap();
                let pool_info = response.pools[0].pool_info.clone();

                // swapped 1000 uwhale
                // fees:
                // swap -> 149 (~15%)
                // protocol -> 0 (0%)
                // burn ->  49 (~5%)
                // total_fees = 198, of which 149 stay in the pool (for LPs).
                // Going out of the pool is 49 (burned)

                assert_eq!(
                    pool_info,
                    PoolInfo {
                        pool_identifier: O_WHALE_ULUNA_POOL_2_ID.to_string(),
                        asset_denoms: vec![DENOM_UWHALE.to_string(), DENOM_ULUNA.to_string()],
                        lp_denom: O_WHALE_ULUNA_POOL_2_LP.to_string(),
                        asset_decimals: vec![DECIMAL_PLACES, DECIMAL_PLACES],
                        assets: vec![
                            coin(POOL_2_WHALE_AFTER_SWAP_1, DENOM_UWHALE),
                            coin(POOL_2_LUNA_AFTER_SWAP_1, DENOM_ULUNA)
                        ],
                        pool_type: PoolType::ConstantProduct,
                        pool_fees: pool_fees_2.clone(),
                        status: PoolStatus::default(),
                    }
                );
            },
        );

    suite
        .swap(
            &creator,
            DENOM_UWHALE.to_string(),
            None,
            Some(Decimal::percent(SLIPPAGE_PERCENT_21)),
            None,
            O_WHALE_ULUNA_POOL_2_ID.to_string(),
            vec![coin(SWAP_AMOUNT_2K, DENOM_ULUNA.to_string())],
            |result| {
                result.unwrap();
            },
        )
        .query_pools(
            Some(O_WHALE_ULUNA_POOL_2_ID.to_string()),
            None,
            None,
            |result| {
                let response = result.unwrap();
                let pool_info = response.pools[0].pool_info.clone();

                // swapped 2000 uluna
                // fees:
                // swap -> 299 (~15%)
                // protocol -> 0 (0%)
                // burn ->  99 (~5%)
                // total_fees = 398, of which 299 stay in the pool (for LPs).
                // Going out of the pool is 99 (burned)

                assert_eq!(
                    pool_info,
                    PoolInfo {
                        pool_identifier: O_WHALE_ULUNA_POOL_2_ID.to_string(),
                        asset_denoms: vec![DENOM_UWHALE.to_string(), DENOM_ULUNA.to_string()],
                        lp_denom: O_WHALE_ULUNA_POOL_2_LP.to_string(),
                        asset_decimals: vec![DECIMAL_PLACES, DECIMAL_PLACES],
                        assets: vec![
                            coin(POOL_2_WHALE_AFTER_SWAP_2, DENOM_UWHALE),
                            coin(POOL_2_LUNA_AFTER_SWAP_2, DENOM_ULUNA)
                        ],
                        pool_type: PoolType::ConstantProduct,
                        pool_fees: pool_fees_2.clone(),
                        status: PoolStatus::default(),
                    }
                );
            },
        );

    suite
        .query_balance(&fee_collector_addr.to_string(), DENOM_UWHALE, |result| {
            // no additional funds were sent to the fee collector
            assert_eq!(
                result.unwrap(),
                coin(FEE_COLLECTOR_WHALE_AFTER_SWAP_2, DENOM_UWHALE)
            );
        })
        .query_balance(&fee_collector_addr.to_string(), DENOM_ULUNA, |result| {
            // no additional funds were sent to the fee collector
            assert_eq!(
                result.unwrap(),
                coin(FEE_COLLECTOR_LUNA_AFTER_SWAP_1, DENOM_ULUNA)
            );
        });

    // let\'s do swaps in o.uluna.uusd.pool.1 and verify the fees are channeled correctly
    suite
        .swap(
            &creator,
            DENOM_UUSD.to_string(),
            None,
            Some(Decimal::percent(SLIPPAGE_PERCENT_21)),
            None,
            O_ULUNA_UUSD_POOL_1_ID.to_string(),
            vec![coin(3_000u128, DENOM_ULUNA.to_string())],
            |result| {
                result.unwrap();
            },
        )
        .query_pools(
            Some(O_ULUNA_UUSD_POOL_1_ID.to_string()),
            None,
            None,
            |result| {
                let response = result.unwrap();
                let pool_info = response.pools[0].pool_info.clone();

                // swapped 3000 uluna
                // fees:
                // swap -> 209 (~7%)
                // protocol -> 299 (~10%)
                // burn ->  89 (~3%)
                // total_fees = 597, of which 209 stay in the pool (for LPs).
                // Going out of the pool is 299 (fee collector) + 89 (burned)

                assert_eq!(
                    pool_info,
                    PoolInfo {
                        pool_identifier: O_ULUNA_UUSD_POOL_1_ID.to_string(),
                        asset_denoms: vec![DENOM_ULUNA.to_string(), DENOM_UUSD.to_string()],
                        lp_denom: O_ULUNA_UUSD_POOL_1_LP.to_string(),
                        asset_decimals: vec![DECIMAL_PLACES, DECIMAL_PLACES],
                        assets: vec![
                            coin(POOL_3_LUNA_AFTER_SWAP_1, DENOM_ULUNA),
                            coin(POOL_3_UUSD_AFTER_SWAP_1, DENOM_UUSD)
                        ],
                        pool_type: PoolType::ConstantProduct,
                        pool_fees: pool_fees_1.clone(),
                        status: PoolStatus::default(),
                    }
                );
            },
        );

    suite.query_balance(&fee_collector_addr.to_string(), DENOM_UUSD, |result| {
        // 3000 of pool creation fees + 299 from the previous swap
        assert_eq!(
            result.unwrap(),
            coin(FEE_COLLECTOR_UUSD_AFTER_SWAP_1, DENOM_UUSD)
        );
    });

    suite
        .swap(
            &creator,
            DENOM_ULUNA.to_string(),
            None,
            Some(Decimal::percent(SLIPPAGE_PERCENT_21)),
            None,
            O_ULUNA_UUSD_POOL_1_ID.to_string(),
            vec![coin(1_500u128, DENOM_UUSD.to_string())],
            |result| {
                result.unwrap();
            },
        )
        .query_pools(
            Some(O_ULUNA_UUSD_POOL_1_ID.to_string()),
            None,
            None,
            |result| {
                let response = result.unwrap();
                let pool_info = response.pools[0].pool_info.clone();

                // swapped 1500 uusd
                // fees:
                // swap -> 105 (~7%)
                // protocol -> 150 (~10%)
                // burn ->  45 (~3%)
                // total_fees = 300, of which 105 stay in the pool (for LPs).
                // Going out of the pool is 150 (fee collector) + 45 (burned)

                assert_eq!(
                    pool_info,
                    PoolInfo {
                        pool_identifier: O_ULUNA_UUSD_POOL_1_ID.to_string(),
                        asset_denoms: vec![DENOM_ULUNA.to_string(), DENOM_UUSD.to_string()],
                        lp_denom: O_ULUNA_UUSD_POOL_1_LP.to_string(),
                        asset_decimals: vec![DECIMAL_PLACES, DECIMAL_PLACES],
                        assets: vec![
                            coin(POOL_3_LUNA_AFTER_SWAP_2, DENOM_ULUNA),
                            coin(POOL_3_UUSD_AFTER_SWAP_2, DENOM_UUSD)
                        ],
                        pool_type: PoolType::ConstantProduct,
                        pool_fees: pool_fees_1.clone(),
                        status: PoolStatus::default(),
                    }
                );
            },
        );

    suite
        .query_balance(&fee_collector_addr.to_string(), DENOM_UWHALE, |result| {
            // no additional funds were sent to the fee collector
            assert_eq!(
                result.unwrap(),
                coin(FEE_COLLECTOR_WHALE_AFTER_SWAP_2, DENOM_UWHALE)
            );
        })
        .query_balance(&fee_collector_addr.to_string(), DENOM_ULUNA, |result| {
            // 99 + 150
            assert_eq!(
                result.unwrap(),
                coin(FEE_COLLECTOR_LUNA_AFTER_SWAP_2_POOL_3, DENOM_ULUNA)
            );
        })
        .query_balance(&fee_collector_addr.to_string(), DENOM_UUSD, |result| {
            // FEE_COLLECTOR_UUSD_AFTER_SWAP_1 (3299) remains as this swap does not involve UUSD for protocol fees
            assert_eq!(
                result.unwrap(),
                coin(FEE_COLLECTOR_UUSD_AFTER_SWAP_1, DENOM_UUSD)
            );
        });

    // query pools with pagination // This comment might be from old code, the assertion below is for pool manager balances
    suite.query_all_balances(&pool_manager_addr.to_string(), |result| {
        let balances = result.unwrap();
        assert_eq!(
            balances,
            vec![
                Coin {
                    denom: O_ULUNA_UUSD_POOL_1_LP.to_string(),
                    amount: Uint128::from(CONTRACT_LP_BALANCE_1K),
                },
                Coin {
                    denom: WHALE_ULUNA_POOL_1_LP.to_string(),
                    amount: Uint128::from(CONTRACT_LP_BALANCE_1K),
                },
                Coin {
                    denom: O_WHALE_ULUNA_POOL_2_LP.to_string(),
                    amount: Uint128::from(CONTRACT_LP_BALANCE_1K),
                },
                Coin {
                    denom: DENOM_ULUNA.to_string(),
                    amount: Uint128::from(POOL_MANAGER_ULUNA_FINAL),
                },
                Coin {
                    denom: DENOM_UUSD.to_string(),
                    amount: Uint128::from(POOL_MANAGER_UUSD_FINAL),
                },
                Coin {
                    denom: DENOM_UWHALE.to_string(),
                    amount: Uint128::from(POOL_MANAGER_UWHALE_FINAL),
                },
            ]
        );
    });

    // swap via the router now
    let swap_operations = vec![
        SwapOperation::MantraSwap {
            token_in_denom: DENOM_UWHALE.to_string(),
            token_out_denom: DENOM_ULUNA.to_string(),
            pool_identifier: O_WHALE_ULUNA_POOL_2_ID.to_string(),
        },
        SwapOperation::MantraSwap {
            token_in_denom: DENOM_ULUNA.to_string(),
            token_out_denom: DENOM_UUSD.to_string(),
            pool_identifier: O_ULUNA_UUSD_POOL_1_ID.to_string(),
        },
    ];

    suite
        .execute_swap_operations(
            &creator,
            swap_operations,
            None,
            None,
            Some(Decimal::percent(SLIPPAGE_PERCENT_21)),
            vec![coin(5_000u128, DENOM_UWHALE.to_string())],
            |result| {
                result.unwrap();
            },
        )
        .query_pools(
            Some(O_WHALE_ULUNA_POOL_1_ID.to_string()),
            None,
            None,
            |result| {
                let response = result.unwrap();
                let pool_info = response.pools[0].pool_info.clone();

                // this should have not changed since last time, since we didn\'t touch this pool
                assert_eq!(
                    pool_info,
                    PoolInfo {
                        pool_identifier: O_WHALE_ULUNA_POOL_1_ID.to_string(),
                        asset_denoms: vec![DENOM_UWHALE.to_string(), DENOM_ULUNA.to_string()],
                        lp_denom: WHALE_ULUNA_POOL_1_LP.to_string(),
                        asset_decimals: vec![DECIMAL_PLACES, DECIMAL_PLACES],
                        assets: vec![
                            coin(POOL_1_WHALE_AFTER_SWAP_2, DENOM_UWHALE),
                            coin(POOL_1_LUNA_AFTER_SWAP_2, DENOM_ULUNA)
                        ],
                        pool_type: PoolType::ConstantProduct,
                        pool_fees: pool_fees_1.clone(),
                        status: PoolStatus::default(),
                    }
                );
            },
        )
        .query_pools(
            Some(O_WHALE_ULUNA_POOL_2_ID.to_string()),
            None,
            None,
            |result| {
                let response = result.unwrap();
                let pool_info = response.pools[0].pool_info.clone();

                // the swap above was:
                // SwapComputation { return_amount: Uint128(3988),
                // spread_amount: Uint128(25), swap_fee_amount: Uint128(747),
                // protocol_fee_amount: Uint128(0), burn_fee_amount: Uint128(249) }

                assert_eq!(
                    pool_info,
                    PoolInfo {
                        pool_identifier: O_WHALE_ULUNA_POOL_2_ID.to_string(),
                        asset_denoms: vec![DENOM_UWHALE.to_string(), DENOM_ULUNA.to_string()],
                        lp_denom: O_WHALE_ULUNA_POOL_2_LP.to_string(),
                        asset_decimals: vec![DECIMAL_PLACES, DECIMAL_PLACES],
                        assets: vec![
                            coin(POOL_2_WHALE_AFTER_ROUTER_SWAP, DENOM_UWHALE),
                            coin(POOL_2_LUNA_AFTER_ROUTER_SWAP, DENOM_ULUNA)
                        ],
                        pool_type: PoolType::ConstantProduct,
                        pool_fees: pool_fees_2.clone(),
                        status: PoolStatus::default(),
                    }
                );
            },
        )
        .query_pools(
            Some(O_ULUNA_UUSD_POOL_1_ID.to_string()),
            None,
            None,
            |result| {
                let response = result.unwrap();
                let pool_info = response.pools[0].pool_info.clone();

                // the swap above was:
                // SwapComputation { return_amount: Uint128(3169),
                // spread_amount: Uint128(16), swap_fee_amount: Uint128(277),
                // protocol_fee_amount: Uint128(396), burn_fee_amount: Uint128(118) }

                assert_eq!(
                    pool_info,
                    PoolInfo {
                        pool_identifier: O_ULUNA_UUSD_POOL_1_ID.to_string(),
                        asset_denoms: vec![DENOM_ULUNA.to_string(), DENOM_UUSD.to_string()],
                        lp_denom: O_ULUNA_UUSD_POOL_1_LP.to_string(),
                        asset_decimals: vec![DECIMAL_PLACES, DECIMAL_PLACES],
                        assets: vec![
                            coin(POOL_3_LUNA_AFTER_ROUTER_SWAP, DENOM_ULUNA),
                            coin(POOL_3_UUSD_AFTER_ROUTER_SWAP, DENOM_UUSD)
                        ],
                        pool_type: PoolType::ConstantProduct,
                        pool_fees: pool_fees_1.clone(),
                        status: PoolStatus::default(),
                    }
                );
            },
        );

    suite
        .query_all_balances(&fee_collector_addr.to_string(), |result| {
            let balances = result.unwrap();
            assert_eq!(
                balances,
                vec![
                    // the o.whale.uluna.pool.2 doesn\'t have protocol fees, hence no luna was accrued
                    // in the last swap
                    Coin {
                        denom: DENOM_ULUNA.to_string(),
                        amount: Uint128::from(FEE_COLLECTOR_LUNA_AFTER_SWAP_2_POOL_3),
                    },
                    Coin {
                        denom: DENOM_UUSD.to_string(),
                        amount: Uint128::from(3_000u128 + 299u128 + 396u128), // Pool creation + swap1_pool3 + swap2_pool3(via router)
                    },
                    Coin {
                        denom: DENOM_UWHALE.to_string(),
                        amount: Uint128::from(FEE_COLLECTOR_WHALE_AFTER_SWAP_2),
                    },
                ]
            );
        })
        .query_all_balances(&pool_manager_addr.to_string(), |result| {
            let balances = result.unwrap();
            assert_eq!(
                balances,
                vec![
                    Coin {
                        denom: O_ULUNA_UUSD_POOL_1_LP.to_string(),
                        amount: Uint128::from(CONTRACT_LP_BALANCE_1K),
                    },
                    Coin {
                        denom: WHALE_ULUNA_POOL_1_LP.to_string(),
                        amount: Uint128::from(CONTRACT_LP_BALANCE_1K),
                    },
                    Coin {
                        denom: O_WHALE_ULUNA_POOL_2_LP.to_string(),
                        amount: Uint128::from(CONTRACT_LP_BALANCE_1K),
                    },
                    Coin {
                        denom: DENOM_ULUNA.to_string(),
                        amount: Uint128::from(3_003_570u128),
                    },
                    Coin {
                        denom: DENOM_UUSD.to_string(),
                        amount: Uint128::from(995_035u128),
                    },
                    Coin {
                        denom: DENOM_UWHALE.to_string(),
                        amount: Uint128::from(2_003_440u128),
                    },
                ]
            );
        });
}
