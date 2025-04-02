use cosmwasm_std::{coin, Coin, Decimal, Uint128};
use mantra_common_testing::multi_test::stargate_mock::StargateMock;
use mantra_dex_std::fee::{Fee, PoolFee};
use mantra_dex_std::pool_manager::{PoolInfo, PoolStatus, PoolType};

use crate::tests::suite::TestingSuite;
use crate::ContractError;

#[test]
fn assert_slippage_tolerance_correct_xyk() {
    let mut suite = TestingSuite::default_with_balances(
        vec![
            coin(1_000_000_001u128, "uwhale".to_string()),
            coin(1_000_000_000u128, "uluna".to_string()),
            coin(1_000_000_001u128, "uusd".to_string()),
            coin(1_000_000_001u128, "uom".to_string()),
        ],
        StargateMock::new(vec![coin(8888u128, "uom".to_string())]),
    );
    let creator = suite.creator();
    // Asset infos with uwhale and uluna

    let asset_infos = vec!["uwhale".to_string(), "uluna".to_string()];

    // Protocol fee is 0.01% and swap fee is 0.02% and burn fee is 0%
    let pool_fees = PoolFee {
        protocol_fee: Fee {
            share: Decimal::from_ratio(1u128, 1000u128),
        },
        swap_fee: Fee {
            share: Decimal::from_ratio(1u128, 10_000_u128),
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
        vec![6u8, 6u8],
        pool_fees,
        PoolType::ConstantProduct {},
        Some("whale.uluna".to_string()),
        vec![coin(1000, "uusd"), coin(8888, "uom")],
        |result| {
            result.unwrap();
        },
    );

    // first liquidity provided as uwhale,uluna
    suite.provide_liquidity(
        &creator,
        "o.whale.uluna".to_string(),
        None,
        None,
        None,
        None,
        None,
        vec![
            Coin {
                denom: "uwhale".to_string(),
                amount: Uint128::from(500_000u128),
            },
            Coin {
                denom: "uluna".to_string(),
                amount: Uint128::from(1_000_000u128),
            },
        ],
        |result| {
            result.unwrap();
        },
    );

    // second liquidity provided as uluna,uwhale
    suite.provide_liquidity(
        &creator,
        "o.whale.uluna".to_string(),
        None,
        None,
        Some(Decimal::percent(60)),
        None,
        None,
        vec![
            Coin {
                denom: "uluna".to_string(),
                amount: Uint128::from(1_000_000u128),
            },
            Coin {
                denom: "uwhale".to_string(),
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
            coin(1_000_000_000_000u128, "uwhale".to_string()),
            coin(1_000_000_000_000u128, "uluna".to_string()),
            coin(1_000_000_000_000u128, "uosmo".to_string()),
            coin(1_000_000_000_000u128, "uusd".to_string()),
            coin(1_000_000_000_000u128, "uom".to_string()),
        ],
        StargateMock::new(vec![coin(8888u128, "uom".to_string())]),
    );
    let creator = suite.creator();
    let other = suite.senders[1].clone();

    // Asset denoms with uwhale and uluna
    let asset_denoms_1 = vec!["uwhale".to_string(), "uluna".to_string()];
    let asset_denoms_2 = vec!["uluna".to_string(), "uusd".to_string()];

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
            vec![6u8, 6u8],
            pool_fees_1.clone(),
            PoolType::ConstantProduct,
            Some("whale.uluna.pool.1".to_string()),
            vec![coin(1000, "uusd"), coin(8888, "uom")],
            |result| {
                result.unwrap();
            },
        )
        .create_pool(
            &creator,
            asset_denoms_1,
            vec![6u8, 6u8],
            pool_fees_2.clone(),
            PoolType::ConstantProduct,
            Some("whale.uluna.pool.2".to_string()),
            vec![coin(1000, "uusd"), coin(8888, "uom")],
            |result| {
                result.unwrap();
            },
        )
        .create_pool(
            &creator,
            asset_denoms_2,
            vec![6u8, 6u8],
            pool_fees_1.clone(),
            PoolType::ConstantProduct,
            Some("uluna.uusd.pool.1".to_string()),
            vec![coin(1000, "uusd"), coin(8888, "uom")],
            |result| {
                result.unwrap();
            },
        );

    let pool_manager_addr = suite.pool_manager_addr.clone();
    let fee_collector_addr = suite.fee_collector_addr.clone();

    // after creating 3 pools, the fee collector should have 3_000 uusd in fees
    suite.query_balance(
        &fee_collector_addr.to_string(),
        "uusd".to_string(),
        |result| {
            assert_eq!(result.unwrap().amount, Uint128::new(3 * 1_000u128));
        },
    );

    // let's provide liquidity with two assets
    suite
            .provide_liquidity(
                &creator,
                "o.whale.uluna".to_string(),
                None,
                None,
                None,
                None,
                None,
                vec![
                    Coin {
                        denom: "uwhale".to_string(),
                        amount: Uint128::from(1_000_000u128),
                    },
                    Coin {
                        denom: "uluna".to_string(),
                        amount: Uint128::from(1_000_000u128),
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
                "o.whale.uluna.pool.1".to_string(),
                None,
                None,
                None,
                None,
                None,
                vec![
                    Coin {
                        denom: "uwhale".to_string(),
                        amount: Uint128::from(1_000_000u128),
                    },
                    Coin {
                        denom: "uluna".to_string(),
                        amount: Uint128::from(1_000_000u128),
                    },
                ],
                |result| {
                    result.unwrap();
                },
            )
            .provide_liquidity(
                &other,
                "o.whale.uluna.pool.2".to_string(),
                None,
                None,
                None,
                None,
                None,
                vec![
                    Coin {
                        denom: "uwhale".to_string(),
                        amount: Uint128::from(1_000_000u128),
                    },
                    Coin {
                        denom: "uluna".to_string(),
                        amount: Uint128::from(1_000_000u128),
                    },
                ],
                |result| {
                    result.unwrap();
                },
            )
            .provide_liquidity(
                &other,
                "o.uluna.uusd.pool.1".to_string(),
                None,
                None,
                None,
                None,
                None,
                vec![
                    Coin {
                        denom: "uluna".to_string(),
                        amount: Uint128::from(1_000_000u128),
                    },
                    Coin {
                        denom: "uusd".to_string(),
                        amount: Uint128::from(1_000_000u128),
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
                            denom: "factory/mantra1zwv6feuzhy6a9wekh96cd57lsarmqlwxdypdsplw6zhfncqw6ftqlydlr9/o.uluna.uusd.pool.1.LP".to_string(),
                            amount: Uint128::from(1_000u128),
                        },
                        Coin {
                            denom: "factory/mantra1zwv6feuzhy6a9wekh96cd57lsarmqlwxdypdsplw6zhfncqw6ftqlydlr9/o.whale.uluna.pool.1.LP".to_string(),
                            amount: Uint128::from(1_000u128),
                        },
                        Coin {
                            denom: "factory/mantra1zwv6feuzhy6a9wekh96cd57lsarmqlwxdypdsplw6zhfncqw6ftqlydlr9/o.whale.uluna.pool.2.LP".to_string(),
                            amount: Uint128::from(1_000u128),
                        },
                        Coin {
                            denom: "uluna".to_string(),
                            amount: Uint128::from(3_000_000u128),
                        },
                        Coin {
                            denom: "uusd".to_string(),
                            amount: Uint128::from(1_000_000u128),
                        },
                        Coin {
                            denom: "uwhale".to_string(),
                            amount: Uint128::from(2_000_000u128),
                        },
                    ]
                );
            });

    // let's do swaps in o.whale.uluna.pool.1 and verify the fees are channeled correctly
    suite
            .swap(
                &creator,
                "uluna".to_string(),
                None,
                Some(Decimal::percent(20)),
                None,
                "o.whale.uluna.pool.1".to_string(),
                vec![coin(1000u128, "uwhale".to_string())],
                |result| {
                    result.unwrap();
                },
            )
            .query_pools(Some("o.whale.uluna.pool.1".to_string()), None, None, |result| {
                let response = result.unwrap();
                let pool_info = response.pools[0].pool_info.clone();

                // swapped 1000 uwhale
                // fees:
                // swap -> 69 (~7%)
                // protocol -> 99 (~10%)
                // burn ->  29 (~3%)
                // total_fees = 197, of which 69 stay in the pool (for LPs).
                // Going out of the pool is 99 (fee collector) + 29 (burned)

                assert_eq!(pool_info, PoolInfo {
                    pool_identifier: "o.whale.uluna.pool.1".to_string(),
                    asset_denoms: vec!["uwhale".to_string(), "uluna".to_string()],
                    lp_denom: "factory/mantra1zwv6feuzhy6a9wekh96cd57lsarmqlwxdypdsplw6zhfncqw6ftqlydlr9/o.whale.uluna.pool.1.LP".to_string(),
                    asset_decimals: vec![6u8, 6u8],
                    assets: vec![coin(1001000, "uwhale"), coin(999070, "uluna")],
                    pool_type: PoolType::ConstantProduct,
                    pool_fees: pool_fees_1.clone(),
                    status: PoolStatus::default(),
                });
            })
        ;

    // verify the fees went to the fee collector
    suite.query_balance(
            &fee_collector_addr.to_string(),
            "uluna",
            |result| {
                assert_eq!(result.unwrap(), coin(99, "uluna"));
            },
        )
            .swap(
                &creator,
                "uwhale".to_string(),
                None,
                Some(Decimal::percent(21)),
                None,
                "o.whale.uluna.pool.1".to_string(),
                vec![coin(2_000u128, "uluna".to_string())],
                |result| {
                    result.unwrap();
                },
            )
            .query_pools(Some("o.whale.uluna.pool.1".to_string()), None, None, |result| {
                let response = result.unwrap();
                let pool_info = response.pools[0].pool_info.clone();

                // swapped 2000 uluna
                // fees:
                // swap -> 139 (~7%)
                // protocol -> 199 (~10%)
                // burn ->  59 (~3%)
                // total_fees = 397, of which 139 stay in the pool (for LPs).
                // Going out of the pool is 199 (fee collector) + 59 (burned)

                assert_eq!(pool_info, PoolInfo {
                    pool_identifier: "o.whale.uluna.pool.1".to_string(),
                    asset_denoms: vec!["uwhale".to_string(), "uluna".to_string()],
                    lp_denom: "factory/mantra1zwv6feuzhy6a9wekh96cd57lsarmqlwxdypdsplw6zhfncqw6ftqlydlr9/o.whale.uluna.pool.1.LP".to_string(),
                    asset_decimals: vec![6u8, 6u8],
                    assets: vec![coin(999_140, "uwhale"), coin(1_001_070, "uluna")],
                    pool_type: PoolType::ConstantProduct,
                    pool_fees: pool_fees_1.clone(),
                    status: PoolStatus::default(),
                });
            })
        ;

    suite
        .query_balance(&fee_collector_addr.to_string(), "uwhale", |result| {
            assert_eq!(result.unwrap(), coin(199, "uwhale"));
        })
        .query_balance(&fee_collector_addr.to_string(), "uluna", |result| {
            assert_eq!(result.unwrap(), coin(99, "uluna"));
        });

    // let's do swaps in o.whale.uluna.pool.2 and verify the fees are channeled correctly
    suite
            .swap(
                &creator,
                "uluna".to_string(),
                None,
                Some(Decimal::percent(21)),
                None,
                "o.whale.uluna.pool.2".to_string(),
                vec![coin(1000u128, "uwhale".to_string())],
                |result| {
                    result.unwrap();
                },
            )
            .query_pools(Some("o.whale.uluna.pool.2".to_string()), None, None, |result| {
                let response = result.unwrap();
                let pool_info = response.pools[0].pool_info.clone();

                // swapped 1000 uwhale
                // fees:
                // swap -> 149 (~15%)
                // protocol -> 0 (0%)
                // burn ->  49 (~5%)
                // total_fees = 198, of which 149 stay in the pool (for LPs).
                // Going out of the pool is 49 (burned)

                assert_eq!(pool_info, PoolInfo {
                    pool_identifier: "o.whale.uluna.pool.2".to_string(),
                    asset_denoms: vec!["uwhale".to_string(), "uluna".to_string()],
                    lp_denom: "factory/mantra1zwv6feuzhy6a9wekh96cd57lsarmqlwxdypdsplw6zhfncqw6ftqlydlr9/o.whale.uluna.pool.2.LP".to_string(),
                    asset_decimals: vec![6u8, 6u8],
                    assets: vec![coin(1001000, "uwhale"), coin(999_150, "uluna")],
                    pool_type: PoolType::ConstantProduct,
                    pool_fees: pool_fees_2.clone(),
                    status: PoolStatus::default(),
                });
            })
        ;

    suite
            .swap(
                &creator,
                "uwhale".to_string(),
                None,
                Some(Decimal::percent(21)),
                None,
                "o.whale.uluna.pool.2".to_string(),
                vec![coin(2_000u128, "uluna".to_string())],
                |result| {
                    result.unwrap();
                },
            )
            .query_pools(Some("o.whale.uluna.pool.2".to_string()), None, None, |result| {
                let response = result.unwrap();
                let pool_info = response.pools[0].pool_info.clone();

                // swapped 2000 uluna
                // fees:
                // swap -> 299 (~15%)
                // protocol -> 0 (0%)
                // burn ->  99 (~5%)
                // total_fees = 398, of which 299 stay in the pool (for LPs).
                // Going out of the pool is 99 (burned)

                assert_eq!(pool_info, PoolInfo {
                    pool_identifier: "o.whale.uluna.pool.2".to_string(),
                    asset_denoms: vec!["uwhale".to_string(), "uluna".to_string()],
                    lp_denom: "factory/mantra1zwv6feuzhy6a9wekh96cd57lsarmqlwxdypdsplw6zhfncqw6ftqlydlr9/o.whale.uluna.pool.2.LP".to_string(),
                    asset_decimals: vec![6u8, 6u8],
                    assets: vec![coin(999_300, "uwhale"), coin(1_001_150, "uluna")],
                    pool_type: PoolType::ConstantProduct,
                    pool_fees: pool_fees_2.clone(),
                    status: PoolStatus::default(),
                });
            });

    suite
        .query_balance(&fee_collector_addr.to_string(), "uwhale", |result| {
            // no additional funds were sent to the fee collector
            assert_eq!(result.unwrap(), coin(199, "uwhale"));
        })
        .query_balance(&fee_collector_addr.to_string(), "uluna", |result| {
            // no additional funds were sent to the fee collector
            assert_eq!(result.unwrap(), coin(99, "uluna"));
        });

    // let's do swaps in o.uluna.uusd.pool.1 and verify the fees are channeled correctly
    suite
            .swap(
                &creator,
                "uusd".to_string(),
                None,
                Some(Decimal::percent(21)),
                None,
                "o.uluna.uusd.pool.1".to_string(),
                vec![coin(3000u128, "uluna".to_string())],
                |result| {
                    result.unwrap();
                },
            )
            .query_pools(Some("o.uluna.uusd.pool.1".to_string()), None, None, |result| {
                let response = result.unwrap();
                let pool_info = response.pools[0].pool_info.clone();

                // swapped 3000 uluna
                // fees:
                // swap -> 209 (~7%)
                // protocol -> 299 (~10%)
                // burn ->  89 (~3%)
                // total_fees = 597, of which 209 stay in the pool (for LPs).
                // Going out of the pool is 299 (fee collector) + 89 (burned)

                assert_eq!(pool_info, PoolInfo {
                    pool_identifier: "o.uluna.uusd.pool.1".to_string(),
                    asset_denoms: vec!["uluna".to_string(), "uusd".to_string()],
                    lp_denom: "factory/mantra1zwv6feuzhy6a9wekh96cd57lsarmqlwxdypdsplw6zhfncqw6ftqlydlr9/o.uluna.uusd.pool.1.LP".to_string(),
                    asset_decimals: vec![6u8, 6u8],
                    assets: vec![coin(1003000, "uluna"), coin(997_218, "uusd")],
                    pool_type: PoolType::ConstantProduct,
                    pool_fees: pool_fees_1.clone(),
                    status: PoolStatus::default(),
                });
            })
        ;

    suite.query_balance(&fee_collector_addr.to_string(), "uusd", |result| {
        // 3000 of pool creation fees + 299 from the previous swap
        assert_eq!(result.unwrap(), coin(3299, "uusd"));
    });

    suite
            .swap(
                &creator,
                "uluna".to_string(),
                None,
                Some(Decimal::percent(21)),
                None,
                "o.uluna.uusd.pool.1".to_string(),
                vec![coin(1_500u128, "uusd".to_string())],
                |result| {
                    result.unwrap();
                },
            )
            .query_pools(Some("o.uluna.uusd.pool.1".to_string()), None, None, |result| {
                let response = result.unwrap();
                let pool_info = response.pools[0].pool_info.clone();

                // swapped 1500 uusd
                // fees:
                // swap -> 105 (~7%)
                // protocol -> 150 (~10%)
                // burn ->  45 (~3%)
                // total_fees = 300, of which 105 stay in the pool (for LPs).
                // Going out of the pool is 150 (fee collector) + 45 (burned)

                assert_eq!(pool_info, PoolInfo {
                    pool_identifier: "o.uluna.uusd.pool.1".to_string(),
                    asset_denoms: vec!["uluna".to_string(), "uusd".to_string()],
                    lp_denom: "factory/mantra1zwv6feuzhy6a9wekh96cd57lsarmqlwxdypdsplw6zhfncqw6ftqlydlr9/o.uluna.uusd.pool.1.LP".to_string(),
                    asset_decimals: vec![6u8, 6u8],
                    assets: vec![coin(1_001_599, "uluna"), coin(998_718, "uusd")],
                    pool_type: PoolType::ConstantProduct,
                    pool_fees: pool_fees_1.clone(),
                    status: PoolStatus::default(),
                });
            })
        ;

    suite
            .query_balance(
                &fee_collector_addr.to_string(),
                "uwhale",
                |result| {
                    // no additional funds were sent to the fee collector
                    assert_eq!(result.unwrap(), coin(199, "uwhale"));
                },
            )
            .query_balance(
                &fee_collector_addr.to_string(),
                "uluna",
                |result| {
                    // 99 + 150
                    assert_eq!(result.unwrap(), coin(249, "uluna"));
                },
            ).query_balance(
            &fee_collector_addr.to_string(),
            "uusd",
            |result| {
                // 99 + 150
                assert_eq!(result.unwrap(), coin(3299, "uusd"));
            },
        )
            .query_all_balances(
                &pool_manager_addr.to_string(),
                |result| {
                    let balances = result.unwrap();
                    assert_eq!(balances, vec![
                        Coin {
                            denom: "factory/mantra1zwv6feuzhy6a9wekh96cd57lsarmqlwxdypdsplw6zhfncqw6ftqlydlr9/o.uluna.uusd.pool.1.LP".to_string(),
                            amount: Uint128::from(1_000u128),
                        },
                        Coin {
                            denom: "factory/mantra1zwv6feuzhy6a9wekh96cd57lsarmqlwxdypdsplw6zhfncqw6ftqlydlr9/o.whale.uluna.pool.1.LP".to_string(),
                            amount: Uint128::from(1_000u128),
                        },
                        Coin {
                            denom: "factory/mantra1zwv6feuzhy6a9wekh96cd57lsarmqlwxdypdsplw6zhfncqw6ftqlydlr9/o.whale.uluna.pool.2.LP".to_string(),
                            amount: Uint128::from(1_000u128),
                        },
                        Coin {
                            denom: "uluna".to_string(),
                            amount: Uint128::from(3_003_819u128),
                        },
                        Coin {
                            denom: "uusd".to_string(),
                            amount: Uint128::from(998_718u128),
                        },
                        Coin {
                            denom: "uwhale".to_string(),
                            amount: Uint128::from(1_998_440u128),
                        },
                    ]);
                },
            );

    // swap via the router now
    let swap_operations = vec![
        mantra_dex_std::pool_manager::SwapOperation::MantraSwap {
            token_in_denom: "uwhale".to_string(),
            token_out_denom: "uluna".to_string(),
            pool_identifier: "o.whale.uluna.pool.2".to_string(),
        },
        mantra_dex_std::pool_manager::SwapOperation::MantraSwap {
            token_in_denom: "uluna".to_string(),
            token_out_denom: "uusd".to_string(),
            pool_identifier: "o.uluna.uusd.pool.1".to_string(),
        },
    ];

    suite.execute_swap_operations(
            &creator,
            swap_operations,
            None,
            None,
            Some(Decimal::percent(21)),
            vec![coin(5_000u128, "uwhale".to_string())],
            |result| {
                result.unwrap();
            },
        ).query_pools(Some("o.whale.uluna.pool.1".to_string()), None, None, |result| {
            let response = result.unwrap();
            let pool_info = response.pools[0].pool_info.clone();

            // this should have not changed since last time, since we didn't touch this pool
            assert_eq!(pool_info, PoolInfo {
                pool_identifier: "o.whale.uluna.pool.1".to_string(),
                asset_denoms: vec!["uwhale".to_string(), "uluna".to_string()],
                lp_denom: "factory/mantra1zwv6feuzhy6a9wekh96cd57lsarmqlwxdypdsplw6zhfncqw6ftqlydlr9/o.whale.uluna.pool.1.LP".to_string(),
                asset_decimals: vec![6u8, 6u8],
                assets: vec![coin(999_140, "uwhale"), coin(1_001_070, "uluna")],
                pool_type: PoolType::ConstantProduct,
                pool_fees: pool_fees_1.clone(),
                status: PoolStatus::default(),
            });
        })
            .query_pools(Some("o.whale.uluna.pool.2".to_string()), None, None, |result| {
                let response = result.unwrap();
                let pool_info = response.pools[0].pool_info.clone();

                // the swap above was:
                // SwapComputation { return_amount: Uint128(3988),
                // spread_amount: Uint128(25), swap_fee_amount: Uint128(747),
                // protocol_fee_amount: Uint128(0), burn_fee_amount: Uint128(249) }

                assert_eq!(pool_info, PoolInfo {
                    pool_identifier: "o.whale.uluna.pool.2".to_string(),
                    asset_denoms: vec!["uwhale".to_string(), "uluna".to_string()],
                    lp_denom: "factory/mantra1zwv6feuzhy6a9wekh96cd57lsarmqlwxdypdsplw6zhfncqw6ftqlydlr9/o.whale.uluna.pool.2.LP".to_string(),
                    asset_decimals: vec![6u8, 6u8],
                    assets: vec![coin(1_004_300, "uwhale"), coin(996_913, "uluna")],
                    pool_type: PoolType::ConstantProduct,
                    pool_fees: pool_fees_2.clone(),
                    status: PoolStatus::default(),
                });
            }).query_pools(Some("o.uluna.uusd.pool.1".to_string()), None, None, |result| {
            let response = result.unwrap();
            let pool_info = response.pools[0].pool_info.clone();

            // the swap above was:
            // SwapComputation { return_amount: Uint128(3169),
            // spread_amount: Uint128(16), swap_fee_amount: Uint128(277),
            // protocol_fee_amount: Uint128(396), burn_fee_amount: Uint128(118) }

            assert_eq!(pool_info, PoolInfo {
                pool_identifier: "o.uluna.uusd.pool.1".to_string(),
                asset_denoms: vec!["uluna".to_string(), "uusd".to_string()],
                lp_denom: "factory/mantra1zwv6feuzhy6a9wekh96cd57lsarmqlwxdypdsplw6zhfncqw6ftqlydlr9/o.uluna.uusd.pool.1.LP".to_string(),
                asset_decimals: vec![6u8, 6u8],
                assets: vec![coin(1_005_587, "uluna"), coin(995_035, "uusd")],
                pool_type: PoolType::ConstantProduct,
                pool_fees: pool_fees_1.clone(),
                status: PoolStatus::default(),
            });
        });

    suite.query_all_balances(
            &fee_collector_addr.to_string(),
            |result| {
                let balances = result.unwrap();
                assert_eq!(balances, vec![
                    // the o.whale.uluna.pool.2 doesn't have protocol fees, hence no luna was accrued
                    // in the last swap
                    Coin {
                        denom: "uluna".to_string(),
                        amount: Uint128::from(249u128),
                    },
                    Coin {
                        denom: "uusd".to_string(),
                        amount: Uint128::from(3_695u128),
                    },
                    Coin {
                        denom: "uwhale".to_string(),
                        amount: Uint128::from(199u128),
                    },
                ]);
            },
        ).query_all_balances(
            &pool_manager_addr.to_string(),
            |result| {
                let balances = result.unwrap();
                assert_eq!(balances, vec![
                    Coin {
                        denom: "factory/mantra1zwv6feuzhy6a9wekh96cd57lsarmqlwxdypdsplw6zhfncqw6ftqlydlr9/o.uluna.uusd.pool.1.LP".to_string(),
                        amount: Uint128::from(1_000u128),
                    },
                    Coin {
                        denom: "factory/mantra1zwv6feuzhy6a9wekh96cd57lsarmqlwxdypdsplw6zhfncqw6ftqlydlr9/o.whale.uluna.pool.1.LP".to_string(),
                        amount: Uint128::from(1_000u128),
                    },
                    Coin {
                        denom: "factory/mantra1zwv6feuzhy6a9wekh96cd57lsarmqlwxdypdsplw6zhfncqw6ftqlydlr9/o.whale.uluna.pool.2.LP".to_string(),
                        amount: Uint128::from(1_000u128),
                    },
                    Coin {
                        denom: "uluna".to_string(),
                        amount: Uint128::from(3_003_570u128),
                    },
                    Coin {
                        denom: "uusd".to_string(),
                        amount: Uint128::from(995_035u128),
                    },
                    Coin {
                        denom: "uwhale".to_string(),
                        amount: Uint128::from(2_003_440u128),
                    },
                ]);
            },
        );

    // query pools with pagination
    suite
        .query_pools(None, None, None, |result| {
            let response = result.unwrap();
            assert_eq!(response.pools.len(), 3);
            assert_eq!(
                response.pools[0].pool_info.pool_identifier,
                "o.uluna.uusd.pool.1"
            );
            assert_eq!(
                response.pools[1].pool_info.pool_identifier,
                "o.whale.uluna.pool.1"
            );
            assert_eq!(
                response.pools[2].pool_info.pool_identifier,
                "o.whale.uluna.pool.2"
            );
        })
        .query_pools(None, None, Some(2), |result| {
            let response = result.unwrap();
            assert_eq!(response.pools.len(), 2);
            assert_eq!(
                response.pools[0].pool_info.pool_identifier,
                "o.uluna.uusd.pool.1"
            );
            assert_eq!(
                response.pools[1].pool_info.pool_identifier,
                "o.whale.uluna.pool.1"
            );
        })
        .query_pools(
            None,
            Some("o.uluna.uusd.pool.1".to_string()),
            None,
            |result| {
                let response = result.unwrap();
                assert_eq!(response.pools.len(), 2);
                assert_eq!(
                    response.pools[0].pool_info.pool_identifier,
                    "o.whale.uluna.pool.1"
                );
                assert_eq!(
                    response.pools[1].pool_info.pool_identifier,
                    "o.whale.uluna.pool.2"
                );
            },
        )
        .query_pools(
            None,
            Some("o.whale.uluna.pool.1".to_string()),
            None,
            |result| {
                let response = result.unwrap();
                assert_eq!(response.pools.len(), 1);
                assert_eq!(
                    response.pools[0].pool_info.pool_identifier,
                    "o.whale.uluna.pool.2"
                );
            },
        );
}
