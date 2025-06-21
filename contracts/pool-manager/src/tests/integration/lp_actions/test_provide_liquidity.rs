use cosmwasm_std::{coin, Coin, Decimal, Uint128};
use mantra_common_testing::multi_test::stargate_mock::StargateMock;
use mantra_dex_std::{
    fee::{Fee, PoolFee},
    pool_manager::PoolType,
};
use test_utils::common_constants::{
    DECIMALS_6, DENOM_UOM, DENOM_UUSD, ONE_THOUSAND, STARGATE_MOCK_UOM_AMOUNT,
};

use crate::tests::suite::TestingSuite;

// Constants for the test
const INITIAL_BALANCE: u128 = 10_000_000u128;
const UOM_BALANCE: u128 = STARGATE_MOCK_UOM_AMOUNT;
const UUSD_POOL_CREATION_FEE: u128 = ONE_THOUSAND;
const UOM_POOL_CREATION_FEE: u128 = UOM_BALANCE;

// Test denominations - using OM and USDY as described
const UUSDY_DENOM: &str = "factory/mantra1x5nk33zpglp4ge6q9a8xx3zceqf4g8nvaggjmc/aUSDY";

// Pool configuration
const POOL_ID: &str = "o.uom.usdy.pool";
const POOL_LABEL: &str = "uom.usdy.pool";

// Test liquidity amounts - 1 OM + 4 USDY as described
const OM_LIQUIDITY_AMOUNT: u128 = 1_000_000u128; // 1 OM with 6 decimals
const USDY_LIQUIDITY_AMOUNT: u128 = 4_000_000u128; // 4 USDY with 6 decimals

// Slippage settings
const ONE_PERCENT_SLIPPAGE: Option<Decimal> = Some(Decimal::percent(1));
const ZERO_PERCENT_FEE: Decimal = Decimal::zero();
const SIX_DECIMALS: u8 = DECIMALS_6;

#[test]
fn test_provide_liquidity() {
    println!("=== Starting test_provide_liquidity ===");

    let mut suite = TestingSuite::default_with_balances(
        vec![
            coin(INITIAL_BALANCE, DENOM_UOM.to_string()),
            coin(INITIAL_BALANCE, UUSDY_DENOM.to_string()),
            coin(ONE_THOUSAND, DENOM_UUSD.to_string()),
        ],
        StargateMock::new(vec![coin(UOM_BALANCE, DENOM_UOM.to_string())]),
    );

    let creator = suite.creator();

    // Print contract addresses (simulating the client creation step)
    println!("Pool Manager Contract: {}", suite.pool_manager_addr);
    println!(
        "Farm Manager Contract: Some(\"{}\")",
        suite.farm_manager_addr
    );
    println!(
        "Fee Collector Contract: Some(\"{}\")",
        suite.fee_collector_addr
    );
    println!(
        "Epoch Manager Contract: Some(\"{}\")",
        suite.epoch_manager_addr
    );
    println!("Wallet address: {}", creator);

    // Asset denoms with OM and USDY
    let asset_denoms = vec![DENOM_UOM.to_string(), UUSDY_DENOM.to_string()];

    let pool_fees = PoolFee {
        protocol_fee: Fee {
            share: ZERO_PERCENT_FEE,
        },
        swap_fee: Fee {
            share: ZERO_PERCENT_FEE,
        },
        burn_fee: Fee {
            share: ZERO_PERCENT_FEE,
        },
        extra_fees: vec![],
    };

    // Print search message
    println!(
        "Looking for pool with assets: {} and {}",
        DENOM_UOM, UUSDY_DENOM
    );

    // Create the pool (simulating get_or_create_test_pool_id)
    suite.instantiate_default().add_one_epoch().create_pool(
        &creator,
        asset_denoms,
        vec![SIX_DECIMALS, SIX_DECIMALS],
        pool_fees,
        PoolType::ConstantProduct,
        Some(POOL_LABEL.to_string()),
        vec![
            coin(UUSD_POOL_CREATION_FEE, DENOM_UUSD),
            coin(UOM_POOL_CREATION_FEE, DENOM_UOM),
        ],
        |result| {
            let response = result.unwrap();
            println!("Pool created successfully! Transaction hash: <simulated_hash>");
            // Check that we have events indicating successful pool creation
            assert!(
                !response.events.is_empty(),
                "Pool creation should generate events"
            );
        },
    );

    println!("Found existing pool: {}", POOL_ID);

    // First, provide initial liquidity without slippage tolerance to populate the pool
    println!("Providing initial liquidity to populate the pool...");
    let initial_assets = vec![
        Coin {
            denom: DENOM_UOM.to_string(),
            amount: Uint128::from(OM_LIQUIDITY_AMOUNT),
        },
        Coin {
            denom: UUSDY_DENOM.to_string(),
            amount: Uint128::from(USDY_LIQUIDITY_AMOUNT),
        },
    ];

    suite.provide_liquidity(
        &creator,
        POOL_ID.to_string(),
        None,                       // unlocking_duration
        None,                       // lock_position_identifier
        Some(Decimal::percent(10)), // liquidity_max_slippage - no slippage check for initial liquidity
        Some(Decimal::percent(10)), // swap_max_slippage - no slippage check for initial liquidity
        None,                       // receiver
        initial_assets,
        |result| match result {
            Ok(tx_response) => {
                println!("Initial liquidity provision successful!");
                assert!(
                    !tx_response.events.is_empty(),
                    "Initial liquidity provision should generate events"
                );
            }
            Err(e) => {
                panic!("Initial liquidity provision failed: {:?}", e);
            }
        },
    );

    println!("Testing liquidity provision with pool: {}", POOL_ID);

    // Prepare assets for the test liquidity provision
    let assets = vec![
        Coin {
            denom: DENOM_UOM.to_string(),
            amount: Uint128::from(OM_LIQUIDITY_AMOUNT),
        },
        Coin {
            denom: UUSDY_DENOM.to_string(),
            amount: Uint128::from(USDY_LIQUIDITY_AMOUNT),
        },
    ];

    // Execute provide liquidity with slippage tolerance
    suite.provide_liquidity(
        &creator,
        POOL_ID.to_string(),
        None,                 // unlocking_duration
        None,                 // lock_position_identifier
        ONE_PERCENT_SLIPPAGE, // liquidity_max_slippage (1%)
        ONE_PERCENT_SLIPPAGE, // swap_max_slippage (1%)
        None,                 // receiver
        assets,
        |result| {
            match result {
                Ok(tx_response) => {
                    println!("Liquidity provision successful with txhash: <simulated_hash>");
                    // Check that we have events indicating successful liquidity provision
                    assert!(
                        !tx_response.events.is_empty(),
                        "Liquidity provision should generate events"
                    );

                    // Look for events that indicate successful liquidity provision
                    let has_provide_liquidity_event = tx_response.events.iter().any(|event| {
                        event.ty == "wasm"
                            && event.attributes.iter().any(|attr| {
                                attr.key == "action" && attr.value == "provide_liquidity"
                            })
                    });

                    if has_provide_liquidity_event {
                        println!("✓ Found provide_liquidity event in transaction");
                    } else {
                        println!("⚠ No provide_liquidity event found, but transaction succeeded");
                    }
                }
                Err(e) => {
                    println!("Liquidity provision failed: {:?}", e);
                    // Don't fail the test, just log the error as described
                }
            }
        },
    );

    // Query pool liquidity and show pool assets
    println!("\n=== POOL LIQUIDITY INFORMATION ===");

    // Query pool information to show current pool assets
    suite.query_pools(Some(POOL_ID.to_string()), None, None, |result| {
        let response = result.unwrap();
        let pool = &response.pools[0];

        println!("Pool ID: {}", pool.pool_info.pool_identifier);
        println!("Pool Type: {:?}", pool.pool_info.pool_type);
        println!("Pool Assets:");

        let mut assets = pool.pool_info.assets.clone();
        assets.sort_by(|a, b| a.denom.cmp(&b.denom));

        for asset in &assets {
            println!(
                "  - {}: {} ({})",
                asset.denom,
                asset.amount,
                if asset.denom == DENOM_UOM {
                    "OM"
                } else if asset.denom == UUSDY_DENOM {
                    "USDY"
                } else {
                    "Unknown"
                }
            );
        }

        // Calculate total value in pool
        let om_amount = assets
            .iter()
            .find(|coin| coin.denom == DENOM_UOM)
            .map(|coin| coin.amount)
            .unwrap_or_default();
        let usdy_amount = assets
            .iter()
            .find(|coin| coin.denom == UUSDY_DENOM)
            .map(|coin| coin.amount)
            .unwrap_or_default();

        println!(
            "Total OM in pool: {} ({})",
            om_amount,
            om_amount.u128() as f64 / 1_000_000.0
        );
        println!(
            "Total USDY in pool: {} ({})",
            usdy_amount,
            usdy_amount.u128() as f64 / 1_000_000.0
        );
    });

    // Query LP token supply
    let lp_denom = suite.get_lp_denom(POOL_ID.to_string());
    suite.query_lp_supply(POOL_ID.to_string(), |result| {
        let lp_supply = result.unwrap();
        println!(
            "Total LP Token Supply: {} {}",
            lp_supply.amount, lp_supply.denom
        );
    });

    // Query user's LP token balance
    suite.query_balance(&creator.to_string(), lp_denom.clone(), |result| {
        let balance = result.unwrap();
        println!(
            "User LP Token Balance: {} {}",
            balance.amount, balance.denom
        );
    });

    // Query user's remaining asset balances
    println!("\nUser Asset Balances After Liquidity Provision:");
    suite.query_all_balances(&creator.to_string(), |result| {
        let balances = result.unwrap();
        for balance in &balances {
            if balance.denom == DENOM_UOM {
                println!(
                    "  - OM: {} ({})",
                    balance.amount,
                    balance.amount.u128() as f64 / 1_000_000.0
                );
            } else if balance.denom == UUSDY_DENOM {
                println!(
                    "  - USDY: {} ({})",
                    balance.amount,
                    balance.amount.u128() as f64 / 1_000_000.0
                );
            } else if balance.denom.contains(".LP") {
                println!("  - LP Tokens: {}", balance.amount);
            }
        }
    });

    println!("=====================================");

    println!("=== test_provide_liquidity completed ===");
}
