use std::cell::RefCell;

use cosmwasm_std::{assert_approx_eq, coin, Coin, Decimal, Uint128};
use mantra_common_testing::multi_test::stargate_mock::StargateMock;
use mantra_dex_std::{
    fee::{Fee, PoolFee},
    lp_common::MINIMUM_LIQUIDITY_AMOUNT,
    pool_manager::PoolType,
};

use crate::tests::{integration::helpers::extract_pool_reserves, suite::TestingSuite};

const UWHALE_DENOM: &str = "uwhale";
const ULUNA_DENOM: &str = "uluna";
const UUSD_DENOM: &str = "uusd";
const UTEST_DENOM: &str = "utest";
const UOM_DENOM: &str = "uom";
const UUSDC_DENOM: &str = "uusdc";
const UUSDT_DENOM: &str = "uusdt";
const AUSDT_DENOM: &str = "ausdy";
const PUSDC_DENOM: &str = "pusdc";

const ONE_MILLION: Uint128 = Uint128::new(1_000_000u128);
const ONE_THOUSAND: Uint128 = Uint128::new(1_000u128);
const TEN_THOUSAND: Uint128 = Uint128::new(10_000u128);
const NINE_NINE_NINE_THOUSAND: Uint128 = Uint128::new(999_000u128);
const LP_AMOUNT_18_DECIMALS: Uint128 = Uint128::new(300_000_000_000_000_000000000000000000u128);
const INITIAL_BALANCE_STABLESWAP: Uint128 = Uint128::new(1_000_00000000000000u128);
const LIQUIDITY_ADD_UUSDC_STABLESWAP: Uint128 = Uint128::new(10u128.pow(3));
const LIQUIDITY_ADD_UUSDT_STABLESWAP: Uint128 = Uint128::new(10u128.pow(15));
const LIQUIDITY_ADD_UUSDC_LOW_DECIMALS: Uint128 = Uint128::new(1_000u128);
const LIQUIDITY_ADD_UUSDT_LOW_DECIMALS: Uint128 = Uint128::new(1_000_000u128);
const ONE_TRILLION: Uint128 = Uint128::new(1_000_000_000_000u128);
const FEE_AMOUNT_UOM: Uint128 = Uint128::new(8888u128);

const POOL_ID_WHALE_LUNA: &str = "o.whale.uluna";

const DEFAULT_DECIMALS: u8 = 6u8;

#[test]
fn deposit_and_withdraw_sanity_check() {
    let mut suite = TestingSuite::default_with_balances(
        vec![
            coin(ONE_MILLION.u128(), UWHALE_DENOM.to_string()),
            coin(ONE_MILLION.u128(), ULUNA_DENOM.to_string()),
            coin(ONE_THOUSAND.u128(), UUSD_DENOM.to_string()),
            coin(TEN_THOUSAND.u128(), UOM_DENOM.to_string()),
            coin(TEN_THOUSAND.u128(), UTEST_DENOM.to_string()),
        ],
        StargateMock::new(vec![coin(ONE_THOUSAND.u128(), UTEST_DENOM.to_string())]),
    );
    let creator = suite.creator();
    let _other = suite.senders[1].clone();
    let _unauthorized = suite.senders[2].clone();

    // Asset denoms with uwhale and uluna
    let asset_denoms = vec![UWHALE_DENOM.to_string(), ULUNA_DENOM.to_string()];

    let pool_fees = PoolFee {
        protocol_fee: Fee {
            share: Decimal::zero(),
        },
        swap_fee: Fee {
            share: Decimal::zero(),
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
        vec![DEFAULT_DECIMALS, DEFAULT_DECIMALS],
        pool_fees,
        PoolType::ConstantProduct,
        Some("whale.uluna".to_string()),
        vec![
            coin(ONE_THOUSAND.u128(), UUSD_DENOM.to_string()),
            coin(ONE_THOUSAND.u128(), UTEST_DENOM.to_string()),
        ],
        |result| {
            result.unwrap();
        },
    );

    let contract_addr = suite.pool_manager_addr.clone();
    let lp_denom = suite.get_lp_denom(POOL_ID_WHALE_LUNA.to_string());
    let expected_pool_reserves = RefCell::<Vec<Vec<Coin>>>::new(vec![]);

    // Let's try to add liquidity
    suite
        .provide_liquidity(
            &creator,
            POOL_ID_WHALE_LUNA.to_string(),
            None,
            None,
            None,
            None,
            None,
            vec![
                Coin {
                    denom: UWHALE_DENOM.to_string(),
                    amount: ONE_MILLION,
                },
                Coin {
                    denom: ULUNA_DENOM.to_string(),
                    amount: ONE_MILLION,
                },
            ],
            |result| {
                let events = result.unwrap().events;
                // Ensure we got 999_000 in the response which is 1_000_000 less the initial liquidity amount
                assert!(events.iter().any(|event| {
                    if event.ty == "wasm" {
                        for attribute in &event.attributes {
                            if attribute.key.as_str() == "pool_reserves" {
                                extract_pool_reserves(attribute, &expected_pool_reserves);
                            }
                        }
                    }
                    event.attributes.iter().any(|attr| {
                        attr.key == "added_shares"
                            && attr.value == (ONE_MILLION - MINIMUM_LIQUIDITY_AMOUNT).to_string()
                    })
                }));
            },
        )
        // creator should have 999_000 LP shares (1M - MINIMUM_LIQUIDITY_AMOUNT)
        .query_all_balances(&creator.to_string(), |result| {
            let balances = result.unwrap();

            assert!(balances
                .iter()
                .any(|coin| { coin.denom == lp_denom && coin.amount == NINE_NINE_NINE_THOUSAND }));
        })
        // contract should have 1_000 LP shares (MINIMUM_LIQUIDITY_AMOUNT)
        .query_all_balances(&contract_addr.to_string(), |result| {
            let balances = result.unwrap();
            // check that balances has 999_000 factory/mantra1zwv6feuzhy6a9wekh96cd57lsarmqlwxdypdsplw6zhfncqw6ftqlydlr9/o.whale.uluna.LP
            assert!(balances.iter().any(|coin| {
                coin.denom == lp_denom.clone() && coin.amount == MINIMUM_LIQUIDITY_AMOUNT
            }));
        })
        .query_pools(Some(POOL_ID_WHALE_LUNA.to_string()), None, None, |result| {
            let response = result.unwrap();
            let mut assets = response.pools[0].pool_info.assets.clone();
            assets.sort_by(|a, b| a.denom.cmp(&b.denom));
            assert_eq!(assets, expected_pool_reserves.borrow()[0]);
        });

    // Let's try to withdraw liquidity
    expected_pool_reserves.borrow_mut().clear();
    suite
        .withdraw_liquidity(
            &creator,
            POOL_ID_WHALE_LUNA.to_string(),
            vec![Coin {
                denom: lp_denom.clone(),
                amount: NINE_NINE_NINE_THOUSAND,
            }],
            |result| {
                let events = result.unwrap().events;
                for event in &events {
                    if event.ty.as_str() == "wasm" {
                        for attribute in &event.attributes {
                            match attribute.key.as_str() {
                                "pool_reserves" => {
                                    extract_pool_reserves(attribute, &expected_pool_reserves);
                                }
                                "pool_identifier" => {
                                    assert_eq!(attribute.value, POOL_ID_WHALE_LUNA);
                                }
                                _ => {}
                            }
                        }
                    }
                }
                // we're trading 999_000 shares for 1_000_000 of our liquidity
                assert!(events.iter().any(|event| {
                    event.attributes.iter().any(|attr| {
                        attr.key == "withdrawn_shares"
                            && attr.value == (ONE_MILLION - MINIMUM_LIQUIDITY_AMOUNT).to_string()
                    })
                }));
            },
        )
        // creator should have 0 LP shares in the contract and 0 LP shares in their account balance
        .query_amount_of_lp_token(
            POOL_ID_WHALE_LUNA.to_string(),
            &creator.to_string(),
            |result| {
                assert_eq!(result.unwrap(), Uint128::zero());
            },
        )
        .query_balance(&creator.to_string(), lp_denom, |result| {
            assert_eq!(result.unwrap().amount, Uint128::zero());
        })
        // creator should 999_000 uwhale and 999_000 uluna (1M - MINIMUM_LIQUIDITY_AMOUNT)
        .query_all_balances(&creator.to_string(), |result| {
            let balances = result.unwrap();
            assert!(balances.iter().any(|coin| {
                coin.denom == *UWHALE_DENOM && coin.amount == ONE_MILLION - MINIMUM_LIQUIDITY_AMOUNT
            }));
            assert!(balances.iter().any(|coin| {
                coin.denom == *ULUNA_DENOM && coin.amount == ONE_MILLION - MINIMUM_LIQUIDITY_AMOUNT
            }));
        })
        .query_pools(Some(POOL_ID_WHALE_LUNA.to_string()), None, None, |result| {
            let response = result.unwrap();
            let mut assets = response.pools[0].pool_info.assets.clone();
            assets.sort_by(|a, b| a.denom.cmp(&b.denom));
            assert_eq!(assets, expected_pool_reserves.borrow()[0]);
        });
}

#[test]
fn lp_mint_stableswap_different_decimals_scaling_min_liquidity() {
    let mut suite = TestingSuite::default_with_balances(
        vec![
            coin(INITIAL_BALANCE_STABLESWAP.u128(), UUSDC_DENOM.to_string()),
            coin(INITIAL_BALANCE_STABLESWAP.u128(), UUSDT_DENOM.to_string()),
            coin(ONE_THOUSAND.u128(), UUSD_DENOM.to_string()),
            coin(TEN_THOUSAND.u128(), UOM_DENOM.to_string()),
            coin(TEN_THOUSAND.u128(), UTEST_DENOM.to_string()),
        ],
        StargateMock::new(vec![coin(ONE_THOUSAND.u128(), UTEST_DENOM.to_string())]),
    );
    let alice = suite.creator();
    let bob = suite.senders[1].clone();
    let asset_denoms = vec![UUSDC_DENOM.to_string(), UUSDT_DENOM.to_string()];

    let pool_fees = PoolFee {
        protocol_fee: Fee {
            share: Decimal::zero(),
        },
        swap_fee: Fee {
            share: Decimal::zero(),
        },
        burn_fee: Fee {
            share: Decimal::zero(),
        },
        extra_fees: vec![],
    };

    // Create a pool
    suite.instantiate_default().add_one_epoch().create_pool(
        &alice,
        asset_denoms,
        vec![6u8, 18u8],
        pool_fees,
        PoolType::StableSwap { amp: 85 },
        None,
        vec![
            coin(ONE_THOUSAND.u128(), UUSD_DENOM.to_string()),
            coin(ONE_THOUSAND.u128(), UTEST_DENOM.to_string()),
        ],
        |result| {
            result.unwrap();
        },
    );

    // Let's try to add liquidity
    suite
        .provide_liquidity(
            &alice,
            "p.1".to_string(),
            None,
            None,
            None,
            None,
            None,
            vec![
                coin(
                    LIQUIDITY_ADD_UUSDC_STABLESWAP.u128(),
                    UUSDC_DENOM.to_string(),
                ),
                coin(
                    LIQUIDITY_ADD_UUSDT_STABLESWAP.u128(),
                    UUSDT_DENOM.to_string(),
                ),
            ],
            |result| {
                println!("result: {:?}", result);
                result.unwrap();
                println!("---");
            },
        )
        .provide_liquidity(
            &bob,
            "p.1".to_string(),
            None,
            None,
            None,
            None,
            None,
            vec![
                coin(
                    LIQUIDITY_ADD_UUSDC_STABLESWAP.u128(),
                    UUSDC_DENOM.to_string(),
                ),
                coin(
                    LIQUIDITY_ADD_UUSDT_STABLESWAP.u128(),
                    UUSDT_DENOM.to_string(),
                ),
            ],
            |result| {
                println!("result: {:?}", result);
                result.unwrap();
            },
        );

    let lp_shares_alice = RefCell::new(Coin::new(0u128, "".to_string()));
    let uusdc_alice = RefCell::new(Coin::new(0u128, "".to_string()));
    let uusdt_alice = RefCell::new(Coin::new(0u128, "".to_string()));
    suite.query_all_balances(&alice.to_string(), |balances| {
        for coin in balances.unwrap().iter() {
            if coin.denom.contains("p.1") {
                *lp_shares_alice.borrow_mut() = coin.clone();
                println!("lp_shares_alice: {:?}", lp_shares_alice.borrow());
            }
            if coin.denom.contains(UUSDC_DENOM) {
                *uusdc_alice.borrow_mut() = coin.clone();
                println!("uusdc_alice: {:?}", uusdc_alice.borrow());
            }
            if coin.denom.contains(UUSDT_DENOM) {
                *uusdt_alice.borrow_mut() = coin.clone();
                println!("uusdt_alice: {:?}", uusdt_alice.borrow());
            }
        }
    });

    let lp_shares_bob = RefCell::new(Coin::new(0u128, "".to_string()));
    let uusdc_bob = RefCell::new(Coin::new(0u128, "".to_string()));
    let uusdt_bob = RefCell::new(Coin::new(0u128, "".to_string()));
    suite.query_all_balances(&bob.to_string(), |balances| {
        for coin in balances.unwrap().iter() {
            if coin.denom.contains("p.1") {
                *lp_shares_bob.borrow_mut() = coin.clone();
                println!("lp_shares_bob: {:?}", lp_shares_bob.borrow());
            }
            if coin.denom.contains(UUSDC_DENOM) {
                *uusdc_bob.borrow_mut() = coin.clone();
                println!("uusdc_bob: {:?}", uusdc_bob.borrow());
            }
            if coin.denom.contains(UUSDT_DENOM) {
                *uusdt_bob.borrow_mut() = coin.clone();
                println!("uusdt_bob: {:?}", uusdt_bob.borrow());
            }
        }
    });

    suite
        .withdraw_liquidity(
            &bob,
            "p.1".to_string(),
            vec![lp_shares_bob.borrow().clone()],
            |result| {
                println!("result: {:?}", result);
                result.unwrap();
            },
        )
        .query_all_balances(&bob.to_string(), |balances| {
            for coin in balances.unwrap().iter() {
                if coin.denom.contains(UUSDC_DENOM) {
                    assert_eq!(
                        coin.amount,
                        uusdc_bob.borrow().clone().amount + LIQUIDITY_ADD_UUSDC_STABLESWAP
                    );
                }
                if coin.denom.contains(UUSDT_DENOM) {
                    assert_eq!(
                        coin.amount,
                        uusdt_bob.borrow().clone().amount + LIQUIDITY_ADD_UUSDT_STABLESWAP
                    );
                }
            }
        });

    suite
        .withdraw_liquidity(
            &alice,
            "p.1".to_string(),
            vec![lp_shares_alice.borrow().clone()],
            |result| {
                println!("result: {:?}", result);
                result.unwrap();
            },
        )
        .query_all_balances(&alice.to_string(), |balances| {
            for coin in balances.unwrap().iter() {
                if coin.denom.contains(UUSDC_DENOM) {
                    assert_approx_eq!(
                        coin.amount,
                        uusdc_alice.borrow().clone().amount + LIQUIDITY_ADD_UUSDC_STABLESWAP,
                        "50"
                    );
                }
                if coin.denom.contains(UUSDT_DENOM) {
                    assert_approx_eq!(
                        coin.amount,
                        uusdt_alice.borrow().clone().amount + LIQUIDITY_ADD_UUSDT_STABLESWAP,
                        "50"
                    );
                }
            }
        });
}

#[test]
fn lp_mint_stableswap_low_decimals_scaling_min_liquidity() {
    let mut suite = TestingSuite::default_with_balances(
        vec![
            coin(INITIAL_BALANCE_STABLESWAP.u128(), UUSDC_DENOM.to_string()),
            coin(INITIAL_BALANCE_STABLESWAP.u128(), UUSDT_DENOM.to_string()),
            coin(ONE_THOUSAND.u128(), UUSD_DENOM.to_string()),
            coin(TEN_THOUSAND.u128(), UOM_DENOM.to_string()),
            coin(TEN_THOUSAND.u128(), UTEST_DENOM.to_string()),
        ],
        StargateMock::new(vec![coin(ONE_THOUSAND.u128(), UTEST_DENOM.to_string())]),
    );
    let alice = suite.creator();
    let bob = suite.senders[1].clone();
    let asset_denoms = vec![UUSDC_DENOM.to_string(), UUSDT_DENOM.to_string()];

    let pool_fees = PoolFee {
        protocol_fee: Fee {
            share: Decimal::zero(),
        },
        swap_fee: Fee {
            share: Decimal::zero(),
        },
        burn_fee: Fee {
            share: Decimal::zero(),
        },
        extra_fees: vec![],
    };

    // Create a pool
    suite.instantiate_default().add_one_epoch().create_pool(
        &alice,
        asset_denoms,
        vec![3u8, 6u8],
        pool_fees,
        PoolType::StableSwap { amp: 85 },
        None,
        vec![
            coin(ONE_THOUSAND.u128(), UUSD_DENOM.to_string()),
            coin(ONE_THOUSAND.u128(), UTEST_DENOM.to_string()),
        ],
        |result| {
            result.unwrap();
        },
    );

    // Let's try to add liquidity
    suite
        .provide_liquidity(
            &alice,
            "p.1".to_string(),
            None,
            None,
            None,
            None,
            None,
            vec![
                coin(
                    LIQUIDITY_ADD_UUSDC_LOW_DECIMALS.u128(),
                    UUSDC_DENOM.to_string(),
                ),
                coin(
                    LIQUIDITY_ADD_UUSDT_LOW_DECIMALS.u128(),
                    UUSDT_DENOM.to_string(),
                ),
            ],
            |result| {
                println!("result: {:?}", result);
                result.unwrap();
                println!("---");
            },
        )
        .provide_liquidity(
            &bob,
            "p.1".to_string(),
            None,
            None,
            None,
            None,
            None,
            vec![
                coin(
                    LIQUIDITY_ADD_UUSDC_LOW_DECIMALS.u128(),
                    UUSDC_DENOM.to_string(),
                ),
                coin(
                    LIQUIDITY_ADD_UUSDT_LOW_DECIMALS.u128(),
                    UUSDT_DENOM.to_string(),
                ),
            ],
            |result| {
                println!("result: {:?}", result);
                result.unwrap();
            },
        );

    let lp_shares_alice = RefCell::new(Coin::new(0u128, "".to_string()));
    let uusdc_alice = RefCell::new(Coin::new(0u128, "".to_string()));
    let uusdt_alice = RefCell::new(Coin::new(0u128, "".to_string()));
    suite.query_all_balances(&alice.to_string(), |balances| {
        for coin in balances.unwrap().iter() {
            if coin.denom.contains("p.1") {
                *lp_shares_alice.borrow_mut() = coin.clone();
                println!("lp_shares_alice: {:?}", lp_shares_alice.borrow());
            }
            if coin.denom.contains(UUSDC_DENOM) {
                *uusdc_alice.borrow_mut() = coin.clone();
                println!("uusdc_alice: {:?}", uusdc_alice.borrow());
            }
            if coin.denom.contains(UUSDT_DENOM) {
                *uusdt_alice.borrow_mut() = coin.clone();
                println!("uusdt_alice: {:?}", uusdt_alice.borrow());
            }
        }
    });

    let lp_shares_bob = RefCell::new(Coin::new(0u128, "".to_string()));
    let uusdc_bob = RefCell::new(Coin::new(0u128, "".to_string()));
    let uusdt_bob = RefCell::new(Coin::new(0u128, "".to_string()));
    suite.query_all_balances(&bob.to_string(), |balances| {
        for coin in balances.unwrap().iter() {
            if coin.denom.contains("p.1") {
                *lp_shares_bob.borrow_mut() = coin.clone();
                println!("lp_shares_bob: {:?}", lp_shares_bob.borrow());
            }
            if coin.denom.contains(UUSDC_DENOM) {
                *uusdc_bob.borrow_mut() = coin.clone();
                println!("uusdc_bob: {:?}", uusdc_bob.borrow());
            }
            if coin.denom.contains(UUSDT_DENOM) {
                *uusdt_bob.borrow_mut() = coin.clone();
                println!("uusdt_bob: {:?}", uusdt_bob.borrow());
            }
        }
    });

    suite
        .withdraw_liquidity(
            &bob,
            "p.1".to_string(),
            vec![lp_shares_bob.borrow().clone()],
            |result| {
                println!("result: {:?}", result);
                result.unwrap();
            },
        )
        .query_all_balances(&bob.to_string(), |balances| {
            for coin in balances.unwrap().iter() {
                if coin.denom.contains(UUSDC_DENOM) {
                    assert_eq!(
                        coin.amount,
                        uusdc_bob.borrow().clone().amount + LIQUIDITY_ADD_UUSDC_LOW_DECIMALS
                    );
                }
                if coin.denom.contains(UUSDT_DENOM) {
                    assert_eq!(
                        coin.amount,
                        uusdt_bob.borrow().clone().amount + LIQUIDITY_ADD_UUSDT_LOW_DECIMALS
                    );
                }
            }
        });

    suite
        .withdraw_liquidity(
            &alice,
            "p.1".to_string(),
            vec![lp_shares_alice.borrow().clone()],
            |result| {
                println!("result: {:?}", result);
                result.unwrap();
            },
        )
        .query_all_balances(&alice.to_string(), |balances| {
            for coin in balances.unwrap().iter() {
                if coin.denom.contains(UUSDC_DENOM) {
                    assert_approx_eq!(
                        coin.amount,
                        uusdc_alice.borrow().clone().amount + LIQUIDITY_ADD_UUSDC_LOW_DECIMALS,
                        "50"
                    );
                }
                if coin.denom.contains(UUSDT_DENOM) {
                    assert_approx_eq!(
                        coin.amount,
                        uusdt_alice.borrow().clone().amount + LIQUIDITY_ADD_UUSDT_LOW_DECIMALS,
                        "50"
                    );
                }
            }
        });
}

#[allow(clippy::inconsistent_digit_grouping)]
#[test]
fn provide_and_remove_liquidity_18_decimals() {
    let mut suite = TestingSuite::default_with_balances(
        vec![
            coin(ONE_TRILLION.u128(), UUSD_DENOM.to_string()),
            coin(ONE_TRILLION.u128(), UUSDC_DENOM.to_string()),
            coin(LP_AMOUNT_18_DECIMALS.u128(), AUSDT_DENOM.to_string()),
            coin(LP_AMOUNT_18_DECIMALS.u128(), PUSDC_DENOM.to_string()),
            coin(ONE_TRILLION.u128(), UOM_DENOM.to_string()),
        ],
        StargateMock::new(vec![coin(FEE_AMOUNT_UOM.u128(), UOM_DENOM.to_string())]),
    );
    let alice = suite.creator();

    let asset_denoms = vec![AUSDT_DENOM.to_string(), PUSDC_DENOM.to_string()];

    let pool_fees = PoolFee {
        protocol_fee: Fee {
            share: Decimal::zero(),
        },
        swap_fee: Fee {
            share: Decimal::permille(5),
        },
        burn_fee: Fee {
            share: Decimal::zero(),
        },
        extra_fees: vec![],
    };

    // Create a pool
    suite.instantiate_default().add_one_epoch().create_pool(
        &alice,
        asset_denoms,
        vec![18u8, 18u8],
        pool_fees,
        PoolType::ConstantProduct,
        None,
        vec![
            coin(ONE_THOUSAND.u128(), UUSD_DENOM.to_string()),
            coin(FEE_AMOUNT_UOM.u128(), UOM_DENOM.to_string()),
        ],
        |result| {
            result.unwrap();
        },
    );

    // let's provide liquidity 300T pusdc, 300T usdy
    suite.provide_liquidity(
        &alice,
        "p.1".to_string(),
        None,
        None,
        None,
        None,
        None,
        vec![
            Coin {
                denom: PUSDC_DENOM.to_string(),
                amount: LP_AMOUNT_18_DECIMALS,
            },
            Coin {
                denom: AUSDT_DENOM.to_string(),
                amount: LP_AMOUNT_18_DECIMALS,
            },
        ],
        |result| {
            result.unwrap();
        },
    );

    let lp_shares = RefCell::new(Coin::new(0u128, "".to_string()));
    suite.query_all_balances(&alice.to_string(), |balances| {
        for coin in balances.unwrap().iter() {
            if coin.denom.contains("p.1") {
                *lp_shares.borrow_mut() = coin.clone();
            }
        }
    });

    suite
        .query_balance(&alice.to_string(), PUSDC_DENOM.to_string(), |result| {
            assert_eq!(result.unwrap().amount, Uint128::zero());
        })
        .query_balance(&alice.to_string(), AUSDT_DENOM.to_string(), |result| {
            assert_eq!(result.unwrap().amount, Uint128::zero());
        })
        .withdraw_liquidity(
            &alice,
            "p.1".to_string(),
            vec![lp_shares.borrow().clone()],
            |result| {
                result.unwrap();
            },
        )
        .query_balance(&alice.to_string(), PUSDC_DENOM.to_string(), |result| {
            assert_approx_eq!(
                result.unwrap().amount,
                LP_AMOUNT_18_DECIMALS,
                "0.000000000000000001"
            );
        })
        .query_balance(&alice.to_string(), AUSDT_DENOM.to_string(), |result| {
            assert_approx_eq!(
                result.unwrap().amount,
                LP_AMOUNT_18_DECIMALS,
                "0.000000000000000001"
            );
        });
}
