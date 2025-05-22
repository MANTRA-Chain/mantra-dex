// Token Denominations
pub const DENOM_UOM: &str = "uom";
pub const DENOM_UWHALE: &str = "uwhale";
pub const DENOM_ULUNA: &str = "uluna";
pub const DENOM_UUSD: &str = "uusd";
pub const DENOM_UUSDC: &str = "uusdc";
pub const DENOM_UUSDT: &str = "uusdt";
pub const DENOM_UUSDY: &str = "uusdy";
pub const DENOM_UOSMO: &str = "uosmo";

// Token Amounts
pub const INITIAL_BALANCE: u128 = 1_000_000_000;
pub const INITIAL_BALANCE_PLUS_ONE: u128 = 1_000_000_001;
pub const STARGATE_MOCK_UOM_AMOUNT: u128 = 8888;
pub const LIQUIDITY_AMOUNT: u128 = 1_000_000;
pub const POOL_CREATION_FEE: u128 = 1000;
pub const SWAP_AMOUNT: u128 = 1000;

// Fee Constants
pub const PROTOCOL_FEE_RATIO_1_1000: (u128, u128) = (1u128, 1000u128);
pub const SWAP_FEE_RATIO_1_10000: (u128, u128) = (1u128, 10_000u128);

// Pool Constants
pub const STABLESWAP_AMP_FACTOR: u64 = 85;

// Decimal Places
pub const DECIMAL_PLACES: u8 = 6;

// Unlocking Duration
pub const UNLOCKING_DURATION: u64 = 86_400; // 1 day in seconds
