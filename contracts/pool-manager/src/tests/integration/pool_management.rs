use super::super::suite::TestingSuite;
use crate::ContractError;
use cosmwasm_std::{coin, coins, Addr, Coin, Decimal, StdResult, Timestamp, Uint128};
use mantra_common_testing::multi_test::stargate_mock::StargateMock;
use mantra_dex_std::constants::{LP_SYMBOL, MONTH_IN_SECONDS};
use mantra_dex_std::farm_manager::{Position, PositionsBy};
use mantra_dex_std::fee::{Fee, PoolFee};
use mantra_dex_std::lp_common::MINIMUM_LIQUIDITY_AMOUNT;
use mantra_dex_std::pool_manager::{Config, FeatureToggle, PoolType};
use test_utils::common_constants::{
    DECIMAL_PLACES, DEFAULT_DECIMAL_PRECISION, DENOM_ULUNA, DENOM_UOM, DENOM_UOSMO, DENOM_UUSD,
    DENOM_UUSDC, DENOM_UUSDT, DENOM_UUSDY, DENOM_UWHALE, INITIAL_BALANCE, INITIAL_BALANCE_PLUS_ONE,
    LIQUIDITY_AMOUNT, POOL_CREATION_FEE, STABLESWAP_AMP_FACTOR, STARGATE_MOCK_UOM_AMOUNT,
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
const LOCK_POOL_BALANCE_AMOUNT: u128 = 1_000_000;
const LOCK_POOL_TF_FEE: u128 = 1_000;
const LOCK_POOL_LIQUIDITY_AMOUNT: u128 = 1_000;
const LOCK_POOL_LIQUIDITY_AMOUNT_2: u128 = 8_000;
const LOCK_POOL_SWAP_AMOUNT: u128 = 1_000;
const LOCK_POOL_SWAP_AMOUNT_2: u128 = 2_000;
const LOCK_POOL_ID_1: &str = "uom.uusd.1";
const LOCK_POOL_ID_2: &str = "uom.uusd.2";
const LOCK_POOL_PREFIX_1: &str = "o.uom.uusd.1";
const LOCK_POOL_PREFIX_2: &str = "o.uom.uusd.2";

// Toggle pool test constants
const TOGGLE_POOL_BALANCE_AMOUNT: u128 = 1_000_000;
const TOGGLE_POOL_TF_FEE: u128 = 1_000;
const TOGGLE_POOL_ID: &str = "uom.uusd.1";
const TOGGLE_INVALID_POOL_ID: &str = "xxx";

// Define balance and denom constants for test clarity
const BALANCE_AMOUNT_LARGE: u128 = INITIAL_BALANCE_PLUS_ONE;
const BALANCE_AMOUNT_MEDIUM: u128 = INITIAL_BALANCE;
const DENOM_LUNA: &str = DENOM_ULUNA;
const DENOM_WHALE: &str = DENOM_UWHALE;

// ... existing code ...
