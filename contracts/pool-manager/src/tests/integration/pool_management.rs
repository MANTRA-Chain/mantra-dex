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
use super::common_constants::{
    DENOM_UWHALE, 
    DENOM_ULUNA, 
    DENOM_UUSD, 
    DENOM_UOM, 
    DENOM_UUSDC, 
    DENOM_UUSDY, 
    DENOM_UUSDT, 
    DENOM_UOSMO,
    INITIAL_BALANCE as BALANCE_AMOUNT_MEDIUM,
    INITIAL_BALANCE_PLUS_ONE as BALANCE_AMOUNT_LARGE,
    STARGATE_MOCK_UOM_AMOUNT as MOCK_AMOUNT_UOM,
    LIQUIDITY_AMOUNT,
    POOL_CREATION_FEE as REQUIRED_POOL_CREATION_FEE,
    DECIMAL_PLACES as DEFAULT_DECIMAL_PRECISION,
    STABLESWAP_AMP_FACTOR,
    UNLOCKING_DURATION,
};

// Test constants
const DENOM_UTEST: &str = "utest";
const DENOM_IBC_1: &str = "ibc/3A6F4C8D5B2E7A1F0C4D5B6E7A8F9C3D4E5B6A7F8E9C4D5B6E7A8F9C3D4E5B6A";
const DENOM_IBC_2: &str = "ibc/A1B2C3D4E5F6G7H8I9J0K1L2M3N4O5P6Q7R8S9T0U1V2W3X4Y5Z6A7B8C9D0E1F2";
const DENOM_FACTORY: &str = "factory/mantra158xlpsqqkqpkmcrgnlcrc5fjyhy7j7x2vpa79r/subdenom";

// Common token amounts
const MOCK_AMOUNT_UTEST: u128 = 1000;
const INSUFFICIENT_TF_FEE: u128 = 999;
const INSUFFICIENT_POOL_AMOUNT: u128 = 999;
const MOCK_AMOUNT_ULUNA: u128 = 1000;
const INSUFFICIENT_POOL_CREATION_AMOUNT: u128 = 900;
const TWICE_POOL_CREATION_FEE: u128 = 2000;
const INSUFFICIENT_TWICE_POOL_CREATION_FEE: u128 = 1999;
const EXCESSIVE_POOL_CREATION_FEE: u128 = 3000;
const ATTACKER_BALANCE_AMOUNT: u128 = 10_000_000;
const SMALL_BALANCE_AMOUNT: u128 = 10_000;

// Fee constants
const INSUFFICIENT_POOL_CREATION_FEE: u128 = 90;

// Fee percentages
const DEFAULT_FEE_PERCENT: u64 = 1;
const PROTOCOL_FEE_PERCENT: u64 = 10;
const SWAP_FEE_PERCENT: u64 = 7;
const BURN_FEE_PERCENT: u64 = 3;
const SINGLE_SIDED_LP_PERCENT: u64 = 50;

// Pool constants
const STABLESWAP_TEST_AMP_FACTOR: u64 = 80;
const STABLESWAP_POOL_ID: &str = "stableswap";

// Pool identifier constants
const INVALID_POOL_IDENTIFIER_DASH: &str = "invalid-identifier";
const INVALID_POOL_IDENTIFIER_LONG: &str = "this.is.a.loooooooooooooooooong.identifier";
const VALID_POOL_IDENTIFIER: &str = "mycoolpool";
const WHALE_LUNA_POOL_IDENTIFIER: &str = "whale.uluna.pool.1";
const OTHER_WHALE_LUNA_POOL_IDENTIFIER: &str = "o.whale.uluna.pool.1";
const WHALE_LUNA_POOL_RAW_ID: &str = "whale.uluna";
const WHALE_LUNA_POOL_PREFIX: &str = "o.whale.uluna";
const POOL_ID_NUMERIC: &str = "1";
const POOL_ID_PREFIX_O: &str = "o.1";
const POOL_ID_PREFIX_P: &str = "p.1";
const CUSTOM_POOL_ID_1: &str = "pool.1";
const CUSTOM_POOL_ID_2: &str = "pool.2";
const CUSTOM_POOL_PREFIX_1: &str = "o.pool.1";
const CUSTOM_POOL_PREFIX_2: &str = "o.pool.2";

// Position constants
const SPAM_POSITION_ID: &str = "spam_position";
const LEGIT_POSITION_ID: &str = "legit_position";

// Lock pool test constants
const LOCK_POOL_BALANCE_AMOUNT: u128 = 1_000_000_000u128;
const LOCK_POOL_TF_FEE: u128 = 1000u128;
const LOCK_POOL_LIQUIDITY_AMOUNT: u128 = 1_000u128;
const LOCK_POOL_LIQUIDITY_AMOUNT_2: u128 = 8_000u128;
const LOCK_POOL_SWAP_AMOUNT: u128 = 100u128;
const LOCK_POOL_SWAP_AMOUNT_2: u128 = 50u128;
const LOCK_POOL_ID_1: &str = "uom.uusd.1";
const LOCK_POOL_ID_2: &str = "uom.uusd.2";
const LOCK_POOL_PREFIX_1: &str = "o.uom.uusd.1";
const LOCK_POOL_PREFIX_2: &str = "o.uom.uusd.2";

// Toggle pool test constants
const TOGGLE_POOL_BALANCE_AMOUNT: u128 = 1_000_000_000u128;
const TOGGLE_POOL_TF_FEE: u128 = 1000u128;
const TOGGLE_POOL_ID: &str = "uom.uusd.1";
const TOGGLE_INVALID_POOL_ID: &str = "xxx";

// Insufficient fee to create pool; 90 instead of 100
#[test]
fn insufficient_pool_creation_fee() {
    let mut suite = TestingSuite::default_with_balances(
        vec![
            coin(BALANCE_AMOUNT_LARGE, DENOM_UWHALE),
            coin(BALANCE_AMOUNT_MEDIUM, DENOM_ULUNA),
            coin(BALANCE_AMOUNT_LARGE, DENOM_UUSD),
            coin(BALANCE_AMOUNT_LARGE, DENOM_UOM),
        ],
        StargateMock::new(vec![
            coin(MOCK_AMOUNT_UOM, DENOM_UOM),
            coin(MOCK_AMOUNT_UTEST, DENOM_UTEST),
        ]),
    );
    let creator = suite.creator();
    let _other = suite.senders[1].clone();
    let _unauthorized = suite.senders[2].clone();
    // Asset infos with uwhale

    let asset_infos = vec![DENOM_UWHALE.to_string(), DENOM_UOM.to_string()];

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
        vec![DEFAULT_DECIMAL_PRECISION, DEFAULT_DECIMAL_PRECISION],
        pool_fees,
        PoolType::ConstantProduct,
        None,
        vec![coin(INSUFFICIENT_POOL_CREATION_FEE, DENOM_UUSD)],
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
            coin(BALANCE_AMOUNT_LARGE, DENOM_UWHALE),
            coin(BALANCE_AMOUNT_MEDIUM, DENOM_ULUNA),
            coin(BALANCE_AMOUNT_LARGE, DENOM_UUSD),
            coin(BALANCE_AMOUNT_LARGE, DENOM_UOM),
        ],
        StargateMock::new(vec![coin(MOCK_AMOUNT_UOM, DENOM_UOM)]),
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
            vec![DEFAULT_DECIMAL_PRECISION, DEFAULT_DECIMAL_PRECISION],
            pool_fees.clone(),
            PoolType::ConstantProduct,
            None,
            vec![
                coin(REQUIRED_POOL_CREATION_FEE, DENOM_UUSD),
                coin(MOCK_AMOUNT_UOM, DENOM_UOM),
            ],
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
            vec![DENOM_UOM.to_string()],
            vec![DEFAULT_DECIMAL_PRECISION],
            pool_fees.clone(),
            PoolType::ConstantProduct,
            None,
            vec![
                coin(REQUIRED_POOL_CREATION_FEE, DENOM_UUSD),
                coin(MOCK_AMOUNT_UOM, DENOM_UOM),
            ],
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
            vec![DENOM_UOM.to_string(), DENOM_UOM.to_string()],
            vec![DEFAULT_DECIMAL_PRECISION, DEFAULT_DECIMAL_PRECISION],
            pool_fees,
            PoolType::ConstantProduct,
            None,
            vec![
                coin(REQUIRED_POOL_CREATION_FEE, DENOM_UUSD),
                coin(MOCK_AMOUNT_UOM, DENOM_UOM),
            ],
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
            coin(BALANCE_AMOUNT_LARGE, DENOM_UWHALE),
            coin(BALANCE_AMOUNT_MEDIUM, DENOM_ULUNA),
            coin(BALANCE_AMOUNT_LARGE, DENOM_UUSD),
            coin(BALANCE_AMOUNT_LARGE, DENOM_UUSDC),
            coin(BALANCE_AMOUNT_LARGE, DENOM_UOM),
        ],
        StargateMock::new(vec![coin(MOCK_AMOUNT_UOM, DENOM_UOM)]),
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
            vec![DENOM_UOM.to_string(), DENOM_UUSDC.to_string(), DENOM_UUSD.to_string()],
            vec![DEFAULT_DECIMAL_PRECISION, DEFAULT_DECIMAL_PRECISION, DEFAULT_DECIMAL_PRECISION],
            pool_fees.clone(),
            PoolType::ConstantProduct,
            None,
            vec![coin(REQUIRED_POOL_CREATION_FEE, DENOM_UUSD), coin(MOCK_AMOUNT_UOM, DENOM_UOM)],
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
            vec![DENOM_UOM.to_string()],
            vec![DEFAULT_DECIMAL_PRECISION],
            pool_fees.clone(),
            PoolType::ConstantProduct,
            None,
            vec![coin(REQUIRED_POOL_CREATION_FEE, DENOM_UUSD), coin(MOCK_AMOUNT_UOM, DENOM_UOM)],
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
            vec![DENOM_UOM.to_string(), DENOM_UUSD.to_string()],
            vec![DEFAULT_DECIMAL_PRECISION, DEFAULT_DECIMAL_PRECISION],
            pool_fees.clone(),
            PoolType::ConstantProduct,
            None,
            vec![coin(REQUIRED_POOL_CREATION_FEE, DENOM_UUSD), coin(MOCK_AMOUNT_UOM, DENOM_UOM)],
            |result| {
                result.unwrap();
            },
        )
        .create_pool(
            &creator,
            vec![DENOM_UOM.to_string(), DENOM_UUSDC.to_string(), DENOM_UUSD.to_string()],
            vec![DEFAULT_DECIMAL_PRECISION, DEFAULT_DECIMAL_PRECISION, DEFAULT_DECIMAL_PRECISION],
            pool_fees.clone(),
            PoolType::StableSwap { amp: STABLESWAP_AMP_FACTOR },
            None,
            vec![coin(REQUIRED_POOL_CREATION_FEE, DENOM_UUSD), coin(MOCK_AMOUNT_UOM, DENOM_UOM)],
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
            coin(BALANCE_AMOUNT_LARGE, DENOM_UWHALE),
            coin(BALANCE_AMOUNT_MEDIUM, DENOM_ULUNA),
            coin(BALANCE_AMOUNT_LARGE, DENOM_UUSD),
            coin(BALANCE_AMOUNT_LARGE, DENOM_UOM),
        ],
        StargateMock::new(vec![coin(MOCK_AMOUNT_UOM, DENOM_UOM)]),
    );
    let creator = suite.creator();

    let asset_infos = vec![DENOM_UOM.to_string(), DENOM_UUSD.to_string()];

    // Additional constants specific to this test
    const EXTRA_FEE_AMOUNT: u128 = 2000;
    const INVALID_TF_FEE_AMOUNT: u128 = 10_000;

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
            vec![DEFAULT_DECIMAL_PRECISION, DEFAULT_DECIMAL_PRECISION],
            pool_fees.clone(),
            PoolType::ConstantProduct,
            None,
            vec![
                coin(MOCK_AMOUNT_UOM, DENOM_UOM),
                coin(REQUIRED_POOL_CREATION_FEE, DENOM_UUSD),
                coin(REQUIRED_POOL_CREATION_FEE, DENOM_ULUNA),
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
            vec![DEFAULT_DECIMAL_PRECISION, DEFAULT_DECIMAL_PRECISION],
            pool_fees.clone(),
            PoolType::ConstantProduct,
            None,
            vec![
                coin(MOCK_AMOUNT_UOM, DENOM_UOM),
                coin(EXTRA_FEE_AMOUNT, DENOM_UUSD),
            ],
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();
                match err {
                    ContractError::InvalidPoolCreationFee { amount, expected } => {
                        assert_eq!(amount, Uint128::new(EXTRA_FEE_AMOUNT));
                        assert_eq!(expected, Uint128::new(REQUIRED_POOL_CREATION_FEE));
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
            vec![DEFAULT_DECIMAL_PRECISION, DEFAULT_DECIMAL_PRECISION],
            pool_fees.clone(),
            PoolType::ConstantProduct,
            None,
            vec![
                coin(INVALID_TF_FEE_AMOUNT, DENOM_UOM),
                coin(REQUIRED_POOL_CREATION_FEE, DENOM_UUSD),
            ],
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();
                match err {
                    ContractError::InvalidTokenFactoryFee {
                        denom,
                        amount,
                        expected,
                    } => {
                        assert_eq!(denom, DENOM_UOM);
                        assert_eq!(amount, Uint128::new(INVALID_TF_FEE_AMOUNT));
                        assert_eq!(expected, Uint128::new(MOCK_AMOUNT_UOM));
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
            vec![DEFAULT_DECIMAL_PRECISION, DEFAULT_DECIMAL_PRECISION],
            pool_fees,
            PoolType::ConstantProduct,
            None,
            vec![
                coin(MOCK_AMOUNT_UOM, DENOM_UOM),
                coin(REQUIRED_POOL_CREATION_FEE, DENOM_UUSD),
            ],
            |result| {
                result.unwrap();
            },
        );
}

#[test]
fn sends_less_tf_denoms_than_needed_with_funds_in_pools() {
    let mut suite = TestingSuite::default_with_balances(
        vec![
            coin(BALANCE_AMOUNT_LARGE, DENOM_UWHALE),
            coin(BALANCE_AMOUNT_MEDIUM, DENOM_ULUNA),
            coin(BALANCE_AMOUNT_LARGE, DENOM_UUSD),
            coin(BALANCE_AMOUNT_LARGE, DENOM_UOM),
        ],
        StargateMock::new(vec![
            coin(MOCK_AMOUNT_UOM, DENOM_UOM),
            coin(REQUIRED_POOL_CREATION_FEE, DENOM_UUSD),
        ]),
    );
    let creator = suite.creator();

    let asset_infos = vec![DENOM_UOM.to_string(), DENOM_UUSD.to_string()];

    // Additional constants specific to this test
    const LIQUIDITY_AMOUNT_UOM: u128 = 1_000_000;
    const LIQUIDITY_AMOUNT_UUSD: u128 = 6_000_000;
    const DOUBLE_POOL_CREATION_FEE: u128 = 2000;
    const INSUFFICIENT_TF_FEE: u128 = 8887;

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

    // First create a pool with the proper fees
    suite.instantiate_default().add_one_epoch().create_pool(
        &creator,
        asset_infos.clone(),
        vec![DEFAULT_DECIMAL_PRECISION, DEFAULT_DECIMAL_PRECISION],
        pool_fees.clone(),
        PoolType::ConstantProduct,
        Some("uom.uusd".to_string()),
        vec![
            coin(DOUBLE_POOL_CREATION_FEE, DENOM_UUSD),
            coin(MOCK_AMOUNT_UOM, DENOM_UOM),
        ],
        |result| {
            result.unwrap();
        },
    );

    // Then provide liquidity separately
    suite.provide_liquidity(
        &creator,
        "o.uom.uusd".to_string(),
        None,
        None,
        None,
        None,
        None,
        vec![
            coin(LIQUIDITY_AMOUNT_UOM, DENOM_UOM),
            coin(LIQUIDITY_AMOUNT_UUSD, DENOM_UUSD),
        ],
        |result| {
            result.unwrap();
        },
    );

    // Now proceed with the test cases for validation
    suite.create_pool(
        &creator,
        asset_infos.clone(),
        vec![DEFAULT_DECIMAL_PRECISION, DEFAULT_DECIMAL_PRECISION],
        pool_fees.clone(),
        PoolType::ConstantProduct,
        None,
        vec![],
        |result| {
            let err = result.unwrap_err().downcast::<ContractError>().unwrap();
            match err {
                ContractError::InvalidPoolCreationFee { amount, expected } => {
                    assert_eq!(amount, Uint128::zero());
                    assert_eq!(expected, Uint128::new(DOUBLE_POOL_CREATION_FEE));
                }
                _ => {
                    panic!("Wrong error type, should return ContractError::InvalidPoolCreationFee")
                }
            }
        },
    );

    suite.create_pool(
        &creator,
        asset_infos.clone(),
        vec![DEFAULT_DECIMAL_PRECISION, DEFAULT_DECIMAL_PRECISION],
        pool_fees.clone(),
        PoolType::ConstantProduct,
        None,
        vec![coin(DOUBLE_POOL_CREATION_FEE, DENOM_UUSD)],
        |result| {
            let err = result.unwrap_err().downcast::<ContractError>().unwrap();
            match err {
                ContractError::InvalidTokenFactoryFee {
                    denom,
                    amount,
                    expected,
                } => {
                    assert_eq!(denom, DENOM_UOM);
                    assert_eq!(amount, Uint128::zero());
                    assert_eq!(expected, Uint128::new(MOCK_AMOUNT_UOM));
                }
                _ => {
                    panic!("Wrong error type, should return ContractError::InvalidTokenFactoryFee")
                }
            }
        },
    );

    suite.create_pool(
        &creator,
        asset_infos.clone(),
        vec![DEFAULT_DECIMAL_PRECISION, DEFAULT_DECIMAL_PRECISION],
        pool_fees.clone(),
        PoolType::ConstantProduct,
        None,
        vec![
            coin(DOUBLE_POOL_CREATION_FEE, DENOM_UUSD),
            coin(INSUFFICIENT_TF_FEE, DENOM_UOM),
        ],
        |result| {
            let err = result.unwrap_err().downcast::<ContractError>().unwrap();
            match err {
                ContractError::InvalidTokenFactoryFee {
                    denom,
                    amount,
                    expected,
                } => {
                    assert_eq!(denom, DENOM_UOM);
                    assert_eq!(amount, Uint128::new(INSUFFICIENT_TF_FEE));
                    assert_eq!(expected, Uint128::new(MOCK_AMOUNT_UOM));
                }
                _ => {
                    panic!("Wrong error type, should return ContractError::InvalidTokenFactoryFee")
                }
            }
        },
    );

    suite.create_pool(
        &creator,
        asset_infos.clone(),
        vec![DEFAULT_DECIMAL_PRECISION, DEFAULT_DECIMAL_PRECISION],
        pool_fees.clone(),
        PoolType::ConstantProduct,
        None,
        vec![
            coin(DOUBLE_POOL_CREATION_FEE, DENOM_UUSD),
            coin(MOCK_AMOUNT_UOM, DENOM_UOM),
        ],
        |result| {
            result.unwrap();
        },
    );
}

#[test]
fn sends_more_funds_than_needed_3_tf_denoms() {
    let mut suite = TestingSuite::default_with_balances(
        vec![
            coin(BALANCE_AMOUNT_LARGE, DENOM_UWHALE),
            coin(BALANCE_AMOUNT_MEDIUM, DENOM_ULUNA),
            coin(BALANCE_AMOUNT_LARGE, DENOM_UUSD),
            coin(BALANCE_AMOUNT_LARGE, DENOM_UOM),
            coin(BALANCE_AMOUNT_LARGE, DENOM_UTEST),
        ],
        StargateMock::new(vec![
            coin(MOCK_AMOUNT_UOM, DENOM_UOM),
            coin(REQUIRED_POOL_CREATION_FEE, DENOM_UTEST),
            coin(REQUIRED_POOL_CREATION_FEE, DENOM_UUSD),
        ]),
    );
    let creator = suite.creator();

    let asset_infos = vec![DENOM_UOM.to_string(), DENOM_UUSD.to_string()];

    // Additional constants specific to this test
    const EXCESSIVE_MOCK_AMOUNT_UOM: u128 = 9000;
    const EXCESSIVE_FEE_AMOUNT: u128 = 3000;
    const EXCESSIVE_TF_FEE_AMOUNT: u128 = 2000;
    const COMBINED_FEES_AMOUNT: u128 = 2000;

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
            vec![DEFAULT_DECIMAL_PRECISION, DEFAULT_DECIMAL_PRECISION],
            pool_fees.clone(),
            PoolType::ConstantProduct,
            None,
            vec![
                coin(MOCK_AMOUNT_UOM, DENOM_UOM),
                coin(COMBINED_FEES_AMOUNT, DENOM_UUSD),
                coin(REQUIRED_POOL_CREATION_FEE, DENOM_UTEST),
                coin(REQUIRED_POOL_CREATION_FEE, DENOM_ULUNA),
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
            vec![DEFAULT_DECIMAL_PRECISION, DEFAULT_DECIMAL_PRECISION],
            pool_fees.clone(),
            PoolType::ConstantProduct,
            None,
            vec![
                coin(MOCK_AMOUNT_UOM, DENOM_UOM),
                coin(EXCESSIVE_FEE_AMOUNT, DENOM_UUSD),
                coin(REQUIRED_POOL_CREATION_FEE, DENOM_UTEST),
            ],
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();
                match err {
                    ContractError::InvalidPoolCreationFee { amount, expected } => {
                        assert_eq!(amount, Uint128::new(EXCESSIVE_FEE_AMOUNT));
                        assert_eq!(expected, Uint128::new(COMBINED_FEES_AMOUNT));
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
            vec![DEFAULT_DECIMAL_PRECISION, DEFAULT_DECIMAL_PRECISION],
            pool_fees.clone(),
            PoolType::ConstantProduct,
            None,
            vec![
                coin(MOCK_AMOUNT_UOM, DENOM_UOM),
                coin(COMBINED_FEES_AMOUNT, DENOM_UUSD),
                coin(EXCESSIVE_TF_FEE_AMOUNT, DENOM_UTEST),
            ],
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();
                match err {
                    ContractError::InvalidTokenFactoryFee {
                        denom,
                        amount,
                        expected,
                    } => {
                        assert_eq!(denom, DENOM_UTEST);
                        assert_eq!(amount, Uint128::new(EXCESSIVE_TF_FEE_AMOUNT));
                        assert_eq!(expected, Uint128::new(REQUIRED_POOL_CREATION_FEE));
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
            vec![DEFAULT_DECIMAL_PRECISION, DEFAULT_DECIMAL_PRECISION],
            pool_fees.clone(),
            PoolType::ConstantProduct,
            None,
            vec![
                coin(EXCESSIVE_MOCK_AMOUNT_UOM, DENOM_UOM),
                coin(COMBINED_FEES_AMOUNT, DENOM_UUSD),
                coin(REQUIRED_POOL_CREATION_FEE, DENOM_UTEST),
            ],
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();
                match err {
                    ContractError::InvalidTokenFactoryFee {
                        denom,
                        amount,
                        expected,
                    } => {
                        assert_eq!(denom, DENOM_UOM);
                        assert_eq!(amount, Uint128::new(EXCESSIVE_MOCK_AMOUNT_UOM));
                        assert_eq!(expected, Uint128::new(MOCK_AMOUNT_UOM));
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
            vec![DEFAULT_DECIMAL_PRECISION, DEFAULT_DECIMAL_PRECISION],
            pool_fees,
            PoolType::ConstantProduct,
            None,
            vec![
                coin(MOCK_AMOUNT_UOM, DENOM_UOM),
                coin(REQUIRED_POOL_CREATION_FEE, DENOM_UTEST),
                coin(COMBINED_FEES_AMOUNT, DENOM_UUSD),
            ],
            |result| {
                result.unwrap();
            },
        );
}

#[test]
fn wrong_pool_label() {
    let mut suite = TestingSuite::default_with_balances(
        vec![
            coin(BALANCE_AMOUNT_LARGE, DENOM_IBC_1),
            coin(BALANCE_AMOUNT_MEDIUM, DENOM_IBC_2),
            coin(BALANCE_AMOUNT_LARGE, DENOM_FACTORY),
            coin(BALANCE_AMOUNT_LARGE, DENOM_UUSD),
            coin(BALANCE_AMOUNT_LARGE, DENOM_UOM),
        ],
        StargateMock::new(vec![coin(MOCK_AMOUNT_UOM, DENOM_UOM)]),
    );
    let creator = suite.creator();
    let _other = suite.senders[1].clone();
    let _unauthorized = suite.senders[2].clone();
    // Asset infos with uwhale

    let asset_infos = vec![DENOM_IBC_1.to_string(), DENOM_IBC_2.to_string()];

    let pool_fees = PoolFee {
        protocol_fee: Fee {
            share: Decimal::percent(DEFAULT_FEE_PERCENT),
        },
        swap_fee: Fee {
            share: Decimal::percent(DEFAULT_FEE_PERCENT),
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
            vec![DEFAULT_DECIMAL_PRECISION, DEFAULT_DECIMAL_PRECISION],
            pool_fees.clone(),
            PoolType::ConstantProduct,
            Some(INVALID_POOL_IDENTIFIER_DASH.to_string()),
            vec![
                coin(REQUIRED_POOL_CREATION_FEE, DENOM_UUSD),
                coin(MOCK_AMOUNT_UOM, DENOM_UOM),
            ],
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
            vec![DEFAULT_DECIMAL_PRECISION, DEFAULT_DECIMAL_PRECISION],
            pool_fees.clone(),
            PoolType::ConstantProduct,
            //42 chars long
            Some(INVALID_POOL_IDENTIFIER_LONG.to_string()),
            vec![
                coin(REQUIRED_POOL_CREATION_FEE, DENOM_UUSD),
                coin(MOCK_AMOUNT_UOM, DENOM_UOM),
            ],
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
            coin(BALANCE_AMOUNT_LARGE, DENOM_UWHALE),
            coin(BALANCE_AMOUNT_MEDIUM, DENOM_ULUNA),
            coin(BALANCE_AMOUNT_LARGE, DENOM_UUSD),
            coin(BALANCE_AMOUNT_LARGE, DENOM_UOM),
        ],
        StargateMock::new(vec![coin(MOCK_AMOUNT_UOM, DENOM_UOM)]),
    );
    let creator = suite.creator();
    let _other = suite.senders[1].clone();
    let _unauthorized = suite.senders[2].clone();
    // Asset infos with uwhale

    let asset_infos = vec![DENOM_UWHALE.to_string(), DENOM_UOM.to_string()];

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
            vec![DEFAULT_DECIMAL_PRECISION, DEFAULT_DECIMAL_PRECISION],
            pool_fees.clone(),
            PoolType::ConstantProduct,
            Some(VALID_POOL_IDENTIFIER.to_string()),
            vec![
                coin(REQUIRED_POOL_CREATION_FEE, DENOM_UUSD),
                coin(MOCK_AMOUNT_UOM, DENOM_UOM),
            ],
            |result| {
                result.unwrap();
            },
        )
        .create_pool(
            &creator,
            asset_infos,
            vec![DEFAULT_DECIMAL_PRECISION, DEFAULT_DECIMAL_PRECISION],
            pool_fees,
            PoolType::ConstantProduct,
            Some(VALID_POOL_IDENTIFIER.to_string()),
            vec![
                coin(REQUIRED_POOL_CREATION_FEE, DENOM_UUSD),
                coin(MOCK_AMOUNT_UOM, DENOM_UOM),
            ],
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
            coin(BALANCE_AMOUNT_LARGE, DENOM_UWHALE),
            coin(BALANCE_AMOUNT_MEDIUM, DENOM_ULUNA),
            coin(BALANCE_AMOUNT_LARGE, DENOM_UUSD),
            coin(BALANCE_AMOUNT_LARGE, DENOM_UOM),
        ],
        StargateMock::new(vec![coin(MOCK_AMOUNT_UOM, DENOM_UOM)]),
    );
    let creator = suite.creator();
    let _other = suite.senders[1].clone();
    let _unauthorized = suite.senders[2].clone();
    // Asset infos with uwhale

    let asset_infos = vec![DENOM_UWHALE.to_string(), DENOM_UOM.to_string()];

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
        vec![DEFAULT_DECIMAL_PRECISION, DEFAULT_DECIMAL_PRECISION],
        pool_fees.clone(),
        PoolType::StableSwap { amp: 0u64 },
        None,
        vec![
            coin(REQUIRED_POOL_CREATION_FEE, DENOM_UUSD),
            coin(MOCK_AMOUNT_UOM, DENOM_UOM),
        ],
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
            coin(BALANCE_AMOUNT_LARGE, DENOM_UWHALE),
            coin(BALANCE_AMOUNT_MEDIUM, DENOM_ULUNA),
            coin(BALANCE_AMOUNT_LARGE, DENOM_UUSD),
            coin(BALANCE_AMOUNT_LARGE, DENOM_UOM),
            coin(BALANCE_AMOUNT_LARGE, DENOM_UTEST),
        ],
        StargateMock::new(vec![
            coin(MOCK_AMOUNT_UOM, DENOM_UOM),
            coin(MOCK_AMOUNT_UTEST, DENOM_UTEST),
        ]),
    );
    let creator = suite.creator();

    let asset_denoms = vec![DENOM_UWHALE.to_string(), DENOM_ULUNA.to_string()];

    let pool_fees = PoolFee {
        protocol_fee: Fee {
            share: Decimal::percent(PROTOCOL_FEE_PERCENT),
        },
        swap_fee: Fee {
            share: Decimal::percent(SWAP_FEE_PERCENT),
        },
        burn_fee: Fee {
            share: Decimal::percent(BURN_FEE_PERCENT),
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
            vec![DEFAULT_DECIMAL_PRECISION, DEFAULT_DECIMAL_PRECISION],
            pool_fees.clone(),
            PoolType::ConstantProduct,
            Some(WHALE_LUNA_POOL_IDENTIFIER.to_string()),
            vec![
                coin(REQUIRED_POOL_CREATION_FEE, DENOM_UUSD),
                coin(MOCK_AMOUNT_UOM, DENOM_UOM),
            ],
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();
                match err {
                    ContractError::InvalidTokenFactoryFee {
                        denom,
                        amount,
                        expected,
                    } => {
                        assert_eq!(denom, DENOM_UTEST);
                        assert_eq!(amount, Uint128::zero());
                        assert_eq!(expected, Uint128::new(MOCK_AMOUNT_UTEST));
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
            vec![DEFAULT_DECIMAL_PRECISION, DEFAULT_DECIMAL_PRECISION],
            pool_fees.clone(),
            PoolType::ConstantProduct,
            Some(OTHER_WHALE_LUNA_POOL_IDENTIFIER.to_string()),
            vec![
                coin(REQUIRED_POOL_CREATION_FEE, DENOM_UUSD),
                coin(INSUFFICIENT_TF_FEE, DENOM_UTEST),
                coin(MOCK_AMOUNT_UOM, DENOM_UOM),
            ],
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();

                match err {
                    ContractError::InvalidTokenFactoryFee {
                        denom,
                        amount,
                        expected,
                    } => {
                        assert_eq!(denom, DENOM_UTEST);
                        assert_eq!(amount, Uint128::new(INSUFFICIENT_TF_FEE));
                        assert_eq!(expected, Uint128::new(MOCK_AMOUNT_UTEST));
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
            vec![DEFAULT_DECIMAL_PRECISION, DEFAULT_DECIMAL_PRECISION],
            pool_fees.clone(),
            PoolType::ConstantProduct,
            Some(OTHER_WHALE_LUNA_POOL_IDENTIFIER.to_string()),
            vec![
                coin(REQUIRED_POOL_CREATION_FEE, DENOM_UUSD),
                coin(INSUFFICIENT_TF_FEE, DENOM_UOM),
                coin(MOCK_AMOUNT_UTEST, DENOM_UTEST),
            ],
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();

                match err {
                    ContractError::InvalidTokenFactoryFee {
                        denom,
                        amount,
                        expected,
                    } => {
                        assert_eq!(denom, DENOM_UOM);
                        assert_eq!(amount, Uint128::new(INSUFFICIENT_TF_FEE));
                        assert_eq!(expected, Uint128::new(MOCK_AMOUNT_UOM));
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
            vec![DEFAULT_DECIMAL_PRECISION, DEFAULT_DECIMAL_PRECISION],
            pool_fees.clone(),
            PoolType::ConstantProduct,
            Some(OTHER_WHALE_LUNA_POOL_IDENTIFIER.to_string()),
            vec![
                coin(INSUFFICIENT_POOL_AMOUNT, DENOM_UUSD),
                coin(MOCK_AMOUNT_UTEST, DENOM_UTEST),
                coin(MOCK_AMOUNT_UOM, DENOM_UOM),
            ],
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();

                match err {
                    ContractError::InvalidPoolCreationFee { amount, expected } => {
                        assert_eq!(amount, Uint128::new(INSUFFICIENT_POOL_AMOUNT));
                        assert_eq!(expected, Uint128::new(REQUIRED_POOL_CREATION_FEE));
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
            vec![DEFAULT_DECIMAL_PRECISION, DEFAULT_DECIMAL_PRECISION],
            pool_fees.clone(),
            PoolType::ConstantProduct,
            Some(OTHER_WHALE_LUNA_POOL_IDENTIFIER.to_string()),
            vec![
                coin(REQUIRED_POOL_CREATION_FEE, DENOM_UUSD),
                coin(MOCK_AMOUNT_UOM, DENOM_UOM),
                coin(MOCK_AMOUNT_UTEST, DENOM_UTEST),
                coin(MOCK_AMOUNT_ULUNA, DENOM_ULUNA),
            ],
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();

                match err {
                    ContractError::ExtraFundsSent => {}
                    _ => panic!("Wrong error type, should return ContractError::ExtraFundsSent"),
                }
            },
        )
        .create_pool(
            &creator,
            asset_denoms.clone(),
            vec![DEFAULT_DECIMAL_PRECISION, DEFAULT_DECIMAL_PRECISION],
            pool_fees.clone(),
            PoolType::ConstantProduct,
            Some(OTHER_WHALE_LUNA_POOL_IDENTIFIER.to_string()),
            vec![
                coin(REQUIRED_POOL_CREATION_FEE, DENOM_UUSD),
                coin(MOCK_AMOUNT_UOM, DENOM_UOM),
                coin(MOCK_AMOUNT_UTEST, DENOM_UTEST),
            ],
            |result| {
                result.unwrap();
            },
        );
}

#[test]
fn cant_create_pool_without_paying_tf_fees_same_denom() {
    let mut suite = TestingSuite::default_with_balances(
        vec![
            coin(BALANCE_AMOUNT_LARGE, DENOM_UWHALE),
            coin(BALANCE_AMOUNT_MEDIUM, DENOM_ULUNA),
            coin(BALANCE_AMOUNT_LARGE, DENOM_UUSD),
            coin(BALANCE_AMOUNT_LARGE, DENOM_UOM),
        ],
        StargateMock::new(vec![coin(REQUIRED_POOL_CREATION_FEE, DENOM_UUSD)]),
    );
    let creator = suite.creator();

    let asset_denoms = vec![DENOM_UWHALE.to_string(), DENOM_ULUNA.to_string()];

    let pool_fees = PoolFee {
        protocol_fee: Fee {
            share: Decimal::percent(PROTOCOL_FEE_PERCENT),
        },
        swap_fee: Fee {
            share: Decimal::percent(SWAP_FEE_PERCENT),
        },
        burn_fee: Fee {
            share: Decimal::percent(BURN_FEE_PERCENT),
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
            vec![DEFAULT_DECIMAL_PRECISION, DEFAULT_DECIMAL_PRECISION],
            pool_fees.clone(),
            PoolType::ConstantProduct,
            Some(WHALE_LUNA_POOL_IDENTIFIER.to_string()),
            vec![coin(INSUFFICIENT_POOL_CREATION_AMOUNT, DENOM_UUSD)],
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
            vec![DEFAULT_DECIMAL_PRECISION, DEFAULT_DECIMAL_PRECISION],
            pool_fees.clone(),
            PoolType::ConstantProduct,
            Some(WHALE_LUNA_POOL_IDENTIFIER.to_string()),
            vec![coin(INSUFFICIENT_TWICE_POOL_CREATION_FEE, DENOM_UUSD)],
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();
                match err {
                    ContractError::InvalidPoolCreationFee { amount, expected } => {
                        assert_eq!(amount.u128(), INSUFFICIENT_TWICE_POOL_CREATION_FEE);
                        assert_eq!(expected.u128(), TWICE_POOL_CREATION_FEE);
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
            vec![DEFAULT_DECIMAL_PRECISION, DEFAULT_DECIMAL_PRECISION],
            pool_fees.clone(),
            PoolType::ConstantProduct,
            Some(WHALE_LUNA_POOL_IDENTIFIER.to_string()),
            vec![coin(EXCESSIVE_POOL_CREATION_FEE, DENOM_UUSD)],
            |result| {
                let err = result.unwrap_err().downcast::<ContractError>().unwrap();

                match err {
                    ContractError::InvalidPoolCreationFee { amount, expected } => {
                        assert_eq!(amount.u128(), EXCESSIVE_POOL_CREATION_FEE);
                        assert_eq!(expected.u128(), TWICE_POOL_CREATION_FEE);
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
            vec![DEFAULT_DECIMAL_PRECISION, DEFAULT_DECIMAL_PRECISION],
            pool_fees.clone(),
            PoolType::ConstantProduct,
            Some(WHALE_LUNA_POOL_IDENTIFIER.to_string()),
            vec![coin(TWICE_POOL_CREATION_FEE, DENOM_UUSD)],
            |result| {
                result.unwrap();
            },
        );
}

#[test]
fn attacker_creates_farm_positions_through_pool_manager() {
    let mut suite = TestingSuite::default_with_balances(
        vec![
            coin(ATTACKER_BALANCE_AMOUNT, DENOM_UWHALE),
            coin(ATTACKER_BALANCE_AMOUNT, DENOM_ULUNA),
            coin(SMALL_BALANCE_AMOUNT, DENOM_UUSD),
            coin(SMALL_BALANCE_AMOUNT, DENOM_UOM),
        ],
        StargateMock::new(vec![coin(MOCK_AMOUNT_UOM, DENOM_UOM)]),
    );
    let creator = suite.creator();
    let attacker = suite.senders[1].clone();
    let victim = suite.senders[2].clone();

    let asset_denoms = vec![DENOM_UWHALE.to_string(), DENOM_ULUNA.to_string()];

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
        vec![DEFAULT_DECIMAL_PRECISION, DEFAULT_DECIMAL_PRECISION],
        pool_fees,
        PoolType::ConstantProduct,
        Some(WHALE_LUNA_POOL_RAW_ID.to_string()),
        vec![
            coin(REQUIRED_POOL_CREATION_FEE, DENOM_UUSD),
            coin(MOCK_AMOUNT_UOM, DENOM_UOM),
        ],
        |result| {
            result.unwrap();
        },
    );

    // Let's try to add liquidity
    suite
        .provide_liquidity(
            &creator,
            WHALE_LUNA_POOL_PREFIX.to_string(),
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
                // Ensure we got 999_000 in the response which is 1_000_000 less the initial liquidity amount
                assert!(result.unwrap().events.iter().any(|event| {
                    event.attributes.iter().any(|attr| {
                        attr.key == "added_shares"
                            && attr.value
                                == (Uint128::from(LIQUIDITY_AMOUNT) - MINIMUM_LIQUIDITY_AMOUNT)
                                    .to_string()
                    })
                }));
            },
        )
        .provide_liquidity(
            &attacker,
            WHALE_LUNA_POOL_PREFIX.to_string(),
            Some(UNLOCKING_DURATION),
            Some(SPAM_POSITION_ID.to_string()),
            None,
            None,
            Some(victim.to_string()),
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
                    ContractError::Unauthorized => {}
                    _ => panic!("Wrong error type, should return ContractError::Unauthorized"),
                }
            },
        )
        // try creating a position in the farm with a single-sided lp
        .provide_liquidity(
            &attacker,
            WHALE_LUNA_POOL_PREFIX.to_string(),
            Some(UNLOCKING_DURATION),
            Some(SPAM_POSITION_ID.to_string()),
            None,
            Some(Decimal::percent(SINGLE_SIDED_LP_PERCENT)),
            Some(victim.to_string()),
            vec![Coin {
                denom: DENOM_UWHALE.to_string(),
                amount: Uint128::from(LIQUIDITY_AMOUNT),
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
            WHALE_LUNA_POOL_PREFIX.to_string(),
            Some(UNLOCKING_DURATION),
            Some(LEGIT_POSITION_ID.to_string()),
            None,
            None,
            Some(attacker.to_string()),
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
        );

    suite.query_farm_positions(Some(PositionsBy::Receiver(attacker.to_string())), None, None, None, |result| {
        let positions = result.unwrap().positions;
        // the position should be updated
        assert_eq!(positions.len(), 1);
        assert_eq!(positions[0], Position {
            identifier: "u-legit_position".to_string(),
            lp_asset: Coin { denom: "factory/mantra1zwv6feuzhy6a9wekh96cd57lsarmqlwxdypdsplw6zhfncqw6ftqlydlr9/o.whale.uluna.LP".to_string(), amount: Uint128::new(LIQUIDITY_AMOUNT) },
            unlocking_duration: UNLOCKING_DURATION,
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
            coin(BALANCE_AMOUNT_MEDIUM, DENOM_UUSDY),
            coin(BALANCE_AMOUNT_MEDIUM, DENOM_UUSDC),
            coin(BALANCE_AMOUNT_MEDIUM, DENOM_UTEST),
            coin(BALANCE_AMOUNT_MEDIUM, DENOM_UUSD),
            coin(BALANCE_AMOUNT_MEDIUM, DENOM_UOM),
        ],
        StargateMock::new(vec![coin(MOCK_AMOUNT_UOM, DENOM_UOM)]),
    );
    let creator = suite.creator();

    let pool_fees = PoolFee {
        protocol_fee: Fee {
            share: Decimal::percent(PROTOCOL_FEE_PERCENT),
        },
        swap_fee: Fee {
            share: Decimal::percent(SWAP_FEE_PERCENT),
        },
        burn_fee: Fee {
            share: Decimal::percent(BURN_FEE_PERCENT),
        },
        extra_fees: vec![],
    };

    // Create pools
    suite
        .instantiate_default()
        .add_one_epoch()
        .create_pool(
            &creator,
            vec![DENOM_UOM.to_string(), DENOM_UTEST.to_string()],
            vec![DEFAULT_DECIMAL_PRECISION, DEFAULT_DECIMAL_PRECISION],
            pool_fees.clone(),
            PoolType::ConstantProduct,
            Some(POOL_ID_NUMERIC.to_string()),
            vec![
                coin(REQUIRED_POOL_CREATION_FEE, DENOM_UUSD),
                coin(MOCK_AMOUNT_UOM, DENOM_UOM),
            ],
            |result| {
                result.unwrap();
            },
        )
        .create_pool(
            &creator,
            vec![DENOM_UOM.to_string(), DENOM_UUSDC.to_string()],
            vec![DEFAULT_DECIMAL_PRECISION, DEFAULT_DECIMAL_PRECISION],
            pool_fees.clone(),
            PoolType::ConstantProduct,
            None,
            vec![
                coin(REQUIRED_POOL_CREATION_FEE, DENOM_UUSD),
                coin(MOCK_AMOUNT_UOM, DENOM_UOM),
            ],
            |result| {
                result.unwrap();
            },
        )
        .query_pools(Some(POOL_ID_NUMERIC.to_string()), None, None, |result| {
            let err = result.unwrap_err();
            assert!(err.to_string().contains("not found"));
        })
        .query_pools(None, None, None, |result| {
            let response = result.unwrap();
            assert_eq!(response.pools.len(), 2);
            assert_eq!(
                response.pools[0].pool_info.pool_identifier,
                POOL_ID_PREFIX_O
            );
            assert_eq!(
                response.pools[1].pool_info.pool_identifier,
                POOL_ID_PREFIX_P
            );
        });

    suite.create_pool(
        &creator,
        vec![DENOM_UOM.to_string(), DENOM_UTEST.to_string()],
        vec![DEFAULT_DECIMAL_PRECISION, DEFAULT_DECIMAL_PRECISION],
        pool_fees.clone(),
        PoolType::ConstantProduct,
        Some(POOL_ID_NUMERIC.to_string()),
        vec![
            coin(REQUIRED_POOL_CREATION_FEE, DENOM_UUSD),
            coin(MOCK_AMOUNT_UOM, DENOM_UOM),
        ],
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
            coin(BALANCE_AMOUNT_MEDIUM, DENOM_UUSDY),
            coin(BALANCE_AMOUNT_MEDIUM, DENOM_UUSDC),
            coin(BALANCE_AMOUNT_MEDIUM, DENOM_UUSDT),
            coin(BALANCE_AMOUNT_MEDIUM, DENOM_UUSD),
            coin(BALANCE_AMOUNT_MEDIUM, DENOM_UOM),
        ],
        StargateMock::new(vec![coin(MOCK_AMOUNT_UOM, DENOM_UOM)]),
    );
    let creator = suite.creator();

    // Asset denoms with uwhale and uluna
    let asset_denoms = vec![
        DENOM_UUSDY.to_string(),
        DENOM_UUSDC.to_string(),
        DENOM_UUSDT.to_string(),
        DENOM_UUSD.to_string(),
    ];

    let pool_fees = PoolFee {
        protocol_fee: Fee {
            share: Decimal::percent(PROTOCOL_FEE_PERCENT),
        },
        swap_fee: Fee {
            share: Decimal::percent(SWAP_FEE_PERCENT),
        },
        burn_fee: Fee {
            share: Decimal::percent(BURN_FEE_PERCENT),
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
            vec![DEFAULT_DECIMAL_PRECISION, DEFAULT_DECIMAL_PRECISION],
            pool_fees.clone(),
            PoolType::StableSwap {
                amp: STABLESWAP_TEST_AMP_FACTOR,
            },
            Some(STABLESWAP_POOL_ID.to_string()),
            vec![
                coin(REQUIRED_POOL_CREATION_FEE, DENOM_UUSD),
                coin(MOCK_AMOUNT_UOM, DENOM_UOM),
            ],
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
            vec![
                DEFAULT_DECIMAL_PRECISION,
                DEFAULT_DECIMAL_PRECISION,
                DEFAULT_DECIMAL_PRECISION,
            ],
            pool_fees.clone(),
            PoolType::StableSwap {
                amp: STABLESWAP_TEST_AMP_FACTOR,
            },
            Some(STABLESWAP_POOL_ID.to_string()),
            vec![
                coin(REQUIRED_POOL_CREATION_FEE, DENOM_UUSD),
                coin(MOCK_AMOUNT_UOM, DENOM_UOM),
            ],
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
                DENOM_UUSDY.to_string(),
                DENOM_UUSDC.to_string(),
                DENOM_UUSDT.to_string(),
                DENOM_UUSD.to_string(),
                DENOM_UOM.to_string(),
            ],
            vec![
                DEFAULT_DECIMAL_PRECISION,
                DEFAULT_DECIMAL_PRECISION,
                DEFAULT_DECIMAL_PRECISION,
                DEFAULT_DECIMAL_PRECISION,
                DEFAULT_DECIMAL_PRECISION,
            ],
            pool_fees.clone(),
            PoolType::StableSwap {
                amp: STABLESWAP_TEST_AMP_FACTOR,
            },
            Some(STABLESWAP_POOL_ID.to_string()),
            vec![
                coin(REQUIRED_POOL_CREATION_FEE, DENOM_UUSD),
                coin(MOCK_AMOUNT_UOM, DENOM_UOM),
            ],
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
                DENOM_UUSDY.to_string(),
                DENOM_UUSDC.to_string(),
                DENOM_UUSDT.to_string(),
                DENOM_UUSD.to_string(),
            ],
            vec![
                DEFAULT_DECIMAL_PRECISION,
                DEFAULT_DECIMAL_PRECISION,
                DEFAULT_DECIMAL_PRECISION,
                DEFAULT_DECIMAL_PRECISION,
            ],
            pool_fees.clone(),
            PoolType::StableSwap {
                amp: STABLESWAP_TEST_AMP_FACTOR,
            },
            Some(STABLESWAP_POOL_ID.to_string()),
            vec![
                coin(REQUIRED_POOL_CREATION_FEE, DENOM_UUSD),
                coin(MOCK_AMOUNT_UOM, DENOM_UOM),
            ],
            |result| {
                result.unwrap();
            },
        );
}

#[test]
fn providing_custom_pool_id_doesnt_increment_pool_counter() {
    let mut suite = TestingSuite::default_with_balances(
        vec![
            coin(BALANCE_AMOUNT_MEDIUM, DENOM_UWHALE),
            coin(BALANCE_AMOUNT_MEDIUM, DENOM_ULUNA),
            coin(BALANCE_AMOUNT_MEDIUM, DENOM_UOSMO),
            coin(BALANCE_AMOUNT_MEDIUM, DENOM_UUSD),
            coin(BALANCE_AMOUNT_MEDIUM, DENOM_UOM),
        ],
        StargateMock::new(vec![coin(MOCK_AMOUNT_UOM, DENOM_UOM)]),
    );
    let creator = suite.creator();

    let asset_denoms = vec![DENOM_UOM.to_string(), DENOM_ULUNA.to_string()];

    let pool_fees = PoolFee {
        protocol_fee: Fee {
            share: Decimal::percent(PROTOCOL_FEE_PERCENT),
        },
        swap_fee: Fee {
            share: Decimal::percent(SWAP_FEE_PERCENT),
        },
        burn_fee: Fee {
            share: Decimal::percent(BURN_FEE_PERCENT),
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
            vec![DEFAULT_DECIMAL_PRECISION, DEFAULT_DECIMAL_PRECISION],
            pool_fees.clone(),
            PoolType::ConstantProduct,
            Some(CUSTOM_POOL_ID_1.to_string()),
            vec![
                coin(REQUIRED_POOL_CREATION_FEE, DENOM_UUSD),
                coin(MOCK_AMOUNT_UOM, DENOM_UOM),
            ],
            |result| {
                result.unwrap();
            },
        )
        .create_pool(
            &creator,
            asset_denoms.clone(),
            vec![DEFAULT_DECIMAL_PRECISION, DEFAULT_DECIMAL_PRECISION],
            pool_fees.clone(),
            PoolType::ConstantProduct,
            Some(CUSTOM_POOL_ID_2.to_string()),
            vec![
                coin(REQUIRED_POOL_CREATION_FEE, DENOM_UUSD),
                coin(MOCK_AMOUNT_UOM, DENOM_UOM),
            ],
            |result| {
                result.unwrap();
            },
        )
        .create_pool(
            &creator,
            asset_denoms,
            vec![DEFAULT_DECIMAL_PRECISION, DEFAULT_DECIMAL_PRECISION],
            pool_fees,
            PoolType::ConstantProduct,
            None,
            vec![
                coin(REQUIRED_POOL_CREATION_FEE, DENOM_UUSD),
                coin(MOCK_AMOUNT_UOM, DENOM_UOM),
            ],
            |result| {
                result.unwrap();
            },
        )
        .query_pools(None, None, None, |result| {
            let response = result.unwrap();
            assert_eq!(response.pools.len(), 3);
            assert_eq!(
                response.pools[0].pool_info.pool_identifier,
                CUSTOM_POOL_PREFIX_1
            );
            assert_eq!(
                response.pools[1].pool_info.pool_identifier,
                CUSTOM_POOL_PREFIX_2
            );
            assert_eq!(
                response.pools[2].pool_info.pool_identifier,
                POOL_ID_PREFIX_P
            );
        });
}

#[test]
fn lock_single_pool() {
    let mut suite = TestingSuite::default_with_balances(
        vec![
            coin(LOCK_POOL_BALANCE_AMOUNT, DENOM_UOM),
            coin(LOCK_POOL_BALANCE_AMOUNT, DENOM_UUSD),
        ],
        StargateMock::new(vec![coin(LOCK_POOL_TF_FEE, DENOM_UOM)]),
    );
    let creator = suite.creator();

    let asset_denoms = vec![DENOM_UOM.to_string(), DENOM_UUSD.to_string()];

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
            vec![DEFAULT_DECIMAL_PRECISION, DEFAULT_DECIMAL_PRECISION],
            pool_fees.clone(),
            PoolType::ConstantProduct,
            Some(LOCK_POOL_ID_1.to_string()),
            vec![
                coin(LOCK_POOL_TF_FEE, DENOM_UUSD),
                coin(LOCK_POOL_TF_FEE, DENOM_UOM),
            ],
            |result| {
                result.unwrap();
            },
        )
        .create_pool(
            &creator,
            asset_denoms,
            vec![DEFAULT_DECIMAL_PRECISION, DEFAULT_DECIMAL_PRECISION],
            pool_fees,
            PoolType::ConstantProduct,
            Some(LOCK_POOL_ID_2.to_string()),
            vec![
                coin(LOCK_POOL_TF_FEE, DENOM_UUSD),
                coin(LOCK_POOL_TF_FEE, DENOM_UOM),
            ],
            |result| {
                result.unwrap();
            },
        );

    suite.provide_liquidity(
        &creator,
        LOCK_POOL_PREFIX_1.to_string(),
        None,
        None,
        None,
        None,
        None,
        vec![
            Coin {
                denom: DENOM_UOM.to_string(),
                amount: Uint128::from(LOCK_POOL_LIQUIDITY_AMOUNT),
            },
            Coin {
                denom: DENOM_UUSD.to_string(),
                amount: Uint128::from(LOCK_POOL_LIQUIDITY_AMOUNT_2),
            },
        ],
        |result| {
            result.unwrap();
        },
    );

    let lp_denom_1 = suite.get_lp_denom(LOCK_POOL_PREFIX_1.to_string());
    let lp_denom_2 = suite.get_lp_denom(LOCK_POOL_PREFIX_2.to_string());

    suite.update_config(
        &creator,
        None,
        None,
        None,
        Some(FeatureToggle {
            pool_identifier: LOCK_POOL_PREFIX_1.to_string(),
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
            LOCK_POOL_PREFIX_1.to_string(),
            None,
            None,
            None,
            None,
            None,
            vec![
                Coin {
                    denom: DENOM_UOM.to_string(),
                    amount: Uint128::from(LOCK_POOL_LIQUIDITY_AMOUNT),
                },
                Coin {
                    denom: DENOM_UUSD.to_string(),
                    amount: Uint128::from(LOCK_POOL_LIQUIDITY_AMOUNT_2),
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
            DENOM_UOM.to_string(),
            None,
            None,
            None,
            LOCK_POOL_PREFIX_1.to_string(),
            vec![coin(LOCK_POOL_TF_FEE, DENOM_UUSD)],
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
            LOCK_POOL_PREFIX_1.to_string(),
            vec![Coin {
                denom: lp_denom_1.clone(),
                amount: Uint128::from(LOCK_POOL_SWAP_AMOUNT),
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
            LOCK_POOL_PREFIX_2.to_string(),
            None,
            None,
            None,
            None,
            None,
            vec![
                Coin {
                    denom: DENOM_UOM.to_string(),
                    amount: Uint128::from(LOCK_POOL_LIQUIDITY_AMOUNT),
                },
                Coin {
                    denom: DENOM_UUSD.to_string(),
                    amount: Uint128::from(LOCK_POOL_LIQUIDITY_AMOUNT_2),
                },
            ],
            |result| {
                result.unwrap();
            },
        )
        .swap(
            &creator,
            DENOM_UOM.to_string(),
            None,
            None,
            None,
            LOCK_POOL_PREFIX_2.to_string(),
            vec![coin(LOCK_POOL_SWAP_AMOUNT, DENOM_UUSD)],
            |result| {
                result.unwrap();
            },
        )
        .withdraw_liquidity(
            &creator,
            LOCK_POOL_PREFIX_2.to_string(),
            vec![Coin {
                denom: lp_denom_2.clone(),
                amount: Uint128::from(LOCK_POOL_SWAP_AMOUNT),
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
            pool_identifier: LOCK_POOL_PREFIX_1.to_string(),
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
            LOCK_POOL_PREFIX_1.to_string(),
            None,
            None,
            None,
            None,
            None,
            vec![
                Coin {
                    denom: DENOM_UOM.to_string(),
                    amount: Uint128::from(LOCK_POOL_LIQUIDITY_AMOUNT),
                },
                Coin {
                    denom: DENOM_UUSD.to_string(),
                    amount: Uint128::from(LOCK_POOL_LIQUIDITY_AMOUNT_2),
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
            DENOM_UOM.to_string(),
            None,
            None,
            None,
            LOCK_POOL_PREFIX_1.to_string(),
            vec![coin(LOCK_POOL_SWAP_AMOUNT_2, DENOM_UUSD)],
            |result| {
                // should work, as it was enabled
                result.unwrap();
            },
        )
        .withdraw_liquidity(
            &creator,
            LOCK_POOL_PREFIX_1.to_string(),
            vec![Coin {
                denom: lp_denom_1.clone(),
                amount: Uint128::from(LOCK_POOL_SWAP_AMOUNT),
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
            coin(TOGGLE_POOL_BALANCE_AMOUNT, DENOM_UOM),
            coin(TOGGLE_POOL_BALANCE_AMOUNT, DENOM_UUSD),
        ],
        StargateMock::new(vec![coin(TOGGLE_POOL_TF_FEE, DENOM_UOM)]),
    );
    let creator = suite.creator();

    let asset_denoms = vec![DENOM_UOM.to_string(), DENOM_UUSD.to_string()];

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
        vec![DEFAULT_DECIMAL_PRECISION, DEFAULT_DECIMAL_PRECISION],
        pool_fees.clone(),
        PoolType::ConstantProduct,
        Some(TOGGLE_POOL_ID.to_string()),
        vec![
            coin(TOGGLE_POOL_TF_FEE, DENOM_UUSD),
            coin(TOGGLE_POOL_TF_FEE, DENOM_UOM),
        ],
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
            pool_identifier: TOGGLE_INVALID_POOL_ID.to_string(),
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
