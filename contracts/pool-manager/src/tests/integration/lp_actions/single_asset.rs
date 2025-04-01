use cosmwasm_std::{coin, Coin, Decimal, StdError, Uint128};
use mantra_common_testing::multi_test::stargate_mock::StargateMock;
use mantra_dex_std::fee::{Fee, PoolFee};
use mantra_dex_std::lp_common::MINIMUM_LIQUIDITY_AMOUNT;
use mantra_dex_std::pool_manager::PoolType;

use crate::tests::suite::TestingSuite;
use crate::ContractError;

#[test]
fn provide_liquidity_with_single_asset() {
    let mut suite = TestingSuite::default_with_balances(
        vec![
            coin(10_000_000u128, "uwhale".to_string()),
            coin(10_000_000u128, "uluna".to_string()),
            coin(10_000_000u128, "uosmo".to_string()),
            coin(10_000u128, "uusd".to_string()),
            coin(10_000u128, "uom".to_string()),
        ],
        StargateMock::new(vec![coin(8888u128, "uom".to_string())]),
    );
    let creator = suite.creator();
    let other = suite.senders[1].clone();
    let _unauthorized = suite.senders[2].clone();

    // Asset denoms with uwhale and uluna
    let asset_denoms = vec!["uwhale".to_string(), "uluna".to_string()];

    let pool_fees = PoolFee {
        protocol_fee: Fee {
            share: Decimal::percent(1),
        },
        swap_fee: Fee {
            share: Decimal::percent(1),
        },
        burn_fee: Fee {
            share: Decimal::zero(),
        },
        extra_fees: vec![],
    };

    // Create a pool
    suite.instantiate_default().add_one_epoch().create_pool(
        &creator,
        asset_denoms,
        vec![6u8, 6u8],
        pool_fees,
        PoolType::ConstantProduct,
        Some("whale.uluna".to_string()),
        vec![coin(1000, "uusd"), coin(8888, "uom")],
        |result| {
            result.unwrap();
        },
    );

    let contract_addr = suite.pool_manager_addr.clone();
    let lp_denom = suite.get_lp_denom("o.whale.uluna".to_string());

    // Let's try to add liquidity
    suite
            .provide_liquidity(
                &creator,
                "o.whale.uluna".to_string(),
                None,
                None,
                None,
                None,
                None,
                vec![],
                |result| {
                    let err = result.unwrap_err().downcast::<ContractError>().unwrap();

                    match err {
                        ContractError::EmptyAssets => {}
                        _ => panic!("Wrong error type, should return ContractError::EmptyAssets"),
                    }
                },
            )
            .provide_liquidity(
                &creator,
                "o.whale.uluna".to_string(),
                None,
                None,
                None,
                None,
                None,
                vec![Coin {
                    denom: "uosmo".to_string(),
                    amount: Uint128::from(1_000_000u128),
                }],
                |result| {
                    let err = result.unwrap_err().downcast::<ContractError>().unwrap();

                    match err {
                        ContractError::AssetMismatch => {}
                        _ => panic!("Wrong error type, should return ContractError::AssetMismatch"),
                    }
                },
            )
            .provide_liquidity(
                &creator,
                "o.whale.uluna".to_string(),
                None,
                None,
                None,
                None,
                None,
                vec![Coin {
                    denom: "uwhale".to_string(),
                    amount: Uint128::from(1_000_000u128),
                }],
                |result| {
                    let err = result.unwrap_err().downcast::<ContractError>().unwrap();

                    match err {
                        ContractError::EmptyPoolForSingleSideLiquidityProvision {} => {}
                        _ => panic!(
                            "Wrong error type, should return ContractError::EmptyPoolForSingleSideLiquidityProvision"
                        ),
                    }
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
                    denom: "uosmo".to_string(),
                    amount: Uint128::from(1_000_000u128),
                },
                Coin {
                    denom: "uwhale".to_string(),
                    amount: Uint128::from(1_000_000u128),
                },
            ],
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();

                match err {
                    ContractError::AssetMismatch => {}
                    _ => panic!("Wrong error type, should return ContractError::AssetMismatch"),
                }
            },
        )
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
                result.unwrap();
            },
        )
        .query_all_balances(&creator.to_string(), |result| {
            let balances = result.unwrap();

            assert!(balances.iter().any(|coin| {
                coin.denom == lp_denom && coin.amount == Uint128::from(999_000u128)
            }));
        })
        // contract should have 1_000 LP shares (MINIMUM_LIQUIDITY_AMOUNT)
        .query_all_balances(&contract_addr.to_string(), |result| {
            let balances = result.unwrap();
            // check that balances has 999_000 factory/mantra1zwv6feuzhy6a9wekh96cd57lsarmqlwxdypdsplw6zhfncqw6ftqlydlr9/o.whale.uluna.LP
            assert!(balances.iter().any(|coin| {
                coin.denom == lp_denom.clone() && coin.amount == MINIMUM_LIQUIDITY_AMOUNT
            }));
        });

    // now let's provide liquidity with a single asset
    suite
        .provide_liquidity(
            &other,
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
                    denom: "uwhale".to_string(),
                    amount: Uint128::from(1_000_000u128),
                },
            ],
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();
                assert_eq!(
                    err,
                    ContractError::Std(StdError::generic_err("Spread limit exceeded"))
                );
            },
        )
        .provide_liquidity(
            &other,
            "o.whale.uluna".to_string(),
            None,
            None,
            None,
            Some(Decimal::percent(50)),
            None,
            vec![
                Coin {
                    denom: "uwhale".to_string(),
                    amount: Uint128::from(10_000u128),
                },
                Coin {
                    denom: "uwhale".to_string(),
                    amount: Uint128::from(10_000u128),
                },
            ],
            |result| {
                result.unwrap();
            },
        )
        .query_all_balances(&other.to_string(), |result| {
            let balances = result.unwrap();
            println!("{:?}", balances);
            // the new minted LP tokens should be 10_000 * 1_000_000 / 1_000_000 = ~10_000 lp shares - slippage
            // of swapping half of one asset to the other
            assert!(balances
                .iter()
                .any(|coin| { coin.denom == lp_denom && coin.amount == Uint128::from(9_798u128) }));
        })
        .query_all_balances(&contract_addr.to_string(), |result| {
            let balances = result.unwrap();
            // check that balances has 999_000 factory/mantra1zwv6feuzhy6a9wekh96cd57lsarmqlwxdypdsplw6zhfncqw6ftqlydlr9/o.whale.uluna.LP
            assert!(balances.iter().any(|coin| {
                coin.denom == lp_denom.clone() && coin.amount == MINIMUM_LIQUIDITY_AMOUNT
            }));
        });

    suite
        .query_lp_supply("o.whale.uluna".to_string(), |res| {
            // total amount of LP tokens issued should be 1_009_798 = 999_000 to the first LP,
            // 1_000 to the contract, and 9_798 to the second, single-side LP
            assert_eq!(res.unwrap().amount, Uint128::from(1_009_798u128));
        })
        .query_pools(Some("o.whale.uluna".to_string()), None, None, |res| {
            let response = res.unwrap();

            let whale = response.pools[0]
                .pool_info
                .assets
                .iter()
                .find(|coin| coin.denom == "uwhale".to_string())
                .unwrap();
            let luna = response.pools[0]
                .pool_info
                .assets
                .iter()
                .find(|coin| coin.denom == "uluna".to_string())
                .unwrap();

            assert_eq!(whale.amount, Uint128::from(1_020_000u128));
            assert_eq!(luna.amount, Uint128::from(999_901u128));
        });

    let pool_manager = suite.pool_manager_addr.clone();
    // let's withdraw both LPs
    suite
        .query_all_balances(&pool_manager.to_string(), |result| {
            let balances = result.unwrap();
            assert_eq!(
                balances,
                vec![
                    Coin {
                        denom: lp_denom.clone(),
                        amount: Uint128::from(1_000u128),
                    },
                    Coin {
                        denom: "uluna".to_string(),
                        amount: Uint128::from(999_901u128),
                    },
                    Coin {
                        denom: "uwhale".to_string(),
                        amount: Uint128::from(1_020_000u128),
                    },
                ]
            );
        })
        .query_all_balances(&creator.to_string(), |result| {
            let balances = result.unwrap();
            assert_eq!(
                balances,
                vec![
                    Coin {
                        denom: lp_denom.clone(),
                        amount: Uint128::from(999_000u128),
                    },
                    Coin {
                        denom: "uluna".to_string(),
                        amount: Uint128::from(9_000_000u128),
                    },
                    Coin {
                        denom: "uom".to_string(),
                        amount: Uint128::from(10_000u128 - 8_888u128),
                    },
                    Coin {
                        denom: "uosmo".to_string(),
                        amount: Uint128::from(10_000_000u128),
                    },
                    Coin {
                        denom: "uusd".to_string(),
                        amount: Uint128::from(9_000u128),
                    },
                    Coin {
                        denom: "uwhale".to_string(),
                        amount: Uint128::from(9_000_000u128),
                    },
                ]
            );
        })
        .withdraw_liquidity(
            &creator,
            "o.whale.uluna".to_string(),
            vec![Coin {
                denom: lp_denom.clone(),
                amount: Uint128::from(999_000u128),
            }],
            |result| {
                result.unwrap();
            },
        )
        .query_all_balances(&creator.to_string(), |result| {
            let balances = result.unwrap();
            assert_eq!(
                balances,
                vec![
                    Coin {
                        denom: "uluna".to_string(),
                        amount: Uint128::from(9_989_208u128),
                    },
                    Coin {
                        denom: "uom".to_string(),
                        amount: Uint128::from(10_000u128 - 8_888u128),
                    },
                    Coin {
                        denom: "uosmo".to_string(),
                        amount: Uint128::from(10_000_000u128),
                    },
                    Coin {
                        denom: "uusd".to_string(),
                        amount: Uint128::from(9_000u128),
                    },
                    Coin {
                        denom: "uwhale".to_string(),
                        amount: Uint128::from(10_009_092u128),
                    },
                ]
            );
        });

    let fee_collector = suite.fee_collector_addr.clone();

    suite
        .query_all_balances(&other.to_string(), |result| {
            let balances = result.unwrap();
            assert_eq!(
                balances,
                vec![
                    Coin {
                        denom: lp_denom.clone(),
                        amount: Uint128::from(9_798u128),
                    },
                    Coin {
                        denom: "uluna".to_string(),
                        amount: Uint128::from(10_000_000u128),
                    },
                    Coin {
                        denom: "uom".to_string(),
                        amount: Uint128::from(10_000u128),
                    },
                    Coin {
                        denom: "uosmo".to_string(),
                        amount: Uint128::from(10_000_000u128),
                    },
                    Coin {
                        denom: "uusd".to_string(),
                        amount: Uint128::from(10_000u128),
                    },
                    Coin {
                        denom: "uwhale".to_string(),
                        amount: Uint128::from(9_980_000u128),
                    },
                ]
            );
        })
        .withdraw_liquidity(
            &other,
            "o.whale.uluna".to_string(),
            vec![Coin {
                denom: lp_denom.clone(),
                amount: Uint128::from(9_798u128),
            }],
            |result| {
                result.unwrap();
            },
        )
        .query_all_balances(&other.to_string(), |result| {
            let balances = result.unwrap();
            assert_eq!(
                balances,
                vec![
                    Coin {
                        denom: "uluna".to_string(),
                        amount: Uint128::from(10_009_702u128),
                    },
                    Coin {
                        denom: "uom".to_string(),
                        amount: Uint128::from(10_000u128),
                    },
                    Coin {
                        denom: "uosmo".to_string(),
                        amount: Uint128::from(10_000_000u128),
                    },
                    Coin {
                        denom: "uusd".to_string(),
                        amount: Uint128::from(10_000u128),
                    },
                    Coin {
                        denom: "uwhale".to_string(),
                        amount: Uint128::from(9_989_897u128),
                    },
                ]
            );
        })
        .query_all_balances(&fee_collector.to_string(), |result| {
            let balances = result.unwrap();
            // check that the fee collector got the luna fees for the single-side lp
            // plus the pool creation fee
            assert_eq!(
                balances,
                vec![
                    Coin {
                        denom: "uluna".to_string(),
                        amount: Uint128::from(99u128),
                    },
                    Coin {
                        denom: "uusd".to_string(),
                        amount: Uint128::from(1_000u128),
                    },
                ]
            );
        })
        .query_all_balances(&contract_addr.to_string(), |result| {
            let balances = result.unwrap();
            // the contract should have some dust left, and 1000 LP tokens
            assert_eq!(
                balances,
                vec![
                    Coin {
                        denom: lp_denom.clone(),
                        amount: Uint128::from(1_000u128),
                    },
                    Coin {
                        denom: "uluna".to_string(),
                        amount: Uint128::from(991u128),
                    },
                    Coin {
                        denom: "uwhale".to_string(),
                        amount: Uint128::from(1_011u128),
                    },
                ]
            );
        });
}

#[test]
fn provide_liquidity_with_single_asset_to_third_party() {
    let mut suite = TestingSuite::default_with_balances(
        vec![
            coin(10_000_000u128, "uwhale".to_string()),
            coin(10_000_000u128, "uluna".to_string()),
            coin(10_000_000u128, "uosmo".to_string()),
            coin(10_000u128, "uusd".to_string()),
            coin(10_000u128, "uom".to_string()),
        ],
        StargateMock::new(vec![coin(8888u128, "uom".to_string())]),
    );
    let creator = suite.creator();
    let other = suite.senders[1].clone();
    let another = suite.senders[2].clone();

    // Asset denoms with uwhale and uluna
    let asset_denoms = vec!["uwhale".to_string(), "uluna".to_string()];

    let pool_fees = PoolFee {
        protocol_fee: Fee {
            share: Decimal::percent(1),
        },
        swap_fee: Fee {
            share: Decimal::percent(1),
        },
        burn_fee: Fee {
            share: Decimal::zero(),
        },
        extra_fees: vec![],
    };

    // Create a pool
    suite.instantiate_default().add_one_epoch().create_pool(
        &creator,
        asset_denoms,
        vec![6u8, 6u8],
        pool_fees,
        PoolType::ConstantProduct,
        Some("whale.uluna".to_string()),
        vec![coin(1000, "uusd"), coin(8888, "uom")],
        |result| {
            result.unwrap();
        },
    );

    let contract_addr = suite.pool_manager_addr.clone();
    let lp_denom = suite.get_lp_denom("o.whale.uluna".to_string());

    // Let's try to add liquidity
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
                result.unwrap();
            },
        )
        .query_all_balances(&creator.to_string(), |result| {
            let balances = result.unwrap();

            assert!(balances.iter().any(|coin| {
                coin.denom == lp_denom && coin.amount == Uint128::from(999_000u128)
            }));
        })
        // contract should have 1_000 LP shares (MINIMUM_LIQUIDITY_AMOUNT)
        .query_all_balances(&contract_addr.to_string(), |result| {
            let balances = result.unwrap();
            // check that balances has 999_000 factory/mantra1zwv6feuzhy6a9wekh96cd57lsarmqlwxdypdsplw6zhfncqw6ftqlydlr9/o.whale.uluna.LP
            assert!(balances.iter().any(|coin| {
                coin.denom == lp_denom.clone() && coin.amount == MINIMUM_LIQUIDITY_AMOUNT
            }));
        });

    // now let's provide liquidity with a single asset
    suite
        .provide_liquidity(
            &other,
            "o.whale.uluna".to_string(),
            None,
            None,
            None,
            Some(Decimal::percent(50)),
            Some(another.to_string()),
            vec![
                Coin {
                    denom: "uwhale".to_string(),
                    amount: Uint128::from(1_000u128),
                },
                Coin {
                    denom: "uwhale".to_string(),
                    amount: Uint128::from(1_000u128),
                },
            ],
            |result| {
                result.unwrap();
            },
        )
        .query_all_balances(&other.to_string(), |result| {
            let balances = result.unwrap();
            //other should not have any LP tokens as it provided for another
            assert!(!balances.iter().any(|coin| coin.denom == lp_denom));
        })
        .query_all_balances(&another.to_string(), |result| {
            let balances = result.unwrap();
            assert!(balances
                .iter()
                .any(|coin| { coin.denom == lp_denom && coin.amount == Uint128::from(981u128) }));
        })
        .query_all_balances(&contract_addr.to_string(), |result| {
            let balances = result.unwrap();
            // check that balances has 999_000 factory/mantra1zwv6feuzhy6a9wekh96cd57lsarmqlwxdypdsplw6zhfncqw6ftqlydlr9/o.whale.uluna.LP
            assert!(balances.iter().any(|coin| {
                coin.denom == lp_denom.clone() && coin.amount == MINIMUM_LIQUIDITY_AMOUNT
            }));
        });

    suite.provide_liquidity(
        &other,
        "o.whale.uluna".to_string(),
        Some(86_400u64),
        None,
        None,
        Some(Decimal::percent(50)),
        Some(another.to_string()),
        vec![
            Coin {
                denom: "uwhale".to_string(),
                amount: Uint128::from(1_000u128),
            },
            Coin {
                denom: "uwhale".to_string(),
                amount: Uint128::from(1_000u128),
            },
        ],
        |result| {
            let err = result.unwrap_err().downcast::<ContractError>().unwrap();
            match err {
                ContractError::Unauthorized => {}
                _ => panic!("Wrong error type, should return ContractError::Unauthorized"),
            }
        },
    );
}

#[test]
fn provide_liquidity_with_single_asset_edge_case() {
    let mut suite = TestingSuite::default_with_balances(
        vec![
            coin(1_000_000u128, "uwhale".to_string()),
            coin(1_000_000u128, "uluna".to_string()),
            coin(1_000_000u128, "uosmo".to_string()),
            coin(10_000u128, "uusd".to_string()),
            coin(10_000u128, "uom".to_string()),
        ],
        StargateMock::new(vec![coin(8888u128, "uom".to_string())]),
    );
    let creator = suite.creator();
    let other = suite.senders[1].clone();
    let _unauthorized = suite.senders[2].clone();

    // Asset denoms with uwhale and uluna
    let asset_denoms = vec!["uwhale".to_string(), "uluna".to_string()];

    let pool_fees = PoolFee {
        protocol_fee: Fee {
            share: Decimal::percent(15),
        },
        swap_fee: Fee {
            share: Decimal::percent(5),
        },
        burn_fee: Fee {
            share: Decimal::zero(),
        },
        extra_fees: vec![],
    };

    // Create a pool
    suite.instantiate_default().add_one_epoch().create_pool(
        &creator,
        asset_denoms,
        vec![6u8, 6u8],
        pool_fees,
        PoolType::ConstantProduct,
        Some("whale.uluna".to_string()),
        vec![coin(1000, "uusd"), coin(8888, "uom")],
        |result| {
            result.unwrap();
        },
    );

    let contract_addr = suite.pool_manager_addr.clone();

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
                    amount: Uint128::from(1_100u128),
                },
                Coin {
                    denom: "uluna".to_string(),
                    amount: Uint128::from(1_100u128),
                },
            ],
            |result| {
                result.unwrap();
            },
        )
        .query_all_balances(&contract_addr.to_string(), |result| {
            let balances = result.unwrap();
            println!("contract_addr {:?}", balances);
        });

    // now let's provide liquidity with a single asset
    suite
        .provide_liquidity(
            &other,
            "o.whale.uluna".to_string(),
            None,
            None,
            None,
            Some(Decimal::percent(50)),
            None,
            vec![Coin {
                denom: "uwhale".to_string(),
                amount: Uint128::from(1_760u128),
            }],
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();
                assert_eq!(
                    err,
                    ContractError::Std(StdError::generic_err("Spread limit exceeded"))
                );
            },
        )
        .provide_liquidity(
            &other,
            "o.whale.uluna".to_string(),
            None,
            None,
            None,
            Some(Decimal::percent(50)),
            None,
            vec![Coin {
                denom: "uwhale".to_string(),
                amount: Uint128::from(10_000u128),
            }],
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();
                assert_eq!(
                    err,
                    ContractError::Std(StdError::generic_err("Spread limit exceeded"))
                );
            },
        )
        .provide_liquidity(
            &other,
            "o.whale.uluna".to_string(),
            None,
            None,
            None,
            Some(Decimal::percent(50)),
            None,
            vec![Coin {
                denom: "uwhale".to_string(),
                amount: Uint128::from(1_000u128),
            }],
            |result| {
                result.unwrap();
            },
        );
}
