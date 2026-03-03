# DEX Reconfiguration Summary: uOM → aMANTRA

## Overview
Successfully reconfigured the MANTRA DEX to use aMANTRA (10^18 decimals) as the native gas token, replacing uOM (10^6 decimals).

## ✅ All Changes Completed

### 1. Core Configuration
- **test-utils/src/common_constants.rs**: Updated DENOM_UOM constant
  ```rust
  pub const DENOM_UOM: &str = "amantra";
  ```

### 2. Deployment Environments
Both mainnet and testnet environment files updated:
- **scripts/deployment/deploy_env/mainnets/mantra.env**
- **scripts/deployment/deploy_env/testnets/mantra.env**
  ```bash
  DENOM="amantra"
  ```

### 3. Deployment Fees (scripts/deployment/deploy_mantra_dex.sh)
```json
{
  "pool_creation_fee": {
    "denom": "amantra",
    "amount": "10000000000000000000"  // 10 MANTRA
  },
  "create_farm_fee": {
    "denom": "amantra", 
    "amount": "10000000000000000000"  // 10 MANTRA
  }
}
```

### 4. Gas Prices (Emergency Scripts)
- **scripts/emergency/close_farms.js**
- **scripts/emergency/toggle_pool_features.js**
  ```javascript
  const GAS_PRICE_STRING = "25000000000amantra";
  ```

### 5. Test Files
Updated 9 test files across pool-manager and farm-manager:
- Replaced 90+ hardcoded "uom" strings with DENOM_UOM constant
- Updated pool identifiers from "uom" to "amantra"
- All tests passing ✅

### 6. Documentation
- Created comprehensive migration guide: **docs/MIGRATION_UOM_TO_AMANTRA.md**

## Value Conversions

| Item | Old Value (uOM) | New Value (aMANTRA) | Actual Value |
|------|-----------------|---------------------|--------------|
| Pool Creation Fee | 10,000,000 | 10,000,000,000,000,000,000 | 10 tokens |
| Farm Creation Fee | 10,000,000 | 10,000,000,000,000,000,000 | 10 tokens |
| Gas Price | 0.025 | 25,000,000,000 | 0.000000025 tokens |

### Decimal Scaling
- **Factor**: 10^12 multiplier (from 10^6 to 10^18)
- **Calculation**: All old values × 10^12 = new values

## Verification ✅

### Build Status
```bash
cargo build
# ✅ Finished `dev` profile in 41.95s
```

### Test Results
```bash
# Pool Manager - Swap Tests
✅ 14 tests passed

# Farm Manager - Position Management
✅ 11 tests passed

# Farm Manager - Reward Claiming  
✅ 8 tests passed
```

## Files Changed
Total: 17 files
- 9 test files
- 4 deployment/configuration files
- 2 emergency script files
- 1 core constants file
- 1 documentation file

## Next Steps for Deployment

1. **Review the PR** and ensure all changes are correct
2. **Test on testnet** using the updated configuration:
   ```bash
   just deploy mantra-testnet
   ```
3. **Deploy to mainnet** when ready:
   ```bash
   just deploy mantra
   ```

## Important Notes

⚠️ **Critical**: The native token denomination has changed from "uom" to "amantra"
⚠️ **Fee amounts**: All fees are now 10^12 times larger numerically to maintain the same economic value
⚠️ **Gas prices**: Gas price is now "25000000000amantra" instead of "0.025uom"

## Chain Information
- **RPC**: https://rpc.mantrachain.io
- **API**: https://api.mantrachain.io  
- **Fee Collector**: mantra1ufs3tlq4umljk0qfe8k5ya0x6hpavn897u2cnf9k0en9jr7qarqq4ha9c7
- **Pool Manager**: mantra1466nf3zuxpya8q9emxukd7vftaf6h4psr0a07srl5zw74zh84yjqagspfm

## Contact
For questions or issues with this migration, please refer to the detailed migration guide at `docs/MIGRATION_UOM_TO_AMANTRA.md`.
