use cosmwasm_std::{coin, Coin, Decimal, StdError, Uint128};
use mantra_common_testing::multi_test::stargate_mock::StargateMock;
use mantra_dex_std::fee::{Fee, PoolFee};
use mantra_dex_std::lp_common::MINIMUM_LIQUIDITY_AMOUNT;
use mantra_dex_std::pool_manager::PoolType;

use crate::tests::suite::TestingSuite;
use crate::ContractError;

const UWHALE_DENOM: &str = "uwhale";
const ULUNA_DENOM: &str = "uluna";
const UOSMO_DENOM: &str = "uosmo";
const UUSD_DENOM: &str = "uusd";
const UOM_DENOM: &str = "uom";
const O_WHALE_ULUNA_DENOM: &str = "o.whale.uluna";
const WHALE_ULUNA_LABEL: &str = "whale.uluna";

const INITIAL_BALANCE: u128 = 10_000_000u128;
const SMALL_BALANCE: u128 = 10_000u128;
const UOM_STARGATE_BALANCE: u128 = 8888u128;
const UUSD_POOL_CREATION_FEE: u128 = 1000u128;
const UOM_POOL_CREATION_FEE: u128 = UOM_STARGATE_BALANCE; // 8888u128

const LIQUIDITY_AMOUNT: u128 = 1_000_000u128;
const INITIAL_LP_TOKENS_MINTED: u128 = 999_000u128; // LIQUIDITY_AMOUNT - MINIMUM_LIQUIDITY_AMOUNT (1_000_000 - 1_000)
const SINGLE_ASSET_DEPOSIT_SMALL: u128 = 10_000u128;
const SINGLE_ASSET_DEPOSIT_THIRD_PARTY: u128 = 1_000u128;

const LP_TOKENS_FOR_OTHER_USER: u128 = 9_798u128;
const TOTAL_LP_SUPPLY_AFTER_SINGLE_ASSET_DEPOSIT: u128 =
    INITIAL_LP_TOKENS_MINTED + MINIMUM_LIQUIDITY_AMOUNT.u128() + LP_TOKENS_FOR_OTHER_USER; // 999_000 + 1_000 + 9_798 = 1_009_798

const FINAL_UWHALE_IN_POOL: u128 = 1_020_000u128;
const FINAL_ULUNA_IN_POOL: u128 = 999_901u128;

const CREATOR_REMAINING_ULUNA: u128 = INITIAL_BALANCE - LIQUIDITY_AMOUNT; // 9_000_000
const CREATOR_REMAINING_UOM: u128 = SMALL_BALANCE - UOM_POOL_CREATION_FEE; // 10_000 - 8888 = 1112
const CREATOR_REMAINING_UUSD: u128 = SMALL_BALANCE - UUSD_POOL_CREATION_FEE; // 9_000
const CREATOR_REMAINING_UWHALE: u128 = INITIAL_BALANCE - LIQUIDITY_AMOUNT; // 9_000_000

const CREATOR_ULUNA_AFTER_WITHDRAW: u128 = 9_989_208u128;
const CREATOR_UWHALE_AFTER_WITHDRAW: u128 = 10_009_092u128;

const OTHER_REMAINING_UWHALE: u128 = INITIAL_BALANCE - SINGLE_ASSET_DEPOSIT_SMALL * 2; // 9_980_000 (10_000_000 - 20_000)

const OTHER_ULUNA_AFTER_WITHDRAW: u128 = 10_009_702u128;
const OTHER_UWHALE_AFTER_WITHDRAW: u128 = 9_989_897u128;

const FEE_COLLECTOR_ULUNA_FEES: u128 = 99u128;
const CONTRACT_DUST_ULUNA: u128 = 991u128;
const CONTRACT_DUST_UWHALE: u128 = 1_011u128;

const LP_TOKENS_FOR_ANOTHER_USER: u128 = 981u128;

const EDGE_CASE_INITIAL_LIQUIDITY: u128 = 1_100u128;
const EDGE_CASE_SINGLE_ASSET_DEPOSIT_SLIPPAGE_FAIL: u128 = 1_760u128;
const EDGE_CASE_SINGLE_ASSET_DEPOSIT_SLIPPAGE_FAIL_LARGE: u128 = 10_000u128;
const EDGE_CASE_SINGLE_ASSET_DEPOSIT_SUCCESS: u128 = 1_000u128;

const FIFTY_PERCENT_SLIPPAGE: Option<Decimal> = Some(Decimal::percent(50));
const ONE_PERCENT_FEE: Decimal = Decimal::percent(1);
const FIFTEEN_PERCENT_FEE: Decimal = Decimal::percent(15);
const FIVE_PERCENT_FEE: Decimal = Decimal::percent(5);
const ZERO_PERCENT_FEE: Decimal = Decimal::zero();
const SIX_DECIMALS: u8 = 6u8;

#[test]
fn provide_liquidity_with_single_asset() {
    let mut suite = TestingSuite::default_with_balances(
        vec![
            coin(INITIAL_BALANCE, UWHALE_DENOM.to_string()),
            coin(INITIAL_BALANCE, ULUNA_DENOM.to_string()),
            coin(INITIAL_BALANCE, UOSMO_DENOM.to_string()),
            coin(SMALL_BALANCE, UUSD_DENOM.to_string()),
            coin(SMALL_BALANCE, UOM_DENOM.to_string()),
        ],
        StargateMock::new(vec![coin(UOM_STARGATE_BALANCE, UOM_DENOM.to_string())]),
    );
    let creator = suite.creator();
    let other = suite.senders[1].clone();
    let _unauthorized = suite.senders[2].clone();

    // Asset denoms with uwhale and uluna
    let asset_denoms = vec![UWHALE_DENOM.to_string(), ULUNA_DENOM.to_string()];

    let pool_fees = PoolFee {
        protocol_fee: Fee {
            share: ONE_PERCENT_FEE,
        },
        swap_fee: Fee {
            share: ONE_PERCENT_FEE,
        },
        burn_fee: Fee {
            share: ZERO_PERCENT_FEE,
        },
        extra_fees: vec![],
    };

    // Create a pool
    suite.instantiate_default().add_one_epoch().create_pool(
        &creator,
        asset_denoms,
        vec![SIX_DECIMALS, SIX_DECIMALS],
        pool_fees,
        PoolType::ConstantProduct,
        Some(WHALE_ULUNA_LABEL.to_string()),
        vec![
            coin(UUSD_POOL_CREATION_FEE, UUSD_DENOM),
            coin(UOM_POOL_CREATION_FEE, UOM_DENOM),
        ],
        |result| {
            result.unwrap();
        },
    );

    let contract_addr = suite.pool_manager_addr.clone();
    let lp_denom = suite.get_lp_denom(O_WHALE_ULUNA_DENOM.to_string());

    // Let's try to add liquidity
    suite
            .provide_liquidity(
                &creator,
                O_WHALE_ULUNA_DENOM.to_string(),
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
                O_WHALE_ULUNA_DENOM.to_string(),
                None,
                None,
                None,
                None,
                None,
                vec![Coin {
                    denom: UOSMO_DENOM.to_string(),
                    amount: Uint128::from(LIQUIDITY_AMOUNT),
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
                O_WHALE_ULUNA_DENOM.to_string(),
                None,
                None,
                None,
                None,
                None,
                vec![Coin {
                    denom: UWHALE_DENOM.to_string(),
                    amount: Uint128::from(LIQUIDITY_AMOUNT),
                }],
                |result| {
                    let err = result.unwrap_err().downcast::<ContractError>().unwrap();

                    match err {
                        ContractError::EmptyPoolForSingleSideLiquidityProvision => {}
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
            O_WHALE_ULUNA_DENOM.to_string(),
            None,
            None,
            None,
            None,
            None,
            vec![
                Coin {
                    denom: UOSMO_DENOM.to_string(),
                    amount: Uint128::from(LIQUIDITY_AMOUNT),
                },
                Coin {
                    denom: UWHALE_DENOM.to_string(),
                    amount: Uint128::from(LIQUIDITY_AMOUNT),
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
            O_WHALE_ULUNA_DENOM.to_string(),
            None,
            None,
            None,
            None,
            None,
            vec![
                Coin {
                    denom: UWHALE_DENOM.to_string(),
                    amount: Uint128::from(LIQUIDITY_AMOUNT),
                },
                Coin {
                    denom: ULUNA_DENOM.to_string(),
                    amount: Uint128::from(LIQUIDITY_AMOUNT),
                },
            ],
            |result| {
                result.unwrap();
            },
        )
        .query_all_balances(&creator.to_string(), |result| {
            let balances = result.unwrap();

            assert!(balances.iter().any(|coin| {
                coin.denom == lp_denom && coin.amount == Uint128::from(INITIAL_LP_TOKENS_MINTED)
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
            O_WHALE_ULUNA_DENOM.to_string(),
            None,
            None,
            None,
            None,
            None,
            vec![
                Coin {
                    denom: UWHALE_DENOM.to_string(),
                    amount: Uint128::from(1_000u128),
                },
                Coin {
                    denom: UWHALE_DENOM.to_string(),
                    amount: Uint128::from(1_000u128),
                },
            ],
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();
                assert_eq!(
                    err,
                    ContractError::Std(StdError::generic_err("Slippage limit exceeded"))
                );
            },
        )
        .provide_liquidity(
            &other,
            O_WHALE_ULUNA_DENOM.to_string(),
            None,
            None,
            None,
            FIFTY_PERCENT_SLIPPAGE,
            None,
            vec![
                Coin {
                    denom: UWHALE_DENOM.to_string(),
                    amount: Uint128::from(SINGLE_ASSET_DEPOSIT_SMALL),
                },
                Coin {
                    denom: UWHALE_DENOM.to_string(),
                    amount: Uint128::from(SINGLE_ASSET_DEPOSIT_SMALL),
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
            assert!(balances.iter().any(|coin| {
                coin.denom == lp_denom && coin.amount == Uint128::from(LP_TOKENS_FOR_OTHER_USER)
            }));
        })
        .query_all_balances(&contract_addr.to_string(), |result| {
            let balances = result.unwrap();
            // check that balances has 999_000 factory/mantra1zwv6feuzhy6a9wekh96cd57lsarmqlwxdypdsplw6zhfncqw6ftqlydlr9/o.whale.uluna.LP
            assert!(balances.iter().any(|coin| {
                coin.denom == lp_denom.clone() && coin.amount == MINIMUM_LIQUIDITY_AMOUNT
            }));
        });

    suite
        .query_lp_supply(O_WHALE_ULUNA_DENOM.to_string(), |res| {
            // total amount of LP tokens issued should be 1_009_798 = 999_000 to the first LP,
            // 1_000 to the contract, and 9_798 to the second, single-side LP
            assert_eq!(
                res.unwrap().amount,
                Uint128::from(TOTAL_LP_SUPPLY_AFTER_SINGLE_ASSET_DEPOSIT)
            );
        })
        .query_pools(Some(O_WHALE_ULUNA_DENOM.to_string()), None, None, |res| {
            let response = res.unwrap();

            let whale = response.pools[0]
                .pool_info
                .assets
                .iter()
                .find(|coin| coin.denom == *UWHALE_DENOM)
                .unwrap();
            let luna = response.pools[0]
                .pool_info
                .assets
                .iter()
                .find(|coin| coin.denom == *ULUNA_DENOM)
                .unwrap();

            assert_eq!(whale.amount, Uint128::from(FINAL_UWHALE_IN_POOL));
            assert_eq!(luna.amount, Uint128::from(FINAL_ULUNA_IN_POOL));
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
                        denom: ULUNA_DENOM.to_string(),
                        amount: Uint128::from(FINAL_ULUNA_IN_POOL),
                    },
                    Coin {
                        denom: UWHALE_DENOM.to_string(),
                        amount: Uint128::from(FINAL_UWHALE_IN_POOL),
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
                        amount: Uint128::from(INITIAL_LP_TOKENS_MINTED),
                    },
                    Coin {
                        denom: ULUNA_DENOM.to_string(),
                        amount: Uint128::from(CREATOR_REMAINING_ULUNA),
                    },
                    Coin {
                        denom: UOM_DENOM.to_string(),
                        amount: Uint128::from(CREATOR_REMAINING_UOM),
                    },
                    Coin {
                        denom: UOSMO_DENOM.to_string(),
                        amount: Uint128::from(INITIAL_BALANCE),
                    },
                    Coin {
                        denom: UUSD_DENOM.to_string(),
                        amount: Uint128::from(CREATOR_REMAINING_UUSD),
                    },
                    Coin {
                        denom: UWHALE_DENOM.to_string(),
                        amount: Uint128::from(CREATOR_REMAINING_UWHALE),
                    },
                ]
            );
        })
        .withdraw_liquidity(
            &creator,
            O_WHALE_ULUNA_DENOM.to_string(),
            vec![Coin {
                denom: lp_denom.clone(),
                amount: Uint128::from(INITIAL_LP_TOKENS_MINTED),
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
                        denom: ULUNA_DENOM.to_string(),
                        amount: Uint128::from(CREATOR_ULUNA_AFTER_WITHDRAW),
                    },
                    Coin {
                        denom: UOM_DENOM.to_string(),
                        amount: Uint128::from(CREATOR_REMAINING_UOM),
                    },
                    Coin {
                        denom: UOSMO_DENOM.to_string(),
                        amount: Uint128::from(INITIAL_BALANCE),
                    },
                    Coin {
                        denom: UUSD_DENOM.to_string(),
                        amount: Uint128::from(CREATOR_REMAINING_UUSD),
                    },
                    Coin {
                        denom: UWHALE_DENOM.to_string(),
                        amount: Uint128::from(CREATOR_UWHALE_AFTER_WITHDRAW),
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
                        amount: Uint128::from(LP_TOKENS_FOR_OTHER_USER),
                    },
                    Coin {
                        denom: ULUNA_DENOM.to_string(),
                        amount: Uint128::from(INITIAL_BALANCE),
                    },
                    Coin {
                        denom: UOM_DENOM.to_string(),
                        amount: Uint128::from(SMALL_BALANCE),
                    },
                    Coin {
                        denom: UOSMO_DENOM.to_string(),
                        amount: Uint128::from(INITIAL_BALANCE),
                    },
                    Coin {
                        denom: UUSD_DENOM.to_string(),
                        amount: Uint128::from(SMALL_BALANCE),
                    },
                    Coin {
                        denom: UWHALE_DENOM.to_string(),
                        amount: Uint128::from(OTHER_REMAINING_UWHALE),
                    },
                ]
            );
        })
        .withdraw_liquidity(
            &other,
            O_WHALE_ULUNA_DENOM.to_string(),
            vec![Coin {
                denom: lp_denom.clone(),
                amount: Uint128::from(LP_TOKENS_FOR_OTHER_USER),
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
                        denom: ULUNA_DENOM.to_string(),
                        amount: Uint128::from(OTHER_ULUNA_AFTER_WITHDRAW),
                    },
                    Coin {
                        denom: UOM_DENOM.to_string(),
                        amount: Uint128::from(SMALL_BALANCE),
                    },
                    Coin {
                        denom: UOSMO_DENOM.to_string(),
                        amount: Uint128::from(INITIAL_BALANCE),
                    },
                    Coin {
                        denom: UUSD_DENOM.to_string(),
                        amount: Uint128::from(SMALL_BALANCE),
                    },
                    Coin {
                        denom: UWHALE_DENOM.to_string(),
                        amount: Uint128::from(OTHER_UWHALE_AFTER_WITHDRAW),
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
                        denom: ULUNA_DENOM.to_string(),
                        amount: Uint128::from(FEE_COLLECTOR_ULUNA_FEES),
                    },
                    Coin {
                        denom: UUSD_DENOM.to_string(),
                        amount: Uint128::from(UUSD_POOL_CREATION_FEE),
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
                        denom: ULUNA_DENOM.to_string(),
                        amount: Uint128::from(CONTRACT_DUST_ULUNA),
                    },
                    Coin {
                        denom: UWHALE_DENOM.to_string(),
                        amount: Uint128::from(CONTRACT_DUST_UWHALE),
                    },
                ]
            );
        });
}

#[test]
fn provide_liquidity_with_single_asset_to_third_party() {
    let mut suite = TestingSuite::default_with_balances(
        vec![
            coin(INITIAL_BALANCE, UWHALE_DENOM.to_string()),
            coin(INITIAL_BALANCE, ULUNA_DENOM.to_string()),
            coin(INITIAL_BALANCE, UOSMO_DENOM.to_string()),
            coin(SMALL_BALANCE, UUSD_DENOM.to_string()),
            coin(SMALL_BALANCE, UOM_DENOM.to_string()),
        ],
        StargateMock::new(vec![coin(UOM_STARGATE_BALANCE, UOM_DENOM.to_string())]),
    );
    let creator = suite.creator();
    let other = suite.senders[1].clone();
    let another = suite.senders[2].clone();

    // Asset denoms with uwhale and uluna
    let asset_denoms = vec![UWHALE_DENOM.to_string(), ULUNA_DENOM.to_string()];

    let pool_fees = PoolFee {
        protocol_fee: Fee {
            share: ONE_PERCENT_FEE,
        },
        swap_fee: Fee {
            share: ONE_PERCENT_FEE,
        },
        burn_fee: Fee {
            share: ZERO_PERCENT_FEE,
        },
        extra_fees: vec![],
    };

    // Create a pool
    suite.instantiate_default().add_one_epoch().create_pool(
        &creator,
        asset_denoms,
        vec![SIX_DECIMALS, SIX_DECIMALS],
        pool_fees,
        PoolType::ConstantProduct,
        Some(WHALE_ULUNA_LABEL.to_string()),
        vec![
            coin(UUSD_POOL_CREATION_FEE, UUSD_DENOM),
            coin(UOM_POOL_CREATION_FEE, UOM_DENOM),
        ],
        |result| {
            result.unwrap();
        },
    );

    let contract_addr = suite.pool_manager_addr.clone();
    let lp_denom = suite.get_lp_denom(O_WHALE_ULUNA_DENOM.to_string());

    // Let's try to add liquidity
    suite
        .provide_liquidity(
            &creator,
            O_WHALE_ULUNA_DENOM.to_string(),
            None,
            None,
            None,
            None,
            None,
            vec![
                Coin {
                    denom: UWHALE_DENOM.to_string(),
                    amount: Uint128::from(LIQUIDITY_AMOUNT),
                },
                Coin {
                    denom: ULUNA_DENOM.to_string(),
                    amount: Uint128::from(LIQUIDITY_AMOUNT),
                },
            ],
            |result| {
                result.unwrap();
            },
        )
        .query_all_balances(&creator.to_string(), |result| {
            let balances = result.unwrap();

            assert!(balances.iter().any(|coin| {
                coin.denom == lp_denom && coin.amount == Uint128::from(INITIAL_LP_TOKENS_MINTED)
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
            O_WHALE_ULUNA_DENOM.to_string(),
            None,
            None,
            None,
            FIFTY_PERCENT_SLIPPAGE,
            Some(another.to_string()),
            vec![
                Coin {
                    denom: UWHALE_DENOM.to_string(),
                    amount: Uint128::from(SINGLE_ASSET_DEPOSIT_THIRD_PARTY),
                },
                Coin {
                    denom: UWHALE_DENOM.to_string(),
                    amount: Uint128::from(SINGLE_ASSET_DEPOSIT_THIRD_PARTY),
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
            assert!(balances.iter().any(|coin| {
                coin.denom == lp_denom && coin.amount == Uint128::from(LP_TOKENS_FOR_ANOTHER_USER)
            }));
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
        O_WHALE_ULUNA_DENOM.to_string(),
        Some(86_400u64),
        None,
        None,
        FIFTY_PERCENT_SLIPPAGE,
        Some(another.to_string()),
        vec![
            Coin {
                denom: UWHALE_DENOM.to_string(),
                amount: Uint128::from(SINGLE_ASSET_DEPOSIT_THIRD_PARTY),
            },
            Coin {
                denom: UWHALE_DENOM.to_string(),
                amount: Uint128::from(SINGLE_ASSET_DEPOSIT_THIRD_PARTY),
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
            coin(LIQUIDITY_AMOUNT, UWHALE_DENOM.to_string()),
            coin(LIQUIDITY_AMOUNT, ULUNA_DENOM.to_string()),
            coin(LIQUIDITY_AMOUNT, UOSMO_DENOM.to_string()),
            coin(SMALL_BALANCE, UUSD_DENOM.to_string()),
            coin(SMALL_BALANCE, UOM_DENOM.to_string()),
        ],
        StargateMock::new(vec![coin(UOM_STARGATE_BALANCE, UOM_DENOM.to_string())]),
    );
    let creator = suite.creator();
    let other = suite.senders[1].clone();
    let _unauthorized = suite.senders[2].clone();

    // Asset denoms with uwhale and uluna
    let asset_denoms = vec![UWHALE_DENOM.to_string(), ULUNA_DENOM.to_string()];

    let pool_fees = PoolFee {
        protocol_fee: Fee {
            share: FIFTEEN_PERCENT_FEE,
        },
        swap_fee: Fee {
            share: FIVE_PERCENT_FEE,
        },
        burn_fee: Fee {
            share: ZERO_PERCENT_FEE,
        },
        extra_fees: vec![],
    };

    // Create a pool
    suite.instantiate_default().add_one_epoch().create_pool(
        &creator,
        asset_denoms,
        vec![SIX_DECIMALS, SIX_DECIMALS],
        pool_fees,
        PoolType::ConstantProduct,
        Some(WHALE_ULUNA_LABEL.to_string()),
        vec![
            coin(UUSD_POOL_CREATION_FEE, UUSD_DENOM),
            coin(UOM_POOL_CREATION_FEE, UOM_DENOM),
        ],
        |result| {
            result.unwrap();
        },
    );

    let contract_addr = suite.pool_manager_addr.clone();

    // let's provide liquidity with two assets
    suite
        .provide_liquidity(
            &creator,
            O_WHALE_ULUNA_DENOM.to_string(),
            None,
            None,
            None,
            None,
            None,
            vec![
                Coin {
                    denom: UWHALE_DENOM.to_string(),
                    amount: Uint128::from(EDGE_CASE_INITIAL_LIQUIDITY),
                },
                Coin {
                    denom: ULUNA_DENOM.to_string(),
                    amount: Uint128::from(EDGE_CASE_INITIAL_LIQUIDITY),
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
            O_WHALE_ULUNA_DENOM.to_string(),
            None,
            None,
            None,
            FIFTY_PERCENT_SLIPPAGE,
            None,
            vec![Coin {
                denom: UWHALE_DENOM.to_string(),
                amount: Uint128::from(EDGE_CASE_SINGLE_ASSET_DEPOSIT_SLIPPAGE_FAIL),
            }],
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();
                assert_eq!(
                    err,
                    ContractError::Std(StdError::generic_err("Slippage limit exceeded"))
                );
            },
        )
        .provide_liquidity(
            &other,
            O_WHALE_ULUNA_DENOM.to_string(),
            None,
            None,
            None,
            FIFTY_PERCENT_SLIPPAGE,
            None,
            vec![Coin {
                denom: UWHALE_DENOM.to_string(),
                amount: Uint128::from(EDGE_CASE_SINGLE_ASSET_DEPOSIT_SLIPPAGE_FAIL_LARGE),
            }],
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();
                assert_eq!(
                    err,
                    ContractError::Std(StdError::generic_err("Slippage limit exceeded"))
                );
            },
        )
        .provide_liquidity(
            &other,
            O_WHALE_ULUNA_DENOM.to_string(),
            None,
            None,
            None,
            FIFTY_PERCENT_SLIPPAGE,
            None,
            vec![Coin {
                denom: UWHALE_DENOM.to_string(),
                amount: Uint128::from(EDGE_CASE_SINGLE_ASSET_DEPOSIT_SUCCESS),
            }],
            |result| {
                result.unwrap();
            },
        );
}
