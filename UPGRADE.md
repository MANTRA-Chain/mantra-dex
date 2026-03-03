# MANTRA DEX Upgrade Guide: uOM to aMANTRA Migration

## Overview

This guide provides step-by-step instructions for upgrading the already deployed MANTRA DEX contracts to use the new aMANTRA native token (10^18 decimals) instead of uOM (10^6 decimals).

## Deployed Contract Addresses

- **Fee Collector**: `mantra1ufs3tlq4umljk0qfe8k5ya0x6hpavn897u2cnf9k0en9jr7qarqq4ha9c7`
- **Pool Manager**: `mantra1466nf3zuxpya8q9emxukd7vftaf6h4psr0a07srl5zw74zh84yjqagspfm`
- **Chain RPC**: `https://rpc.mantrachain.io`
- **Chain API**: `https://api.mantrachain.io`

## Prerequisites

1. **Owner/Admin Access**: You must be the owner of the contracts to perform these updates
2. **mantrachaind CLI**: Installed and configured
3. **Wallet**: With sufficient aMANTRA balance for transaction fees
4. **Chain Access**: Access to mantra-1 mainnet

## Part 1: Update Fee Configurations

### 1.1 Query Current Configuration

First, verify the current configuration:

```bash
# Query Pool Manager config
mantrachaind q wasm contract-state smart mantra1466nf3zuxpya8q9emxukd7vftaf6h4psr0a07srl5zw74zh84yjqagspfm \
  '{"config":{}}' \
  --node https://rpc.mantrachain.io:443 \
  --output json | jq

# Query Farm Manager config (if you have the address)
mantrachaind q wasm contract-state smart <FARM_MANAGER_ADDRESS> \
  '{"config":{}}' \
  --node https://rpc.mantrachain.io:443 \
  --output json | jq
```

**Expected Current Values:**
- Pool creation fee: `{"amount":"10000000","denom":"uom"}` (10 OM)
- Farm creation fee: `{"amount":"10000000","denom":"uom"}` (10 OM)

### 1.2 Update Pool Creation Fee

**New Fee Value**: 10 MANTRA = `10000000000000000000` aMANTRA

#### Option A: Using the Update Script (Recommended)

```bash
./scripts/upgrade/update_pool_creation_fee.sh \
  mantra1466nf3zuxpya8q9emxukd7vftaf6h4psr0a07srl5zw74zh84yjqagspfm \
  10000000000000000000 \
  amantra
```

#### Option B: Manual Execution

```bash
# Prepare the execute message
EXECUTE_MSG='{"update_config":{"pool_creation_fee":{"amount":"10000000000000000000","denom":"amantra"}}}'

# Execute the update (replace YOUR_WALLET_NAME with your actual wallet)
mantrachaind tx wasm execute \
  mantra1466nf3zuxpya8q9emxukd7vftaf6h4psr0a07srl5zw74zh84yjqagspfm \
  "$EXECUTE_MSG" \
  --from YOUR_WALLET_NAME \
  --chain-id mantra-1 \
  --node https://rpc.mantrachain.io:443 \
  --gas auto \
  --gas-adjustment 1.5 \
  --gas-prices 25000000000amantra \
  --yes
```

### 1.3 Update Farm Creation Fee

**New Fee Value**: 10 MANTRA = `10000000000000000000` aMANTRA

#### Option A: Using the Update Script (Recommended)

```bash
./scripts/upgrade/update_farm_creation_fee.sh \
  <FARM_MANAGER_ADDRESS> \
  10000000000000000000 \
  amantra
```

#### Option B: Manual Execution

```bash
# Prepare the execute message
EXECUTE_MSG='{"update_config":{"create_farm_fee":{"amount":"10000000000000000000","denom":"amantra"}}}'

# Execute the update (replace YOUR_WALLET_NAME and FARM_MANAGER_ADDRESS)
mantrachaind tx wasm execute \
  <FARM_MANAGER_ADDRESS> \
  "$EXECUTE_MSG" \
  --from YOUR_WALLET_NAME \
  --chain-id mantra-1 \
  --node https://rpc.mantrachain.io:443 \
  --gas auto \
  --gas-adjustment 1.5 \
  --gas-prices 25000000000amantra \
  --yes
```

### 1.4 Verify Updated Configuration

```bash
# Verify Pool Manager config
mantrachaind q wasm contract-state smart mantra1466nf3zuxpya8q9emxukd7vftaf6h4psr0a07srl5zw74zh84yjqagspfm \
  '{"config":{}}' \
  --node https://rpc.mantrachain.io:443 \
  --output json | jq '.data.pool_creation_fee'

# Expected output:
# {
#   "amount": "10000000000000000000",
#   "denom": "amantra"
# }
```

## Part 2: Create MANTRA/USDC Pool

### 2.1 Pool Configuration

A pool configuration file has been created at `scripts/upgrade/pools/mantra_usdc.json`:

```json
{
  "protocol_fee": "0.001",
  "swap_fee": "0.002",
  "burn_fee": "0",
  "pool_type": "constant_product",
  "pool_identifier": "mantra.usdc",
  "assets": [
    {
      "denom": "amantra",
      "decimals": 18
    },
    {
      "denom": "ibc/...",
      "decimals": 6
    }
  ]
}
```

**Note**: Replace `ibc/...` with the actual IBC denom for USDC on MANTRA Chain.

### 2.2 Create the Pool

#### Prerequisites
- Pool creation fee: `10000000000000000000amantra` (10 MANTRA)
- Token factory fee: Query from chain (typically ~176000000amantra or similar)
- Initial liquidity amounts (optional)

#### Option A: Using deploy_pool.js Script

```bash
# First, update the pool config with correct USDC IBC denom
# Edit scripts/upgrade/pools/mantra_usdc.json

# Install dependencies if not already installed
cd scripts/deployment
npm install

# Create the pool (with initial liquidity - optional)
# Format: node deploy_pool.js <chain> <pool_config> <amount_asset0> <amount_asset1>
node deploy_pool.js mantra scripts/upgrade/pools/mantra_usdc.json 1000000000000000000 1000000

# Above example: 1 MANTRA and 1 USDC for initial liquidity
```

#### Option B: Using deploy_pool.sh Script

```bash
# Load chain environment
source scripts/deployment/deploy_env/chain_env.sh -c mantra

# Create pool with initial liquidity
./scripts/deployment/deploy_pool.sh \
  -c mantra \
  -p scripts/upgrade/pools/mantra_usdc.json \
  -a 1000000000000000000,1000000

# Without initial liquidity, omit the -a parameter
```

#### Option C: Manual Pool Creation

```bash
# 1. Query the total fee required
POOL_CREATION_FEE=$(mantrachaind q wasm contract-state smart mantra1466nf3zuxpya8q9emxukd7vftaf6h4psr0a07srl5zw74zh84yjqagspfm '{"config":{}}' --node https://rpc.mantrachain.io:443 -o json | jq -r '.data.pool_creation_fee.amount')
TOKEN_FACTORY_FEE=$(mantrachaind q tokenfactory params --node https://rpc.mantrachain.io:443 -o json | jq -r '.params.denom_creation_fee[0].amount')
TOTAL_FEE=$((POOL_CREATION_FEE + TOKEN_FACTORY_FEE))

echo "Total fee required: ${TOTAL_FEE}amantra"

# 2. Create the pool (update USDC_IBC_DENOM with actual value)
USDC_IBC_DENOM="ibc/..."  # Replace with actual IBC denom

CREATE_POOL_MSG='{
  "create_pool": {
    "asset_denoms": ["amantra", "'$USDC_IBC_DENOM'"],
    "asset_decimals": [18, 6],
    "pool_fees": {
      "protocol_fee": {"share": "0.001"},
      "swap_fee": {"share": "0.002"},
      "burn_fee": {"share": "0"},
      "extra_fees": []
    },
    "pool_type": "constant_product",
    "pool_identifier": "mantra.usdc"
  }
}'

# Execute pool creation
mantrachaind tx wasm execute \
  mantra1466nf3zuxpya8q9emxukd7vftaf6h4psr0a07srl5zw74zh84yjqagspfm \
  "$CREATE_POOL_MSG" \
  --from YOUR_WALLET_NAME \
  --amount ${TOTAL_FEE}amantra \
  --chain-id mantra-1 \
  --node https://rpc.mantrachain.io:443 \
  --gas auto \
  --gas-adjustment 1.5 \
  --gas-prices 25000000000amantra \
  --yes

# 3. Query to get the pool identifier
# After creation, query pools to find your newly created pool
mantrachaind q wasm contract-state smart mantra1466nf3zuxpya8q9emxukd7vftaf6h4psr0a07srl5zw74zh84yjqagspfm \
  '{"pools":{"pool_identifier":"o.mantra.usdc","limit":1}}' \
  --node https://rpc.mantrachain.io:443 \
  --output json | jq
```

### 2.3 Provide Initial Liquidity (Optional)

```bash
# Provide liquidity to the newly created pool
PROVIDE_LIQUIDITY_MSG='{"provide_liquidity":{"pool_identifier":"o.mantra.usdc"}}'

# Example: Add 10 MANTRA and 10 USDC
mantrachaind tx wasm execute \
  mantra1466nf3zuxpya8q9emxukd7vftaf6h4psr0a07srl5zw74zh84yjqagspfm \
  "$PROVIDE_LIQUIDITY_MSG" \
  --from YOUR_WALLET_NAME \
  --amount 10000000000000000000amantra,10000000${USDC_IBC_DENOM} \
  --chain-id mantra-1 \
  --node https://rpc.mantrachain.io:443 \
  --gas auto \
  --gas-adjustment 1.5 \
  --gas-prices 25000000000amantra \
  --yes
```

## Part 3: Verification

### 3.1 Verify Fee Updates

```bash
# Check Pool Manager config
mantrachaind q wasm contract-state smart mantra1466nf3zuxpya8q9emxukd7vftaf6h4psr0a07srl5zw74zh84yjqagspfm \
  '{"config":{}}' \
  --node https://rpc.mantrachain.io:443 -o json | jq

# Expected pool_creation_fee: {"amount":"10000000000000000000","denom":"amantra"}
```

### 3.2 Verify Pool Creation

```bash
# Query the MANTRA/USDC pool
mantrachaind q wasm contract-state smart mantra1466nf3zuxpya8q9emxukd7vftaf6h4psr0a07srl5zw74zh84yjqagspfm \
  '{"pools":{"pool_identifier":"o.mantra.usdc","limit":1}}' \
  --node https://rpc.mantrachain.io:443 -o json | jq
```

## Important Notes

### Gas Prices
- **Old**: `0.025uom`
- **New**: `25000000000amantra`
- Always use `--gas-prices 25000000000amantra` for transactions

### Fee Amounts
All fees are now 10^12 times larger numerically to account for the decimal change:
- Pool creation: 10,000,000 uOM → 10,000,000,000,000,000,000 aMANTRA
- Farm creation: 10,000,000 uOM → 10,000,000,000,000,000,000 aMANTRA
- Both represent **10 tokens** economically

### USDC IBC Denom
To find the correct USDC IBC denom on MANTRA Chain:

```bash
# Query your wallet balance to find the USDC denom
mantrachaind q bank balances YOUR_ADDRESS --node https://rpc.mantrachain.io:443

# Or check with the chain explorer/documentation
```

### Pool Identifier Format
- **User-provided identifier**: `mantra.usdc`
- **On-chain identifier**: `o.mantra.usdc` (with "o." prefix for official pools)

## Rollback Considerations

If you need to revert the fee changes (not recommended once pools are created with new fees):

```bash
# Revert pool creation fee
EXECUTE_MSG='{"update_config":{"pool_creation_fee":{"amount":"10000000","denom":"uom"}}}'
mantrachaind tx wasm execute mantra1466nf3zuxpya8q9emxukd7vftaf6h4psr0a07srl5zw74zh84yjqagspfm \
  "$EXECUTE_MSG" --from YOUR_WALLET_NAME --chain-id mantra-1 \
  --node https://rpc.mantrachain.io:443 --gas auto --gas-adjustment 1.5 \
  --gas-prices 25000000000amantra --yes
```

**Warning**: This is only possible if uOM still exists on the chain. Since uOM no longer exists, rollback is not feasible.

## Support

For issues or questions:
- Check transaction status: `mantrachaind q tx <TX_HASH> --node https://rpc.mantrachain.io:443`
- Review contract state: Use the query commands provided above
- Refer to `docs/MIGRATION_UOM_TO_AMANTRA.md` for technical details

## Summary Checklist

- [ ] Query current configuration
- [ ] Update pool creation fee to 10000000000000000000 aMANTRA
- [ ] Update farm creation fee to 10000000000000000000 aMANTRA
- [ ] Verify fee updates
- [ ] Update USDC IBC denom in pool config
- [ ] Create MANTRA/USDC pool
- [ ] Provide initial liquidity (optional)
- [ ] Verify pool creation
- [ ] Test pool functionality (swaps, liquidity provision)
