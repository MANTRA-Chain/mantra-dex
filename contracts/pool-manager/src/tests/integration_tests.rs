use cosmwasm_std::{coin, Addr, Coin, Decimal, Uint128};

use amm::fee::Fee;
use amm::fee::PoolFee;
use amm::lp_common::MINIMUM_LIQUIDITY_AMOUNT;
use amm::pool_manager::PoolType;
use common_testing::multi_test::stargate_mock::StargateMock;

use crate::ContractError;

use super::suite::TestingSuite;

#[test]
fn instantiate_normal() {
    let mut suite = TestingSuite::default_with_balances(
        vec![],
        StargateMock::new("uom".to_string(), "8888".to_string()),
    );

    suite.instantiate(suite.senders[0].to_string(), suite.senders[1].to_string());
}

#[test]
fn deposit_and_withdraw_sanity_check() {
    let mut suite = TestingSuite::default_with_balances(
        vec![
            coin(1_000_000u128, "uwhale".to_string()),
            coin(1_000_000u128, "uluna".to_string()),
            coin(1_000u128, "uusd".to_string()),
            coin(10_000u128, "uom".to_string()),
        ],
        StargateMock::new("uom".to_string(), "8888".to_string()),
    );
    let creator = suite.creator();
    let _other = suite.senders[1].clone();
    let _unauthorized = suite.senders[2].clone();

    // Asset denoms with uwhale and uluna
    let asset_denoms = vec!["uwhale".to_string(), "uluna".to_string()];

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
                // Ensure we got 999_000 in the response which is 1_000_000 less the initial liquidity amount
                assert!(result.unwrap().events.iter().any(|event| {
                    event.attributes.iter().any(|attr| {
                        attr.key == "share"
                            && attr.value
                                == (Uint128::from(1_000_000u128) - MINIMUM_LIQUIDITY_AMOUNT)
                                    .to_string()
                    })
                }));
            },
        )
        // creator should have 999_000 LP shares (1M - MINIMUM_LIQUIDITY_AMOUNT)
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

    // Let's try to withdraw liquidity
    suite
        .withdraw_liquidity(
            &creator,
            "o.whale.uluna".to_string(),
            vec![Coin {
                denom: lp_denom.clone(),
                amount: Uint128::from(999_000u128),
            }],
            |result| {
                // we're trading 999_000 shares for 1_000_000 of our liquidity
                assert!(result.unwrap().events.iter().any(|event| {
                    event.attributes.iter().any(|attr| {
                        attr.key == "withdrawn_share"
                            && attr.value
                                == (Uint128::from(1_000_000u128) - MINIMUM_LIQUIDITY_AMOUNT)
                                    .to_string()
                    })
                }));
            },
        )
        // creator should have 0 LP shares in the contract and 0 LP shares in their account balance
        .query_amount_of_lp_token(
            "o.whale.uluna".to_string(),
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
                coin.denom == *"uwhale"
                    && coin.amount == Uint128::from(1_000_000u128) - MINIMUM_LIQUIDITY_AMOUNT
            }));
            assert!(balances.iter().any(|coin| {
                coin.denom == *"uluna"
                    && coin.amount == Uint128::from(1_000_000u128) - MINIMUM_LIQUIDITY_AMOUNT
            }));
        });
}

mod pool_creation_failures {
    use common_testing::multi_test::stargate_mock::StargateMock;

    use super::*;

    // Insufficient fee to create pool; 90 instead of 100
    #[test]
    fn insufficient_pool_creation_fee() {
        let mut suite = TestingSuite::default_with_balances(
            vec![
                coin(1_000_000_001u128, "uwhale".to_string()),
                coin(1_000_000_000u128, "uluna".to_string()),
                coin(1_000_000_001u128, "uusd".to_string()),
                coin(1_000_000_001u128, "uom".to_string()),
            ],
            StargateMock::new("uom".to_string(), "8888".to_string()),
        );
        let creator = suite.creator();
        let _other = suite.senders[1].clone();
        let _unauthorized = suite.senders[2].clone();
        // Asset infos with uwhale

        let asset_infos = vec!["uwhale".to_string(), "uom".to_string()];

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
            asset_infos,
            vec![6u8, 6u8],
            pool_fees,
            PoolType::ConstantProduct,
            None,
            vec![coin(90, "uusd")],
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();
                match err {
                    ContractError::InvalidPoolCreationFee { .. } => {}
                    _ => panic!(
                        "Wrong error type, should return ContractError::InvalidPoolCreationFee"
                    ),
                }
            },
        );
    }

    // Only 1 asset provided, or none
    #[test]
    fn invalid_assets_on_pool_creation() {
        let mut suite = TestingSuite::default_with_balances(
            vec![
                coin(1_000_000_001u128, "uwhale".to_string()),
                coin(1_000_000_000u128, "uluna".to_string()),
                coin(1_000_000_001u128, "uusd".to_string()),
                coin(1_000_000_001u128, "uom".to_string()),
            ],
            StargateMock::new("uom".to_string(), "8888".to_string()),
        );
        let creator = suite.creator();

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
        suite
            .instantiate_default()
            .add_one_epoch()
            .create_pool(
                &creator,
                vec![],
                vec![6u8, 6u8],
                pool_fees.clone(),
                PoolType::ConstantProduct,
                None,
                vec![coin(1000, "uusd"), coin(8888, "uom")],
                |result| {
                    let err = result.unwrap_err().downcast::<ContractError>().unwrap();
                    match err {
                        ContractError::AssetMismatch { .. } => {}
                        _ => panic!("Wrong error type, should return ContractError::AssetMismatch"),
                    }
                },
            )
            .create_pool(
                &creator,
                vec!["uom".to_string()],
                vec![6u8],
                pool_fees.clone(),
                PoolType::ConstantProduct,
                None,
                vec![coin(1000, "uusd"), coin(8888, "uom")],
                |result| {
                    let err = result.unwrap_err().downcast::<ContractError>().unwrap();
                    match err {
                        ContractError::AssetMismatch { .. } => {}
                        _ => panic!("Wrong error type, should return ContractError::AssetMismatch"),
                    }
                },
            )
            .create_pool(
                &creator,
                vec!["uom".to_string(), "uom".to_string()],
                vec![6u8, 6u8],
                pool_fees,
                PoolType::ConstantProduct,
                None,
                vec![coin(1000, "uusd"), coin(8888, "uom")],
                |result| {
                    let err = result.unwrap_err().downcast::<ContractError>().unwrap();
                    match err {
                        ContractError::SameAsset { .. } => {}
                        _ => panic!("Wrong error type, should return ContractError::SameAsset"),
                    }
                },
            );
    }

    #[test]
    fn sends_more_funds_than_needed() {
        let mut suite = TestingSuite::default_with_balances(
            vec![
                coin(1_000_000_001u128, "uwhale".to_string()),
                coin(1_000_000_000u128, "uluna".to_string()),
                coin(1_000_000_001u128, "uusd".to_string()),
                coin(1_000_000_001u128, "uom".to_string()),
            ],
            StargateMock::new("uom".to_string(), "8888".to_string()),
        );
        let creator = suite.creator();

        let asset_infos = vec!["uom".to_string(), "uusd".to_string()];

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

        suite
            .instantiate_default()
            .add_one_epoch()
            .create_pool(
                &creator,
                asset_infos.clone(),
                vec![6u8, 6u8],
                pool_fees.clone(),
                PoolType::ConstantProduct,
                None,
                vec![coin(8888, "uom"), coin(1000, "uusd"), coin(1000, "uluna")],
                |result| {
                    let err = result.unwrap_err().downcast::<ContractError>().unwrap();
                    match err {
                        ContractError::ExtraFundsSent { .. } => {}
                        _ => {
                            panic!("Wrong error type, should return ContractError::ExtraFundsSent")
                        }
                    }
                },
            )
            .create_pool(
                &creator,
                asset_infos,
                vec![6u8, 6u8],
                pool_fees,
                PoolType::ConstantProduct,
                None,
                vec![coin(8888, "uom"), coin(1000, "uusd")],
                |result| {
                    result.unwrap();
                },
            );
    }

    #[test]
    fn wrong_pool_label() {
        let mut suite = TestingSuite::default_with_balances(
            vec![
                coin(
                    1_000_000_001u128,
                    "ibc/3A6F4C8D5B2E7A1F0C4D5B6E7A8F9C3D4E5B6A7F8E9C4D5B6E7A8F9C3D4E5B6A"
                        .to_string(),
                ),
                coin(
                    1_000_000_000u128,
                    "ibc/A1B2C3D4E5F6G7H8I9J0K1L2M3N4O5P6Q7R8S9T0U1V2W3X4Y5Z6A7B8C9D0E1F2"
                        .to_string(),
                ),
                coin(
                    1_000_000_001u128,
                    "factory/mantra158xlpsqqkqpkmcrgnlcrc5fjyhy7j7x2vpa79r/subdenom".to_string(),
                ),
                coin(1_000_000_001u128, "uusd".to_string()),
                coin(1_000_000_001u128, "uom".to_string()),
            ],
            StargateMock::new("uom".to_string(), "8888".to_string()),
        );
        let creator = suite.creator();
        let _other = suite.senders[1].clone();
        let _unauthorized = suite.senders[2].clone();
        // Asset infos with uwhale

        let asset_infos = vec![
            "ibc/3A6F4C8D5B2E7A1F0C4D5B6E7A8F9C3D4E5B6A7F8E9C4D5B6E7A8F9C3D4E5B6A".to_string(),
            "ibc/A1B2C3D4E5F6G7H8I9J0K1L2M3N4O5P6Q7R8S9T0U1V2W3X4Y5Z6A7B8C9D0E1F2".to_string(),
        ];

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

        suite
            .instantiate_default()
            .add_one_epoch()
            .create_pool(
                &creator,
                asset_infos.clone(),
                vec![6u8, 6u8],
                pool_fees.clone(),
                PoolType::ConstantProduct,
                Some("invalid-identifier".to_string()),
                vec![coin(1_000, "uusd"), coin(8888, "uom")],
                |result| {
                    let err = result.unwrap_err().downcast::<ContractError>().unwrap();
                    match err {
                        ContractError::InvalidPoolIdentifier { .. } => {}
                        _ => panic!(
                            "Wrong error type, should return ContractError::InvalidPoolIdentifier"
                        ),
                    }
                },
            )
            .create_pool(
                &creator,
                asset_infos.clone(),
                vec![6u8, 6u8],
                pool_fees.clone(),
                PoolType::ConstantProduct,
                //42 chars long
                Some("this.is.a.loooooooooooooooooong.identifier".to_string()),
                vec![coin(1_000, "uusd"), coin(8888, "uom")],
                |result| {
                    let err = result.unwrap_err().downcast::<ContractError>().unwrap();
                    match err {
                        ContractError::InvalidPoolIdentifier { .. } => {}
                        _ => panic!(
                            "Wrong error type, should return ContractError::InvalidPoolIdentifier"
                        ),
                    }
                },
            );
    }

    #[test]
    fn cant_recreate_existing_pool() {
        let mut suite = TestingSuite::default_with_balances(
            vec![
                coin(1_000_000_001u128, "uwhale".to_string()),
                coin(1_000_000_000u128, "uluna".to_string()),
                coin(1_000_000_001u128, "uusd".to_string()),
                coin(1_000_000_001u128, "uom".to_string()),
            ],
            StargateMock::new("uom".to_string(), "8888".to_string()),
        );
        let creator = suite.creator();
        let _other = suite.senders[1].clone();
        let _unauthorized = suite.senders[2].clone();
        // Asset infos with uwhale

        let asset_infos = vec!["uwhale".to_string(), "uom".to_string()];

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
        suite
            .instantiate_default()
            .add_one_epoch()
            .create_pool(
                &creator,
                asset_infos.clone(),
                vec![6u8, 6u8],
                pool_fees.clone(),
                PoolType::ConstantProduct,
                Some("mycoolpool".to_string()),
                vec![coin(1000, "uusd"), coin(8888, "uom")],
                |result| {
                    result.unwrap();
                },
            )
            .create_pool(
                &creator,
                asset_infos,
                vec![6u8, 6u8],
                pool_fees,
                PoolType::ConstantProduct,
                Some("mycoolpool".to_string()),
                vec![coin(1000, "uusd"), coin(8888, "uom")],
                |result| {
                    let err = result.unwrap_err().downcast::<ContractError>().unwrap();
                    match err {
                        ContractError::PoolExists { .. } => {}
                        _ => panic!("Wrong error type, should return ContractError::PoolExists"),
                    }
                },
            );
    }

    #[test]
    fn cant_create_pool_without_paying_tf_fees() {
        let mut suite = TestingSuite::default_with_balances(
            vec![
                coin(1_000_000_001u128, "uwhale".to_string()),
                coin(1_000_000_000u128, "uluna".to_string()),
                coin(1_000_000_001u128, "uusd".to_string()),
                coin(1_000_000_001u128, "uom".to_string()),
            ],
            StargateMock::new("uom".to_string(), "8888".to_string()),
        );
        let creator = suite.creator();

        let asset_denoms = vec!["uwhale".to_string(), "uluna".to_string()];

        let pool_fees = PoolFee {
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

        // Create a pool without paying the pool creation fee
        suite
            .instantiate_default()
            .add_one_epoch()
            .create_pool(
                &creator,
                asset_denoms.clone(),
                vec![6u8, 6u8],
                pool_fees.clone(),
                PoolType::ConstantProduct,
                Some("whale.uluna.pool.1".to_string()),
                vec![coin(900, "uusd"), coin(8888, "uom")],
                |result| {
                    let err = result.unwrap_err().downcast::<ContractError>().unwrap();
                    match err {
                        ContractError::InvalidPoolCreationFee { .. } => {}
                        _ => panic!(
                            "Wrong error type, should return ContractError::InvalidPoolCreationFee"
                        ),
                    }
                },
            )
            // add enough to cover the pool creation fee, but not token factory
            .create_pool(
                &creator,
                asset_denoms.clone(),
                vec![6u8, 6u8],
                pool_fees.clone(),
                PoolType::ConstantProduct,
                Some("o.whale.uluna.pool.1".to_string()),
                vec![coin(1000, "uusd"), coin(8887, "uom")],
                |result| {
                    let err = result.unwrap_err().downcast::<ContractError>().unwrap();

                    match err {
                        ContractError::TokenFactoryFeeNotPaid => {}
                        _ => panic!(
                            "Wrong error type, should return ContractError::TokenFactoryFeeNotPaid"
                        ),
                    }
                },
            )
            // add enough to cover for the pool creation fee and token factory
            .create_pool(
                &creator,
                asset_denoms.clone(),
                vec![6u8, 6u8],
                pool_fees.clone(),
                PoolType::ConstantProduct,
                Some("o.whale.uluna.pool.1".to_string()),
                vec![coin(1000, "uusd"), coin(8888, "uom")],
                |result| {
                    result.unwrap();
                },
            );
    }

    #[test]
    fn cant_create_pool_without_paying_tf_fees_same_denom() {
        let mut suite = TestingSuite::default_with_balances(
            vec![
                coin(1_000_000_001u128, "uwhale".to_string()),
                coin(1_000_000_000u128, "uluna".to_string()),
                coin(1_000_000_001u128, "uusd".to_string()),
                coin(1_000_000_001u128, "uom".to_string()),
            ],
            StargateMock::new("uusd".to_string(), "1000".to_string()),
        );
        let creator = suite.creator();

        let asset_denoms = vec!["uwhale".to_string(), "uluna".to_string()];

        let pool_fees = PoolFee {
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

        // Create a pool without paying the pool creation fee
        suite
            .instantiate_default()
            .add_one_epoch()
            .create_pool(
                &creator,
                asset_denoms.clone(),
                vec![6u8, 6u8],
                pool_fees.clone(),
                PoolType::ConstantProduct,
                Some("whale.uluna.pool.1".to_string()),
                vec![coin(900, "uusd")],
                |result| {
                    let err = result.unwrap_err().downcast::<ContractError>().unwrap();
                    match err {
                        ContractError::InvalidPoolCreationFee { .. } => {}
                        _ => panic!(
                            "Wrong error type, should return ContractError::InvalidPoolCreationFee"
                        ),
                    }
                },
            )
            // add enough to cover the pool creation fee, but not token factory
            .create_pool(
                &creator,
                asset_denoms.clone(),
                vec![6u8, 6u8],
                pool_fees.clone(),
                PoolType::ConstantProduct,
                Some("whale.uluna.pool.1".to_string()),
                vec![coin(1999, "uusd")],
                |result| {
                    let err = result.unwrap_err().downcast::<ContractError>().unwrap();
                    match err {
                        ContractError::InvalidPoolCreationFee { amount, expected } => {
                            assert_eq!(amount.u128(), 1999);
                            assert_eq!(expected.u128(), 2000);
                        }
                        _ => panic!(
                            "Wrong error type, should return ContractError::InvalidPoolCreationFee"
                        ),
                    }
                },
            )
            // overpay
            .create_pool(
                &creator,
                asset_denoms.clone(),
                vec![6u8, 6u8],
                pool_fees.clone(),
                PoolType::ConstantProduct,
                Some("whale.uluna.pool.1".to_string()),
                vec![coin(3000, "uusd")],
                |result| {
                    let err = result.unwrap_err().downcast::<ContractError>().unwrap();

                    match err {
                        ContractError::InvalidPoolCreationFee { amount, expected } => {
                            assert_eq!(amount.u128(), 3000);
                            assert_eq!(expected.u128(), 2000);
                        }
                        _ => panic!(
                            "Wrong error type, should return ContractError::InvalidPoolCreationFee"
                        ),
                    }
                },
            )
            // add enough to cover for the pool creation fee and token factory
            .create_pool(
                &creator,
                asset_denoms.clone(),
                vec![6u8, 6u8],
                pool_fees.clone(),
                PoolType::ConstantProduct,
                Some("whale.uluna.pool.1".to_string()),
                vec![coin(2000, "uusd")],
                |result| {
                    result.unwrap();
                },
            );
    }
}

mod router {
    use cosmwasm_std::{assert_approx_eq, Event, StdError};

    use super::*;

    #[test]
    fn basic_swap_operations_test() {
        let mut suite = TestingSuite::default_with_balances(
            vec![
                coin(1_000_000_000u128, "uwhale".to_string()),
                coin(1_000_000_000u128, "uluna".to_string()),
                coin(1_000_000_000u128, "uusd".to_string()),
                coin(1_000_000_000u128, "uom".to_string()),
            ],
            StargateMock::new("uom".to_string(), "8888".to_string()),
        );
        let creator = suite.creator();
        let _other = suite.senders[1].clone();
        let _unauthorized = suite.senders[2].clone();
        // Asset infos with uwhale and uluna

        let first_pool = vec!["uwhale".to_string(), "uluna".to_string()];
        let second_pool = vec!["uluna".to_string(), "uusd".to_string()];

        let pool_fees = PoolFee {
            protocol_fee: Fee {
                share: Decimal::bps(50), // 0.5%
            },
            swap_fee: Fee {
                share: Decimal::bps(50), // 0.5%
            },
            burn_fee: Fee {
                share: Decimal::bps(50), // 0.5%
            },
            extra_fees: vec![],
        };

        // Create a pool
        suite
            .instantiate_default()
            .add_one_epoch()
            .create_pool(
                &creator,
                first_pool,
                vec![6u8, 6u8],
                pool_fees.clone(),
                PoolType::ConstantProduct,
                Some("whale.uluna".to_string()),
                vec![coin(1000, "uusd"), coin(8888, "uom")],
                |result| {
                    result.unwrap();
                },
            )
            .create_pool(
                &creator,
                second_pool,
                vec![6u8, 6u8],
                pool_fees,
                PoolType::ConstantProduct,
                Some("uluna.uusd".to_string()),
                vec![coin(1000, "uusd"), coin(8888, "uom")],
                |result| {
                    result.unwrap();
                },
            );

        // Let's try to add liquidity
        suite.provide_liquidity(
            &creator,
            "o.whale.uluna".to_string(),
            None,
            None,
            None,
            None,
            vec![
                Coin {
                    denom: "uwhale".to_string(),
                    amount: Uint128::from(1000000u128),
                },
                Coin {
                    denom: "uluna".to_string(),
                    amount: Uint128::from(1000000u128),
                },
            ],
            |result| {
                // ensure we got 999,000 in the response (1m - initial liquidity amount)
                let result = result.unwrap();
                assert!(result.has_event(&Event::new("wasm").add_attribute("share", "999000")));
            },
        );

        // Let's try to add liquidity
        suite.provide_liquidity(
            &creator,
            "o.uluna.uusd".to_string(),
            None,
            None,
            None,
            None,
            vec![
                Coin {
                    denom: "uluna".to_string(),
                    amount: Uint128::from(1000000u128),
                },
                Coin {
                    denom: "uusd".to_string(),
                    amount: Uint128::from(1000000u128),
                },
            ],
            |result| {
                // ensure we got 999,000 in the response (1m - initial liquidity amount)
                let result = result.unwrap();
                assert!(result.has_event(&Event::new("wasm").add_attribute("share", "999000")));
            },
        );

        // Prepare the swap operations, we want to go from WHALE -> UUSD
        // We will use the o.uluna.uusd pool as the intermediary pool

        let swap_operations = vec![
            amm::pool_manager::SwapOperation::MantraSwap {
                token_in_denom: "uwhale".to_string(),
                token_out_denom: "uluna".to_string(),
                pool_identifier: "o.whale.uluna".to_string(),
            },
            amm::pool_manager::SwapOperation::MantraSwap {
                token_in_denom: "uluna".to_string(),
                token_out_denom: "uusd".to_string(),
                pool_identifier: "o.uluna.uusd".to_string(),
            },
        ];

        // before swap uusd balance = 1_000_000_000
        // - 2*1_000 pool creation fee
        // - 1_000_000 liquidity provision
        // = 998_998_000
        let pre_swap_amount = 998_998_000;
        suite.query_balance(&creator.to_string(), "uusd".to_string(), |amt| {
            assert_eq!(amt.unwrap().amount.u128(), pre_swap_amount);
        });

        suite.execute_swap_operations(
            &creator,
            swap_operations,
            None,
            None,
            None,
            vec![coin(1000u128, "uwhale".to_string())],
            |result| {
                result.unwrap();
            },
        );

        // ensure that the whale got swapped to an appropriate amount of uusd
        // we swap 1000 whale for 974 uusd
        // with a fee of 4*6 = 24 uusd
        let post_swap_amount = pre_swap_amount + 974;
        suite.query_balance(&creator.to_string(), "uusd".to_string(), |amt| {
            assert_eq!(amt.unwrap().amount.u128(), post_swap_amount);
        });

        // ensure that fees got sent to the appropriate place
        suite.query_balance(
            &suite.fee_collector_addr.to_string(),
            "uusd".to_string(),
            |amt| {
                assert_eq!(amt.unwrap().amount.u128(), 2000 + 4);
            },
        );
        suite.query_balance(
            &suite.fee_collector_addr.to_string(),
            "uwhale".to_string(),
            |amt| {
                assert_eq!(amt.unwrap().amount.u128(), 0);
            },
        );
        suite.query_balance(
            &suite.fee_collector_addr.to_string(),
            "uluna".to_string(),
            |amt| {
                assert_eq!(amt.unwrap().amount.u128(), 4);
            },
        );
    }

    #[test]
    fn rejects_empty_swaps() {
        let mut suite = TestingSuite::default_with_balances(
            vec![
                coin(1_000_000_001u128, "uwhale".to_string()),
                coin(1_000_000_000u128, "uluna".to_string()),
                coin(1_000_000_001u128, "uusd".to_string()),
                coin(1_000_000_001u128, "uom".to_string()),
            ],
            StargateMock::new("uom".to_string(), "8888".to_string()),
        );
        let creator = suite.creator();
        let _other = suite.senders[1].clone();
        let _unauthorized = suite.senders[2].clone();
        // Asset infos with uwhale and uluna

        let first_pool = vec!["uwhale".to_string(), "uluna".to_string()];
        let second_pool = vec!["uluna".to_string(), "uusd".to_string()];

        // Protocol fee is 0.01% and swap fee is 0.02% and burn fee is 0%
        let pool_fees = PoolFee {
            protocol_fee: Fee {
                share: Decimal::from_ratio(1u128, 100_000u128),
            },
            swap_fee: Fee {
                share: Decimal::from_ratio(1u128, 100_000u128),
            },
            burn_fee: Fee {
                share: Decimal::zero(),
            },
            extra_fees: vec![],
        };

        // Create a pool
        suite
            .instantiate_default()
            .add_one_epoch()
            .create_pool(
                &creator,
                first_pool,
                vec![6u8, 6u8],
                pool_fees.clone(),
                PoolType::ConstantProduct,
                Some("whale.uluna".to_string()),
                vec![coin(1000, "uusd"), coin(8888, "uom")],
                |result| {
                    result.unwrap();
                },
            )
            .create_pool(
                &creator,
                second_pool,
                vec![6u8, 6u8],
                pool_fees,
                PoolType::ConstantProduct,
                Some("uluna.uusd".to_string()),
                vec![coin(1000, "uusd"), coin(8888, "uom")],
                |result| {
                    result.unwrap();
                },
            );

        // Let's try to add liquidity
        suite.provide_liquidity(
            &creator,
            "o.whale.uluna".to_string(),
            None,
            None,
            None,
            None,
            vec![
                Coin {
                    denom: "uwhale".to_string(),
                    amount: Uint128::from(1000000u128),
                },
                Coin {
                    denom: "uluna".to_string(),
                    amount: Uint128::from(1000000u128),
                },
            ],
            |result| {
                // ensure we got 999,000 in the response (1m - initial liquidity amount)
                let result = result.unwrap();
                assert!(result.has_event(&Event::new("wasm").add_attribute("share", "999000")));
            },
        );

        // Let's try to add liquidity
        suite.provide_liquidity(
            &creator,
            "o.uluna.uusd".to_string(),
            None,
            None,
            None,
            None,
            vec![
                Coin {
                    denom: "uluna".to_string(),
                    amount: Uint128::from(1000000u128),
                },
                Coin {
                    denom: "uusd".to_string(),
                    amount: Uint128::from(1000000u128),
                },
            ],
            |result| {
                // ensure we got 999,000 in the response (1m - initial liquidity amount)
                let result = result.unwrap();
                assert!(result.has_event(&Event::new("wasm").add_attribute("share", "999000")));
            },
        );

        // attempt to perform a 0 swap operations
        let swap_operations = vec![];

        suite.execute_swap_operations(
            &creator,
            swap_operations,
            None,
            None,
            None,
            vec![coin(1000u128, "uwhale".to_string())],
            |result| {
                assert_eq!(
                    result.unwrap_err().downcast_ref::<ContractError>(),
                    Some(&ContractError::NoSwapOperationsProvided)
                )
            },
        );
    }

    #[test]
    fn rejects_non_consecutive_swaps() {
        let mut suite = TestingSuite::default_with_balances(
            vec![
                coin(1_000_000_001u128, "uwhale".to_string()),
                coin(1_000_000_000u128, "uluna".to_string()),
                coin(1_000_000_001u128, "uusd".to_string()),
                coin(1_000_000_001u128, "uom".to_string()),
            ],
            StargateMock::new("uom".to_string(), "8888".to_string()),
        );
        let creator = suite.creator();
        let other = suite.senders[1].clone();
        let _unauthorized = suite.senders[2].clone();
        // Asset infos with uwhale and uluna

        let first_pool = vec!["uwhale".to_string(), "uluna".to_string()];
        let second_pool = vec!["uluna".to_string(), "uusd".to_string()];

        // Protocol fee is 0.01% and swap fee is 0.02% and burn fee is 0%
        let pool_fees = PoolFee {
            protocol_fee: Fee {
                share: Decimal::from_ratio(1u128, 100_000u128),
            },
            swap_fee: Fee {
                share: Decimal::from_ratio(1u128, 100_000u128),
            },
            burn_fee: Fee {
                share: Decimal::zero(),
            },
            extra_fees: vec![],
        };

        // Create a pool
        suite
            .instantiate_default()
            .add_one_epoch()
            .create_pool(
                &creator,
                first_pool,
                vec![6u8, 6u8],
                pool_fees.clone(),
                PoolType::ConstantProduct,
                Some("whale.uluna".to_string()),
                vec![coin(1000, "uusd"), coin(8888, "uom")],
                |result| {
                    result.unwrap();
                },
            )
            .create_pool(
                &creator,
                second_pool,
                vec![6u8, 6u8],
                pool_fees,
                PoolType::ConstantProduct,
                Some("uluna.uusd".to_string()),
                vec![coin(1000, "uusd"), coin(8888, "uom")],
                |result| {
                    result.unwrap();
                },
            );

        // Let's try to add liquidity
        suite.provide_liquidity(
            &creator,
            "o.whale.uluna".to_string(),
            None,
            None,
            None,
            None,
            vec![
                Coin {
                    denom: "uwhale".to_string(),
                    amount: Uint128::from(1000000u128),
                },
                Coin {
                    denom: "uluna".to_string(),
                    amount: Uint128::from(1000000u128),
                },
            ],
            |result| {
                // ensure we got 999,000 in the response (1m - initial liquidity amount)
                let result = result.unwrap();
                assert!(result.has_event(&Event::new("wasm").add_attribute("share", "999000")));
            },
        );

        // Let's try to add liquidity
        suite.provide_liquidity(
            &creator,
            "o.uluna.uusd".to_string(),
            None,
            None,
            None,
            None,
            vec![
                Coin {
                    denom: "uluna".to_string(),
                    amount: Uint128::from(1000000u128),
                },
                Coin {
                    denom: "uusd".to_string(),
                    amount: Uint128::from(1000000u128),
                },
            ],
            |result| {
                // ensure we got 999,000 in the response (1m - initial liquidity amount)
                let result = result.unwrap();
                assert!(result.has_event(&Event::new("wasm").add_attribute("share", "999000")));
            },
        );

        // Prepare the swap operations, we want to go from WHALE -> UUSD
        // We will use the o.uluna.uusd pool as the intermediary pool

        let swap_operations = vec![
            amm::pool_manager::SwapOperation::MantraSwap {
                token_in_denom: "uwhale".to_string(),
                token_out_denom: "uluna".to_string(),
                pool_identifier: "o.whale.uluna".to_string(),
            },
            amm::pool_manager::SwapOperation::MantraSwap {
                token_in_denom: "uwhale".to_string(),
                token_out_denom: "uluna".to_string(),
                pool_identifier: "o.whale.uluna".to_string(),
            },
        ];

        suite.execute_swap_operations(
            &other,
            swap_operations,
            None,
            None,
            None,
            vec![coin(1000u128, "uwhale".to_string())],
            |result| {
                assert_eq!(
                    result.unwrap_err().downcast_ref::<self::ContractError>(),
                    Some(&ContractError::NonConsecutiveSwapOperations {
                        previous_output: "uluna".to_string(),
                        next_input: "uwhale".to_string(),
                    })
                );
            },
        );
    }

    #[test]
    fn sends_to_correct_receiver() {
        let mut suite = TestingSuite::default_with_balances(
            vec![
                coin(1_000_000_000u128, "uwhale".to_string()),
                coin(1_000_000_000u128, "uluna".to_string()),
                coin(1_000_000_000u128, "uusd".to_string()),
                coin(1_000_000_000u128, "uom".to_string()),
            ],
            StargateMock::new("uom".to_string(), "8888".to_string()),
        );
        let creator = suite.creator();
        let other = suite.senders[1].clone();
        let unauthorized = suite.senders[2].clone();
        // Asset infos with uwhale and uluna

        let first_pool = vec!["uwhale".to_string(), "uluna".to_string()];
        let second_pool = vec!["uluna".to_string(), "uusd".to_string()];

        // Protocol fee is 0.01% and swap fee is 0.02% and burn fee is 0%
        let pool_fees = PoolFee {
            protocol_fee: Fee {
                share: Decimal::from_ratio(1u128, 100_000u128),
            },
            swap_fee: Fee {
                share: Decimal::from_ratio(1u128, 100_000u128),
            },
            burn_fee: Fee {
                share: Decimal::zero(),
            },
            extra_fees: vec![],
        };

        // Create a pool
        suite
            .instantiate_default()
            .add_one_epoch()
            .create_pool(
                &creator,
                first_pool,
                vec![6u8, 6u8],
                pool_fees.clone(),
                PoolType::ConstantProduct,
                Some("whale.uluna".to_string()),
                vec![coin(1000, "uusd"), coin(8888, "uom")],
                |result| {
                    result.unwrap();
                },
            )
            .create_pool(
                &creator,
                second_pool,
                vec![6u8, 6u8],
                pool_fees,
                PoolType::ConstantProduct,
                Some("uluna.uusd".to_string()),
                vec![coin(1000, "uusd"), coin(8888, "uom")],
                |result| {
                    result.unwrap();
                },
            );

        // Let's try to add liquidity
        let liquidity_amount = 1_000_000u128;
        suite.provide_liquidity(
            &creator,
            "o.whale.uluna".to_string(),
            None,
            None,
            None,
            None,
            vec![
                Coin {
                    denom: "uwhale".to_string(),
                    amount: Uint128::from(liquidity_amount),
                },
                Coin {
                    denom: "uluna".to_string(),
                    amount: Uint128::from(liquidity_amount),
                },
            ],
            |result| {
                // ensure we got 999,000 in the response (1m - initial liquidity amount)
                let result = result.unwrap();
                assert!(result.has_event(&Event::new("wasm").add_attribute("share", "999000")));
            },
        );

        // Let's try to add liquidity
        suite.provide_liquidity(
            &creator,
            "o.uluna.uusd".to_string(),
            None,
            None,
            None,
            None,
            vec![
                Coin {
                    denom: "uluna".to_string(),
                    amount: Uint128::from(liquidity_amount),
                },
                Coin {
                    denom: "uusd".to_string(),
                    amount: Uint128::from(liquidity_amount),
                },
            ],
            |result| {
                // ensure we got 999,000 in the response (1m - initial liquidity amount)
                let result = result.unwrap();
                assert!(result.has_event(&Event::new("wasm").add_attribute("share", "999000")));
            },
        );

        // Prepare the swap operations, we want to go from WHALE -> UUSD
        // We will use the o.uluna.uusd pool as the intermediary pool

        let swap_operations = vec![
            amm::pool_manager::SwapOperation::MantraSwap {
                token_in_denom: "uwhale".to_string(),
                token_out_denom: "uluna".to_string(),
                pool_identifier: "o.whale.uluna".to_string(),
            },
            amm::pool_manager::SwapOperation::MantraSwap {
                token_in_denom: "uluna".to_string(),
                token_out_denom: "uusd".to_string(),
                pool_identifier: "o.uluna.uusd".to_string(),
            },
        ];

        // before swap uusd balance = 1_000_000_000
        // before swap uwhale balance = 1_000_000_000
        // before swap uluna balance = 1_000_000_000
        let pre_swap_amount = 1_000_000_000;
        suite.query_balance(&other.to_string(), "uusd".to_string(), |amt| {
            assert_eq!(amt.unwrap().amount.u128(), pre_swap_amount);
        });
        suite.query_balance(&other.to_string(), "uwhale".to_string(), |amt| {
            assert_eq!(amt.unwrap().amount.u128(), pre_swap_amount);
        });
        suite.query_balance(&other.to_string(), "uluna".to_string(), |amt| {
            assert_eq!(amt.unwrap().amount.u128(), pre_swap_amount);
        });
        // also check the same for unauthorized receiver
        suite.query_balance(&other.to_string(), "uusd".to_string(), |amt| {
            assert_eq!(amt.unwrap().amount.u128(), pre_swap_amount);
        });
        suite.query_balance(&other.to_string(), "uwhale".to_string(), |amt| {
            assert_eq!(amt.unwrap().amount.u128(), pre_swap_amount);
        });
        suite.query_balance(&other.to_string(), "uluna".to_string(), |amt| {
            assert_eq!(amt.unwrap().amount.u128(), pre_swap_amount);
        });
        // also check for contract
        suite.query_balance(
            &suite.pool_manager_addr.to_string(),
            "uusd".to_string(),
            |amt| {
                assert_eq!(amt.unwrap().amount.u128(), liquidity_amount);
            },
        );
        suite.query_balance(
            &suite.pool_manager_addr.to_string(),
            "uwhale".to_string(),
            |amt| {
                assert_eq!(amt.unwrap().amount.u128(), liquidity_amount);
            },
        );
        suite.query_balance(
            &suite.pool_manager_addr.to_string(),
            "uluna".to_string(),
            |amt| {
                assert_eq!(amt.unwrap().amount.u128(), 2 * liquidity_amount);
            },
        );

        // perform swaps
        suite.execute_swap_operations(
            &other,
            swap_operations,
            None,
            Some(unauthorized.to_string()),
            None,
            vec![coin(1000u128, "uwhale".to_string())],
            |result| {
                result.unwrap();
            },
        );

        // ensure that the whale got swapped to an appropriate amount of uusd
        // we swap 1000 whale for 998 uusd
        let post_swap_amount = pre_swap_amount + 998;
        suite.query_balance(&unauthorized.to_string(), "uusd".to_string(), |amt| {
            assert_eq!(amt.unwrap().amount.u128(), post_swap_amount);
        });
        // check that the balances of the contract are ok
        suite.query_balance(
            &suite.pool_manager_addr.to_string(),
            "uusd".to_string(),
            |amt| {
                assert_eq!(amt.unwrap().amount.u128(), liquidity_amount - 998);
            },
        );
        suite.query_balance(
            &suite.pool_manager_addr.to_string(),
            "uwhale".to_string(),
            |amt| {
                assert_eq!(amt.unwrap().amount.u128(), liquidity_amount + 1000);
            },
        );
        suite.query_balance(
            &suite.pool_manager_addr.to_string(),
            "uluna".to_string(),
            |amt| {
                assert_eq!(amt.unwrap().amount.u128(), 2 * liquidity_amount);
            },
        );
    }

    #[test]
    fn checks_minimum_receive() {
        let mut suite = TestingSuite::default_with_balances(
            vec![
                coin(1_000_000_000u128, "uwhale".to_string()),
                coin(1_000_000_000u128, "uluna".to_string()),
                coin(1_000_000_000u128, "uusd".to_string()),
                coin(1_000_000_000u128, "uom".to_string()),
            ],
            StargateMock::new("uom".to_string(), "8888".to_string()),
        );
        let creator = suite.creator();
        let _other = suite.senders[1].clone();
        let _unauthorized = suite.senders[2].clone();
        // Asset infos with uwhale and uluna

        let first_pool = vec!["uwhale".to_string(), "uluna".to_string()];
        let second_pool = vec!["uluna".to_string(), "uusd".to_string()];

        let pool_fees = PoolFee {
            protocol_fee: Fee {
                share: Decimal::bps(50), // 0.5%
            },
            swap_fee: Fee {
                share: Decimal::bps(50), // 0.5%
            },
            burn_fee: Fee {
                share: Decimal::bps(50), // 0.5%
            },
            extra_fees: vec![],
        };

        // Create a pool
        suite
            .instantiate_default()
            .add_one_epoch()
            .create_pool(
                &creator,
                first_pool,
                vec![6u8, 6u8],
                pool_fees.clone(),
                PoolType::ConstantProduct,
                Some("whale.uluna".to_string()),
                vec![coin(1000, "uusd"), coin(8888, "uom")],
                |result| {
                    result.unwrap();
                },
            )
            .create_pool(
                &creator,
                second_pool,
                vec![6u8, 6u8],
                pool_fees,
                PoolType::ConstantProduct,
                Some("uluna.uusd".to_string()),
                vec![coin(1000, "uusd"), coin(8888, "uom")],
                |result| {
                    result.unwrap();
                },
            );

        // Let's try to add liquidity
        suite.provide_liquidity(
            &creator,
            "o.whale.uluna".to_string(),
            None,
            None,
            None,
            None,
            vec![
                Coin {
                    denom: "uwhale".to_string(),
                    amount: Uint128::from(1000000u128),
                },
                Coin {
                    denom: "uluna".to_string(),
                    amount: Uint128::from(1000000u128),
                },
            ],
            |result| {
                // ensure we got 999,000 in the response (1m - initial liquidity amount)
                let result = result.unwrap();
                assert!(result.has_event(&Event::new("wasm").add_attribute("share", "999000")));
            },
        );

        // Let's try to add liquidity
        suite.provide_liquidity(
            &creator,
            "o.uluna.uusd".to_string(),
            None,
            None,
            None,
            None,
            vec![
                Coin {
                    denom: "uluna".to_string(),
                    amount: Uint128::from(1000000u128),
                },
                Coin {
                    denom: "uusd".to_string(),
                    amount: Uint128::from(1000000u128),
                },
            ],
            |result| {
                // ensure we got 999,000 in the response (1m - initial liquidity amount)
                let result = result.unwrap();
                assert!(result.has_event(&Event::new("wasm").add_attribute("share", "999000")));
            },
        );

        // Prepare the swap operations, we want to go from WHALE -> UUSD
        // We will use the o.uluna.uusd pool as the intermediary pool

        let swap_operations = vec![
            amm::pool_manager::SwapOperation::MantraSwap {
                token_in_denom: "uwhale".to_string(),
                token_out_denom: "uluna".to_string(),
                pool_identifier: "o.whale.uluna".to_string(),
            },
            amm::pool_manager::SwapOperation::MantraSwap {
                token_in_denom: "uluna".to_string(),
                token_out_denom: "uusd".to_string(),
                pool_identifier: "o.uluna.uusd".to_string(),
            },
        ];

        // before swap uusd balance = 1_000_000_000
        // - 2*1_000 pool creation fee
        // - 1_000_000 liquidity provision
        // = 998_998_000
        let pre_swap_amount = 998_998_000;
        suite.query_balance(&creator.to_string(), "uusd".to_string(), |amt| {
            assert_eq!(amt.unwrap().amount.u128(), pre_swap_amount);
        });

        // require an output of 975 uusd
        suite.execute_swap_operations(
            &creator,
            swap_operations,
            Some(Uint128::new(975)),
            None,
            None,
            vec![coin(1000u128, "uwhale".to_string())],
            |result| {
                assert_eq!(
                    result.unwrap_err().downcast_ref::<ContractError>(),
                    Some(&ContractError::MinimumReceiveAssertion {
                        minimum_receive: Uint128::new(975),
                        swap_amount: Uint128::new(974),
                    })
                )
            },
        );
    }

    #[test]
    fn query_swap_operations() {
        let mut suite = TestingSuite::default_with_balances(
            vec![
                coin(1_000_000_000u128, "uwhale".to_string()),
                coin(1_000_000_000u128, "uluna".to_string()),
                coin(1_000_000_000u128, "uusd".to_string()),
                coin(1_000_000_000u128, "uom".to_string()),
            ],
            StargateMock::new("uom".to_string(), "8888".to_string()),
        );
        let creator = suite.creator();
        let _other = suite.senders[1].clone();
        let _unauthorized = suite.senders[2].clone();

        // Asset infos with uwhale and uluna
        let first_pool = vec!["uwhale".to_string(), "uluna".to_string()];
        let second_pool = vec!["uluna".to_string(), "uusd".to_string()];

        let pool_fees = PoolFee {
            protocol_fee: Fee {
                share: Decimal::bps(50), // 0.5%
            },
            swap_fee: Fee {
                share: Decimal::bps(50), // 0.5%
            },
            burn_fee: Fee {
                share: Decimal::bps(50), // 0.5%
            },
            extra_fees: vec![],
        };

        // Create a pool
        suite
            .instantiate_default()
            .add_one_epoch()
            .create_pool(
                &creator,
                first_pool,
                vec![6u8, 6u8],
                pool_fees.clone(),
                PoolType::ConstantProduct,
                Some("whale.uluna".to_string()),
                vec![coin(1_000, "uusd"), coin(8888, "uom")],
                |result| {
                    result.unwrap();
                },
            )
            .create_pool(
                &creator,
                second_pool,
                vec![6u8, 6u8],
                pool_fees,
                PoolType::ConstantProduct,
                Some("uluna.uusd".to_string()),
                vec![coin(1_000, "uusd"), coin(8888, "uom")],
                |result| {
                    result.unwrap();
                },
            );

        // Let's try to add liquidity
        suite.provide_liquidity(
            &creator,
            "o.whale.uluna".to_string(),
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
                // ensure we got 999,000 in the response (1m - initial liquidity amount)
                let result = result.unwrap();
                assert!(result.has_event(&Event::new("wasm").add_attribute("share", "999000")));
            },
        );

        // Let's try to add liquidity
        suite.provide_liquidity(
            &creator,
            "o.uluna.uusd".to_string(),
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
                // ensure we got 999,000 in the response (1m - initial liquidity amount)
                let result = result.unwrap();
                assert!(result.has_event(&Event::new("wasm").add_attribute("share", "999000")));
            },
        );

        // Prepare the swap operations, we want to go from WHALE -> UUSD
        // We will use the o.uluna.uusd pool as the intermediary pool

        let swap_operations = vec![
            amm::pool_manager::SwapOperation::MantraSwap {
                token_in_denom: "uwhale".to_string(),
                token_out_denom: "uluna".to_string(),
                pool_identifier: "o.whale.uluna".to_string(),
            },
            amm::pool_manager::SwapOperation::MantraSwap {
                token_in_denom: "uluna".to_string(),
                token_out_denom: "uusd".to_string(),
                pool_identifier: "o.uluna.uusd".to_string(),
            },
        ];

        // simulating (reverse) swap operations should return the correct same amount as the pools are balanced
        // going from whale -> uusd should return 974 uusd
        // going from uusd -> whale should return 974 whale
        suite.query_simulate_swap_operations(
            Uint128::new(1_000),
            swap_operations.clone(),
            |result| {
                let result = result.unwrap();
                assert_eq!(result.amount.u128(), 974);
            },
        );
        suite.query_reverse_simulate_swap_operations(
            Uint128::new(1_000),
            swap_operations.clone(),
            |result| {
                let result = result.unwrap();
                assert_eq!(result.amount.u128(), 974);
            },
        );

        // execute the swap operations to unbalance the pools
        // sold 10_000 whale for some uusd, so the price of whale should go down
        suite
            .execute_swap_operations(
                &creator,
                swap_operations.clone(),
                None,
                None,
                None,
                vec![coin(10_000u128, "uwhale".to_string())],
                |result| {
                    let err = result.unwrap_err().downcast::<ContractError>().unwrap();

                    assert_eq!(
                        err,
                        ContractError::Std(StdError::generic_err("Spread limit exceeded"))
                    );
                },
            )
            .execute_swap_operations(
                &creator,
                swap_operations.clone(),
                None,
                None,
                Some(Decimal::percent(5)),
                vec![coin(10_000u128, "uwhale".to_string())],
                |result| {
                    result.unwrap();
                },
            );

        // now to get 1_000 uusd we should swap more whale than before
        suite.query_reverse_simulate_swap_operations(
            Uint128::new(1_000),
            swap_operations.clone(),
            |result| {
                let result = result.unwrap();
                assert_approx_eq!(result.amount.u128(), 1_007, "0.1");
            },
        );

        // and if simulate swap operations with 1_000 more whale we should get even less uusd than before
        suite.query_simulate_swap_operations(
            Uint128::new(1_000),
            swap_operations.clone(),
            |result| {
                let result = result.unwrap();
                assert_eq!(result.amount.u128(), 935);
            },
        );
    }
}

mod swapping {
    use std::cell::RefCell;

    use cosmwasm_std::assert_approx_eq;

    use amm::pool_manager::PoolType;

    use super::*;

    #[test]
    fn basic_swapping_test() {
        let mut suite = TestingSuite::default_with_balances(
            vec![
                coin(1_000_000_001u128, "uwhale".to_string()),
                coin(1_000_000_000u128, "uluna".to_string()),
                coin(1_000_000_001u128, "uusd".to_string()),
                coin(1_000_000_001u128, "uom".to_string()),
            ],
            StargateMock::new("uom".to_string(), "8888".to_string()),
        );
        let creator = suite.creator();
        let _other = suite.senders[1].clone();
        let _unauthorized = suite.senders[2].clone();
        // Asset infos with uwhale and uluna

        let asset_infos = vec!["uwhale".to_string(), "uluna".to_string()];

        // Protocol fee is 0.01% and swap fee is 0.02% and burn fee is 0%
        let pool_fees = PoolFee {
            protocol_fee: Fee {
                share: Decimal::from_ratio(1u128, 100_000u128),
            },
            swap_fee: Fee {
                share: Decimal::from_ratio(1u128, 100_000u128),
            },
            burn_fee: Fee {
                share: Decimal::zero(),
            },
            extra_fees: vec![],
        };

        // Create a pool
        suite.instantiate_default().add_one_epoch().create_pool(
            &creator,
            asset_infos,
            vec![6u8, 6u8],
            pool_fees,
            PoolType::ConstantProduct,
            Some("whale.uluna".to_string()),
            vec![coin(1000, "uusd"), coin(8888, "uom")],
            |result| {
                result.unwrap();
            },
        );

        // Query pool info to ensure the query is working fine
        suite.query_pools(Some("o.whale.uluna".to_string()), None, None, |result| {
            assert_eq!(
                result.unwrap().pools[0].pool_info.asset_decimals,
                vec![6u8, 6u8]
            );
        });

        // Let's try to add liquidity
        suite
            .provide_liquidity(
                &creator,
                "o.whale.uluna".to_string(),
                None,
                None,
                None,
                None,
                vec![
                    Coin {
                        denom: "uwhale".to_string(),
                        amount: Uint128::from(1000000u128),
                    },
                    Coin {
                        denom: "uluna".to_string(),
                        amount: Uint128::from(1000000u128),
                    },
                ],
                |result| {
                    // Ensure we got 999_000 in the response which is 1mil less the initial liquidity amount
                    assert!(result.unwrap().events.iter().any(|event| {
                        event.attributes.iter().any(|attr| {
                            attr.key == "share"
                                && attr.value
                                    == (Uint128::from(1_000_000u128) - MINIMUM_LIQUIDITY_AMOUNT)
                                        .to_string()
                        })
                    }));
                },
            )
            .query_pools(Some("o.whale.uluna".to_string()), None, None, |result| {
                let response = result.unwrap();
                assert_eq!(
                    response.pools[0].total_share,
                    Coin {
                        denom: response.pools[0].pool_info.lp_denom.clone(),
                        amount: Uint128::from(1_000_000u128),
                    }
                );
            });

        let simulated_return_amount = RefCell::new(Uint128::zero());
        suite.query_simulation(
            "o.whale.uluna".to_string(),
            Coin {
                denom: "uwhale".to_string(),
                amount: Uint128::from(1000u128),
            },
            "uluna".to_string(),
            |result| {
                // Ensure that the return amount is 1_000 minus spread
                assert_eq!(
                    result.as_ref().unwrap().return_amount + result.as_ref().unwrap().spread_amount,
                    Uint128::from(1000u128)
                );
                *simulated_return_amount.borrow_mut() = result.unwrap().return_amount;
            },
        );

        // Now Let's try a swap
        suite.swap(
            &creator,
            "uluna".to_string(),
            None,
            None,
            None,
            "o.whale.uluna".to_string(),
            vec![coin(1000u128, "uwhale".to_string())],
            |result| {
                // Find the key with 'offer_amount' and the key with 'return_amount'
                // Ensure that the offer amount is 1000 and the return amount is greater than 0
                let mut return_amount = String::new();
                let mut offer_amount = String::new();

                for event in result.unwrap().events {
                    if event.ty == "wasm" {
                        for attribute in event.attributes {
                            match attribute.key.as_str() {
                                "return_amount" => return_amount = attribute.value,
                                "offer_amount" => offer_amount = attribute.value,
                                _ => {}
                            }
                        }
                    }
                }
                // Because the Pool was created and 1_000_000 of each token has been provided as liquidity
                // Assuming no fees we should expect a small swap of 1000 to result in not too much slippage
                // Expect 1000 give or take 0.002 difference
                // Once fees are added and being deducted properly only the "0.002" should be changed.
                assert_approx_eq!(
                    offer_amount.parse::<u128>().unwrap(),
                    return_amount.parse::<u128>().unwrap(),
                    "0.002"
                );
                assert_approx_eq!(
                    simulated_return_amount.borrow().u128(),
                    return_amount.parse::<u128>().unwrap(),
                    "0.002"
                );
            },
        );

        let simulated_offer_amount = RefCell::new(Uint128::zero());
        suite.query_reverse_simulation(
            "o.whale.uluna".to_string(),
            Coin {
                denom: "uwhale".to_string(),
                amount: Uint128::from(1000u128),
            },
            "uluna".to_string(),
            |result| {
                *simulated_offer_amount.borrow_mut() = result.unwrap().offer_amount;
            },
        );
        // Another swap but this time the other way around
        // Now Let's try a swap
        suite.swap(
            &creator,
            "uwhale".to_string(),
            None,
            None,
            None,
            "o.whale.uluna".to_string(),
            vec![coin(
                simulated_offer_amount.borrow().u128(),
                "uluna".to_string(),
            )],
            |result| {
                // Find the key with 'offer_amount' and the key with 'return_amount'
                // Ensure that the offer amount is 1000 and the return amount is greater than 0
                let mut return_amount = String::new();
                let mut offer_amount = String::new();

                for event in result.unwrap().events {
                    if event.ty == "wasm" {
                        for attribute in event.attributes {
                            match attribute.key.as_str() {
                                "return_amount" => return_amount = attribute.value,
                                "offer_amount" => offer_amount = attribute.value,
                                _ => {}
                            }
                        }
                    }
                }
                assert_approx_eq!(
                    simulated_offer_amount.borrow().u128(),
                    offer_amount.parse::<u128>().unwrap(),
                    "0.002"
                );

                assert_approx_eq!(1000u128, return_amount.parse::<u128>().unwrap(), "0.003");
            },
        );
    }

    #[test]
    fn basic_swapping_test_stable_swap_two_assets() {
        let mut suite = TestingSuite::default_with_balances(
            vec![
                coin(1_000_000_001u128, "uwhale".to_string()),
                coin(1_000_000_000u128, "uluna".to_string()),
                coin(1_000_000_001u128, "uusd".to_string()),
                coin(1_000_000_001u128, "uom".to_string()),
            ],
            StargateMock::new("uom".to_string(), "8888".to_string()),
        );
        let creator = suite.creator();
        let _other = suite.senders[1].clone();
        let _unauthorized = suite.senders[2].clone();
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

        // Create a stableswap pool with amp = 100
        suite.instantiate_default().add_one_epoch().create_pool(
            &creator,
            asset_infos,
            vec![6u8, 6u8],
            pool_fees,
            PoolType::StableSwap { amp: 100 },
            Some("whale.uluna".to_string()),
            vec![coin(1000, "uusd"), coin(8888, "uom")],
            |result| {
                result.unwrap();
            },
        );

        // Let's try to add liquidity
        suite.provide_liquidity(
            &creator,
            "o.whale.uluna".to_string(),
            None,
            None,
            None,
            None,
            vec![
                Coin {
                    denom: "uwhale".to_string(),
                    amount: Uint128::from(1000000u128),
                },
                Coin {
                    denom: "uluna".to_string(),
                    amount: Uint128::from(1000000u128),
                },
            ],
            |result| {
                // Ensure we got 999000 in the response which is 1mil less the initial liquidity amount
                for event in result.unwrap().events {
                    println!("{:?}", event);
                }
            },
        );
        let simulated_return_amount = RefCell::new(Uint128::zero());
        suite.query_simulation(
            "o.whale.uluna".to_string(),
            Coin {
                denom: "uwhale".to_string(),
                amount: Uint128::from(1000u128),
            },
            "uluna".to_string(),
            |result| {
                *simulated_return_amount.borrow_mut() = result.unwrap().return_amount;
            },
        );

        // Now Let's try a swap
        suite.swap(
            &creator,
            "uluna".to_string(),
            None,
            None,
            None,
            "o.whale.uluna".to_string(),
            vec![coin(1000u128, "uwhale".to_string())],
            |result| {
                // Find the key with 'offer_amount' and the key with 'return_amount'
                // Ensure that the offer amount is 1000 and the return amount is greater than 0
                let mut return_amount = String::new();
                let mut offer_amount = String::new();

                for event in result.unwrap().events {
                    if event.ty == "wasm" {
                        for attribute in event.attributes {
                            match attribute.key.as_str() {
                                "return_amount" => return_amount = attribute.value,
                                "offer_amount" => offer_amount = attribute.value,
                                _ => {}
                            }
                        }
                    }
                }
                // Because the Pool was created and 1_000_000 of each token has been provided as liquidity
                // Assuming no fees we should expect a small swap of 1000 to result in not too much slippage
                // Expect 1000 give or take 0.002 difference
                // Once fees are added and being deducted properly only the "0.002" should be changed.
                assert_approx_eq!(
                    offer_amount.parse::<u128>().unwrap(),
                    return_amount.parse::<u128>().unwrap(),
                    "0.002"
                );
                assert_approx_eq!(
                    simulated_return_amount.borrow().u128(),
                    return_amount.parse::<u128>().unwrap(),
                    "0.002"
                );
            },
        );

        let simulated_offer_amount = RefCell::new(Uint128::zero());
        suite.query_reverse_simulation(
            "o.whale.uluna".to_string(),
            Coin {
                denom: "uwhale".to_string(),
                amount: Uint128::from(1000u128),
            },
            "uluna".to_string(),
            |result| {
                *simulated_offer_amount.borrow_mut() = result.unwrap().offer_amount;
            },
        );
        // Another swap but this time the other way around
        // Now Let's try a swap
        suite.swap(
            &creator,
            "uwhale".to_string(),
            None,
            None,
            None,
            "o.whale.uluna".to_string(),
            vec![coin(
                simulated_offer_amount.borrow().u128(),
                "uluna".to_string(),
            )],
            |result| {
                // Find the key with 'offer_amount' and the key with 'return_amount'
                // Ensure that the offer amount is 1000 and the return amount is greater than 0
                let mut return_amount = String::new();
                let mut offer_amount = String::new();

                for event in result.unwrap().events {
                    if event.ty == "wasm" {
                        for attribute in event.attributes {
                            match attribute.key.as_str() {
                                "return_amount" => return_amount = attribute.value,
                                "offer_amount" => offer_amount = attribute.value,
                                _ => {}
                            }
                        }
                    }
                }
                assert_approx_eq!(
                    simulated_offer_amount.borrow().u128(),
                    offer_amount.parse::<u128>().unwrap(),
                    "0.002"
                );

                assert_approx_eq!(1000u128, return_amount.parse::<u128>().unwrap(), "0.003");
            },
        );
    }

    #[test]
    fn swap_with_fees() {
        let mut suite = TestingSuite::default_with_balances(
            vec![
                coin(1_000_000_000_001u128, "uwhale".to_string()),
                coin(1_000_000_000_000u128, "uluna".to_string()),
                coin(1_000_000_000_001u128, "uusd".to_string()),
                coin(1_000_000_000_001u128, "uom".to_string()),
            ],
            StargateMock::new("uom".to_string(), "8888".to_string()),
        );
        let creator = suite.creator();
        let _other = suite.senders[1].clone();
        let _unauthorized = suite.senders[2].clone();
        // Asset infos with uwhale and uluna

        let asset_infos = vec!["uwhale".to_string(), "uluna".to_string()];

        // Protocol fee is 0.001% and swap fee is 0.002% and burn fee is 0%
        let pool_fees = PoolFee {
            protocol_fee: Fee {
                share: Decimal::from_ratio(1u128, 100_000u128),
            },
            swap_fee: Fee {
                share: Decimal::from_ratio(2u128, 100_000u128),
            },
            burn_fee: Fee {
                share: Decimal::zero(),
            },
            extra_fees: vec![],
        };

        // Create a pool
        suite.instantiate_default().add_one_epoch().create_pool(
            &creator,
            asset_infos,
            vec![6u8, 6u8],
            pool_fees,
            PoolType::ConstantProduct,
            Some("whale.uluna".to_string()),
            vec![coin(1000, "uusd"), coin(8888, "uom")],
            |result| {
                result.unwrap();
            },
        );

        // Let's try to add liquidity, 1000 of each token.
        suite.provide_liquidity(
            &creator,
            "o.whale.uluna".to_string(),
            None,
            None,
            None,
            None,
            vec![
                Coin {
                    denom: "uwhale".to_string(),
                    amount: Uint128::from(1000_000000u128),
                },
                Coin {
                    denom: "uluna".to_string(),
                    amount: Uint128::from(1000_000000u128),
                },
            ],
            |result| {
                // Ensure we got 999000 in the response which is 1mil less the initial liquidity amount
                for event in result.unwrap().events {
                    println!("{:?}", event);
                }
            },
        );

        // Now Let's try a swap, max spread is set to 1%
        // With 1000 of each token and a swap of 10 WHALE
        // We should expect a return of 9900792 of ULUNA
        // Applying Fees on the swap:
        //    - Protocol Fee: 0.001% on uLUNA -> 99.
        //    - Swap Fee: 0.002% on uLUNA -> 198.
        // Total Fees: 297 uLUNA

        // Spread Amount: 99,010 uLUNA.
        // Swap Fee Amount: 198 uLUNA.
        // Protocol Fee Amount: 99 uLUNA.
        // Burn Fee Amount: 0 uLUNA (as expected since burn fee is set to 0%).
        // Total -> 9,900,693 (Returned Amount) + 99,010 (Spread)(0.009x%) + 198 (Swap Fee) + 99 (Protocol Fee) = 10,000,000 uLUNA
        suite.swap(
            &creator,
            "uluna".to_string(),
            None,
            Some(Decimal::percent(1)),
            None,
            "o.whale.uluna".to_string(),
            vec![coin(10000000u128, "uwhale".to_string())],
            |result| {
                // Find the key with 'offer_amount' and the key with 'return_amount'
                // Ensure that the offer amount is 1000 and the return amount is greater than 0
                let mut return_amount = String::new();
                let mut offer_amount = String::new();

                for event in result.unwrap().events {
                    if event.ty == "wasm" {
                        for attribute in event.attributes {
                            match attribute.key.as_str() {
                                "return_amount" => return_amount = attribute.value,
                                "offer_amount" => offer_amount = attribute.value,
                                _ => {}
                            }
                        }
                    }
                }
                // Because the Pool was created and 1_000_000 of each token has been provided as liquidity
                // Assuming no fees we should expect a small swap of 1000 to result in not too much slippage
                // Expect 1000 give or take 0.002 difference
                // Once fees are added and being deducted properly only the "0.002" should be changed.
                assert_approx_eq!(
                    offer_amount.parse::<u128>().unwrap(),
                    return_amount.parse::<u128>().unwrap(),
                    "0.011"
                );
            },
        );

        // Verify fee collection by querying the address of the fee collector and checking its balance
        // Should be 99 uLUNA
        suite.query_balance(
            &suite.fee_collector_addr.to_string(),
            "uluna".to_string(),
            |result| {
                assert_eq!(result.unwrap().amount, Uint128::from(99u128));
            },
        );
    }

    #[allow(clippy::inconsistent_digit_grouping)]
    #[test]
    fn swap_large_digits_xyk() {
        let mut suite = TestingSuite::default_with_balances(
            vec![
                coin(1_000_000_000_000u128, "uwhale".to_string()),
                coin(1_000_000_000_000u128, "uluna".to_string()),
                coin(1_000_000_000_000u128, "uosmo".to_string()),
                coin(1_000_000_000_000u128, "uusd".to_string()),
                coin(100_000_000_000_000_000000u128, "uusdc".to_string()),
                coin(
                    100_000_000_000_000_000000000000000000u128,
                    "ausdy".to_string(),
                ),
                coin(150_000_000_000_000_000000u128, "uom".to_string()),
            ],
            StargateMock::new("uom".to_string(), "8888".to_string()),
        );
        let alice = suite.creator();
        let bob = suite.senders[1].clone();
        let carol = suite.senders[2].clone();
        let dan = suite.senders[3].clone();

        let asset_denoms = vec!["uom".to_string(), "ausdy".to_string()];

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
            &alice,
            asset_denoms,
            vec![6u8, 18u8],
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

        // let's provide liquidity 150T om, 100T usdy
        suite
            .provide_liquidity(
                &bob,
                "p.1".to_string(),
                None,
                None,
                None,
                None,
                vec![
                    Coin {
                        denom: "uom".to_string(),
                        amount: Uint128::new(150_000_000_000_000_000000u128),
                    },
                    Coin {
                        denom: "ausdy".to_string(),
                        amount: Uint128::new(100_000_000_000_000_000000000000000000u128),
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
            .query_all_balances(&bob.to_string(), |result| {
                let balances = result.unwrap();
                assert!(balances.iter().any(|coin| {
                    coin.denom == lp_denom.clone()
                        && coin.amount == Uint128::new(122_474_487_139_158_904_909_863_203u128)
                }));
            });

        // swap 2T usdy for om
        suite
            .query_balance(&carol.to_string(), "ausdy".to_string(), |result| {
                assert_eq!(
                    result.unwrap().amount,
                    Uint128::new(100_000_000_000_000_000000000000000000u128)
                );
            })
            .query_balance(&carol.to_string(), "uom".to_string(), |result| {
                assert_eq!(
                    result.unwrap().amount,
                    Uint128::new(150_000_000_000_000_000000u128)
                );
            })
            .query_simulation(
                "p.1".to_string(),
                Coin {
                    denom: "ausdy".to_string(),
                    amount: Uint128::new(2_000_000_000_000_000000000000000000u128),
                },
                "uom".to_string(),
                |result| {
                    assert_eq!(
                        result.unwrap().return_amount,
                        Uint128::new(2_852_941_176_470_588236u128)
                    );
                },
            )
            .swap(
                &carol,
                "uom".to_string(),
                None,
                Some(Decimal::percent(3)),
                None,
                "p.1".to_string(),
                vec![coin(
                    2_000_000_000_000_000000000000000000u128,
                    "ausdy".to_string(),
                )],
                |result| {
                    result.unwrap();
                },
            )
            .query_balance(&carol.to_string(), "ausdy".to_string(), |result| {
                assert_eq!(
                    result.unwrap().amount,
                    Uint128::new(
                        100_000_000_000_000_000000000000000000u128
                            - 2_000_000_000_000_000000000000000000u128
                    )
                );
            })
            .query_balance(&carol.to_string(), "uom".to_string(), |result| {
                assert_eq!(
                    result.unwrap().amount,
                    Uint128::new(150_000_000_000_000_000000u128 + 2_852_941_176_470_588236u128)
                );
            });

        // swap 10T om for usdy
        suite
            .query_balance(&dan.to_string(), "ausdy".to_string(), |result| {
                assert_eq!(
                    result.unwrap().amount,
                    Uint128::new(100_000_000_000_000_000000000000000000u128)
                );
            })
            .query_balance(&dan.to_string(), "uom".to_string(), |result| {
                assert_eq!(
                    result.unwrap().amount,
                    Uint128::new(150_000_000_000_000_000000u128)
                );
            })
            .query_simulation(
                "p.1".to_string(),
                Coin {
                    denom: "uom".to_string(),
                    amount: Uint128::new(10_000_000_000_000_000000u128),
                },
                "ausdy".to_string(),
                |result| {
                    assert_eq!(
                        result.unwrap().return_amount,
                        Uint128::new(6_296_013_475_575_519371168089897701u128)
                    );
                },
            )
            .swap(
                &dan,
                "ausdy".to_string(),
                None,
                Some(Decimal::percent(20)),
                None,
                "p.1".to_string(),
                vec![coin(10_000_000_000_000_000000u128, "uom".to_string())],
                |result| {
                    result.unwrap();
                },
            )
            .query_balance(&dan.to_string(), "ausdy".to_string(), |result| {
                assert_eq!(
                    result.unwrap().amount,
                    Uint128::new(
                        100_000_000_000_000_000000000000000000u128
                            + 6_296_013_475_575_519371168089897701u128
                    )
                );
            })
            .query_balance(&dan.to_string(), "uom".to_string(), |result| {
                assert_eq!(
                    result.unwrap().amount,
                    Uint128::new(150_000_000_000_000_000000u128 - 10_000_000_000_000_000000u128)
                );
            });
    }

    #[allow(clippy::inconsistent_digit_grouping)]
    #[test]
    fn swap_large_digits_stable() {
        let mut suite = TestingSuite::default_with_balances(
            vec![
                coin(1_000_000_000_000u128, "uwhale".to_string()),
                coin(1_000_000_000_000u128, "uluna".to_string()),
                coin(1_000_000_000_000u128, "uosmo".to_string()),
                coin(1_000_000_000_000u128, "uusd".to_string()),
                coin(100_000_000_000_000_000000u128, "uusdc".to_string()),
                coin(
                    100_000_000_000_000_000000000000000000u128,
                    "ausdy".to_string(),
                ),
                coin(150_000_000_000_000_000000u128, "uom".to_string()),
            ],
            StargateMock::new("uom".to_string(), "8888".to_string()),
        );
        let alice = suite.creator();
        let bob = suite.senders[1].clone();
        let carol = suite.senders[2].clone();
        let dan = suite.senders[3].clone();

        let asset_denoms = vec!["ausdy".to_string(), "uusdc".to_string()];

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
            vec![18u8, 6u8],
            pool_fees,
            PoolType::ConstantProduct,
            None,
            vec![coin(1000, "uusd"), coin(8888, "uom")],
            |result| {
                result.unwrap();
            },
        );

        // let's provide liquidity 200T usdc, 200T usdy
        suite
            .provide_liquidity(
                &alice,
                "p.1".to_string(),
                None,
                None,
                None,
                None,
                vec![
                    Coin {
                        denom: "uusdc".to_string(),
                        amount: Uint128::new(100_000_000_000_000_000000u128),
                    },
                    Coin {
                        denom: "ausdy".to_string(),
                        amount: Uint128::new(100_000_000_000_000_000000000000000000u128),
                    },
                ],
                |result| {
                    result.unwrap();
                },
            )
            .provide_liquidity(
                &bob,
                "p.1".to_string(),
                None,
                None,
                None,
                None,
                vec![
                    Coin {
                        denom: "uusdc".to_string(),
                        amount: Uint128::new(100_000_000_000_000_000000u128),
                    },
                    Coin {
                        denom: "ausdy".to_string(),
                        amount: Uint128::new(100_000_000_000_000_000000000000000000u128),
                    },
                ],
                |result| {
                    result.unwrap();
                },
            );

        // swap 10T usdc for usdy
        suite
            .query_balance(&carol.to_string(), "ausdy".to_string(), |result| {
                assert_eq!(
                    result.unwrap().amount,
                    Uint128::new(100_000_000_000_000_000000000000000000u128)
                );
            })
            .query_balance(&carol.to_string(), "uusdc".to_string(), |result| {
                assert_eq!(
                    result.unwrap().amount,
                    Uint128::new(100_000_000_000_000_000000u128)
                );
            })
            .query_simulation(
                "p.1".to_string(),
                Coin {
                    denom: "uusdc".to_string(),
                    amount: Uint128::new(10_000_000_000_000_000000u128),
                },
                "ausdy".to_string(),
                |result| {
                    assert_eq!(
                        result.unwrap().return_amount,
                        Uint128::new(9_476_190_476_190_476190476190476190u128)
                    );
                },
            )
            .swap(
                &carol,
                "ausdy".to_string(),
                None,
                Some(Decimal::percent(5)),
                None,
                "p.1".to_string(),
                vec![coin(10_000_000_000_000_000000u128, "uusdc".to_string())],
                |result| {
                    result.unwrap();
                },
            )
            .query_balance(&carol.to_string(), "ausdy".to_string(), |result| {
                assert_eq!(
                    result.unwrap().amount,
                    Uint128::new(
                        100_000_000_000_000_000000000000000000u128
                            + 9_476_190_476_190_476190476190476190u128
                    )
                );
            })
            .query_balance(&carol.to_string(), "uusdc".to_string(), |result| {
                assert_eq!(
                    result.unwrap().amount,
                    Uint128::new(100_000_000_000_000_000000u128 - 10_000_000_000_000_000000u128)
                );
            });

        // swap 20T usdy for usdc
        suite
            .query_balance(&dan.to_string(), "ausdy".to_string(), |result| {
                assert_eq!(
                    result.unwrap().amount,
                    Uint128::new(100_000_000_000_000_000000000000000000u128)
                );
            })
            .query_balance(&dan.to_string(), "uusdc".to_string(), |result| {
                assert_eq!(
                    result.unwrap().amount,
                    Uint128::new(100_000_000_000_000_000000u128)
                );
            })
            .query_simulation(
                "p.1".to_string(),
                Coin {
                    denom: "ausdy".to_string(),
                    amount: Uint128::new(20_000_000_000_000_000000000000000000u128),
                },
                "uusdc".to_string(),
                |result| {
                    assert_eq!(
                        result.unwrap().return_amount,
                        Uint128::new(19_850_486_315_313_277539u128)
                    );
                },
            )
            .swap(
                &dan,
                "uusdc".to_string(),
                None,
                Some(Decimal::percent(10)),
                None,
                "p.1".to_string(),
                vec![coin(
                    20_000_000_000_000_000000000000000000u128,
                    "ausdy".to_string(),
                )],
                |result| {
                    result.unwrap();
                },
            )
            .query_balance(&dan.to_string(), "ausdy".to_string(), |result| {
                assert_eq!(
                    result.unwrap().amount,
                    Uint128::new(
                        100_000_000_000_000_000000000000000000u128
                            - 20_000_000_000_000_000000000000000000u128
                    )
                );
            })
            .query_balance(&dan.to_string(), "uusdc".to_string(), |result| {
                assert_eq!(
                    result.unwrap().amount,
                    Uint128::new(100_000_000_000_000_000000u128 + 19_850_486_315_313_277539u128)
                );
            });
    }

    #[allow(clippy::inconsistent_digit_grouping)]
    #[test]
    fn swap_large_digits_stable_18_digits() {
        let mut suite = TestingSuite::default_with_balances(
            vec![
                coin(1_000_000_000_000u128, "uwhale".to_string()),
                coin(1_000_000_000_000u128, "uluna".to_string()),
                coin(1_000_000_000_000u128, "uosmo".to_string()),
                coin(1_000_000_000_000u128, "uusd".to_string()),
                coin(1_000_000_000_000u128, "uusdc".to_string()),
                coin(
                    300_000_000_000_000_000000000000000000u128,
                    "ausdy".to_string(),
                ),
                coin(
                    300_000_000_000_000_000000000000000000u128,
                    "pusdc".to_string(),
                ),
                coin(1_000_000_000_000u128, "uom".to_string()),
            ],
            StargateMock::new("uom".to_string(), "8888".to_string()),
        );
        let alice = suite.creator();
        let bob = suite.senders[1].clone();
        let carol = suite.senders[2].clone();

        let asset_denoms = vec!["ausdy".to_string(), "pusdc".to_string()];

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
            vec![coin(1000, "uusd"), coin(8888, "uom")],
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
            vec![
                Coin {
                    denom: "pusdc".to_string(),
                    amount: Uint128::new(300_000_000_000_000_000000000000000000u128),
                },
                Coin {
                    denom: "ausdy".to_string(),
                    amount: Uint128::new(300_000_000_000_000_000000000000000000u128),
                },
            ],
            |result| {
                result.unwrap();
            },
        );

        // swap 100T pusdc for usdy
        suite
            .query_balance(&bob.to_string(), "ausdy".to_string(), |result| {
                assert_eq!(
                    result.unwrap().amount,
                    Uint128::new(300_000_000_000_000_000000000000000000u128)
                );
            })
            .query_balance(&bob.to_string(), "pusdc".to_string(), |result| {
                assert_eq!(
                    result.unwrap().amount,
                    Uint128::new(300_000_000_000_000_000000000000000000u128)
                );
            })
            .query_simulation(
                "p.1".to_string(),
                Coin {
                    denom: "pusdc".to_string(),
                    amount: Uint128::new(100_000_000_000_000_000000000000000000u128),
                },
                "ausdy".to_string(),
                |result| {
                    assert_eq!(
                        result.unwrap().return_amount,
                        Uint128::new(74_625_000_000_000_000000000000000000u128)
                    );
                },
            )
            .swap(
                &bob,
                "ausdy".to_string(),
                None,
                Some(Decimal::percent(30)),
                None,
                "p.1".to_string(),
                vec![coin(
                    100_000_000_000_000_000000000000000000u128,
                    "pusdc".to_string(),
                )],
                |result| {
                    result.unwrap();
                },
            )
            .query_balance(&bob.to_string(), "ausdy".to_string(), |result| {
                assert_eq!(
                    result.unwrap().amount,
                    Uint128::new(
                        300_000_000_000_000_000000000000000000u128
                            + 74_625_000_000_000_000000000000000000u128
                    )
                );
            })
            .query_balance(&bob.to_string(), "pusdc".to_string(), |result| {
                assert_eq!(
                    result.unwrap().amount,
                    Uint128::new(
                        300_000_000_000_000_000000000000000000u128
                            - 100_000_000_000_000_000000000000000000u128
                    )
                );
            });

        // swap 50T usdy for pusdc
        suite
            .query_balance(&carol.to_string(), "ausdy".to_string(), |result| {
                assert_eq!(
                    result.unwrap().amount,
                    Uint128::new(300_000_000_000_000_000000000000000000u128)
                );
            })
            .query_balance(&carol.to_string(), "pusdc".to_string(), |result| {
                assert_eq!(
                    result.unwrap().amount,
                    Uint128::new(300_000_000_000_000_000000000000000000u128)
                );
            })
            .query_simulation(
                "p.1".to_string(),
                Coin {
                    denom: "ausdy".to_string(),
                    amount: Uint128::new(50_000_000_000_000_000000000000000000u128),
                },
                "pusdc".to_string(),
                |result| {
                    assert_eq!(
                        result.unwrap().return_amount,
                        Uint128::new(72_265_093_054_925_102133454380390377u128)
                    );
                },
            )
            .swap(
                &carol,
                "pusdc".to_string(),
                None,
                Some(Decimal::percent(20)),
                None,
                "p.1".to_string(),
                vec![coin(
                    50_000_000_000_000_000000000000000000u128,
                    "ausdy".to_string(),
                )],
                |result| {
                    result.unwrap();
                },
            )
            .query_balance(&carol.to_string(), "ausdy".to_string(), |result| {
                assert_eq!(
                    result.unwrap().amount,
                    Uint128::new(
                        300_000_000_000_000_000000000000000000u128
                            - 50_000_000_000_000_000000000000000000u128
                    )
                );
            })
            .query_balance(&carol.to_string(), "pusdc".to_string(), |result| {
                assert_eq!(
                    result.unwrap().amount,
                    Uint128::new(
                        300_000_000_000_000_000000000000000000u128
                            + 72_265_093_054_925_102133454380390377u128
                    )
                );
            });
    }
}

mod ownership {
    use amm::pool_manager::FeatureToggle;

    use super::*;

    #[test]
    fn verify_ownership() {
        let mut suite = TestingSuite::default_with_balances(
            vec![],
            StargateMock::new("uom".to_string(), "8888".to_string()),
        );
        let creator = suite.creator();
        let other = suite.senders[1].clone();
        let unauthorized = suite.senders[2].clone();

        suite
            .instantiate_default()
            .query_ownership(|result| {
                let ownership = result.unwrap();
                assert_eq!(Addr::unchecked(ownership.owner.unwrap()), creator);
            })
            .update_ownership(
                &unauthorized,
                cw_ownable::Action::TransferOwnership {
                    new_owner: other.to_string(),
                    expiry: None,
                },
                |result| {
                    let err = result.unwrap_err().downcast::<ContractError>().unwrap();

                    match err {
                        ContractError::OwnershipError { .. } => {}
                        _ => {
                            panic!("Wrong error type, should return ContractError::OwnershipError")
                        }
                    }
                },
            )
            .update_ownership(
                &creator,
                cw_ownable::Action::TransferOwnership {
                    new_owner: other.to_string(),
                    expiry: None,
                },
                |result| {
                    result.unwrap();
                },
            )
            .update_ownership(&other, cw_ownable::Action::AcceptOwnership, |result| {
                result.unwrap();
            })
            .query_ownership(|result| {
                let ownership = result.unwrap();
                assert_eq!(Addr::unchecked(ownership.owner.unwrap()), other);
            })
            .update_ownership(&other, cw_ownable::Action::RenounceOwnership, |result| {
                result.unwrap();
            })
            .query_ownership(|result| {
                let ownership = result.unwrap();
                assert!(ownership.owner.is_none());
            });
    }

    #[test]
    fn checks_ownership_when_updating_config() {
        let mut suite = TestingSuite::default_with_balances(
            vec![],
            StargateMock::new("uom".to_string(), "8888".to_string()),
        );
        let unauthorized = suite.senders[2].clone();

        suite.instantiate_default().update_config(
            &unauthorized,
            None,
            None,
            None,
            None,
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();

                match err {
                    ContractError::OwnershipError { .. } => {}
                    _ => {
                        panic!("Wrong error type, should return ContractError::OwnershipError")
                    }
                }
            },
        );
    }

    #[test]
    fn updates_config_fields() {
        let mut suite = TestingSuite::default_with_balances(
            vec![],
            StargateMock::new("uom".to_string(), "8888".to_string()),
        );
        let creator = suite.creator();
        let other = suite.senders[1].clone();
        let another = suite.senders[2].clone();

        suite.instantiate_default();
        let current_pool_creation_fee = suite.query_config().pool_creation_fee;
        let initial_config = suite.query_config();

        suite.update_config(
            &creator,
            Some(other),
            Some(another),
            Some(coin(
                current_pool_creation_fee
                    .amount
                    .checked_add(Uint128::from(1u32))
                    .unwrap()
                    .u128(),
                current_pool_creation_fee.denom,
            )),
            Some(FeatureToggle {
                deposits_enabled: false,
                swaps_enabled: false,
                withdrawals_enabled: false,
            }),
            |res| {
                res.unwrap();
            },
        );

        let config = suite.query_config();
        assert_ne!(config.fee_collector_addr, initial_config.fee_collector_addr);
        assert_ne!(config.pool_creation_fee, initial_config.pool_creation_fee);
        assert_ne!(config.feature_toggle, initial_config.feature_toggle);
        assert_ne!(config.farm_manager_addr, initial_config.farm_manager_addr);
    }
}

mod locking_lp {
    use std::cell::RefCell;

    use cosmwasm_std::{coin, Coin, Decimal, Uint128};

    use amm::farm_manager::{Position, PositionsBy};
    use amm::fee::{Fee, PoolFee};
    use amm::lp_common::MINIMUM_LIQUIDITY_AMOUNT;
    use amm::pool_manager::PoolType;
    use common_testing::multi_test::stargate_mock::StargateMock;

    use crate::tests::suite::TestingSuite;
    use crate::ContractError;

    #[test]
    fn provide_liquidity_locking_lp_no_lock_position_identifier() {
        let mut suite = TestingSuite::default_with_balances(
            vec![
                coin(10_000_000u128, "uwhale".to_string()),
                coin(10_000_000u128, "uluna".to_string()),
                coin(10_000u128, "uusd".to_string()),
                coin(10_000u128, "uom".to_string()),
            ],
            StargateMock::new("uom".to_string(), "8888".to_string()),
        );
        let creator = suite.creator();
        let _other = suite.senders[1].clone();
        let _unauthorized = suite.senders[2].clone();

        // Asset denoms with uwhale and uluna
        let asset_denoms = vec!["uwhale".to_string(), "uluna".to_string()];

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
        let farm_manager_addr = suite.farm_manager_addr.clone();
        let lp_denom = suite.get_lp_denom("o.whale.uluna".to_string());

        // Let's try to add liquidity
        suite
            .provide_liquidity(
                &creator,
                "o.whale.uluna".to_string(),
                Some(86_400u64),
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
                    // Ensure we got 999_000 in the response which is 1_000_000 less the initial liquidity amount
                    assert!(result.unwrap().events.iter().any(|event| {
                        event.attributes.iter().any(|attr| {
                            attr.key == "share"
                                && attr.value
                                    == (Uint128::from(1_000_000u128) - MINIMUM_LIQUIDITY_AMOUNT)
                                        .to_string()
                        })
                    }));
                },
            )
            .query_all_balances(&creator.to_string(), |result| {
                let balances = result.unwrap();
                // the lp tokens should have gone to the farm manager
                assert!(!balances
                    .iter()
                    .any(|coin| { coin.denom == lp_denom.clone() }));
            })
            // contract should have 1_000 LP shares (MINIMUM_LIQUIDITY_AMOUNT)
            .query_all_balances(&contract_addr.to_string(), |result| {
                let balances = result.unwrap();
                assert!(balances.iter().any(|coin| {
                    coin.denom == lp_denom.clone() && coin.amount == MINIMUM_LIQUIDITY_AMOUNT
                }));
            })
            // check the LP went to the farm manager
            .query_all_balances(&farm_manager_addr.to_string(), |result| {
                let balances = result.unwrap();
                assert!(balances.iter().any(|coin| {
                    coin.denom == lp_denom && coin.amount == Uint128::from(999_000u128)
                }));
            });

        suite.query_farm_positions(Some(PositionsBy::Receiver(creator.to_string())), None, None, None, |result| {
            let positions = result.unwrap().positions;
            assert_eq!(positions.len(), 1);
            assert_eq!(positions[0], Position {
                identifier: "p-1".to_string(),
                lp_asset: Coin { denom: "factory/mantra1zwv6feuzhy6a9wekh96cd57lsarmqlwxdypdsplw6zhfncqw6ftqlydlr9/o.whale.uluna.LP".to_string(), amount: Uint128::from(999_000u128) },
                unlocking_duration: 86_400,
                open: true,
                expiring_at: None,
                receiver: creator.clone(),
            });
        });

        // let's do it again, it should create another position on the farm manager

        let farm_manager_lp_amount = RefCell::new(Uint128::zero());

        suite
            .query_all_balances(&farm_manager_addr.to_string(), |result| {
                let balances = result.unwrap();
                let lp_balance = balances.iter().find(|coin| coin.denom == lp_denom).unwrap();
                *farm_manager_lp_amount.borrow_mut() = lp_balance.amount;
            })
            .provide_liquidity(
                &creator,
                "o.whale.uluna".to_string(),
                Some(200_000u64),
                None,
                None,
                None,
                vec![
                    Coin {
                        denom: "uwhale".to_string(),
                        amount: Uint128::from(2_000u128),
                    },
                    Coin {
                        denom: "uluna".to_string(),
                        amount: Uint128::from(2_000u128),
                    },
                ],
                |result| {
                    result.unwrap();
                },
            )
            .query_all_balances(&creator.to_string(), |result| {
                let balances = result.unwrap();
                // the lp tokens should have gone to the farm manager
                assert!(!balances
                    .iter()
                    .any(|coin| { coin.denom == lp_denom.clone() }));
            })
            // check the LP went to the farm manager
            .query_all_balances(&farm_manager_addr.to_string(), |result| {
                let balances = result.unwrap();
                // the LP tokens should have gone to the farm manager
                // the new minted LP tokens should be 2_000
                assert!(balances.iter().any(|coin| {
                    coin.denom == lp_denom
                        && coin.amount
                            == farm_manager_lp_amount
                                .borrow()
                                .checked_add(Uint128::from(2_000u128))
                                .unwrap()
                }));

                let lp_balance = balances.iter().find(|coin| coin.denom == lp_denom).unwrap();
                *farm_manager_lp_amount.borrow_mut() = lp_balance.amount;
            });

        suite.query_farm_positions(Some(PositionsBy::Receiver(creator.to_string())), None, None, None, |result| {
            let positions = result.unwrap().positions;
            assert_eq!(positions.len(), 2);
            assert_eq!(positions[0], Position {
                identifier: "p-1".to_string(),
                lp_asset: Coin { denom: "factory/mantra1zwv6feuzhy6a9wekh96cd57lsarmqlwxdypdsplw6zhfncqw6ftqlydlr9/o.whale.uluna.LP".to_string(), amount: Uint128::from(999_000u128) },
                unlocking_duration: 86_400,
                open: true,
                expiring_at: None,
                receiver: creator.clone(),
            });
            assert_eq!(positions[1], Position {
                identifier: "p-2".to_string(),
                lp_asset: Coin { denom: "factory/mantra1zwv6feuzhy6a9wekh96cd57lsarmqlwxdypdsplw6zhfncqw6ftqlydlr9/o.whale.uluna.LP".to_string(), amount: Uint128::from(2_000u128) },
                unlocking_duration: 200_000,
                open: true,
                expiring_at: None,
                receiver: creator.clone(),
            });
        });
    }

    #[test]
    fn provide_liquidity_locking_lp_reusing_position_identifier() {
        let mut suite = TestingSuite::default_with_balances(
            vec![
                coin(10_000_000u128, "uwhale".to_string()),
                coin(10_000_000u128, "uluna".to_string()),
                coin(10_000u128, "uusd".to_string()),
                coin(10_000u128, "uom".to_string()),
            ],
            StargateMock::new("uom".to_string(), "8888".to_string()),
        );
        let creator = suite.creator();
        let _other = suite.senders[1].clone();
        let _unauthorized = suite.senders[2].clone();

        // Asset denoms with uwhale and uluna
        let asset_denoms = vec!["uwhale".to_string(), "uluna".to_string()];

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
        let farm_manager_addr = suite.farm_manager_addr.clone();
        let lp_denom = suite.get_lp_denom("o.whale.uluna".to_string());

        // Let's try to add liquidity
        suite
            .provide_liquidity(
                &creator,
                "o.whale.uluna".to_string(),
                Some(86_400u64),
                Some("farm_identifier".to_string()),
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
                    // Ensure we got 999_000 in the response which is 1_000_000 less the initial liquidity amount
                    assert!(result.unwrap().events.iter().any(|event| {
                        event.attributes.iter().any(|attr| {
                            attr.key == "share"
                                && attr.value
                                    == (Uint128::from(1_000_000u128) - MINIMUM_LIQUIDITY_AMOUNT)
                                        .to_string()
                        })
                    }));
                },
            )
            .query_all_balances(&creator.to_string(), |result| {
                let balances = result.unwrap();
                // the lp tokens should have gone to the farm manager
                assert!(!balances
                    .iter()
                    .any(|coin| { coin.denom == lp_denom.clone() }));
            })
            // contract should have 1_000 LP shares (MINIMUM_LIQUIDITY_AMOUNT)
            .query_all_balances(&contract_addr.to_string(), |result| {
                let balances = result.unwrap();
                assert!(balances.iter().any(|coin| {
                    coin.denom == lp_denom.clone() && coin.amount == MINIMUM_LIQUIDITY_AMOUNT
                }));
            })
            // check the LP went to the farm manager
            .query_all_balances(&farm_manager_addr.to_string(), |result| {
                let balances = result.unwrap();
                assert!(balances.iter().any(|coin| {
                    coin.denom == lp_denom && coin.amount == Uint128::from(999_000u128)
                }));
            });

        suite.query_farm_positions(Some(PositionsBy::Receiver(creator.to_string())), None, None, None, |result| {
            let positions = result.unwrap().positions;
            assert_eq!(positions.len(), 1);
            assert_eq!(positions[0], Position {
                identifier: "u-farm_identifier".to_string(),
                lp_asset: Coin { denom: "factory/mantra1zwv6feuzhy6a9wekh96cd57lsarmqlwxdypdsplw6zhfncqw6ftqlydlr9/o.whale.uluna.LP".to_string(), amount: Uint128::from(999_000u128) },
                unlocking_duration: 86_400,
                open: true,
                expiring_at: None,
                receiver: creator.clone(),
            });
        });

        // let's do it again, reusing the same farm identifier

        let farm_manager_lp_amount = RefCell::new(Uint128::zero());

        suite
            .query_all_balances(&farm_manager_addr.to_string(), |result| {
                let balances = result.unwrap();
                let lp_balance = balances.iter().find(|coin| coin.denom == lp_denom).unwrap();
                *farm_manager_lp_amount.borrow_mut() = lp_balance.amount;
            })
            .provide_liquidity(
                &creator,
                "o.whale.uluna".to_string(),
                Some(200_000u64),
                Some("u-farm_identifier".to_string()),
                None,
                None,
                vec![
                    Coin {
                        denom: "uwhale".to_string(),
                        amount: Uint128::from(2_000u128),
                    },
                    Coin {
                        denom: "uluna".to_string(),
                        amount: Uint128::from(2_000u128),
                    },
                ],
                |result| {
                    result.unwrap();
                },
            )
            .query_all_balances(&creator.to_string(), |result| {
                let balances = result.unwrap();
                // the lp tokens should have gone to the farm manager
                assert!(!balances
                    .iter()
                    .any(|coin| { coin.denom == lp_denom.clone() }));
            })
            // check the LP went to the farm manager
            .query_all_balances(&farm_manager_addr.to_string(), |result| {
                let balances = result.unwrap();
                // the LP tokens should have gone to the farm manager
                // the new minted LP tokens should be 2_000
                assert!(balances.iter().any(|coin| {
                    coin.denom == lp_denom
                        && coin.amount
                            == farm_manager_lp_amount
                                .borrow()
                                .checked_add(Uint128::from(2_000u128))
                                .unwrap()
                }));

                let lp_balance = balances.iter().find(|coin| coin.denom == lp_denom).unwrap();
                *farm_manager_lp_amount.borrow_mut() = lp_balance.amount;
            });

        suite.query_farm_positions(Some(PositionsBy::Receiver(creator.to_string())), None, None, None, |result| {
            let positions = result.unwrap().positions;
            // the position should be updated
            assert_eq!(positions.len(), 1);
            assert_eq!(positions[0], Position {
                identifier: "u-farm_identifier".to_string(),
                lp_asset: Coin { denom: "factory/mantra1zwv6feuzhy6a9wekh96cd57lsarmqlwxdypdsplw6zhfncqw6ftqlydlr9/o.whale.uluna.LP".to_string(), amount: farm_manager_lp_amount.borrow().clone() },
                unlocking_duration: 86_400,
                open: true,
                expiring_at: None,
                receiver: creator.clone(),
            });
        });
    }

    #[test]
    fn provide_liquidity_locking_lp_reusing_position_identifier_2() {
        let mut suite = TestingSuite::default_with_balances(
            vec![
                coin(10_000_000u128, "uwhale".to_string()),
                coin(10_000_000u128, "uluna".to_string()),
                coin(10_000u128, "uusd".to_string()),
                coin(10_000u128, "uom".to_string()),
            ],
            StargateMock::new("uom".to_string(), "8888".to_string()),
        );
        let creator = suite.creator();
        let _other = suite.senders[1].clone();
        let _unauthorized = suite.senders[2].clone();

        // Asset denoms with uwhale and uluna
        let asset_denoms = vec!["uwhale".to_string(), "uluna".to_string()];

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
        let farm_manager_addr = suite.farm_manager_addr.clone();
        let lp_denom = suite.get_lp_denom("o.whale.uluna".to_string());

        // Let's try to add liquidity
        suite
            .provide_liquidity(
                &creator,
                "o.whale.uluna".to_string(),
                Some(86_400u64),
                Some("farm_identifier".to_string()),
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
                    // Ensure we got 999_000 in the response which is 1_000_000 less the initial liquidity amount
                    assert!(result.unwrap().events.iter().any(|event| {
                        event.attributes.iter().any(|attr| {
                            attr.key == "share"
                                && attr.value
                                    == (Uint128::from(1_000_000u128) - MINIMUM_LIQUIDITY_AMOUNT)
                                        .to_string()
                        })
                    }));
                },
            )
            .query_all_balances(&creator.to_string(), |result| {
                let balances = result.unwrap();
                // the lp tokens should have gone to the farm manager
                assert!(!balances
                    .iter()
                    .any(|coin| { coin.denom == lp_denom.clone() }));
            })
            // contract should have 1_000 LP shares (MINIMUM_LIQUIDITY_AMOUNT)
            .query_all_balances(&contract_addr.to_string(), |result| {
                let balances = result.unwrap();
                assert!(balances.iter().any(|coin| {
                    coin.denom == lp_denom.clone() && coin.amount == MINIMUM_LIQUIDITY_AMOUNT
                }));
            })
            // check the LP went to the farm manager
            .query_all_balances(&farm_manager_addr.to_string(), |result| {
                let balances = result.unwrap();
                assert!(balances.iter().any(|coin| {
                    coin.denom == lp_denom && coin.amount == Uint128::from(999_000u128)
                }));
            });

        suite.query_farm_positions(Some(PositionsBy::Receiver(creator.to_string())), None, None, None, |result| {
            let positions = result.unwrap().positions;
            assert_eq!(positions.len(), 1);
            assert_eq!(positions[0], Position {
                identifier: "u-farm_identifier".to_string(),
                lp_asset: Coin { denom: "factory/mantra1zwv6feuzhy6a9wekh96cd57lsarmqlwxdypdsplw6zhfncqw6ftqlydlr9/o.whale.uluna.LP".to_string(), amount: Uint128::from(999_000u128) },
                unlocking_duration: 86_400,
                open: true,
                expiring_at: None,
                receiver: creator.clone(),
            });
        });

        // let's do it again, this time no identifier is used

        let farm_manager_lp_amount = RefCell::new(Uint128::zero());

        suite
            .query_all_balances(&farm_manager_addr.to_string(), |result| {
                let balances = result.unwrap();
                let lp_balance = balances.iter().find(|coin| coin.denom == lp_denom).unwrap();
                *farm_manager_lp_amount.borrow_mut() = lp_balance.amount;
            })
            .provide_liquidity(
                &creator,
                "o.whale.uluna".to_string(),
                Some(200_000u64),
                None,
                None,
                None,
                vec![
                    Coin {
                        denom: "uwhale".to_string(),
                        amount: Uint128::from(2_000u128),
                    },
                    Coin {
                        denom: "uluna".to_string(),
                        amount: Uint128::from(2_000u128),
                    },
                ],
                |result| {
                    result.unwrap();
                },
            )
            .query_all_balances(&creator.to_string(), |result| {
                let balances = result.unwrap();
                // the lp tokens should have gone to the farm manager
                assert!(!balances
                    .iter()
                    .any(|coin| { coin.denom == lp_denom.clone() }));
            })
            // check the LP went to the farm manager
            .query_all_balances(&farm_manager_addr.to_string(), |result| {
                let balances = result.unwrap();
                // the LP tokens should have gone to the farm manager
                // the new minted LP tokens should be 2_000
                assert!(balances.iter().any(|coin| {
                    coin.denom == lp_denom
                        && coin.amount
                            == farm_manager_lp_amount
                                .borrow()
                                .checked_add(Uint128::from(2_000u128))
                                .unwrap()
                }));

                let lp_balance = balances.iter().find(|coin| coin.denom == lp_denom).unwrap();
                *farm_manager_lp_amount.borrow_mut() = lp_balance.amount;
            });

        suite.query_farm_positions(Some(PositionsBy::Receiver(creator.to_string())), None, None, None, |result| {
            let positions = result.unwrap().positions;
            // the position should be updated
            assert_eq!(positions.len(), 2);
            assert_eq!(positions[0], Position {
                identifier: "p-1".to_string(),
                lp_asset: Coin { denom: "factory/mantra1zwv6feuzhy6a9wekh96cd57lsarmqlwxdypdsplw6zhfncqw6ftqlydlr9/o.whale.uluna.LP".to_string(), amount: Uint128::new(2_000u128) },
                unlocking_duration: 200_000,
                open: true,
                expiring_at: None,
                receiver: creator.clone(),
            });
            assert_eq!(positions[1], Position {
                identifier: "u-farm_identifier".to_string(),
                lp_asset: Coin { denom: "factory/mantra1zwv6feuzhy6a9wekh96cd57lsarmqlwxdypdsplw6zhfncqw6ftqlydlr9/o.whale.uluna.LP".to_string(), amount: Uint128::new(999_000u128) },
                unlocking_duration: 86_400,
                open: true,
                expiring_at: None,
                receiver: creator.clone(),
            });
        });
    }

    #[test]
    fn attacker_creates_farm_positions_through_pool_manager() {
        let mut suite = TestingSuite::default_with_balances(
            vec![
                coin(10_000_000u128, "uwhale".to_string()),
                coin(10_000_000u128, "uluna".to_string()),
                coin(10_000u128, "uusd".to_string()),
                coin(10_000u128, "uom".to_string()),
            ],
            StargateMock::new("uom".to_string(), "8888".to_string()),
        );
        let creator = suite.creator();
        let attacker = suite.senders[1].clone();
        let victim = suite.senders[2].clone();

        let asset_denoms = vec!["uwhale".to_string(), "uluna".to_string()];

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
            vec![6u8, 6u8],
            pool_fees,
            PoolType::ConstantProduct,
            Some("whale.uluna".to_string()),
            vec![coin(1000, "uusd"), coin(8888, "uom")],
            |result| {
                result.unwrap();
            },
        );

        // Let's try to add liquidity
        suite
            .provide_liquidity(
                &creator,
                "o.whale.uluna".to_string(),
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
                    // Ensure we got 999_000 in the response which is 1_000_000 less the initial liquidity amount
                    assert!(result.unwrap().events.iter().any(|event| {
                        event.attributes.iter().any(|attr| {
                            attr.key == "share"
                                && attr.value
                                    == (Uint128::from(1_000_000u128) - MINIMUM_LIQUIDITY_AMOUNT)
                                        .to_string()
                        })
                    }));
                },
            )
            .provide_liquidity(
                &attacker,
                "o.whale.uluna".to_string(),
                Some(86_400u64),
                Some("spam_position".to_string()),
                None,
                Some(victim.to_string()),
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
                        ContractError::Unauthorized => {}
                        _ => panic!("Wrong error type, should return ContractError::Unauthorized"),
                    }
                },
            )
            // user can only create positions in farm for himself
            .provide_liquidity(
                &attacker,
                "o.whale.uluna".to_string(),
                Some(86_400u64),
                Some("legit_position".to_string()),
                None,
                Some(attacker.to_string()),
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
            );

        suite.query_farm_positions(Some(PositionsBy::Receiver(attacker.to_string())), None, None, None, |result| {
            let positions = result.unwrap().positions;
            // the position should be updated
            assert_eq!(positions.len(), 1);
            assert_eq!(positions[0], Position {
                identifier: "u-legit_position".to_string(),
                lp_asset: Coin { denom: "factory/mantra1zwv6feuzhy6a9wekh96cd57lsarmqlwxdypdsplw6zhfncqw6ftqlydlr9/o.whale.uluna.LP".to_string(), amount: Uint128::new(1_000_000u128) },
                unlocking_duration: 86_400,
                open: true,
                expiring_at: None,
                receiver: attacker.clone(),
            });
        })
            .query_farm_positions(Some(PositionsBy::Receiver(victim.to_string())), None, None, None, |result| {
                let positions = result.unwrap().positions;
                assert!(positions.is_empty());
            })
        ;
    }
}

mod provide_liquidity {
    use std::cell::RefCell;

    use cosmwasm_std::{assert_approx_eq, coin, Coin, Decimal, StdError, Uint128};

    use amm::fee::{Fee, PoolFee};
    use amm::lp_common::MINIMUM_LIQUIDITY_AMOUNT;
    use amm::pool_manager::PoolType;
    use common_testing::multi_test::stargate_mock::StargateMock;

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
            StargateMock::new("uom".to_string(), "8888".to_string()),
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
                None, None,
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
                None, None,
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
                None, None,
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
                assert!(balances.iter().any(|coin| {
                    coin.denom == lp_denom && coin.amount == Uint128::from(9_798u128)
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
    fn provide_liquidity_with_single_asset_edge_case() {
        let mut suite = TestingSuite::default_with_balances(
            vec![
                coin(1_000_000u128, "uwhale".to_string()),
                coin(1_000_000u128, "uluna".to_string()),
                coin(1_000_000u128, "uosmo".to_string()),
                coin(10_000u128, "uusd".to_string()),
                coin(10_000u128, "uom".to_string()),
            ],
            StargateMock::new("uom".to_string(), "8888".to_string()),
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
            StargateMock::new("uom".to_string(), "8888".to_string()),
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
                assert!(balances.iter().any(|coin| {
                    coin.denom == lp_denom && coin.amount == Uint128::from(9_000u128)
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
            StargateMock::new("uom".to_string(), "8888".to_string()),
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

                assert!(balances.iter().any(|coin| {
                    coin.denom == "uom" && coin.amount == Uint128::new(3_000_000u128)
                }));
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
                    coin.denom == lp_denom.clone()
                        && coin.amount == Uint128::from(1_223_990_208u128)
                }));
            });
    }

    #[test]
    fn provide_liquidity_stable_swap() {
        let mut suite = TestingSuite::default_with_balances(
            vec![
                coin(1_000_000_001u128, "uwhale".to_string()),
                coin(1_000_000_000u128, "uluna".to_string()),
                coin(1_000_000_001u128, "uusd".to_string()),
                coin(1_000_000_001u128, "uom".to_string()),
            ],
            StargateMock::new("uom".to_string(), "8888".to_string()),
        );
        let creator = suite.creator();
        let _other = suite.senders[1].clone();
        let _unauthorized = suite.senders[2].clone();
        // Asset infos with uwhale and uluna

        let asset_infos = vec![
            "uwhale".to_string(),
            "uluna".to_string(),
            "uusd".to_string(),
        ];

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

        // Create a pool
        suite.instantiate_default().create_pool(
            &creator,
            asset_infos,
            vec![6u8, 6u8, 6u8],
            pool_fees,
            PoolType::StableSwap { amp: 100 },
            Some("whale.uluna.uusd".to_string()),
            vec![coin(1000, "uusd"), coin(8888, "uom")],
            |result| {
                result.unwrap();
            },
        );

        // Let's try to add liquidity
        suite.provide_liquidity(
            &creator,
            "o.whale.uluna.uusd".to_string(),
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
                Coin {
                    denom: "uusd".to_string(),
                    amount: Uint128::from(1_000_000u128),
                },
            ],
            |result| {
                // Ensure we got 999000 in the response which is 1mil less the initial liquidity amount
                for event in result.unwrap().events {
                    println!("{:?}", event);
                }
            },
        );
        let simulated_return_amount = RefCell::new(Uint128::zero());
        suite.query_simulation(
            "o.whale.uluna.uusd".to_string(),
            Coin {
                denom: "uwhale".to_string(),
                amount: Uint128::from(1_000u128),
            },
            "uluna".to_string(),
            |result| {
                *simulated_return_amount.borrow_mut() = result.unwrap().return_amount;
            },
        );

        // Now Let's try a swap
        suite.swap(
            &creator,
            "uluna".to_string(),
            None,
            None,
            None,
            "o.whale.uluna.uusd".to_string(),
            vec![coin(1_000u128, "uwhale".to_string())],
            |result| {
                // Find the key with 'offer_amount' and the key with 'return_amount'
                // Ensure that the offer amount is 1000 and the return amount is greater than 0
                let mut return_amount = String::new();
                let mut offer_amount = String::new();

                for event in result.unwrap().events {
                    if event.ty == "wasm" {
                        for attribute in event.attributes {
                            match attribute.key.as_str() {
                                "return_amount" => return_amount = attribute.value,
                                "offer_amount" => offer_amount = attribute.value,
                                _ => {}
                            }
                        }
                    }
                }
                // Because the Pool was created and 1_000_000 of each token has been provided as liquidity
                // Assuming no fees we should expect a small swap of 1000 to result in not too much slippage
                // Expect 1000 give or take 0.002 difference
                // Once fees are added and being deducted properly only the "0.002" should be changed.
                assert_approx_eq!(
                    offer_amount.parse::<u128>().unwrap(),
                    return_amount.parse::<u128>().unwrap(),
                    "0.002"
                );
                assert_approx_eq!(
                    simulated_return_amount.borrow().u128(),
                    return_amount.parse::<u128>().unwrap(),
                    "0.002"
                );
            },
        );

        let simulated_offer_amount = RefCell::new(Uint128::zero());
        // Now Let's try a reverse simulation by swapping uluna to uwhale
        suite.query_reverse_simulation(
            "o.whale.uluna.uusd".to_string(),
            Coin {
                denom: "uwhale".to_string(),
                amount: Uint128::from(1000u128),
            },
            "uluna".to_string(),
            |result| {
                *simulated_offer_amount.borrow_mut() = result.unwrap().offer_amount;
            },
        );

        // Another swap but this time the other way around
        suite.swap(
            &creator,
            "uwhale".to_string(),
            None,
            None,
            None,
            "o.whale.uluna.uusd".to_string(),
            vec![coin(
                simulated_offer_amount.borrow().u128(),
                "uluna".to_string(),
            )],
            |result| {
                // Find the key with 'offer_amount' and the key with 'return_amount'
                // Ensure that the offer amount is 1000 and the return amount is greater than 0
                let mut return_amount = String::new();
                let mut offer_amount = String::new();

                for event in result.unwrap().events {
                    if event.ty == "wasm" {
                        for attribute in event.attributes {
                            match attribute.key.as_str() {
                                "return_amount" => return_amount = attribute.value,
                                "offer_amount" => offer_amount = attribute.value,
                                _ => {}
                            }
                        }
                    }
                }
                assert_approx_eq!(
                    simulated_offer_amount.borrow().u128(),
                    offer_amount.parse::<u128>().unwrap(),
                    "0.002"
                );

                assert_approx_eq!(1000u128, return_amount.parse::<u128>().unwrap(), "0.003");
            },
        );

        // And now uwhale to uusd
        suite.query_reverse_simulation(
            "o.whale.uluna.uusd".to_string(),
            Coin {
                denom: "uusd".to_string(),
                amount: Uint128::from(1000u128),
            },
            "uwhale".to_string(),
            |result| {
                *simulated_return_amount.borrow_mut() = result.unwrap().offer_amount;
            },
        );
        // Another swap but this time uwhale to uusd
        suite.swap(
            &creator,
            "uusd".to_string(),
            None,
            None,
            None,
            "o.whale.uluna.uusd".to_string(),
            vec![coin(
                simulated_return_amount.borrow().u128(),
                "uwhale".to_string(),
            )],
            |result| {
                // Find the key with 'offer_amount' and the key with 'return_amount'
                // Ensure that the offer amount is 1000 and the return amount is greater than 0
                let mut return_amount = String::new();
                let mut offer_amount = String::new();

                for event in result.unwrap().events {
                    if event.ty == "wasm" {
                        for attribute in event.attributes {
                            match attribute.key.as_str() {
                                "return_amount" => return_amount = attribute.value,
                                "offer_amount" => offer_amount = attribute.value,
                                _ => {}
                            }
                        }
                    }
                }
                assert_approx_eq!(
                    simulated_return_amount.borrow().u128(),
                    return_amount.parse::<u128>().unwrap(),
                    "0.002"
                );
                assert_approx_eq!(1000u128, offer_amount.parse::<u128>().unwrap(), "0.003");
            },
        );

        // And now uusd to uluna
        suite.query_reverse_simulation(
            "o.whale.uluna.uusd".to_string(),
            Coin {
                denom: "uluna".to_string(),
                amount: Uint128::from(1000u128),
            },
            "uusd".to_string(),
            |result| {
                *simulated_offer_amount.borrow_mut() = result.unwrap().offer_amount;
            },
        );
        // Another swap but this time uusd to uluna
        suite.swap(
            &creator,
            "uluna".to_string(),
            None,
            None,
            None,
            "o.whale.uluna.uusd".to_string(),
            vec![coin(
                simulated_offer_amount.borrow().u128(),
                "uusd".to_string(),
            )],
            |result| {
                let mut return_amount = String::new();
                let mut offer_amount = String::new();

                for event in result.unwrap().events {
                    if event.ty == "wasm" {
                        for attribute in event.attributes {
                            match attribute.key.as_str() {
                                "return_amount" => return_amount = attribute.value,
                                "offer_amount" => offer_amount = attribute.value,
                                _ => {}
                            }
                        }
                    }
                }
                assert_approx_eq!(
                    simulated_offer_amount.borrow().u128(),
                    offer_amount.parse::<u128>().unwrap(),
                    "0.002"
                );

                assert_approx_eq!(1000u128, return_amount.parse::<u128>().unwrap(), "0.003");
            },
        );
    }

    #[test]
    fn provide_liquidity_stable_swap_shouldnt_double_count_deposits_or_inflate_lp() {
        let mut suite = TestingSuite::default_with_balances(
            vec![
                coin(1_000_000_001u128, "uusd".to_string()),
                coin(1_000_000_001u128, "uusdc".to_string()),
                coin(1_000_000_000u128, "uusdt".to_string()),
                coin(1_000_000_001u128, "uusdy".to_string()),
                coin(1_000_000_001u128, "uom".to_string()),
            ],
            StargateMock::new("uom".to_string(), "8888".to_string()),
        );
        let creator = suite.creator();
        let alice = suite.senders[1].clone();

        let asset_infos = vec![
            "uusdc".to_string(),
            "uusdt".to_string(),
            "uusdy".to_string(),
        ];

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

        // Create a pool
        suite.instantiate_default().create_pool(
            &creator,
            asset_infos,
            vec![6u8, 6u8, 6u8],
            pool_fees,
            PoolType::StableSwap { amp: 100 },
            Some("uusdc.uusdt.uusdy".to_string()),
            vec![coin(1000, "uusd"), coin(8888, "uom")],
            |result| {
                result.unwrap();
            },
        );

        let lp_denom = suite.get_lp_denom("o.uusdc.uusdt.uusdy".to_string());

        // Let's try to add liquidity
        suite
            .provide_liquidity(
                &creator,
                "o.uusdc.uusdt.uusdy".to_string(),
                None,
                None,
                None,
                None,
                vec![
                    Coin {
                        denom: "uusdc".to_string(),
                        amount: Uint128::from(500_000u128),
                    },
                    Coin {
                        denom: "uusdt".to_string(),
                        amount: Uint128::from(500_000u128),
                    },
                    Coin {
                        denom: "uusdy".to_string(),
                        amount: Uint128::from(500_000u128),
                    },
                ],
                |result| {
                    result.unwrap();
                },
            )
            .query_balance(&creator.to_string(), &lp_denom, |result| {
                assert_eq!(
                    result.unwrap().amount,
                    // liquidity provided - MINIMUM_LIQUIDITY_AMOUNT
                    Uint128::from(1_500_000u128 - 1_000u128)
                );
            });

        // let's try providing liquidity again
        suite
            .provide_liquidity(
                &creator,
                "o.uusdc.uusdt.uusdy".to_string(),
                None,
                None,
                None,
                None,
                vec![
                    Coin {
                        denom: "uusdc".to_string(),
                        amount: Uint128::from(500_000u128),
                    },
                    Coin {
                        denom: "uusdt".to_string(),
                        amount: Uint128::from(500_000u128),
                    },
                    Coin {
                        denom: "uusdy".to_string(),
                        amount: Uint128::from(500_000u128),
                    },
                ],
                |result| {
                    result.unwrap();
                },
            )
            .query_balance(&creator.to_string(), &lp_denom, |result| {
                assert_eq!(
                    result.unwrap().amount,
                    // we should expect another ~1_500_000
                    Uint128::from(1_500_000u128 + 1_500_000u128 - 1_000u128)
                );
            });

        let simulated_return_amount = RefCell::new(Uint128::zero());
        suite.query_simulation(
            "o.uusdc.uusdt.uusdy".to_string(),
            Coin {
                denom: "uusdc".to_string(),
                amount: Uint128::from(1_000u128),
            },
            "uusdt".to_string(),
            |result| {
                *simulated_return_amount.borrow_mut() = result.unwrap().return_amount;
            },
        );

        // Now Let's try a swap
        suite.swap(
            &creator,
            "uusdt".to_string(),
            None,
            None,
            None,
            "o.uusdc.uusdt.uusdy".to_string(),
            vec![coin(1_000u128, "uusdc".to_string())],
            |result| {
                // Find the key with 'offer_amount' and the key with 'return_amount'
                // Ensure that the offer amount is 1000 and the return amount is greater than 0
                let mut return_amount = String::new();
                let mut offer_amount = String::new();

                for event in result.unwrap().events {
                    if event.ty == "wasm" {
                        for attribute in event.attributes {
                            match attribute.key.as_str() {
                                "return_amount" => return_amount = attribute.value,
                                "offer_amount" => offer_amount = attribute.value,
                                _ => {}
                            }
                        }
                    }
                }
                // Because the Pool was created and 1_000_000 of each token has been provided as liquidity
                // Assuming no fees we should expect a small swap of 1000 to result in not too much slippage
                // Expect 1000 give or take 0.002 difference
                // Once fees are added and being deducted properly only the "0.002" should be changed.
                assert_approx_eq!(
                    offer_amount.parse::<u128>().unwrap(),
                    return_amount.parse::<u128>().unwrap(),
                    "0.002"
                );
                assert_approx_eq!(
                    simulated_return_amount.borrow().u128(),
                    return_amount.parse::<u128>().unwrap(),
                    "0.002"
                );
            },
        );

        let simulated_offer_amount = RefCell::new(Uint128::zero());
        // Now Let's try a reverse simulation by swapping uluna to uwhale
        suite.query_reverse_simulation(
            "o.uusdc.uusdt.uusdy".to_string(),
            Coin {
                denom: "uusdc".to_string(),
                amount: Uint128::from(1000u128),
            },
            "uusdt".to_string(),
            |result| {
                *simulated_offer_amount.borrow_mut() = result.unwrap().offer_amount;
            },
        );

        // Another swap but this time the other way around
        suite.swap(
            &creator,
            "uusdc".to_string(),
            None,
            None,
            None,
            "o.uusdc.uusdt.uusdy".to_string(),
            vec![coin(
                simulated_offer_amount.borrow().u128(),
                "uusdt".to_string(),
            )],
            |result| {
                // Find the key with 'offer_amount' and the key with 'return_amount'
                // Ensure that the offer amount is 1000 and the return amount is greater than 0
                let mut return_amount = String::new();
                let mut offer_amount = String::new();

                for event in result.unwrap().events {
                    if event.ty == "wasm" {
                        for attribute in event.attributes {
                            match attribute.key.as_str() {
                                "return_amount" => return_amount = attribute.value,
                                "offer_amount" => offer_amount = attribute.value,
                                _ => {}
                            }
                        }
                    }
                }
                assert_approx_eq!(
                    simulated_offer_amount.borrow().u128(),
                    offer_amount.parse::<u128>().unwrap(),
                    "0.002"
                );

                assert_approx_eq!(1000u128, return_amount.parse::<u128>().unwrap(), "0.003");
            },
        );

        // And now uusdc to uusdy
        suite.query_reverse_simulation(
            "o.uusdc.uusdt.uusdy".to_string(),
            Coin {
                denom: "uusdy".to_string(),
                amount: Uint128::from(1000u128),
            },
            "uusdc".to_string(),
            |result| {
                *simulated_return_amount.borrow_mut() = result.unwrap().offer_amount;
            },
        );
        // Another swap but this time uusdc to uusdy
        suite.swap(
            &creator,
            "uusdy".to_string(),
            None,
            None,
            None,
            "o.uusdc.uusdt.uusdy".to_string(),
            vec![coin(
                simulated_return_amount.borrow().u128(),
                "uusdc".to_string(),
            )],
            |result| {
                // Find the key with 'offer_amount' and the key with 'return_amount'
                // Ensure that the offer amount is 1000 and the return amount is greater than 0
                let mut return_amount = String::new();
                let mut offer_amount = String::new();

                for event in result.unwrap().events {
                    if event.ty == "wasm" {
                        for attribute in event.attributes {
                            match attribute.key.as_str() {
                                "return_amount" => return_amount = attribute.value,
                                "offer_amount" => offer_amount = attribute.value,
                                _ => {}
                            }
                        }
                    }
                }
                assert_approx_eq!(
                    simulated_return_amount.borrow().u128(),
                    return_amount.parse::<u128>().unwrap(),
                    "0.002"
                );
                assert_approx_eq!(1000u128, offer_amount.parse::<u128>().unwrap(), "0.003");
            },
        );

        // And now uusdy to uusdt
        suite.query_reverse_simulation(
            "o.uusdc.uusdt.uusdy".to_string(),
            Coin {
                denom: "uusdt".to_string(),
                amount: Uint128::from(1000u128),
            },
            "uusdy".to_string(),
            |result| {
                *simulated_offer_amount.borrow_mut() = result.unwrap().offer_amount;
            },
        );
        // Another swap but this time uusdy to uusdt
        suite.swap(
            &creator,
            "uusdt".to_string(),
            None,
            None,
            None,
            "o.uusdc.uusdt.uusdy".to_string(),
            vec![coin(
                simulated_offer_amount.borrow().u128(),
                "uusdy".to_string(),
            )],
            |result| {
                let mut return_amount = String::new();
                let mut offer_amount = String::new();

                for event in result.unwrap().events {
                    if event.ty == "wasm" {
                        for attribute in event.attributes {
                            match attribute.key.as_str() {
                                "return_amount" => return_amount = attribute.value,
                                "offer_amount" => offer_amount = attribute.value,
                                _ => {}
                            }
                        }
                    }
                }
                assert_approx_eq!(
                    simulated_offer_amount.borrow().u128(),
                    offer_amount.parse::<u128>().unwrap(),
                    "0.002"
                );

                assert_approx_eq!(1000u128, return_amount.parse::<u128>().unwrap(), "0.003");
            },
        );

        // now creator provides even more liquidity
        suite
            .provide_liquidity(
                &creator,
                "o.uusdc.uusdt.uusdy".to_string(),
                None,
                None,
                None,
                None,
                vec![
                    Coin {
                        denom: "uusdc".to_string(),
                        amount: Uint128::from(10_000_000u128),
                    },
                    Coin {
                        denom: "uusdt".to_string(),
                        amount: Uint128::from(10_000_000u128),
                    },
                    Coin {
                        denom: "uusdy".to_string(),
                        amount: Uint128::from(10_000_000u128),
                    },
                ],
                |result| {
                    result.unwrap();
                },
            )
            .query_balance(&creator.to_string(), &lp_denom, |result| {
                assert_approx_eq!(
                    result.unwrap().amount,
                    Uint128::from(30_000_000u128 + 1_500_000u128 + 1_500_000u128 - 1_000u128),
                    "0.000001"
                );
            });

        // now alice provides liquidity
        suite
            .provide_liquidity(
                &alice,
                "o.uusdc.uusdt.uusdy".to_string(),
                None,
                None,
                None,
                None,
                vec![
                    Coin {
                        denom: "uusdc".to_string(),
                        amount: Uint128::from(10_000u128),
                    },
                    Coin {
                        denom: "uusdt".to_string(),
                        amount: Uint128::from(10_000u128),
                    },
                    Coin {
                        denom: "uusdy".to_string(),
                        amount: Uint128::from(10_000u128),
                    },
                ],
                |result| {
                    result.unwrap();
                },
            )
            .query_balance(&alice.to_string(), &lp_denom, |result| {
                // shares are not inflated, alice should have 30_000 LP shares
                assert_eq!(result.unwrap().amount, Uint128::from(30_000u128));
            });
    }

    // This test is to ensure that the edge case of providing liquidity with 3 assets
    #[test]
    fn provide_liquidity_stable_swap_edge_case() {
        let mut suite = TestingSuite::default_with_balances(
            vec![
                coin(1_000_000_001u128, "uwhale".to_string()),
                coin(1_000_000_000u128, "uluna".to_string()),
                coin(1_000_000_001u128, "uusd".to_string()),
                coin(1_000_000_001u128, "uom".to_string()),
            ],
            StargateMock::new("uom".to_string(), "8888".to_string()),
        );
        let creator = suite.creator();
        let _other = suite.senders[1].clone();
        let _unauthorized = suite.senders[2].clone();
        // Asset infos with uwhale and uluna

        let asset_infos = vec![
            "uwhale".to_string(),
            "uluna".to_string(),
            "uusd".to_string(),
        ];

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

        // Create a pool with 3 assets
        suite.instantiate_default().create_pool(
            &creator,
            asset_infos,
            vec![6u8, 6u8, 6u8],
            pool_fees,
            PoolType::StableSwap { amp: 100 },
            Some("whale.uluna.uusd".to_string()),
            vec![coin(1000, "uusd"), coin(8888, "uom")],
            |result| {
                result.unwrap();
            },
        );

        // Adding liquidity with less than the minimum liquidity amount should fail
        suite.provide_liquidity(
            &creator,
            "o.whale.uluna.uusd".to_string(),
            None,
            None,
            None,
            None,
            vec![
                Coin {
                    denom: "uwhale".to_string(),
                    amount: MINIMUM_LIQUIDITY_AMOUNT
                        .checked_div(Uint128::new(3u128))
                        .unwrap(),
                },
                Coin {
                    denom: "uluna".to_string(),
                    amount: MINIMUM_LIQUIDITY_AMOUNT
                        .checked_div(Uint128::new(3u128))
                        .unwrap(),
                },
                Coin {
                    denom: "uusd".to_string(),
                    amount: MINIMUM_LIQUIDITY_AMOUNT
                        .checked_div(Uint128::new(3u128))
                        .unwrap(),
                },
            ],
            |result| {
                assert_eq!(
                    result.unwrap_err().downcast_ref::<ContractError>(),
                    Some(&ContractError::InvalidInitialLiquidityAmount(
                        MINIMUM_LIQUIDITY_AMOUNT
                    ))
                );
            },
        );

        // Let's try to add liquidity with the correct amount (1_000_000 of each asset)
        suite.provide_liquidity(
            &creator,
            "o.whale.uluna.uusd".to_string(),
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
                Coin {
                    denom: "uusd".to_string(),
                    amount: Uint128::from(1_000_000u128),
                },
            ],
            |result| {
                // Ensure we got 999000 in the response which is 1mil less the initial liquidity amount
                for event in result.unwrap().events {
                    for attribute in event.attributes {
                        if attribute.key == "share" {
                            assert_approx_eq!(
                                attribute.value.parse::<u128>().unwrap(),
                                1_000_000u128 * 3,
                                "0.002"
                            );
                        }
                    }
                }
            },
        );

        let simulated_return_amount = RefCell::new(Uint128::zero());
        suite.query_simulation(
            "o.whale.uluna.uusd".to_string(),
            Coin {
                denom: "uwhale".to_string(),
                amount: Uint128::from(1_000u128),
            },
            "uluna".to_string(),
            |result| {
                *simulated_return_amount.borrow_mut() = result.unwrap().return_amount;
            },
        );

        // Now Let's try a swap
        suite.swap(
            &creator,
            "uluna".to_string(),
            None,
            None,
            None,
            "o.whale.uluna.uusd".to_string(),
            vec![coin(1_000u128, "uwhale".to_string())],
            |result| {
                // Find the key with 'offer_amount' and the key with 'return_amount'
                // Ensure that the offer amount is 1000 and the return amount is greater than 0
                let mut return_amount = String::new();
                let mut offer_amount = String::new();

                for event in result.unwrap().events {
                    if event.ty == "wasm" {
                        for attribute in event.attributes {
                            match attribute.key.as_str() {
                                "return_amount" => return_amount = attribute.value,
                                "offer_amount" => offer_amount = attribute.value,
                                _ => {}
                            }
                        }
                    }
                }
                // Because the Pool was created and 1_000_000 of each token has been provided as liquidity
                // Assuming no fees we should expect a small swap of 1000 to result in not too much slippage
                // Expect 1000 give or take 0.002 difference
                // Once fees are added and being deducted properly only the "0.002" should be changed.
                assert_approx_eq!(
                    offer_amount.parse::<u128>().unwrap(),
                    return_amount.parse::<u128>().unwrap(),
                    "0.002"
                );
                assert_approx_eq!(
                    simulated_return_amount.borrow().u128(),
                    return_amount.parse::<u128>().unwrap(),
                    "0.002"
                );
            },
        );
    }

    #[allow(clippy::inconsistent_digit_grouping)]
    #[test]
    fn provide_and_remove_liquidity_18_decimals() {
        let mut suite = TestingSuite::default_with_balances(
            vec![
                coin(1_000_000_000_000u128, "uusd".to_string()),
                coin(1_000_000_000_000u128, "uusdc".to_string()),
                coin(
                    300_000_000_000_000_000000000000000000u128,
                    "ausdy".to_string(),
                ),
                coin(
                    300_000_000_000_000_000000000000000000u128,
                    "pusdc".to_string(),
                ),
                coin(1_000_000_000_000u128, "uom".to_string()),
            ],
            StargateMock::new("uom".to_string(), "8888".to_string()),
        );
        let alice = suite.creator();

        let asset_denoms = vec!["ausdy".to_string(), "pusdc".to_string()];

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
            vec![coin(1000, "uusd"), coin(8888, "uom")],
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
            vec![
                Coin {
                    denom: "pusdc".to_string(),
                    amount: Uint128::new(300_000_000_000_000_000000000000000000u128),
                },
                Coin {
                    denom: "ausdy".to_string(),
                    amount: Uint128::new(300_000_000_000_000_000000000000000000u128),
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
            .query_balance(&alice.to_string(), "pusdc".to_string(), |result| {
                assert_eq!(result.unwrap().amount, Uint128::zero());
            })
            .query_balance(&alice.to_string(), "usdy".to_string(), |result| {
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
            .query_balance(&alice.to_string(), "pusdc".to_string(), |result| {
                assert_approx_eq!(
                    result.unwrap().amount,
                    Uint128::new(300_000_000_000_000_000000000000000000u128),
                    "0.000000000000000001"
                );
            })
            .query_balance(&alice.to_string(), "ausdy".to_string(), |result| {
                assert_approx_eq!(
                    result.unwrap().amount,
                    Uint128::new(300_000_000_000_000_000000000000000000u128),
                    "0.000000000000000001"
                );
            });
    }
}

mod multiple_pools {
    use cosmwasm_std::{coin, Coin, Decimal, Uint128};

    use amm::fee::{Fee, PoolFee};
    use amm::pool_manager::{PoolInfo, PoolType};
    use common_testing::multi_test::stargate_mock::StargateMock;

    use crate::tests::suite::TestingSuite;
    use crate::ContractError;

    #[test]
    fn providing_custom_pool_id_doesnt_increment_pool_counter() {
        let mut suite = TestingSuite::default_with_balances(
            vec![
                coin(1_000_000_000u128, "uwhale".to_string()),
                coin(1_000_000_000u128, "uluna".to_string()),
                coin(1_000_000_000u128, "uosmo".to_string()),
                coin(1_000_000_000u128, "uusd".to_string()),
                coin(1_000_000_000u128, "uom".to_string()),
            ],
            StargateMock::new("uom".to_string(), "8888".to_string()),
        );
        let creator = suite.creator();

        let asset_denoms = vec!["uom".to_string(), "uluna".to_string()];

        let pool_fees = PoolFee {
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

        // Create pools
        suite
            .instantiate_default()
            .add_one_epoch()
            .create_pool(
                &creator,
                asset_denoms.clone(),
                vec![6u8, 6u8],
                pool_fees.clone(),
                PoolType::ConstantProduct,
                Some("pool.1".to_string()),
                vec![coin(1000, "uusd"), coin(8888, "uom")],
                |result| {
                    result.unwrap();
                },
            )
            .create_pool(
                &creator,
                asset_denoms.clone(),
                vec![6u8, 6u8],
                pool_fees.clone(),
                PoolType::ConstantProduct,
                Some("pool.2".to_string()),
                vec![coin(1000, "uusd"), coin(8888, "uom")],
                |result| {
                    result.unwrap();
                },
            )
            .create_pool(
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
            )
            .query_pools(None, None, None, |result| {
                let response = result.unwrap();
                assert_eq!(response.pools.len(), 3);
                assert_eq!(response.pools[0].pool_info.pool_identifier, "o.pool.1");
                assert_eq!(response.pools[1].pool_info.pool_identifier, "o.pool.2");
                assert_eq!(response.pools[2].pool_info.pool_identifier, "p.1");
            });
    }

    #[test]
    fn provide_liquidity_to_multiple_pools_check_fees() {
        let mut suite = TestingSuite::default_with_balances(
            vec![
                coin(1_000_000_000u128, "uwhale".to_string()),
                coin(1_000_000_000u128, "uluna".to_string()),
                coin(1_000_000_000u128, "uosmo".to_string()),
                coin(1_000_000_000u128, "uusd".to_string()),
                coin(1_000_000_000u128, "uom".to_string()),
            ],
            StargateMock::new("uom".to_string(), "8888".to_string()),
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
                None,
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
                None,
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
                None,
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
                });
            })
        ;

        suite
            .swap(
                &creator,
                "uwhale".to_string(),
                None,
                None,
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
                None,
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
                None,
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
            amm::pool_manager::SwapOperation::MantraSwap {
                token_in_denom: "uwhale".to_string(),
                token_out_denom: "uluna".to_string(),
                pool_identifier: "o.whale.uluna.pool.2".to_string(),
            },
            amm::pool_manager::SwapOperation::MantraSwap {
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
            None,
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

    #[test]
    fn cant_create_pool_with_large_number_of_assets() {
        let mut suite = TestingSuite::default_with_balances(
            vec![
                coin(1_000_000_000u128, "uusdy".to_string()),
                coin(1_000_000_000u128, "uusdc".to_string()),
                coin(1_000_000_000u128, "uusdt".to_string()),
                coin(1_000_000_000u128, "uusd".to_string()),
                coin(1_000_000_000u128, "uom".to_string()),
            ],
            StargateMock::new("uom".to_string(), "8888".to_string()),
        );
        let creator = suite.creator();

        // Asset denoms with uwhale and uluna
        let asset_denoms = vec![
            "uusdy".to_string(),
            "uusdc".to_string(),
            "uusdt".to_string(),
            "uusd".to_string(),
        ];

        let pool_fees = PoolFee {
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

        // Create pools
        suite
            .instantiate_default()
            .add_one_epoch()
            .create_pool(
                &creator,
                asset_denoms.clone(),
                vec![6u8, 6u8],
                pool_fees.clone(),
                PoolType::StableSwap { amp: 80 },
                Some("stableswap".to_string()),
                vec![coin(1000, "uusd"), coin(8888, "uom")],
                |result| {
                    let err = result.unwrap_err().downcast::<ContractError>().unwrap();

                    match err {
                        ContractError::AssetMismatch { .. } => {}
                        _ => {
                            panic!("Wrong error type, should return ContractError::AssetMismatch")
                        }
                    }
                },
            )
            .create_pool(
                &creator,
                asset_denoms.clone(),
                vec![6u8, 6u8, 6u8],
                pool_fees.clone(),
                PoolType::StableSwap { amp: 80 },
                Some("stableswap".to_string()),
                vec![coin(1000, "uusd"), coin(8888, "uom")],
                |result| {
                    let err = result.unwrap_err().downcast::<ContractError>().unwrap();

                    match err {
                        ContractError::AssetMismatch { .. } => {}
                        _ => {
                            panic!("Wrong error type, should return ContractError::AssetMismatch")
                        }
                    }
                },
            )
            .create_pool(
                &creator,
                vec![
                    "uusdy".to_string(),
                    "uusdc".to_string(),
                    "uusdt".to_string(),
                    "uusd".to_string(),
                    "uom".to_string(),
                ],
                vec![6u8, 6u8, 6u8, 6u8, 6u8],
                pool_fees.clone(),
                PoolType::StableSwap { amp: 80 },
                Some("stableswap".to_string()),
                vec![coin(1000, "uusd"), coin(8888, "uom")],
                |result| {
                    let err = result.unwrap_err().downcast::<ContractError>().unwrap();

                    match err {
                        ContractError::TooManyAssets { .. } => {}
                        _ => {
                            panic!("Wrong error type, should return ContractError::TooManyAssets")
                        }
                    }
                },
            )
            .create_pool(
                &creator,
                vec![
                    "uusdy".to_string(),
                    "uusdc".to_string(),
                    "uusdt".to_string(),
                    "uusd".to_string(),
                ],
                vec![6u8, 6u8, 6u8, 6u8],
                pool_fees.clone(),
                PoolType::StableSwap { amp: 80 },
                Some("stableswap".to_string()),
                vec![coin(1000, "uusd"), coin(8888, "uom")],
                |result| {
                    result.unwrap();
                },
            );
    }

    #[test]
    fn cant_create_pool_with_bogus_identifier() {
        let mut suite = TestingSuite::default_with_balances(
            vec![
                coin(1_000_000_000u128, "uusdy".to_string()),
                coin(1_000_000_000u128, "uusdc".to_string()),
                coin(1_000_000_000u128, "uusdt".to_string()),
                coin(1_000_000_000u128, "uusd".to_string()),
                coin(1_000_000_000u128, "uom".to_string()),
            ],
            StargateMock::new("uom".to_string(), "8888".to_string()),
        );
        let creator = suite.creator();

        let pool_fees = PoolFee {
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

        // Create pools
        suite
            .instantiate_default()
            .add_one_epoch()
            .create_pool(
                &creator,
                vec![
                    "uusdy".to_string(),
                    "uusdc".to_string(),
                    "uusdt".to_string(),
                    "uusd".to_string(),
                ],
                vec![6u8, 6u8, 6u8, 6u8],
                pool_fees.clone(),
                PoolType::StableSwap { amp: 80 },
                Some("1".to_string()),
                vec![coin(1000, "uusd"), coin(8888, "uom")],
                |result| {
                    result.unwrap();
                },
            )
            .create_pool(
                &creator,
                vec!["uom".to_string(), "uusdc".to_string()],
                vec![6u8, 6u8],
                pool_fees.clone(),
                PoolType::ConstantProduct,
                None,
                vec![coin(1000, "uusd"), coin(8888, "uom")],
                |result| {
                    result.unwrap();
                },
            )
            .query_pools(Some("1".to_string()), None, None, |result| {
                let err = result.unwrap_err();
                assert!(err.to_string().contains("not found"));
            })
            .query_pools(None, None, None, |result| {
                let response = result.unwrap();
                assert_eq!(response.pools.len(), 2);
                assert_eq!(response.pools[0].pool_info.pool_identifier, "o.1");
                assert_eq!(response.pools[1].pool_info.pool_identifier, "p.1");
            });

        suite.create_pool(
            &creator,
            vec![
                "uusdy".to_string(),
                "uusdc".to_string(),
                "uusdt".to_string(),
                "uusd".to_string(),
            ],
            vec![6u8, 6u8, 6u8, 6u8],
            pool_fees.clone(),
            PoolType::StableSwap { amp: 80 },
            Some("1".to_string()),
            vec![coin(1000, "uusd"), coin(8888, "uom")],
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();
                match err {
                    ContractError::PoolExists { .. } => {}
                    _ => {
                        panic!("Wrong error type, should return ContractError::PoolExists")
                    }
                }
            },
        );
    }
}
