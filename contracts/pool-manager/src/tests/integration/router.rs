use crate::tests::suite::TestingSuite;
use crate::ContractError;
use cosmwasm_std::coin;
use cosmwasm_std::Coin;
use cosmwasm_std::Decimal;
use cosmwasm_std::Uint128;
use cosmwasm_std::{assert_approx_eq, Event, StdError};
use mantra_common_testing::multi_test::stargate_mock::StargateMock;
use mantra_dex_std::fee::Fee;
use mantra_dex_std::fee::PoolFee;
use mantra_dex_std::pool_manager::PoolType;

#[test]
fn basic_swap_operations_test() {
    let mut suite = TestingSuite::default_with_balances(
        vec![
            coin(1_000_000_000u128, "uwhale".to_string()),
            coin(1_000_000_000u128, "uluna".to_string()),
            coin(1_000_000_000u128, "uusd".to_string()),
            coin(1_000_000_000u128, "uom".to_string()),
        ],
        StargateMock::new(vec![coin(8888u128, "uom".to_string())]),
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
            assert!(result.has_event(&Event::new("wasm").add_attribute("added_shares", "999000")));
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
            assert!(result.has_event(&Event::new("wasm").add_attribute("added_shares", "999000")));
        },
    );

    // Prepare the swap operations, we want to go from WHALE -> UUSD
    // We will use the o.uluna.uusd pool as the intermediary pool

    let swap_operations = vec![
        mantra_dex_std::pool_manager::SwapOperation::MantraSwap {
            token_in_denom: "uwhale".to_string(),
            token_out_denom: "uluna".to_string(),
            pool_identifier: "o.whale.uluna".to_string(),
        },
        mantra_dex_std::pool_manager::SwapOperation::MantraSwap {
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
        Some(Decimal::percent(2)),
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
        StargateMock::new(vec![coin(8888u128, "uom".to_string())]),
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
            assert!(result.has_event(&Event::new("wasm").add_attribute("added_shares", "999000")));
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
            assert!(result.has_event(&Event::new("wasm").add_attribute("added_shares", "999000")));
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
        StargateMock::new(vec![coin(8888u128, "uom".to_string())]),
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
            assert!(result.has_event(&Event::new("wasm").add_attribute("added_shares", "999000")));
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
            assert!(result.has_event(&Event::new("wasm").add_attribute("added_shares", "999000")));
        },
    );

    // Prepare the swap operations, we want to go from WHALE -> UUSD
    // We will use the o.uluna.uusd pool as the intermediary pool

    let swap_operations = vec![
        mantra_dex_std::pool_manager::SwapOperation::MantraSwap {
            token_in_denom: "uwhale".to_string(),
            token_out_denom: "uluna".to_string(),
            pool_identifier: "o.whale.uluna".to_string(),
        },
        mantra_dex_std::pool_manager::SwapOperation::MantraSwap {
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
        StargateMock::new(vec![coin(8888u128, "uom".to_string())]),
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
            assert!(result.has_event(&Event::new("wasm").add_attribute("added_shares", "999000")));
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
            assert!(result.has_event(&Event::new("wasm").add_attribute("added_shares", "999000")));
        },
    );

    // Prepare the swap operations, we want to go from WHALE -> UUSD
    // We will use the o.uluna.uusd pool as the intermediary pool

    let swap_operations = vec![
        mantra_dex_std::pool_manager::SwapOperation::MantraSwap {
            token_in_denom: "uwhale".to_string(),
            token_out_denom: "uluna".to_string(),
            pool_identifier: "o.whale.uluna".to_string(),
        },
        mantra_dex_std::pool_manager::SwapOperation::MantraSwap {
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
        StargateMock::new(vec![coin(8888u128, "uom".to_string())]),
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
            assert!(result.has_event(&Event::new("wasm").add_attribute("added_shares", "999000")));
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
            assert!(result.has_event(&Event::new("wasm").add_attribute("added_shares", "999000")));
        },
    );

    // Prepare the swap operations, we want to go from WHALE -> UUSD
    // We will use the o.uluna.uusd pool as the intermediary pool

    let swap_operations = vec![
        mantra_dex_std::pool_manager::SwapOperation::MantraSwap {
            token_in_denom: "uwhale".to_string(),
            token_out_denom: "uluna".to_string(),
            pool_identifier: "o.whale.uluna".to_string(),
        },
        mantra_dex_std::pool_manager::SwapOperation::MantraSwap {
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
        Some(Decimal::percent(2)),
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
        StargateMock::new(vec![coin(8888u128, "uom".to_string())]),
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
            assert!(result.has_event(&Event::new("wasm").add_attribute("added_shares", "999000")));
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
            assert!(result.has_event(&Event::new("wasm").add_attribute("added_shares", "999000")));
        },
    );

    // Prepare the swap operations, we want to go from WHALE -> UUSD
    // We will use the o.uluna.uusd pool as the intermediary pool

    let swap_operations = vec![
        mantra_dex_std::pool_manager::SwapOperation::MantraSwap {
            token_in_denom: "uwhale".to_string(),
            token_out_denom: "uluna".to_string(),
            pool_identifier: "o.whale.uluna".to_string(),
        },
        mantra_dex_std::pool_manager::SwapOperation::MantraSwap {
            token_in_denom: "uluna".to_string(),
            token_out_denom: "uusd".to_string(),
            pool_identifier: "o.uluna.uusd".to_string(),
        },
    ];

    // simulating (reverse) swap operations should return the correct same amount as the pools are balanced
    // going from whale -> uusd should return 974 uusd
    // going in reverse, 974 uusd -> whale should require approximately 1000 whale
    suite.query_simulate_swap_operations(Uint128::new(1_000), swap_operations.clone(), |result| {
        let result = result.unwrap();
        assert_eq!(result.return_amount.u128(), 974);
    });
    suite.query_reverse_simulate_swap_operations(
        Uint128::new(974),
        swap_operations.clone(),
        |result| {
            let result = result.unwrap();
            assert_approx_eq!(result.offer_amount.u128(), 1000, "0.006");
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
            assert_approx_eq!(result.offer_amount.u128(), 1_007, "0.1");
        },
    );

    // and if simulate swap operations with 1_000 more whale we should get even less uusd than before
    suite.query_simulate_swap_operations(Uint128::new(1_000), swap_operations.clone(), |result| {
        let result = result.unwrap();
        assert_eq!(result.return_amount.u128(), 935);
    });
}
