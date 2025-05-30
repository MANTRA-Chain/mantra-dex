// Common Token Denominations
pub const DENOM_UOM: &str = "uom";
pub const DENOM_UUSDY: &str = "uusdy";
pub const DENOM_UOSMO: &str = "uosmo";
pub const DENOM_UWHALE: &str = "uwhale";
pub const DENOM_ULUNA: &str = "uluna";
pub const DENOM_UUSD: &str = "uusd";
pub const DENOM_UUSDC: &str = "uusdc";
pub const DENOM_UUSDT: &str = "uusdt";
pub const DENOM_UWETH: &str = "uweth";
pub const DENOM_INVALID_LP: &str = "invalid_lp";

// Common Amounts and Balances
pub const INITIAL_BALANCE: u128 = 1_000_000_000;
pub const INITIAL_BALANCE_PLUS_ONE: u128 = 1_000_000_001;
pub const STARGATE_MOCK_UOM_AMOUNT: u128 = 8888;
pub const ONE_THOUSAND: u128 = 1000;
pub const ONE_MILLION: u128 = 1_000_000;
pub const ONE_BILLION: u128 = 1_000_000_000;
pub const ONE_HUNDRED_TRILLION: u128 = 100_000_000_000_000;

// Fee Constants
pub const PROTOCOL_FEE_RATIO_1_1000: (u128, u128) = (1u128, 1000u128);
pub const SWAP_FEE_RATIO_1_1000: (u128, u128) = (1u128, 1000u128);

// Decimal Constants
pub const DECIMALS_6: u8 = 6;
pub const DECIMALS_12: u8 = 12;
pub const DECIMALS_18: u8 = 18;

// Duration Constants
pub const DEFAULT_UNLOCKING_DURATION_SECONDS: u64 = 86400; // 1 day
pub const MONTH_IN_SECONDS: u64 = 2_592_000; // 30 days in seconds

// Pool Constants
pub const STABLESWAP_AMP_FACTOR: u64 = 100;
