use cosmwasm_std::{coin, Coin, Decimal, Uint128};
use mantra_common_testing::multi_test::stargate_mock::StargateMock;
use mantra_dex_std::fee::{Fee, PoolFee};
use mantra_dex_std::pool_manager::{PoolInfo, PoolStatus, PoolType, SwapOperation};

use crate::tests::integration::common_constants::{
    DECIMAL_PLACES as ASSET_DECIMALS_6, DENOM_ULUNA as ULUNA_DENOM, DENOM_UOM as UOM_DENOM,
    DENOM_UOSMO as UOSMO_DENOM, DENOM_UUSD as UUSD_DENOM, DENOM_UWHALE as UWHALE_DENOM,
    INITIAL_BALANCE as INITIAL_BALANCE_1B, INITIAL_BALANCE_PLUS_ONE as INITIAL_BALANCE_1B_PLUS_1,
    LIQUIDITY_AMOUNT as LIQUIDITY_1M, POOL_CREATION_FEE as POOL_CREATION_FEE_UUSD_AMOUNT,
    PROTOCOL_FEE_RATIO_1_1000, STARGATE_MOCK_UOM_AMOUNT, SWAP_AMOUNT as SWAP_AMOUNT_1K,
    SWAP_FEE_RATIO_1_10000,
};
use crate::tests::suite::TestingSuite;
use crate::ContractError;

// ========== Initial Balances ==========
const INITIAL_BALANCE_1T: u128 = 1_000_000_000_000u128;

// ========== Pool Creation & Liquidity ==========
const LIQUIDITY_500K: u128 = 500_000u128;
const LIQUIDITY_200K: u128 = 200_000u128;

// ========== Swap Amounts ==========
const SWAP_AMOUNT_2K: u128 = 2_000u128;
const SWAP_AMOUNT_3K: u128 = 3_000u128;
const SWAP_AMOUNT_1_5K: u128 = 1_500u128;
const SWAP_AMOUNT_5K: u128 = 5_000u128;

// ========== Fee Ratios & Percentages ==========
const SLIPPAGE_PERCENT_60: u64 = 60;
const SLIPPAGE_PERCENT_20: u64 = 20;
const SLIPPAGE_PERCENT_21: u64 = 21;
const FEE_PERCENT_10: u64 = 10;
const FEE_PERCENT_7: u64 = 7;
const FEE_PERCENT_3: u64 = 3;
const FEE_PERCENT_15: u64 = 15;
const FEE_PERCENT_5: u64 = 5;

// ========== Pool Identifiers & LP Denoms ==========
const WHALE_ULUNA_POOL_LABEL: &str = "whale.uluna";
const O_WHALE_ULUNA_ID: &str = "o.whale.uluna";
const WHALE_ULUNA_POOL_1_LABEL: &str = "whale.uluna.pool.1";
const O_WHALE_ULUNA_POOL_1_ID: &str = "o.whale.uluna.pool.1";
const O_WHALE_ULUNA_POOL_1_LP: &str = "factory/mantra1zwv6feuzhy6a9wekh96cd57lsarmqlwxdypdsplw6zhfncqw6ftqlydlr9/o.whale.uluna.pool.1.LP";
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
const POOL_1_WHALE_AFTER_SWAP_1: u128 = 1_001_000u128;
const POOL_1_LUNA_AFTER_SWAP_1: u128 = 999_070u128;
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
const FEE_COLLECTOR_UUSD_FINAL: u128 = 3_000u128 + 299u128 + 396u128; // Pool creation + swap1_pool3 + swap2_pool3(via router)

// ========== Fee Calculation Constants ==========
// Pool 1 Swap 1 (1000 uwhale)
const SWAP_FEE_STAY_IN_POOL_1: u128 = 69u128; // ~7% of SWAP_AMOUNT_1K
const PROTOCOL_FEE_1: u128 = 99u128; // ~10% of SWAP_AMOUNT_1K
const BURN_FEE_1: u128 = 29u128; // ~3% of SWAP_AMOUNT_1K
const TOTAL_FEES_1: u128 = 197u128; // SWAP_FEE_STAY_IN_POOL_1 + PROTOCOL_FEE_1 + BURN_FEE_1

// Pool 1 Swap 2 (2000 uluna)
const SWAP_FEE_STAY_IN_POOL_2: u128 = 139u128; // ~7% of SWAP_AMOUNT_2K
const PROTOCOL_FEE_2: u128 = 199u128; // ~10% of SWAP_AMOUNT_2K
const BURN_FEE_2: u128 = 59u128; // ~3% of SWAP_AMOUNT_2K
const TOTAL_FEES_2: u128 = 397u128; // SWAP_FEE_STAY_IN_POOL_2 + PROTOCOL_FEE_2 + BURN_FEE_2

// Pool 2 Swap 1 (1000 uwhale)
const SWAP_FEE_STAY_IN_POOL_POOL2_1: u128 = 149u128; // ~15% of SWAP_AMOUNT_1K
const PROTOCOL_FEE_POOL2_1: u128 = 0u128; // 0%
const BURN_FEE_POOL2_1: u128 = 49u128; // ~5% of SWAP_AMOUNT_1K
const TOTAL_FEES_POOL2_1: u128 = 198u128; // SWAP_FEE_STAY_IN_POOL_POOL2_1 + PROTOCOL_FEE_POOL2_1 + BURN_FEE_POOL2_1

// Pool 2 Swap 2 (2000 uluna)
const SWAP_FEE_STAY_IN_POOL_POOL2_2: u128 = 299u128; // ~15% of SWAP_AMOUNT_2K
const PROTOCOL_FEE_POOL2_2: u128 = 0u128; // 0%
const BURN_FEE_POOL2_2: u128 = 99u128; // ~5% of SWAP_AMOUNT_2K
const TOTAL_FEES_POOL2_2: u128 = 398u128; // SWAP_FEE_STAY_IN_POOL_POOL2_2 + PROTOCOL_FEE_POOL2_2 + BURN_FEE_POOL2_2

// Pool 3 Swap 1 (3000 uluna)
const SWAP_FEE_STAY_IN_POOL_POOL3_1: u128 = 209u128; // ~7% of SWAP_AMOUNT_3K
const PROTOCOL_FEE_POOL3_1: u128 = 299u128; // ~10% of SWAP_AMOUNT_3K
const BURN_FEE_POOL3_1: u128 = 89u128; // ~3% of SWAP_AMOUNT_3K
const TOTAL_FEES_POOL3_1: u128 = 597u128; // SWAP_FEE_STAY_IN_POOL_POOL3_1 + PROTOCOL_FEE_POOL3_1 + BURN_FEE_POOL3_1

// Pool 3 Swap 2 (1500 uusd)
const SWAP_FEE_STAY_IN_POOL_POOL3_2: u128 = 105u128; // ~7% of SWAP_AMOUNT_1_5K
const PROTOCOL_FEE_POOL3_2: u128 = 150u128; // ~10% of SWAP_AMOUNT_1_5K
const BURN_FEE_POOL3_2: u128 = 45u128; // ~3% of SWAP_AMOUNT_1_5K
const TOTAL_FEES_POOL3_2: u128 = 300u128; // SWAP_FEE_STAY_IN_POOL_POOL3_2 + PROTOCOL_FEE_POOL3_2 + BURN_FEE_POOL3_2

// Expected Pool Manager balances AFTER the final router swap in provide_liquidity_to_multiple_pools_check_fees
const PM_ULUNA_BAL_AFTER_ROUTER: u128 = 3_003_570;
const PM_UUSD_BAL_AFTER_ROUTER: u128 = 995_035;
const PM_UWHALE_BAL_AFTER_ROUTER: u128 = 2_003_440;

#[test]
fn assert_slippage_tolerance_correct_xyk() {
    let mut suite = TestingSuite::default_with_balances(
        vec![
            coin(INITIAL_BALANCE_1B_PLUS_1, UWHALE_DENOM.to_string()),
            coin(INITIAL_BALANCE_1B, ULUNA_DENOM.to_string()),
            coin(INITIAL_BALANCE_1B_PLUS_1, UUSD_DENOM.to_string()),
            coin(INITIAL_BALANCE_1B_PLUS_1, UOM_DENOM.to_string()),
        ],
        StargateMock::new(vec![coin(STARGATE_MOCK_UOM_AMOUNT, UOM_DENOM.to_string())]),
    );
    let creator = suite.creator();
    // Asset infos with uwhale and uluna

    let asset_infos = vec![UWHALE_DENOM.to_string(), ULUNA_DENOM.to_string()];

    // Protocol fee is 0.01% and swap fee is 0.02% and burn fee is 0%
    let pool_fees = PoolFee {
        protocol_fee: Fee {
            share: Decimal::from_ratio(PROTOCOL_FEE_RATIO_1_1000.0, PROTOCOL_FEE_RATIO_1_1000.1),
        },
        swap_fee: Fee {
            share: Decimal::from_ratio(SWAP_FEE_RATIO_1_10000.0, SWAP_FEE_RATIO_1_10000.1),
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
        vec![ASSET_DECIMALS_6, ASSET_DECIMALS_6],
        pool_fees,
        PoolType::ConstantProduct {},
        Some(WHALE_ULUNA_POOL_LABEL.to_string()),
        vec![
            coin(POOL_CREATION_FEE_UUSD_AMOUNT, UUSD_DENOM),
            coin(STARGATE_MOCK_UOM_AMOUNT, UOM_DENOM),
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
                denom: UWHALE_DENOM.to_string(),
                amount: Uint128::from(LIQUIDITY_500K),
            },
            Coin {
                denom: ULUNA_DENOM.to_string(),
                amount: Uint128::from(LIQUIDITY_1M),
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
        Some(Decimal::percent(SLIPPAGE_PERCENT_60)),
        None,
        None,
        vec![
            Coin {
                denom: ULUNA_DENOM.to_string(),
                amount: Uint128::from(LIQUIDITY_1M),
            },
            Coin {
                denom: UWHALE_DENOM.to_string(),
                amount: Uint128::from(LIQUIDITY_200K),
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
            coin(INITIAL_BALANCE_1T, UWHALE_DENOM.to_string()),
            coin(INITIAL_BALANCE_1T, ULUNA_DENOM.to_string()),
            coin(INITIAL_BALANCE_1T, UOSMO_DENOM.to_string()),
            coin(INITIAL_BALANCE_1T, UUSD_DENOM.to_string()),
            coin(INITIAL_BALANCE_1T, UOM_DENOM.to_string()),
        ],
        StargateMock::new(vec![coin(STARGATE_MOCK_UOM_AMOUNT, UOM_DENOM.to_string())]),
    );
    let creator = suite.creator();
    let other = suite.senders[1].clone();

    // Asset denoms with uwhale and uluna
    let asset_denoms_1 = vec![UWHALE_DENOM.to_string(), ULUNA_DENOM.to_string()];
    let asset_denoms_2 = vec![ULUNA_DENOM.to_string(), UUSD_DENOM.to_string()];

    let pool_fees_1 = PoolFee {
        protocol_fee: Fee {
            share: Decimal::percent(FEE_PERCENT_10),
        },
        swap_fee: Fee {
            share: Decimal::percent(FEE_PERCENT_7),
        },
        burn_fee: Fee {
            share: Decimal::percent(FEE_PERCENT_3),
        },
        extra_fees: vec![],
    };

    let pool_fees_2 = PoolFee {
        protocol_fee: Fee {
            share: Decimal::zero(),
        },
        swap_fee: Fee {
            share: Decimal::percent(FEE_PERCENT_15),
        },
        burn_fee: Fee {
            share: Decimal::percent(FEE_PERCENT_5),
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
            vec![ASSET_DECIMALS_6, ASSET_DECIMALS_6],
            pool_fees_1.clone(),
            PoolType::ConstantProduct,
            Some(WHALE_ULUNA_POOL_1_LABEL.to_string()),
            vec![
                coin(POOL_CREATION_FEE_UUSD_AMOUNT, UUSD_DENOM),
                coin(STARGATE_MOCK_UOM_AMOUNT, UOM_DENOM),
            ],
            |result| {
                result.unwrap();
            },
        )
        .create_pool(
            &creator,
            asset_denoms_1.clone(), // Changed from asset_denoms_1 to asset_denoms_1.clone() to avoid move error
            vec![ASSET_DECIMALS_6, ASSET_DECIMALS_6],
            pool_fees_2.clone(),
            PoolType::ConstantProduct,
            Some(WHALE_ULUNA_POOL_2_LABEL.to_string()),
            vec![
                coin(POOL_CREATION_FEE_UUSD_AMOUNT, UUSD_DENOM),
                coin(STARGATE_MOCK_UOM_AMOUNT, UOM_DENOM),
            ],
            |result| {
                result.unwrap();
            },
        )
        .create_pool(
            &creator,
            asset_denoms_2,
            vec![ASSET_DECIMALS_6, ASSET_DECIMALS_6],
            pool_fees_1.clone(),
            PoolType::ConstantProduct,
            Some(ULUNA_UUSD_POOL_1_LABEL.to_string()),
            vec![
                coin(POOL_CREATION_FEE_UUSD_AMOUNT, UUSD_DENOM),
                coin(STARGATE_MOCK_UOM_AMOUNT, UOM_DENOM),
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
        UUSD_DENOM.to_string(),
        |result| {
            assert_eq!(
                result.unwrap().amount,
                Uint128::new(3 * POOL_CREATION_FEE_UUSD_AMOUNT)
            );
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
                    denom: UWHALE_DENOM.to_string(),
                    amount: Uint128::from(LIQUIDITY_1M),
                },
                Coin {
                    denom: ULUNA_DENOM.to_string(),
                    amount: Uint128::from(LIQUIDITY_1M),
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
                    denom: UWHALE_DENOM.to_string(),
                    amount: Uint128::from(LIQUIDITY_1M),
                },
                Coin {
                    denom: ULUNA_DENOM.to_string(),
                    amount: Uint128::from(LIQUIDITY_1M),
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
                    denom: UWHALE_DENOM.to_string(),
                    amount: Uint128::from(LIQUIDITY_1M),
                },
                Coin {
                    denom: ULUNA_DENOM.to_string(),
                    amount: Uint128::from(LIQUIDITY_1M),
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
                    denom: ULUNA_DENOM.to_string(),
                    amount: Uint128::from(LIQUIDITY_1M),
                },
                Coin {
                    denom: UUSD_DENOM.to_string(),
                    amount: Uint128::from(LIQUIDITY_1M),
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
                        denom: O_WHALE_ULUNA_POOL_1_LP.to_string(),
                        amount: Uint128::from(CONTRACT_LP_BALANCE_1K),
                    },
                    Coin {
                        denom: O_WHALE_ULUNA_POOL_2_LP.to_string(),
                        amount: Uint128::from(CONTRACT_LP_BALANCE_1K),
                    },
                    Coin {
                        denom: ULUNA_DENOM.to_string(),
                        amount: Uint128::from(POOL_MANAGER_ULUNA_BALANCE_3M),
                    },
                    Coin {
                        denom: UUSD_DENOM.to_string(),
                        amount: Uint128::from(POOL_MANAGER_UUSD_BALANCE_1M),
                    },
                    Coin {
                        denom: UWHALE_DENOM.to_string(),
                        amount: Uint128::from(POOL_MANAGER_UWHALE_BALANCE_2M),
                    },
                ]
            );
        });

    // let\'s do swaps in o.whale.uluna.pool.1 and verify the fees are channeled correctly
    suite
        .swap(
            &creator,
            ULUNA_DENOM.to_string(),
            None,
            Some(Decimal::percent(SLIPPAGE_PERCENT_20)),
            None,
            O_WHALE_ULUNA_POOL_1_ID.to_string(),
            vec![coin(SWAP_AMOUNT_1K, UWHALE_DENOM.to_string())],
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
                        asset_denoms: vec![UWHALE_DENOM.to_string(), ULUNA_DENOM.to_string()],
                        lp_denom: O_WHALE_ULUNA_POOL_1_LP.to_string(),
                        asset_decimals: vec![ASSET_DECIMALS_6, ASSET_DECIMALS_6],
                        assets: vec![
                            coin(POOL_1_WHALE_AFTER_SWAP_1, UWHALE_DENOM),
                            coin(POOL_1_LUNA_AFTER_SWAP_1, ULUNA_DENOM)
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
        .query_balance(&fee_collector_addr.to_string(), ULUNA_DENOM, |result| {
            assert_eq!(
                result.unwrap(),
                coin(FEE_COLLECTOR_LUNA_AFTER_SWAP_1, ULUNA_DENOM)
            );
        })
        .swap(
            &creator,
            UWHALE_DENOM.to_string(),
            None,
            Some(Decimal::percent(SLIPPAGE_PERCENT_21)),
            None,
            O_WHALE_ULUNA_POOL_1_ID.to_string(),
            vec![coin(SWAP_AMOUNT_2K, ULUNA_DENOM.to_string())],
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
                        asset_denoms: vec![UWHALE_DENOM.to_string(), ULUNA_DENOM.to_string()],
                        lp_denom: O_WHALE_ULUNA_POOL_1_LP.to_string(),
                        asset_decimals: vec![ASSET_DECIMALS_6, ASSET_DECIMALS_6],
                        assets: vec![
                            coin(POOL_1_WHALE_AFTER_SWAP_2, UWHALE_DENOM),
                            coin(POOL_1_LUNA_AFTER_SWAP_2, ULUNA_DENOM)
                        ],
                        pool_type: PoolType::ConstantProduct,
                        pool_fees: pool_fees_1.clone(),
                        status: PoolStatus::default(),
                    }
                );
            },
        );

    suite
        .query_balance(&fee_collector_addr.to_string(), UWHALE_DENOM, |result| {
            assert_eq!(
                result.unwrap(),
                coin(FEE_COLLECTOR_WHALE_AFTER_SWAP_2, UWHALE_DENOM)
            );
        })
        .query_balance(&fee_collector_addr.to_string(), ULUNA_DENOM, |result| {
            assert_eq!(
                result.unwrap(),
                coin(FEE_COLLECTOR_LUNA_AFTER_SWAP_1, ULUNA_DENOM)
            );
        });

    // let\'s do swaps in o.whale.uluna.pool.2 and verify the fees are channeled correctly
    suite
        .swap(
            &creator,
            ULUNA_DENOM.to_string(),
            None,
            Some(Decimal::percent(SLIPPAGE_PERCENT_21)),
            None,
            O_WHALE_ULUNA_POOL_2_ID.to_string(),
            vec![coin(SWAP_AMOUNT_1K, UWHALE_DENOM.to_string())],
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
                        asset_denoms: vec![UWHALE_DENOM.to_string(), ULUNA_DENOM.to_string()],
                        lp_denom: O_WHALE_ULUNA_POOL_2_LP.to_string(),
                        asset_decimals: vec![ASSET_DECIMALS_6, ASSET_DECIMALS_6],
                        assets: vec![
                            coin(POOL_2_WHALE_AFTER_SWAP_1, UWHALE_DENOM),
                            coin(POOL_2_LUNA_AFTER_SWAP_1, ULUNA_DENOM)
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
            UWHALE_DENOM.to_string(),
            None,
            Some(Decimal::percent(SLIPPAGE_PERCENT_21)),
            None,
            O_WHALE_ULUNA_POOL_2_ID.to_string(),
            vec![coin(SWAP_AMOUNT_2K, ULUNA_DENOM.to_string())],
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
                        asset_denoms: vec![UWHALE_DENOM.to_string(), ULUNA_DENOM.to_string()],
                        lp_denom: O_WHALE_ULUNA_POOL_2_LP.to_string(),
                        asset_decimals: vec![ASSET_DECIMALS_6, ASSET_DECIMALS_6],
                        assets: vec![
                            coin(POOL_2_WHALE_AFTER_SWAP_2, UWHALE_DENOM),
                            coin(POOL_2_LUNA_AFTER_SWAP_2, ULUNA_DENOM)
                        ],
                        pool_type: PoolType::ConstantProduct,
                        pool_fees: pool_fees_2.clone(),
                        status: PoolStatus::default(),
                    }
                );
            },
        );

    suite
        .query_balance(&fee_collector_addr.to_string(), UWHALE_DENOM, |result| {
            // no additional funds were sent to the fee collector
            assert_eq!(
                result.unwrap(),
                coin(FEE_COLLECTOR_WHALE_AFTER_SWAP_2, UWHALE_DENOM)
            );
        })
        .query_balance(&fee_collector_addr.to_string(), ULUNA_DENOM, |result| {
            // no additional funds were sent to the fee collector
            assert_eq!(
                result.unwrap(),
                coin(FEE_COLLECTOR_LUNA_AFTER_SWAP_1, ULUNA_DENOM)
            );
        });

    // let\'s do swaps in o.uluna.uusd.pool.1 and verify the fees are channeled correctly
    suite
        .swap(
            &creator,
            UUSD_DENOM.to_string(),
            None,
            Some(Decimal::percent(SLIPPAGE_PERCENT_21)),
            None,
            O_ULUNA_UUSD_POOL_1_ID.to_string(),
            vec![coin(SWAP_AMOUNT_3K, ULUNA_DENOM.to_string())],
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
                        asset_denoms: vec![ULUNA_DENOM.to_string(), UUSD_DENOM.to_string()],
                        lp_denom: O_ULUNA_UUSD_POOL_1_LP.to_string(),
                        asset_decimals: vec![ASSET_DECIMALS_6, ASSET_DECIMALS_6],
                        assets: vec![
                            coin(POOL_3_LUNA_AFTER_SWAP_1, ULUNA_DENOM),
                            coin(POOL_3_UUSD_AFTER_SWAP_1, UUSD_DENOM)
                        ],
                        pool_type: PoolType::ConstantProduct,
                        pool_fees: pool_fees_1.clone(),
                        status: PoolStatus::default(),
                    }
                );
            },
        );

    suite.query_balance(&fee_collector_addr.to_string(), UUSD_DENOM, |result| {
        // 3000 of pool creation fees + 299 from the previous swap
        assert_eq!(
            result.unwrap(),
            coin(FEE_COLLECTOR_UUSD_AFTER_SWAP_1, UUSD_DENOM)
        );
    });

    suite
        .swap(
            &creator,
            ULUNA_DENOM.to_string(),
            None,
            Some(Decimal::percent(SLIPPAGE_PERCENT_21)),
            None,
            O_ULUNA_UUSD_POOL_1_ID.to_string(),
            vec![coin(SWAP_AMOUNT_1_5K, UUSD_DENOM.to_string())],
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
                        asset_denoms: vec![ULUNA_DENOM.to_string(), UUSD_DENOM.to_string()],
                        lp_denom: O_ULUNA_UUSD_POOL_1_LP.to_string(),
                        asset_decimals: vec![ASSET_DECIMALS_6, ASSET_DECIMALS_6],
                        assets: vec![
                            coin(POOL_3_LUNA_AFTER_SWAP_2, ULUNA_DENOM),
                            coin(POOL_3_UUSD_AFTER_SWAP_2, UUSD_DENOM)
                        ],
                        pool_type: PoolType::ConstantProduct,
                        pool_fees: pool_fees_1.clone(),
                        status: PoolStatus::default(),
                    }
                );
            },
        );

    suite
        .query_balance(&fee_collector_addr.to_string(), UWHALE_DENOM, |result| {
            // no additional funds were sent to the fee collector
            assert_eq!(
                result.unwrap(),
                coin(FEE_COLLECTOR_WHALE_AFTER_SWAP_2, UWHALE_DENOM)
            );
        })
        .query_balance(&fee_collector_addr.to_string(), ULUNA_DENOM, |result| {
            // 99 + 150
            assert_eq!(
                result.unwrap(),
                coin(FEE_COLLECTOR_LUNA_AFTER_SWAP_2_POOL_3, ULUNA_DENOM)
            );
        })
        .query_balance(&fee_collector_addr.to_string(), UUSD_DENOM, |result| {
            // FEE_COLLECTOR_UUSD_AFTER_SWAP_1 (3299) remains as this swap does not involve UUSD for protocol fees
            assert_eq!(
                result.unwrap(),
                coin(FEE_COLLECTOR_UUSD_AFTER_SWAP_1, UUSD_DENOM)
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
                    denom: O_WHALE_ULUNA_POOL_1_LP.to_string(),
                    amount: Uint128::from(CONTRACT_LP_BALANCE_1K),
                },
                Coin {
                    denom: O_WHALE_ULUNA_POOL_2_LP.to_string(),
                    amount: Uint128::from(CONTRACT_LP_BALANCE_1K),
                },
                Coin {
                    denom: ULUNA_DENOM.to_string(),
                    amount: Uint128::from(POOL_MANAGER_ULUNA_FINAL),
                },
                Coin {
                    denom: UUSD_DENOM.to_string(),
                    amount: Uint128::from(POOL_MANAGER_UUSD_FINAL),
                },
                Coin {
                    denom: UWHALE_DENOM.to_string(),
                    amount: Uint128::from(POOL_MANAGER_UWHALE_FINAL),
                },
            ]
        );
    });

    // swap via the router now
    let swap_operations = vec![
        SwapOperation::MantraSwap {
            token_in_denom: UWHALE_DENOM.to_string(),
            token_out_denom: ULUNA_DENOM.to_string(),
            pool_identifier: O_WHALE_ULUNA_POOL_2_ID.to_string(),
        },
        SwapOperation::MantraSwap {
            token_in_denom: ULUNA_DENOM.to_string(),
            token_out_denom: UUSD_DENOM.to_string(),
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
            vec![coin(SWAP_AMOUNT_5K, UWHALE_DENOM.to_string())],
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
                        asset_denoms: vec![UWHALE_DENOM.to_string(), ULUNA_DENOM.to_string()],
                        lp_denom: O_WHALE_ULUNA_POOL_1_LP.to_string(),
                        asset_decimals: vec![ASSET_DECIMALS_6, ASSET_DECIMALS_6],
                        assets: vec![
                            coin(POOL_1_WHALE_AFTER_SWAP_2, UWHALE_DENOM),
                            coin(POOL_1_LUNA_AFTER_SWAP_2, ULUNA_DENOM)
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
                        asset_denoms: vec![UWHALE_DENOM.to_string(), ULUNA_DENOM.to_string()],
                        lp_denom: O_WHALE_ULUNA_POOL_2_LP.to_string(),
                        asset_decimals: vec![ASSET_DECIMALS_6, ASSET_DECIMALS_6],
                        assets: vec![
                            coin(POOL_2_WHALE_AFTER_ROUTER_SWAP, UWHALE_DENOM),
                            coin(POOL_2_LUNA_AFTER_ROUTER_SWAP, ULUNA_DENOM)
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
                        asset_denoms: vec![ULUNA_DENOM.to_string(), UUSD_DENOM.to_string()],
                        lp_denom: O_ULUNA_UUSD_POOL_1_LP.to_string(),
                        asset_decimals: vec![ASSET_DECIMALS_6, ASSET_DECIMALS_6],
                        assets: vec![
                            coin(POOL_3_LUNA_AFTER_ROUTER_SWAP, ULUNA_DENOM),
                            coin(POOL_3_UUSD_AFTER_ROUTER_SWAP, UUSD_DENOM)
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
                        denom: ULUNA_DENOM.to_string(),
                        amount: Uint128::from(FEE_COLLECTOR_LUNA_AFTER_SWAP_2_POOL_3),
                    },
                    Coin {
                        denom: UUSD_DENOM.to_string(),
                        amount: Uint128::from(FEE_COLLECTOR_UUSD_FINAL),
                    },
                    Coin {
                        denom: UWHALE_DENOM.to_string(),
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
                        denom: O_WHALE_ULUNA_POOL_1_LP.to_string(),
                        amount: Uint128::from(CONTRACT_LP_BALANCE_1K),
                    },
                    Coin {
                        denom: O_WHALE_ULUNA_POOL_2_LP.to_string(),
                        amount: Uint128::from(CONTRACT_LP_BALANCE_1K),
                    },
                    Coin {
                        denom: ULUNA_DENOM.to_string(),
                        amount: Uint128::from(PM_ULUNA_BAL_AFTER_ROUTER),
                    },
                    Coin {
                        denom: UUSD_DENOM.to_string(),
                        amount: Uint128::from(PM_UUSD_BAL_AFTER_ROUTER),
                    },
                    Coin {
                        denom: UWHALE_DENOM.to_string(),
                        amount: Uint128::from(PM_UWHALE_BAL_AFTER_ROUTER),
                    },
                ]
            );
        });
}
