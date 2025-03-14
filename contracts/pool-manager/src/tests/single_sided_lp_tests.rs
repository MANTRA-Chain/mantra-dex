use cosmwasm_std::{coin, Decimal, Decimal256, Uint128, Uint256};
use mantra_common_testing::multi_test::stargate_mock::StargateMock;
use mantra_dex_std::fee::{Fee, PoolFee};
use mantra_dex_std::lp_common::MINIMUM_LIQUIDITY_AMOUNT;
use mantra_dex_std::pool_manager::PoolType;
use mantra_dex_std::U256;
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

    // Use the correct pool identifier format (o.whale.usdc) in all subsequent operations
    let pool_id = "o.whale.usdc";
    let uwhale_initial_amount = 20_000_000u128;
    let uusdc_initial_amount = 5_000_000_000u128;

    suite.provide_liquidity(
        &creator,
        pool_id.to_string(),
        None,
        None,
        None,
        None,
        None,
        vec![
            coin(uwhale_initial_amount, "uwhale"),
            coin(uusdc_initial_amount, "uusdc"),
        ],
        |result| {
            result.unwrap();
        },
    );

    // Sqrt(asset_1_amount * asset_2_amount) - MIN_LIQUIDITY_AMOUNT
    let expected_lp_amount = (U256::from(uwhale_initial_amount)
        .checked_mul(U256::from(uusdc_initial_amount))
        .unwrap()
        .integer_sqrt()
        .as_u128())
        - MINIMUM_LIQUIDITY_AMOUNT.u128();
    
    // Verify initial LP token supply
    suite.query_amount_of_lp_token(pool_id.to_string(), &creator.to_string(), |result| {
        let lp_amount = result.unwrap();
        assert_eq!(lp_amount.u128(), expected_lp_amount);
    });

    // Check pool balances before the swap
    let pool_balances_before = RefCell::new(vec![]);
    suite.query_pools(Some(pool_id.to_string()), None, None, |result| {
        let response = result.unwrap();
        let pool_info = response.pools[0].pool_info.clone();
        let uwhale_balance = pool_info.assets[0].amount;
        let uusdc_balance = pool_info.assets[1].amount;
        *pool_balances_before.borrow_mut() = vec![uwhale_balance, uusdc_balance];
    });

    // Front-runner performs 10 swaps to skew the pool ratio
    for _ in 0..5 {
        suite.swap(
            &front_runner,
            "uwhale".to_string(),
            None,
            Some(Decimal::percent(50)), // Allow up to 50% spread
            None,
            pool_id.to_string(),
            vec![coin(1_000_000_000u128, "uusdc")],
            |result| {
                result.unwrap();
            },
        );
    }

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
        let uwhale_balance = pool_info.assets[0].amount;
        let uusdc_balance = pool_info.assets[1].amount;
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


    println!("Pool ratio before front-running: uwhale = {}, uusdc = {}", pool_balances_before.borrow()[0], pool_balances_before.borrow()[1]);
    println!(
        "Pool ratio after front-running: uwhale = {}, uusdc = {}",
        pool_balances.borrow()[0],
        pool_balances.borrow()[1]
    );

    let one_e6 = Uint256::from(1_000_000u128);

    let whale_amount_before = Decimal256::from_ratio(pool_balances_before.borrow()[0], one_e6);
    let whale_amount_after = Decimal256::from_ratio(pool_balances.borrow()[0], one_e6);
    let usdc_amount_before = Decimal256::from_ratio(pool_balances_before.borrow()[1], one_e6);
    let usdc_amount_after = Decimal256::from_ratio(pool_balances.borrow()[1], one_e6);
    println!("Whale amount before: {}", whale_amount_before);
    println!("Whale amount after: {}", whale_amount_after);
    println!("Usdc amount before: {}", usdc_amount_before);
    println!("Usdc amount after: {}", usdc_amount_after);
    let whale_deviation = whale_amount_before.abs_diff(whale_amount_after);
    let usdc_deviation = usdc_amount_before.abs_diff(usdc_amount_after);
    let one_hundred = Decimal256::from_ratio(Uint256::from(100u128), Uint256::one());
    let whale_deviation_pct = whale_deviation.checked_div(whale_amount_before).unwrap().checked_mul(one_hundred).unwrap();
    let usdc_deviation_pct = usdc_deviation.checked_div(usdc_amount_before).unwrap().checked_mul(one_hundred).unwrap();

    println!("Whale deviation percentage: {}%", whale_deviation_pct);
    println!("Usdc deviation percentage: {}%", usdc_deviation_pct);

    // Victim provides single-sided liquidity with slippage protection
    println!("Providing liquidity with slippage protection, amount: {}{}", 10_000_000_000u128, "uusdc");
    suite.provide_liquidity(
        &victim,
        pool_id.to_string(),
        None,
        None,
        Some(Decimal::percent(20)),
        None,
        None,
        vec![coin(10_000_000_000u128, "uusdc")],
        |result| {
            // expect error
            assert!(result.is_err());
            let error_msg_str = format!("{:?}", result.err().unwrap());
            assert!(error_msg_str.contains("Spread limit exceeded"));
            println!("Error message: {}", error_msg_str);
        },
    );

    // Victim provides single-sided liquidity without slippage protection
    println!("Providing liquidity without slippage protection, amount: {}{}", 10_000_000_000u128, "uusdc");
    suite.provide_liquidity(
        &victim,
        pool_id.to_string(),
        None,
        None,
        None,
        None,
        None,
        vec![coin(10_000_000_000u128, "uusdc")],
        |result| {
            // expect error
            assert!(result.is_err());
            let error_msg_str = format!("{:?}", result.err().unwrap());
            assert!(error_msg_str.contains("Spread limit exceeded"));
        },
    );
}
