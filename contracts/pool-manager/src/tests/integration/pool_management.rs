use super::super::suite::TestingSuite;
use crate::ContractError;
use cosmwasm_std::coin;
use cosmwasm_std::Coin;
use cosmwasm_std::Decimal;
use cosmwasm_std::Uint128;
use mantra_common_testing::multi_test::stargate_mock::StargateMock;
use mantra_dex_std::farm_manager::Position;
use mantra_dex_std::farm_manager::PositionsBy;
use mantra_dex_std::fee::Fee;
use mantra_dex_std::fee::PoolFee;
use mantra_dex_std::lp_common::MINIMUM_LIQUIDITY_AMOUNT;
use mantra_dex_std::pool_manager::FeatureToggle;
use mantra_dex_std::pool_manager::PoolType;

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
        StargateMock::new(vec![
            coin(8888u128, "uom".to_string()),
            coin(1000u128, "utest".to_string()),
        ]),
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
                _ => {
                    panic!("Wrong error type, should return ContractError::InvalidPoolCreationFee")
                }
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
        StargateMock::new(vec![coin(8888u128, "uom".to_string())]),
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
                    ContractError::AssetMismatch => {}
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
                    ContractError::AssetMismatch => {}
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
                    ContractError::SameAsset => {}
                    _ => panic!("Wrong error type, should return ContractError::SameAsset"),
                }
            },
        );
}

#[test]
fn invalid_amount_assets_xyk_pool() {
    let mut suite = TestingSuite::default_with_balances(
        vec![
            coin(1_000_000_001u128, "uwhale".to_string()),
            coin(1_000_000_000u128, "uluna".to_string()),
            coin(1_000_000_001u128, "uusd".to_string()),
            coin(1_000_000_001u128, "uusdc".to_string()),
            coin(1_000_000_001u128, "uom".to_string()),
        ],
        StargateMock::new(vec![coin(8888u128, "uom".to_string())]),
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
            vec!["uom".to_string(), "uusdc".to_string(), "uusd".to_string()],
            vec![6u8, 6u8, 6u8],
            pool_fees.clone(),
            PoolType::ConstantProduct,
            None,
            vec![coin(1000, "uusd"), coin(8888, "uom")],
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();
                match err {
                    ContractError::ConstantProductPoolAssetMismatch => {}
                    _ => panic!("Wrong error type, should return ContractError::ConstantProductPoolAssetMismatch"),
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
                    ContractError::AssetMismatch => {}
                    _ => panic!("Wrong error type, should return ContractError::AssetMismatch"),
                }
            },
        )
        .create_pool(
            &creator,
            vec!["uom".to_string(), "uusd".to_string()],
            vec![6u8, 6u8],
            pool_fees.clone(),
            PoolType::ConstantProduct,
            None,
            vec![coin(1000, "uusd"), coin(8888, "uom")],
            |result| {
                result.unwrap();
            },
        )
        .create_pool(
            &creator,
            vec!["uom".to_string(), "uusdc".to_string(), "uusd".to_string()],
            vec![6u8, 6u8, 6u8],
            pool_fees.clone(),
            PoolType::StableSwap { amp: 85 },
            None,
            vec![coin(1000, "uusd"), coin(8888, "uom")],
            |result| {
                result.unwrap();
            },
        )
    ;
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
        StargateMock::new(vec![coin(8888u128, "uom".to_string())]),
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
                    ContractError::ExtraFundsSent => {}
                    _ => {
                        panic!("Wrong error type, should return ContractError::ExtraFundsSent")
                    }
                }
            },
        )
        .create_pool(
            &creator,
            asset_infos.clone(),
            vec![6u8, 6u8],
            pool_fees.clone(),
            PoolType::ConstantProduct,
            None,
            vec![coin(8888, "uom"), coin(2000, "uusd")],
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();
                match err {
                    ContractError::InvalidPoolCreationFee { amount, expected } => {
                        assert_eq!(amount, Uint128::new(2000));
                        assert_eq!(expected, Uint128::new(1000));
                    }
                    _ => {
                        panic!(
                            "Wrong error type, should return ContractError::InvalidPoolCreationFee"
                        )
                    }
                }
            },
        )
        .create_pool(
            &creator,
            asset_infos.clone(),
            vec![6u8, 6u8],
            pool_fees.clone(),
            PoolType::ConstantProduct,
            None,
            vec![coin(10_000, "uom"), coin(1000, "uusd")],
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();
                match err {
                    ContractError::InvalidTokenFactoryFee {
                        denom,
                        amount,
                        expected,
                    } => {
                        assert_eq!(denom, "uom");
                        assert_eq!(amount, Uint128::new(10_000));
                        assert_eq!(expected, Uint128::new(8888));
                    }
                    _ => {
                        panic!(
                            "Wrong error type, should return ContractError::InvalidTokenFactoryFee"
                        )
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
fn sends_less_tf_denoms_than_needed_with_funds_in_pools() {
    let mut suite = TestingSuite::default_with_balances(
        vec![
            coin(1_000_000_001u128, "uwhale".to_string()),
            coin(1_000_000_000u128, "uluna".to_string()),
            coin(1_000_000_001u128, "uusd".to_string()),
            coin(1_000_000_001u128, "uom".to_string()),
        ],
        StargateMock::new(vec![
            coin(8888u128, "uom".to_string()),
            coin(1000u128, "uusd".to_string()),
        ]),
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
            Some("uom.uusd".to_string()),
            vec![coin(8888, "uom"), coin(2000, "uusd")],
            |result| {
                result.unwrap();
            },
        )
        .provide_liquidity(
            &creator,
            "o.uom.uusd".to_string(),
            None,
            None,
            None,
            None,
            None,
            vec![coin(1_000_000, "uom"), coin(6_000_000, "uusd")],
            |result| {
                result.unwrap();
            },
        )
        .create_pool(
            &creator,
            asset_infos.clone(),
            vec![6u8, 6u8],
            pool_fees.clone(),
            PoolType::ConstantProduct,
            None,
            vec![],
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();
                match err {
                    ContractError::InvalidPoolCreationFee { amount, expected } => {
                        assert_eq!(amount, Uint128::zero());
                        assert_eq!(expected, Uint128::new(2000));
                    }
                    _ => {
                        panic!(
                            "Wrong error type, should return ContractError::InvalidPoolCreationFee"
                        )
                    }
                }
            },
        )
        .create_pool(
            &creator,
            asset_infos.clone(),
            vec![6u8, 6u8],
            pool_fees.clone(),
            PoolType::ConstantProduct,
            None,
            vec![coin(2000, "uusd")],
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();
                match err {
                    ContractError::InvalidTokenFactoryFee {
                        denom,
                        amount,
                        expected,
                    } => {
                        assert_eq!(denom, "uom");
                        assert_eq!(amount, Uint128::zero());
                        assert_eq!(expected, Uint128::new(8888));
                    }
                    _ => {
                        panic!(
                            "Wrong error type, should return ContractError::InvalidPoolCreationFee"
                        )
                    }
                }
            },
        )
        .create_pool(
            &creator,
            asset_infos.clone(),
            vec![6u8, 6u8],
            pool_fees.clone(),
            PoolType::ConstantProduct,
            None,
            vec![coin(2000, "uusd"), coin(8887, "uom")],
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();
                match err {
                    ContractError::InvalidTokenFactoryFee {
                        denom,
                        amount,
                        expected,
                    } => {
                        assert_eq!(denom, "uom");
                        assert_eq!(amount, Uint128::new(8887));
                        assert_eq!(expected, Uint128::new(8888));
                    }
                    _ => {
                        panic!(
                            "Wrong error type, should return ContractError::InvalidPoolCreationFee"
                        )
                    }
                }
            },
        )
        .create_pool(
            &creator,
            asset_infos.clone(),
            vec![6u8, 6u8],
            pool_fees.clone(),
            PoolType::ConstantProduct,
            None,
            vec![coin(2000, "uusd"), coin(8888, "uom")],
            |result| {
                result.unwrap();
            },
        );
}
#[test]
fn sends_more_funds_than_needed_3_tf_denoms() {
    let mut suite = TestingSuite::default_with_balances(
        vec![
            coin(1_000_000_001u128, "uwhale".to_string()),
            coin(1_000_000_000u128, "uluna".to_string()),
            coin(1_000_000_001u128, "uusd".to_string()),
            coin(1_000_000_001u128, "uom".to_string()),
            coin(1_000_000_001u128, "utest".to_string()),
        ],
        StargateMock::new(vec![
            coin(8888u128, "uom".to_string()),
            coin(1000u128, "utest".to_string()),
            coin(1000u128, "uusd".to_string()),
        ]),
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
            vec![
                coin(8888, "uom"),
                coin(2000, "uusd"),
                coin(1000, "utest"),
                coin(1000, "uluna"),
            ],
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();
                match err {
                    ContractError::ExtraFundsSent => {}
                    _ => {
                        panic!("Wrong error type, should return ContractError::ExtraFundsSent")
                    }
                }
            },
        )
        .create_pool(
            &creator,
            asset_infos.clone(),
            vec![6u8, 6u8],
            pool_fees.clone(),
            PoolType::ConstantProduct,
            None,
            vec![coin(8888, "uom"), coin(3000, "uusd"), coin(1000, "utest")],
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();
                match err {
                    ContractError::InvalidPoolCreationFee { amount, expected } => {
                        assert_eq!(amount, Uint128::new(3000));
                        assert_eq!(expected, Uint128::new(2000));
                    }
                    _ => {
                        panic!(
                            "Wrong error type, should return ContractError::InvalidPoolCreationFee"
                        )
                    }
                }
            },
        )
        .create_pool(
            &creator,
            asset_infos.clone(),
            vec![6u8, 6u8],
            pool_fees.clone(),
            PoolType::ConstantProduct,
            None,
            vec![coin(8888, "uom"), coin(2000, "uusd"), coin(2000, "utest")],
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();
                match err {
                    ContractError::InvalidTokenFactoryFee {
                        denom,
                        amount,
                        expected,
                    } => {
                        assert_eq!(denom, "utest");
                        assert_eq!(amount, Uint128::new(2_000));
                        assert_eq!(expected, Uint128::new(1_000));
                    }
                    _ => {
                        panic!(
                            "Wrong error type, should return ContractError::InvalidTokenFactoryFee"
                        )
                    }
                }
            },
        )
        .create_pool(
            &creator,
            asset_infos.clone(),
            vec![6u8, 6u8],
            pool_fees.clone(),
            PoolType::ConstantProduct,
            None,
            vec![coin(9000, "uom"), coin(2000, "uusd"), coin(1000, "utest")],
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();
                match err {
                    ContractError::InvalidTokenFactoryFee {
                        denom,
                        amount,
                        expected,
                    } => {
                        assert_eq!(denom, "uom");
                        assert_eq!(amount, Uint128::new(9000));
                        assert_eq!(expected, Uint128::new(8888));
                    }
                    _ => {
                        panic!(
                            "Wrong error type, should return ContractError::InvalidTokenFactoryFee"
                        )
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
            vec![coin(8888, "uom"), coin(1000, "utest"), coin(2000, "uusd")],
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
                "ibc/3A6F4C8D5B2E7A1F0C4D5B6E7A8F9C3D4E5B6A7F8E9C4D5B6E7A8F9C3D4E5B6A".to_string(),
            ),
            coin(
                1_000_000_000u128,
                "ibc/A1B2C3D4E5F6G7H8I9J0K1L2M3N4O5P6Q7R8S9T0U1V2W3X4Y5Z6A7B8C9D0E1F2".to_string(),
            ),
            coin(
                1_000_000_001u128,
                "factory/mantra158xlpsqqkqpkmcrgnlcrc5fjyhy7j7x2vpa79r/subdenom".to_string(),
            ),
            coin(1_000_000_001u128, "uusd".to_string()),
            coin(1_000_000_001u128, "uom".to_string()),
        ],
        StargateMock::new(vec![coin(8888u128, "uom".to_string())]),
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
        StargateMock::new(vec![coin(8888u128, "uom".to_string())]),
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
fn cant_create_stableswap_with_zero_amp_factor() {
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
        asset_infos.clone(),
        vec![6u8, 6u8],
        pool_fees.clone(),
        PoolType::StableSwap { amp: 0u64 },
        None,
        vec![coin(1000, "uusd"), coin(8888, "uom")],
        |result| {
            let err = result.unwrap_err().downcast::<ContractError>().unwrap();
            match err {
                ContractError::InvalidAmpFactor => {}
                _ => {
                    panic!("Wrong error type, should return ContractError::InvalidAmpFactor")
                }
            }
        },
    );
}

#[test]
fn cant_create_pool_not_paying_multiple_tf_fees() {
    let mut suite = TestingSuite::default_with_balances(
        vec![
            coin(1_000_000_001u128, "uwhale".to_string()),
            coin(1_000_000_000u128, "uluna".to_string()),
            coin(1_000_000_001u128, "uusd".to_string()),
            coin(1_000_000_001u128, "uom".to_string()),
            coin(1_000_000_001u128, "utest".to_string()),
        ],
        StargateMock::new(vec![
            coin(8888u128, "uom".to_string()),
            coin(1000u128, "utest".to_string()),
        ]),
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

    suite
        .instantiate_default()
        .add_one_epoch()
        // pay both tf fees
        .create_pool(
            &creator,
            asset_denoms.clone(),
            vec![6u8, 6u8],
            pool_fees.clone(),
            PoolType::ConstantProduct,
            Some("whale.uluna.pool.1".to_string()),
            vec![coin(1000, "uusd"), coin(8888, "uom")],
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();
                match err {
                    ContractError::InvalidTokenFactoryFee {
                        denom,
                        amount,
                        expected,
                    } => {
                        assert_eq!(denom, "utest");
                        assert_eq!(amount, Uint128::zero());
                        assert_eq!(expected, Uint128::new(1_000));
                    }
                    _ => {
                        panic!(
                            "Wrong error type, should return ContractError::InvalidTokenFactoryFee"
                        )
                    }
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
            vec![coin(1000, "uusd"), coin(999, "utest"), coin(8888, "uom")],
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();

                match err {
                    ContractError::InvalidTokenFactoryFee {
                        denom,
                        amount,
                        expected,
                    } => {
                        assert_eq!(denom, "utest");
                        assert_eq!(amount, Uint128::new(999));
                        assert_eq!(expected, Uint128::new(1_000));
                    }
                    _ => panic!(
                        "Wrong error type, should return ContractError::InvalidTokenFactoryFee"
                    ),
                }
            },
        )
        .create_pool(
            &creator,
            asset_denoms.clone(),
            vec![6u8, 6u8],
            pool_fees.clone(),
            PoolType::ConstantProduct,
            Some("o.whale.uluna.pool.1".to_string()),
            vec![coin(1000, "uusd"), coin(8887, "uom"), coin(1000, "utest")],
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();

                match err {
                    ContractError::InvalidTokenFactoryFee {
                        denom,
                        amount,
                        expected,
                    } => {
                        assert_eq!(denom, "uom");
                        assert_eq!(amount, Uint128::new(8887));
                        assert_eq!(expected, Uint128::new(8888));
                    }
                    _ => panic!(
                        "Wrong error type, should return ContractError::InvalidTokenFactoryFee"
                    ),
                }
            },
        )
        .create_pool(
            &creator,
            asset_denoms.clone(),
            vec![6u8, 6u8],
            pool_fees.clone(),
            PoolType::ConstantProduct,
            Some("o.whale.uluna.pool.1".to_string()),
            vec![coin(999, "uusd"), coin(1000, "utest"), coin(8888, "uom")],
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();

                match err {
                    ContractError::InvalidPoolCreationFee { amount, expected } => {
                        assert_eq!(amount, Uint128::new(999));
                        assert_eq!(expected, Uint128::new(1000));
                    }
                    _ => panic!(
                        "Wrong error type, should return ContractError::InvalidPoolCreationFee"
                    ),
                }
            },
        )
        .create_pool(
            &creator,
            asset_denoms.clone(),
            vec![6u8, 6u8],
            pool_fees.clone(),
            PoolType::ConstantProduct,
            Some("o.whale.uluna.pool.1".to_string()),
            vec![coin(1000, "uusd"), coin(8888, "uom"), coin(1000, "utest")],
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
        StargateMock::new(vec![coin(1000u128, "uusd".to_string())]),
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

#[test]
fn attacker_creates_farm_positions_through_pool_manager() {
    let mut suite = TestingSuite::default_with_balances(
        vec![
            coin(10_000_000u128, "uwhale".to_string()),
            coin(10_000_000u128, "uluna".to_string()),
            coin(10_000u128, "uusd".to_string()),
            coin(10_000u128, "uom".to_string()),
        ],
        StargateMock::new(vec![coin(8888u128, "uom".to_string())]),
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
                        attr.key == "added_shares"
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
        // try creating a position in the farm with a single-sided lp
        .provide_liquidity(
            &attacker,
            "o.whale.uluna".to_string(),
            Some(86_400u64),
            Some("spam_position".to_string()),
            None,
            Some(Decimal::percent(50)),
            Some(victim.to_string()),
            vec![Coin {
                denom: "uwhale".to_string(),
                amount: Uint128::from(1_000_000u128),
            }],
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
        StargateMock::new(vec![coin(8888u128, "uom".to_string())]),
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
            vec!["uom".to_string(), "uusdt".to_string()],
            vec![6u8, 6u8],
            pool_fees.clone(),
            PoolType::ConstantProduct,
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
        vec!["uom".to_string(), "uusdt".to_string()],
        vec![6u8, 6u8],
        pool_fees.clone(),
        PoolType::ConstantProduct,
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
        StargateMock::new(vec![coin(8888u128, "uom".to_string())]),
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
                    ContractError::AssetMismatch => {}
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
                    ContractError::AssetMismatch => {}
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
fn providing_custom_pool_id_doesnt_increment_pool_counter() {
    let mut suite = TestingSuite::default_with_balances(
        vec![
            coin(1_000_000_000u128, "uwhale".to_string()),
            coin(1_000_000_000u128, "uluna".to_string()),
            coin(1_000_000_000u128, "uosmo".to_string()),
            coin(1_000_000_000u128, "uusd".to_string()),
            coin(1_000_000_000u128, "uom".to_string()),
        ],
        StargateMock::new(vec![coin(8888u128, "uom".to_string())]),
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
fn lock_single_pool() {
    let mut suite = TestingSuite::default_with_balances(
        vec![
            coin(1_000_000_000u128, "uom".to_string()),
            coin(1_000_000_000u128, "uusd".to_string()),
        ],
        StargateMock::new(vec![coin(1000u128, "uom".to_string())]),
    );
    let creator = suite.creator();

    let asset_denoms = vec!["uom".to_string(), "uusd".to_string()];

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

    // Create two pools, one will be locked, the other won't be
    suite
        .instantiate_default()
        .add_one_epoch()
        .create_pool(
            &creator,
            asset_denoms.clone(),
            vec![6u8, 6u8],
            pool_fees.clone(),
            PoolType::ConstantProduct,
            Some("uom.uusd.1".to_string()),
            vec![coin(1000, "uusd"), coin(1000, "uom")],
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
            Some("uom.uusd.2".to_string()),
            vec![coin(1000, "uusd"), coin(1000, "uom")],
            |result| {
                result.unwrap();
            },
        );

    suite.provide_liquidity(
        &creator,
        "o.uom.uusd.1".to_string(),
        None,
        None,
        None,
        None,
        None,
        vec![
            Coin {
                denom: "uom".to_string(),
                amount: Uint128::from(1_000u128),
            },
            Coin {
                denom: "uusd".to_string(),
                amount: Uint128::from(8_000u128),
            },
        ],
        |result| {
            result.unwrap();
        },
    );

    let lp_denom_1 = suite.get_lp_denom("o.uom.uusd.1".to_string());
    let lp_denom_2 = suite.get_lp_denom("o.uom.uusd.2".to_string());

    suite.update_config(
        &creator,
        None,
        None,
        None,
        Some(FeatureToggle {
            pool_identifier: "o.uom.uusd.1".to_string(),
            withdrawals_enabled: Some(false),
            deposits_enabled: Some(false),
            swaps_enabled: Some(false),
        }),
        |res| {
            res.unwrap();
        },
    );

    suite
        .provide_liquidity(
            &creator,
            "o.uom.uusd.1".to_string(),
            None,
            None,
            None,
            None,
            None,
            vec![
                Coin {
                    denom: "uom".to_string(),
                    amount: Uint128::from(1_000u128),
                },
                Coin {
                    denom: "uusd".to_string(),
                    amount: Uint128::from(8_000u128),
                },
            ],
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();
                match err {
                    ContractError::OperationDisabled(o) => {
                        assert_eq!(o, "provide_liquidity")
                    }
                    _ => panic!("Wrong error type, should return ContractError::OperationDisabled"),
                }
            },
        )
        .swap(
            &creator,
            "uom".to_string(),
            None,
            None,
            None,
            "o.uom.uusd.1".to_string(),
            vec![coin(1000u128, "uusd".to_string())],
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();
                match err {
                    ContractError::OperationDisabled(o) => {
                        assert_eq!(o, "swap")
                    }
                    _ => panic!("Wrong error type, should return ContractError::OperationDisabled"),
                }
            },
        )
        .withdraw_liquidity(
            &creator,
            "o.uom.uusd.1".to_string(),
            vec![Coin {
                denom: lp_denom_1.clone(),
                amount: Uint128::from(100u128),
            }],
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();
                match err {
                    ContractError::OperationDisabled(o) => {
                        assert_eq!(o, "withdraw_liquidity")
                    }
                    _ => panic!("Wrong error type, should return ContractError::OperationDisabled"),
                }
            },
        );

    // the second pool should still work
    suite
        .provide_liquidity(
            &creator,
            "o.uom.uusd.2".to_string(),
            None,
            None,
            None,
            None,
            None,
            vec![
                Coin {
                    denom: "uom".to_string(),
                    amount: Uint128::from(1_000u128),
                },
                Coin {
                    denom: "uusd".to_string(),
                    amount: Uint128::from(8_000u128),
                },
            ],
            |result| {
                result.unwrap();
            },
        )
        .swap(
            &creator,
            "uom".to_string(),
            None,
            None,
            None,
            "o.uom.uusd.2".to_string(),
            vec![coin(100u128, "uusd".to_string())],
            |result| {
                result.unwrap();
            },
        )
        .withdraw_liquidity(
            &creator,
            "o.uom.uusd.2".to_string(),
            vec![Coin {
                denom: lp_denom_2.clone(),
                amount: Uint128::from(100u128),
            }],
            |result| {
                result.unwrap();
            },
        );

    suite.update_config(
        &creator,
        None,
        None,
        None,
        Some(FeatureToggle {
            pool_identifier: "o.uom.uusd.1".to_string(),
            withdrawals_enabled: None,
            deposits_enabled: None,
            swaps_enabled: Some(true),
        }),
        |res| {
            res.unwrap();
        },
    );

    suite
        .provide_liquidity(
            &creator,
            "o.uom.uusd.1".to_string(),
            None,
            None,
            None,
            None,
            None,
            vec![
                Coin {
                    denom: "uom".to_string(),
                    amount: Uint128::from(1_000u128),
                },
                Coin {
                    denom: "uusd".to_string(),
                    amount: Uint128::from(8_000u128),
                },
            ],
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();
                match err {
                    ContractError::OperationDisabled(o) => {
                        assert_eq!(o, "provide_liquidity")
                    }
                    _ => panic!("Wrong error type, should return ContractError::OperationDisabled"),
                }
            },
        )
        .swap(
            &creator,
            "uom".to_string(),
            None,
            None,
            None,
            "o.uom.uusd.1".to_string(),
            vec![coin(50u128, "uusd".to_string())],
            |result| {
                // should work, as it was enabled
                result.unwrap();
            },
        )
        .withdraw_liquidity(
            &creator,
            "o.uom.uusd.1".to_string(),
            vec![Coin {
                denom: lp_denom_1.clone(),
                amount: Uint128::from(100u128),
            }],
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();
                match err {
                    ContractError::OperationDisabled(o) => {
                        assert_eq!(o, "withdraw_liquidity")
                    }
                    _ => panic!("Wrong error type, should return ContractError::OperationDisabled"),
                }
            },
        );
}
#[test]
fn cant_toggle_unexisting_pool() {
    let mut suite = TestingSuite::default_with_balances(
        vec![
            coin(1_000_000_000u128, "uom".to_string()),
            coin(1_000_000_000u128, "uusd".to_string()),
        ],
        StargateMock::new(vec![coin(1000u128, "uom".to_string())]),
    );
    let creator = suite.creator();

    let asset_denoms = vec!["uom".to_string(), "uusd".to_string()];

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

    suite.instantiate_default().add_one_epoch().create_pool(
        &creator,
        asset_denoms.clone(),
        vec![6u8, 6u8],
        pool_fees.clone(),
        PoolType::ConstantProduct,
        Some("uom.uusd.1".to_string()),
        vec![coin(1000, "uusd"), coin(1000, "uom")],
        |result| {
            result.unwrap();
        },
    );

    suite.update_config(
        &creator,
        None,
        None,
        None,
        Some(FeatureToggle {
            pool_identifier: "xxx".to_string(),
            withdrawals_enabled: Some(false),
            deposits_enabled: Some(false),
            swaps_enabled: Some(false),
        }),
        |res| {
            let err = res.unwrap_err().downcast::<ContractError>().unwrap();
            match err {
                ContractError::UnExistingPool => {}
                _ => panic!("Wrong error type, should return ContractError::UnExistingPool"),
            }
        },
    );
}
