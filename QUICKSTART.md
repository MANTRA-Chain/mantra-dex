# Quick Start: MANTRA DEX Upgrade

This is a quick reference for upgrading the deployed MANTRA DEX. For complete details, see `UPGRADE.md`.

## ⚡ Quick Commands

### 1. Update Pool Creation Fee

```bash
./scripts/upgrade/update_pool_creation_fee.sh \
  mantra1466nf3zuxpya8q9emxukd7vftaf6h4psr0a07srl5zw74zh84yjqagspfm \
  10000000000000000000 \
  amantra
```

### 2. Update Farm Creation Fee

```bash
# Replace <FARM_MANAGER_ADDRESS> with actual address
./scripts/upgrade/update_farm_creation_fee.sh \
  <FARM_MANAGER_ADDRESS> \
  10000000000000000000 \
  amantra
```

### 3. Create MANTRA/USDC Pool

**First**, edit `scripts/upgrade/pools/mantra_usdc.json` and replace the USDC IBC denom:
```json
{
  "denom": "ibc/YOUR_ACTUAL_USDC_IBC_DENOM_HERE",
  "decimals": 6
}
```

**Then**, create the pool:
```bash
# Option 1: Using Node.js script (recommended)
cd scripts/deployment
npm install
node deploy_pool.js mantra ../../scripts/upgrade/pools/mantra_usdc.json 1000000000000000000 1000000

# Option 2: Using bash script
./scripts/deployment/deploy_pool.sh \
  -c mantra \
  -p scripts/upgrade/pools/mantra_usdc.json \
  -a 1000000000000000000,1000000
```

## 📋 Checklist

- [ ] Find USDC IBC denom for MANTRA Chain
- [ ] Update pool creation fee
- [ ] Update farm creation fee (if needed)
- [ ] Update USDC denom in mantra_usdc.json
- [ ] Create MANTRA/USDC pool
- [ ] Provide initial liquidity (optional)
- [ ] Verify all changes

## 🔍 Find USDC IBC Denom

```bash
# Check your wallet for USDC
mantrachaind q bank balances YOUR_ADDRESS --node https://rpc.mantrachain.io:443 | grep -i usdc
```

## ✅ Verify Updates

```bash
# Check pool creation fee
mantrachaind q wasm contract-state smart mantra1466nf3zuxpya8q9emxukd7vftaf6h4psr0a07srl5zw74zh84yjqagspfm \
  '{"config":{}}' --node https://rpc.mantrachain.io:443 -o json | jq '.data.pool_creation_fee'

# Check the MANTRA/USDC pool
mantrachaind q wasm contract-state smart mantra1466nf3zuxpya8q9emxukd7vftaf6h4psr0a07srl5zw74zh84yjqagspfm \
  '{"pools":{"pool_identifier":"o.mantra.usdc","limit":1}}' \
  --node https://rpc.mantrachain.io:443 -o json | jq
```

## 💡 Key Information

- **Pool Manager**: mantra1466nf3zuxpya8q9emxukd7vftaf6h4psr0a07srl5zw74zh84yjqagspfm
- **Fee Collector**: mantra1ufs3tlq4umljk0qfe8k5ya0x6hpavn897u2cnf9k0en9jr7qarqq4ha9c7
- **Chain RPC**: https://rpc.mantrachain.io
- **New Fee Amount**: 10000000000000000000 aMANTRA (10 MANTRA)
- **Gas Price**: 25000000000amantra

## 📚 Documentation

- **UPGRADE.md** - Complete upgrade instructions with all options
- **scripts/upgrade/README.md** - Script documentation
- **docs/MIGRATION_UOM_TO_AMANTRA.md** - Technical migration details

## ⚠️ Important

- You must be the contract owner to update fees
- Always verify current config before updating
- The scripts will ask for confirmation before executing
- Make sure you have sufficient aMANTRA for transaction fees
