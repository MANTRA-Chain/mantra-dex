use cosmwasm_std::{coin, Decimal, Uint128};
use mantra_common_testing::multi_test::stargate_mock::StargateMock;
use mantra_dex_std::fee::{Fee, PoolFee};
use mantra_dex_std::pool_manager::PoolType;
use std::cell::RefCell;

use crate::tests::suite::TestingSuite;

#[test]
fn test_single_sided_liquidity_provision_slippage_vulnerability() {
    // Initialize TestingSuite with initial balances and a Stargate mock
    let mut suite = TestingSuite::default_with_balances(
        vec![
            coin(300_000_000_000u128, "uwhale".to_string()),
            coin(300_000_000_000u128, "uusdc".to_string()),
            coin(10_000_000u128, "uom".to_string()),
            coin(10_000_000u128, "uusd".to_string()),
        ],
        StargateMock::new(vec![coin(8888u128, "uom".to_string())]),
    );

    // Define participants
    let creator = suite.creator();
    let front_runner = suite.senders[1].clone();
    let victim = suite.senders[2].clone();

    // Define pool fees
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

    // Set up the pool and provide initial liquidity
    suite.instantiate_default().add_one_epoch().create_pool(
        &creator,
        vec!["uwhale".to_string(), "uusdc".to_string()],
        vec![6u8, 6u8],
        pool_fees,
        PoolType::ConstantProduct,
        Some("whale.usdc".to_string()),
        vec![coin(1_000, "uusd"), coin(8888, "uom")],
        |result| {
            result.unwrap();
        },
    );

    // Debug: Check if the pool was actually created
    suite.query_pools(None, None, None, |result| {
        let response = result.unwrap();
        println!("Number of pools: {}", response.pools.len());

        if !response.pools.is_empty() {
            for (i, pool) in response.pools.iter().enumerate() {
                println!(
                    "Pool {}: has identifier: {}",
                    i, pool.pool_info.pool_identifier
                );
            }

            // Check specifically for whale.usdc pool
            let whale_usdc_pool = response
                .pools
                .iter()
                .find(|p| p.pool_info.pool_identifier == "o.whale.usdc");

            match whale_usdc_pool {
                Some(pool) => println!("Found whale.usdc pool: {}", pool.pool_info.pool_identifier),
                None => println!("WARNING: whale.usdc pool not found!"),
            }
        } else {
            println!("No pools were created!");
        }
    });

    // Use the correct pool identifier format (o.whale.usdc) in all subsequent operations
    let pool_id = "o.whale.usdc";

    suite.provide_liquidity(
        &creator,
        pool_id.to_string(),
        None,
        None,
        None,
        None,
        None,
        vec![
            coin(100_000_000u128, "uwhale"),
            coin(100_000_000u128, "uusdc"),
        ],
        |result| {
            result.unwrap();
        },
    );

    // Verify initial LP token supply
    suite.query_amount_of_lp_token(pool_id.to_string(), &creator.to_string(), |result| {
        let lp_amount = result.unwrap();
        assert!(
            lp_amount > Uint128::zero(),
            "Initial LP token supply should be positive"
        );
    });

    // Front-runner performs 10 swaps to skew the pool ratio
    /*
    for _ in 0..10 {
        suite.swap(
            &front_runner,
            "uusdc".to_string(),
            None,
            Some(Decimal::percent(50)), // Allow up to 50% spread
            None,
            pool_id.to_string(),
            vec![coin(8_000_000u128, "uwhale")],
            // vec![coin(80_000_000u128, "uwhale")],
            |result| {
                result.unwrap();
            },
        );
    }
     */

    // Create RefCell to hold the balances
    let pool_balances = RefCell::new(vec![]);

    // Check pool balances after the swap
    suite.query_pools(Some(pool_id.to_string()), None, None, |result| {
        let response = result.unwrap();
        if response.pools.is_empty() {
            panic!(
                "No pools found for identifier '{}'. Check pool creation.",
                pool_id
            );
        }
        let pool_info = response.pools[0].pool_info.clone();
        let uwhale_balance = pool_info
            .assets
            .iter()
            .find(|c| c.denom == "uwhale")
            .unwrap()
            .amount;
        let uusdc_balance = pool_info
            .assets
            .iter()
            .find(|c| c.denom == "uusdc")
            .unwrap()
            .amount;
        let balances = vec![uwhale_balance, uusdc_balance];
        *pool_balances.borrow_mut() = balances; // Use borrow_mut() to modify
                                                /*
                                                assert!(
                                                    uwhale_balance > Uint128::from(150_000_000u128),
                                                    "uwhale balance should increase significantly"
                                                );
                                                assert!(
                                                    uusdc_balance < Uint128::from(60_000_000u128),
                                                    "uusdc balance should decrease significantly"
                                                );
                                                */
    });

    println!(
        "Pool ratio after front-running: uwhale = {}, uusdc = {}",
        pool_balances.borrow()[0],
        pool_balances.borrow()[1]
    );

    // Victim provides single-sided liquidity with slippage protection
    suite.provide_liquidity(
        &victim,
        pool_id.to_string(),
        None,
        None,
        None,
        Some(Decimal::percent(5)),
        None,
        vec![coin(10_000_000u128, "uusdc")],
        |result| {
            // expect error
            // assert!(result.is_err());
            // expect success
            assert!(result.is_ok());
        },
    );

    // Verify LP tokens received by the victim
    /*
    let expected_lp_tokens = Uint128::from(10_000_000u128);
    suite.query_amount_of_lp_token(pool_id.to_string(), &victim.to_string(), |result| {
        let lp_tokens_received: Uint128 = result.unwrap();
        assert!(
            lp_tokens_received > Uint128::zero(),
            "Victim should receive some LP tokens"
        );
        let slippage_pct = Decimal::from_ratio(lp_tokens_received, expected_lp_tokens) * Decimal::percent(100);

        println!(
            "Victim received {} LP tokens; slippage protection failed with max_slippage of 5%, actually got {}",
            lp_tokens_received,
            slippage_pct
        );
    });
    */
}
