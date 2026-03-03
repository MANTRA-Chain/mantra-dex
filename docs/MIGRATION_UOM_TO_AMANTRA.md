# Migration from uOM to aMANTRA

## Overview
MANTRA Chain has changed their native token from **uOM** (microOM) with 10^6 decimals to **aMANTRA** (attoMANTRA) with 10^18 decimals.

## Token Details

### Old Token: uOM
- **Denom**: uom
- **Decimals**: 6 (10^6)
- **Conversion**: 1 OM = 1,000,000 uOM

### New Token: aMANTRA
- **Denom**: amantra
- **Decimals**: 18 (10^18)
- **Conversion**: 1 MANTRA = 1,000,000,000,000,000,000 aMANTRA

## Changes Made

### 1. Core Constants
**File**: `test-utils/src/common_constants.rs`
```rust
pub const DENOM_UOM: &str = "amantra";
```

### 2. Deployment Environment
**Files**: 
- `scripts/deployment/deploy_env/mainnets/mantra.env`
- `scripts/deployment/deploy_env/testnets/mantra.env`

```bash
DENOM="amantra"
```

### 3. Deployment Fees
**File**: `scripts/deployment/deploy_mantra_dex.sh`

| Fee Type | Old Value | New Value |
|----------|-----------|-----------|
| Pool Creation Fee | 10,000,000 uOM (10 OM) | 10,000,000,000,000,000,000 aMANTRA (10 MANTRA) |
| Farm Creation Fee | 10,000,000 uOM (10 OM) | 10,000,000,000,000,000,000 aMANTRA (10 MANTRA) |

### 4. Gas Prices
**Files**: 
- `scripts/emergency/close_farms.js`
- `scripts/emergency/toggle_pool_features.js`

| Old Value | New Value | Equivalent |
|-----------|-----------|------------|
| 0.025 uOM | 25,000,000,000 aMANTRA | Same value |

**Calculation**:
- 0.025 uOM = 0.025 × 10^-6 OM = 0.000000025 OM
- In aMANTRA: 0.000000025 MANTRA = 0.000000025 × 10^18 aMANTRA = 25,000,000,000 aMANTRA

### 5. Test Files
All test files have been updated to use the `DENOM_UOM` constant from `test-utils/src/common_constants.rs` instead of hardcoded "uom" strings.

**Updated Files**:
- `contracts/pool-manager/src/tests/integration/swap.rs`
- `contracts/pool-manager/src/tests/integration/router.rs`
- `contracts/pool-manager/src/tests/integration/query.rs`
- `contracts/pool-manager/src/tests/integration/pool_management.rs`
- `contracts/pool-manager/src/tests/integration/lp_actions/stableswap.rs`
- `contracts/pool-manager/src/tests/integration/lp_actions/test_provide_liquidity.rs`
- `contracts/farm-manager/tests/common/suite.rs`
- `contracts/farm-manager/tests/integration/position_management.rs`
- `contracts/farm-manager/tests/integration/reward_claiming.rs`

## Decimal Scaling Factor

The decimal change from 10^6 to 10^18 represents a **10^12 multiplier**.

To maintain the same economic value:
- **Fees**: Multiply by 10^12
- **Gas prices**: Multiply by 10^12
- **Example**: 10 OM (10,000,000 uOM) = 10 MANTRA (10,000,000,000,000,000,000 aMANTRA)

## Testing
All tests have been updated and verified:
- ✅ Pool manager tests pass
- ✅ Farm manager tests pass
- ✅ Reward claiming tests pass
- ✅ Position management tests pass

## Deployment
When deploying contracts with the updated configuration:

1. Ensure you're using the correct chain environment file (`mantra.env`)
2. The pool creation fee will be: `10000000000000000000amantra` (10 MANTRA)
3. The farm creation fee will be: `10000000000000000000amantra` (10 MANTRA)
4. Gas prices should be set to: `25000000000amantra`

## Important Notes

1. **Test tokens remain unchanged**: Other test denominations (uusdc, uluna, etc.) remain at their original decimal places as they are mock tokens for testing purposes.

2. **Only the native gas token changed**: This migration only affects the native token used for gas fees and protocol fees.

3. **Pool identifiers**: Test pool identifiers that previously referenced "uom" have been updated to reference "amantra" for consistency.

## Chain Information

- **RPC**: https://rpc.mantrachain.io
- **API**: https://api.mantrachain.io
- **Fee Collector**: mantra1ufs3tlq4umljk0qfe8k5ya0x6hpavn897u2cnf9k0en9jr7qarqq4ha9c7
- **Pool Manager**: mantra1466nf3zuxpya8q9emxukd7vftaf6h4psr0a07srl5zw74zh84yjqagspfm
