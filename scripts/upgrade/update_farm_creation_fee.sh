#!/usr/bin/env bash
# Script to update farm creation fee on an already deployed Farm Manager contract

set -e

# Check if required arguments are provided
if [ "$#" -lt 3 ]; then
    echo "Usage: ./update_farm_creation_fee.sh <FARM_MANAGER_ADDRESS> <NEW_FEE_AMOUNT> <NEW_FEE_DENOM> [WALLET_NAME] [NODE_URL]"
    echo ""
    echo "Example:"
    echo "  ./update_farm_creation_fee.sh mantra1xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx 10000000000000000000 amantra"
    echo ""
    echo "Arguments:"
    echo "  FARM_MANAGER_ADDRESS - The address of the deployed Farm Manager contract"
    echo "  NEW_FEE_AMOUNT      - The new farm creation fee amount (e.g., 10000000000000000000 for 10 MANTRA)"
    echo "  NEW_FEE_DENOM       - The new fee denomination (e.g., amantra)"
    echo "  WALLET_NAME         - (Optional) Your wallet name (default: from environment or prompt)"
    echo "  NODE_URL            - (Optional) RPC node URL (default: https://rpc.mantrachain.io:443)"
    exit 1
fi

FARM_MANAGER_ADDRESS=$1
NEW_FEE_AMOUNT=$2
NEW_FEE_DENOM=$3
WALLET_NAME=${4:-${WALLET_NAME}}
NODE_URL=${5:-https://rpc.mantrachain.io:443}
CHAIN_ID=${CHAIN_ID:-mantra-1}
GAS_PRICES=${GAS_PRICES:-25000000000amantra}

# If wallet name is still not set, prompt for it
if [ -z "$WALLET_NAME" ]; then
    read -p "Enter your wallet name: " WALLET_NAME
fi

echo "================================================"
echo "Updating Farm Creation Fee"
echo "================================================"
echo "Farm Manager Address: $FARM_MANAGER_ADDRESS"
echo "New Fee Amount: $NEW_FEE_AMOUNT"
echo "New Fee Denom: $NEW_FEE_DENOM"
echo "Wallet: $WALLET_NAME"
echo "Chain ID: $CHAIN_ID"
echo "Node: $NODE_URL"
echo "================================================"

# Query current configuration
echo ""
echo "Querying current configuration..."
CURRENT_CONFIG=$(mantrachaind q wasm contract-state smart $FARM_MANAGER_ADDRESS \
    '{"config":{}}' \
    --node $NODE_URL \
    --output json 2>/dev/null || echo "")

if [ -n "$CURRENT_CONFIG" ]; then
    echo "Current farm creation fee:"
    echo "$CURRENT_CONFIG" | jq '.data.create_farm_fee'
else
    echo "Warning: Could not query current configuration"
fi

# Prepare the execute message
EXECUTE_MSG=$(cat <<EOF
{
  "update_config": {
    "create_farm_fee": {
      "amount": "$NEW_FEE_AMOUNT",
      "denom": "$NEW_FEE_DENOM"
    }
  }
}
EOF
)

echo ""
echo "Execute message:"
echo "$EXECUTE_MSG" | jq

# Confirmation prompt
echo ""
read -p "Do you want to proceed with this update? (yes/no): " CONFIRM
if [ "$CONFIRM" != "yes" ]; then
    echo "Update cancelled."
    exit 0
fi

# Execute the update
echo ""
echo "Executing update transaction..."
TX_RESULT=$(mantrachaind tx wasm execute \
    $FARM_MANAGER_ADDRESS \
    "$EXECUTE_MSG" \
    --from $WALLET_NAME \
    --chain-id $CHAIN_ID \
    --node $NODE_URL \
    --gas auto \
    --gas-adjustment 1.5 \
    --gas-prices $GAS_PRICES \
    --yes \
    --output json 2>&1)

# Extract transaction hash
TX_HASH=$(echo "$TX_RESULT" | jq -r '.txhash // empty')

if [ -z "$TX_HASH" ]; then
    echo "Error: Transaction failed"
    echo "$TX_RESULT"
    exit 1
fi

echo "Transaction submitted successfully!"
echo "Transaction hash: $TX_HASH"
echo ""
echo "Waiting for transaction to be included in a block..."
sleep 6

# Query transaction result
echo "Querying transaction result..."
TX_QUERY=$(mantrachaind q tx $TX_HASH --node $NODE_URL --output json 2>/dev/null || echo "")

if [ -n "$TX_QUERY" ]; then
    CODE=$(echo "$TX_QUERY" | jq -r '.code // 0')
    if [ "$CODE" = "0" ]; then
        echo "✅ Transaction successful!"
    else
        echo "❌ Transaction failed with code: $CODE"
        echo "$TX_QUERY" | jq '.raw_log'
        exit 1
    fi
else
    echo "Note: Could not verify transaction immediately. Check manually:"
    echo "  mantrachaind q tx $TX_HASH --node $NODE_URL"
fi

# Verify the update
echo ""
echo "Verifying updated configuration..."
sleep 2
UPDATED_CONFIG=$(mantrachaind q wasm contract-state smart $FARM_MANAGER_ADDRESS \
    '{"config":{}}' \
    --node $NODE_URL \
    --output json 2>/dev/null || echo "")

if [ -n "$UPDATED_CONFIG" ]; then
    echo "Updated farm creation fee:"
    echo "$UPDATED_CONFIG" | jq '.data.create_farm_fee'
    
    # Validate the update
    UPDATED_AMOUNT=$(echo "$UPDATED_CONFIG" | jq -r '.data.create_farm_fee.amount')
    UPDATED_DENOM=$(echo "$UPDATED_CONFIG" | jq -r '.data.create_farm_fee.denom')
    
    if [ "$UPDATED_AMOUNT" = "$NEW_FEE_AMOUNT" ] && [ "$UPDATED_DENOM" = "$NEW_FEE_DENOM" ]; then
        echo ""
        echo "✅ Farm creation fee successfully updated!"
    else
        echo ""
        echo "⚠️  Warning: Fee values don't match expected values"
        echo "Expected: {amount: \"$NEW_FEE_AMOUNT\", denom: \"$NEW_FEE_DENOM\"}"
        echo "Got: {amount: \"$UPDATED_AMOUNT\", denom: \"$UPDATED_DENOM\"}"
    fi
else
    echo "Warning: Could not verify updated configuration"
fi

echo ""
echo "Update complete!"
