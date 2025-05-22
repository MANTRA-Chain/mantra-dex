// Common Token Denominations
pub const DENOM_UOM: &str = "uom";
pub const DENOM_UUSDY: &str = "uusdy";
pub const DENOM_UOSMO: &str = "uosmo";
pub const DENOM_UWHALE: &str = "uwhale";
pub const DENOM_ULUNA: &str = "uluna";
pub const DENOM_UUSD: &str = "uusd";
pub const DENOM_UUSDC: &str = "uusdc";
pub const DENOM_UUSDT: &str = "uusdt";

// Common Amounts and Balances
pub const INITIAL_BALANCE: u128 = 1_000_000_000;
pub const INITIAL_USER_BALANCE: u128 = INITIAL_BALANCE; // Alias for backward compatibility
pub const INITIAL_BALANCE_PLUS_ONE: u128 = 1_000_000_001;
pub const LIQUIDITY_AMOUNT: u128 = 1_000_000;
pub const STARGATE_MOCK_UOM_AMOUNT: u128 = 8888;
pub const SWAP_AMOUNT: u128 = 1000;

// Fee Constants
pub const POOL_CREATION_FEE: u128 = 1000;
pub const UOM_FARM_CREATION_FEE: u128 = 1_000;
pub const PROTOCOL_FEE_RATIO_1_1000: (u128, u128) = (1u128, 1000u128);
pub const SWAP_FEE_RATIO_1_1000: (u128, u128) = (1u128, 1000u128);

// Decimal Constants
pub const DECIMAL_PLACES: u8 = 6;
pub const DEFAULT_DECIMAL_PRECISION: u8 = 6;

// Duration Constants
pub const DEFAULT_UNLOCKING_DURATION_SECONDS: u64 = 86400; // 1 day
pub const MONTH_IN_SECONDS: u64 = 2_592_000; // 30 days in seconds

// Pool Constants
pub const STABLESWAP_AMP_FACTOR: u64 = 100;
