use cosmwasm_std::{coin, Coin, Decimal, Uint128};
use mantra_common_testing::multi_test::stargate_mock::StargateMock;
use mantra_dex_std::{
    fee::{Fee, PoolFee},
    lp_common::MINIMUM_LIQUIDITY_AMOUNT,
    pool_manager::PoolType,
};

use crate::tests::suite::TestingSuite;

#[test]
fn provide_liquidity_emit_proportional_lp_shares() {
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
                    amount: Uint128::from(10_000u128),
                },
                Coin {
                    denom: "uluna".to_string(),
                    amount: Uint128::from(10_000u128),
                },
            ],
            |result| {
                result.unwrap();
            },
        )
        .query_all_balances(&creator.to_string(), |result| {
            let balances = result.unwrap();

            // user should have 10_000u128 LP shares - MINIMUM_LIQUIDITY_AMOUNT
            assert!(balances
                .iter()
                .any(|coin| { coin.denom == lp_denom && coin.amount == Uint128::from(9_000u128) }));
        })
        // contract should have 1_000 LP shares (MINIMUM_LIQUIDITY_AMOUNT)
        .query_all_balances(&contract_addr.to_string(), |result| {
            let balances = result.unwrap();
            // check that balances has 999_000 factory/mantra1zwv6feuzhy6a9wekh96cd57lsarmqlwxdypdsplw6zhfncqw6ftqlydlr9/o.whale.uluna.LP
            assert!(balances.iter().any(|coin| {
                coin.denom == lp_denom.clone() && coin.amount == MINIMUM_LIQUIDITY_AMOUNT
            }));
        });

    println!(">>>> provide liquidity: 5_000 uwhale, 5_000 uluna");
    // other provides liquidity as well, half of the tokens the creator provided
    // this should result in ~half LP tokens given to other
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
                    amount: Uint128::from(5_000u128),
                },
                Coin {
                    denom: "uluna".to_string(),
                    amount: Uint128::from(5_000u128),
                },
            ],
            |result| {
                result.unwrap();
            },
        )
        .query_all_balances(&other.to_string(), |result| {
            let balances = result.unwrap();
            // user should have 5_000 * 10_000 / 10_000 = 5_000 LP shares
            assert!(balances
                .iter()
                .any(|coin| { coin.denom == lp_denom && coin.amount == Uint128::new(5_000) }));
        });
}

#[test]
fn provide_liquidity_emits_right_lp_shares() {
    let mut suite = TestingSuite::default_with_balances(
        vec![
            coin(1_000_000_000_000u128, "uwhale".to_string()),
            coin(1_000_000_000_000u128, "uluna".to_string()),
            coin(1_000_000_000_000u128, "uosmo".to_string()),
            coin(1_000_000_000_000u128, "uusd".to_string()),
            coin(1_000_000_000_000u128, "uusdc".to_string()),
            coin(1_000_000_000_000u128, "uom".to_string()),
        ],
        StargateMock::new(vec![coin(8888u128, "uom".to_string())]),
    );
    let creator = suite.creator();

    let asset_denoms = vec!["uom".to_string(), "uusdc".to_string()];

    let pool_fees = PoolFee {
        protocol_fee: Fee {
            share: Decimal::zero(),
        },
        swap_fee: Fee {
            share: Decimal::permille(30),
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
        None,
        vec![coin(1000, "uusd"), coin(8888, "uom")],
        |result| {
            result.unwrap();
        },
    );

    let contract_addr = suite.pool_manager_addr.clone();
    let lp_denom = suite.get_lp_denom("p.1".to_string());

    // let's provide liquidity 1.5 om, 1 usdc
    suite
        .provide_liquidity(
            &creator,
            "p.1".to_string(),
            None,
            None,
            None,
            None,
            None,
            vec![
                Coin {
                    denom: "uom".to_string(),
                    amount: Uint128::new(1_500_000u128),
                },
                Coin {
                    denom: "uusdc".to_string(),
                    amount: Uint128::new(1_000_000u128),
                },
            ],
            |result| {
                result.unwrap();
            },
        )
        .query_all_balances(&contract_addr.to_string(), |result| {
            let balances = result.unwrap();
            assert!(balances.iter().any(|coin| {
                coin.denom == lp_denom.clone() && coin.amount == MINIMUM_LIQUIDITY_AMOUNT
            }));
        })
        .query_all_balances(&creator.to_string(), |result| {
            let balances = result.unwrap();
            assert!(balances.iter().any(|coin| {
                coin.denom == lp_denom.clone() && coin.amount == Uint128::new(1_223_744u128)
            }));
        });

    suite
        .provide_liquidity(
            &creator,
            "p.1".to_string(),
            None,
            None,
            None,
            None,
            None,
            vec![
                Coin {
                    denom: "uom".to_string(),
                    amount: Uint128::from(1_500_000u128),
                },
                Coin {
                    denom: "uusdc".to_string(),
                    amount: Uint128::from(1_000_000u128),
                },
            ],
            |result| {
                result.unwrap();
            },
        )
        .query_all_balances(&contract_addr.to_string(), |result| {
            let balances = result.unwrap();
            println!("balances contract: {:?}", balances);
            assert!(balances.iter().any(|coin| {
                coin.denom == lp_denom.clone() && coin.amount == MINIMUM_LIQUIDITY_AMOUNT
            }));

            assert!(balances
                .iter()
                .any(|coin| { coin.denom == "uom" && coin.amount == Uint128::new(3_000_000u128) }));
            assert!(balances.iter().any(|coin| {
                coin.denom == "uusdc" && coin.amount == Uint128::new(2_000_000u128)
            }));
        })
        .query_all_balances(&creator.to_string(), |result| {
            let balances = result.unwrap();

            assert!(balances.iter().any(|coin| {
                coin.denom == lp_denom.clone() && coin.amount == Uint128::new(2_448_488u128)
            }));
        });

    suite
        .withdraw_liquidity(
            &creator,
            "p.1".to_string(),
            vec![Coin {
                denom: lp_denom.clone(),
                amount: Uint128::new(2_448_488u128),
            }],
            |result| {
                result.unwrap();
            },
        )
        .query_all_balances(&contract_addr.to_string(), |result| {
            let balances = result.unwrap();
            assert!(balances.iter().any(|coin| {
                coin.denom == lp_denom.clone() && coin.amount == MINIMUM_LIQUIDITY_AMOUNT
            }));

            assert!(balances
                .iter()
                .any(|coin| { coin.denom == "uom" && coin.amount == Uint128::new(1_225u128) }));
            assert!(balances
                .iter()
                .any(|coin| { coin.denom == "uusdc" && coin.amount == Uint128::new(817u128) }));
        });

    suite
        .provide_liquidity(
            &creator,
            "p.1".to_string(),
            None,
            None,
            None,
            None,
            None,
            vec![
                Coin {
                    denom: "uom".to_string(),
                    amount: Uint128::from(1_500_000_000u128),
                },
                Coin {
                    denom: "uusdc".to_string(),
                    amount: Uint128::from(1_000_000_000u128),
                },
            ],
            |result| {
                result.unwrap();
            },
        )
        .query_all_balances(&contract_addr.to_string(), |result| {
            let balances = result.unwrap();
            assert!(balances.iter().any(|coin| {
                coin.denom == lp_denom.clone() && coin.amount == MINIMUM_LIQUIDITY_AMOUNT
            }));

            assert!(balances.iter().any(|coin| {
                coin.denom == "uom" && coin.amount == Uint128::from(1_500_001_225u128)
            }));
            assert!(balances.iter().any(|coin| {
                coin.denom == "uusdc" && coin.amount == Uint128::from(1_000_000_817u128)
            }));
        })
        .query_all_balances(&creator.to_string(), |result| {
            let balances = result.unwrap();
            assert!(balances.iter().any(|coin| {
                coin.denom == lp_denom.clone() && coin.amount == Uint128::from(1_223_990_208u128)
            }));
        });
}
