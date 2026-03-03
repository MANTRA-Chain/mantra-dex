# MANTRA DEX Upgrade Scripts

This directory contains scripts and configuration files for upgrading the already deployed MANTRA DEX contracts from uOM to aMANTRA.

## Scripts

### update_pool_creation_fee.sh

Updates the pool creation fee on an already deployed Pool Manager contract.

**Usage:**
```bash
./update_pool_creation_fee.sh <POOL_MANAGER_ADDRESS> <NEW_FEE_AMOUNT> <NEW_FEE_DENOM> [WALLET_NAME] [NODE_URL]
```

**Example:**
```bash
./update_pool_creation_fee.sh \
  mantra1466nf3zuxpya8q9emxukd7vftaf6h4psr0a07srl5zw74zh84yjqagspfm \
  10000000000000000000 \
  amantra \
  my_wallet
```

### update_farm_creation_fee.sh

Updates the farm creation fee on an already deployed Farm Manager contract.

**Usage:**
```bash
./update_farm_creation_fee.sh <FARM_MANAGER_ADDRESS> <NEW_FEE_AMOUNT> <NEW_FEE_DENOM> [WALLET_NAME] [NODE_URL]
```

**Example:**
```bash
./update_farm_creation_fee.sh \
  mantra1xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx \
  10000000000000000000 \
  amantra \
  my_wallet
```

## Pool Configurations

### pools/mantra_usdc.json

Configuration file for creating the MANTRA/USDC pool.

**Important:** Before using this configuration, replace `ibc/REPLACE_WITH_ACTUAL_USDC_IBC_DENOM` with the actual IBC denomination for USDC on MANTRA Chain.

To find the USDC IBC denom:
```bash
# Check your wallet balance
mantrachaind q bank balances YOUR_ADDRESS --node https://rpc.mantrachain.io:443

# Or query all denoms
mantrachaind q bank total --node https://rpc.mantrachain.io:443
```

## Prerequisites

- `mantrachaind` CLI installed and configured
- `jq` installed for JSON processing
- Wallet with owner/admin permissions for the contracts
- Sufficient aMANTRA balance for transaction fees

## Environment Variables

You can set these environment variables to avoid passing them as arguments:

```bash
export WALLET_NAME="my_wallet"
export CHAIN_ID="mantra-1"
export NODE_URL="https://rpc.mantrachain.io:443"
export GAS_PRICES="25000000000amantra"
```

## Fee Conversion Reference

| Old (uOM, 10^6) | New (aMANTRA, 10^18) | Value |
|-----------------|----------------------|-------|
| 10,000,000 | 10,000,000,000,000,000,000 | 10 tokens |

The decimal change requires multiplying old fee amounts by 10^12.

## Complete Upgrade Workflow

See the root `UPGRADE.md` file for complete step-by-step instructions.

## Support

For issues or questions, refer to:
- `UPGRADE.md` - Complete upgrade instructions
- `docs/MIGRATION_UOM_TO_AMANTRA.md` - Technical migration details
