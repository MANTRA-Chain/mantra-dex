#!/usr/bin/env bash
set -euo pipefail

# --- Emergency Script Runner: Toggle Pool Features OFF ---
# This script provides a safety wrapper for executing the Node.js script
# that toggles OFF all features (deposits, withdrawals, swaps) for
# all pools in a Pool Manager contract.
#
# ⚠️ EXTREME CAUTION: This is a highly disruptive emergency action. ⚠️
# ALWAYS test on a testnet first. Verify all parameters.
NODE_SCRIPT_PATH="$(dirname "$0")/toggle_pool_features.js"
PROJECT_ROOT_GUESS="$(dirname "$0")/../../"

# --- Prerequisites Check ---
if ! command -v node &> /dev/null; then
    echo "🔴 Error: Node.js (node) command could not be found. Please install Node.js."
    exit 1
fi

if [ ! -f "${NODE_SCRIPT_PATH}" ]; then
    echo "🔴 Error: Node.js script not found at ${NODE_SCRIPT_PATH}"
    echo "Ensure it exists and the path is correct within this shell script."
    exit 1
fi

echo "ℹ️  Checking for npm dependencies installation..."
if [ ! -d "${PROJECT_ROOT_GUESS}node_modules/@cosmjs" ]; then
    echo "🤔 Warning: '@cosmjs' dependencies might not be installed in '${PROJECT_ROOT_GUESS}node_modules'."
    echo "   If the script fails, try running 'npm install' in the project root: ${PROJECT_ROOT_GUESS}"
fi


# --- Gather Required Inputs (Environment variables override prompts) ---
# RPC_ENDPOINT
if [ -z "${RPC_ENDPOINT:-}" ]; then
    read -r -p "Enter RPC Endpoint URL: " rpc_url_input
    if [ -z "$rpc_url_input" ]; then
        echo "🔴 Error: RPC Endpoint URL cannot be empty."
        exit 1
    fi
    RPC_ENDPOINT_VAL="$rpc_url_input"
else
    RPC_ENDPOINT_VAL="${RPC_ENDPOINT}"
    echo "ℹ️  Using RPC_ENDPOINT from environment: ${RPC_ENDPOINT_VAL}"
fi

# POOL_MANAGER_CONTRACT_ADDRESS
if [ -z "${POOL_MANAGER_CONTRACT_ADDRESS:-}" ]; then
    read -r -p "Enter Pool Manager Contract Address: " contract_address_input
    if [ -z "$contract_address_input" ]; then
        echo "🔴 Error: Pool Manager Contract Address cannot be empty."
        exit 1
    fi
    POOL_MANAGER_CONTRACT_ADDRESS_VAL="$contract_address_input"
else
    POOL_MANAGER_CONTRACT_ADDRESS_VAL="${POOL_MANAGER_CONTRACT_ADDRESS}"
    echo "ℹ️  Using POOL_MANAGER_CONTRACT_ADDRESS from environment: ${POOL_MANAGER_CONTRACT_ADDRESS_VAL}"
fi

# --- Gather Optional Inputs ---
LEDGER_ACCOUNT_INDEX_VAL="${LEDGER_ACCOUNT_INDEX:-0}" # Default to 0 if not set
if [[ -n "${LEDGER_ACCOUNT_INDEX:-}" ]]; then
    echo "ℹ️  Using LEDGER_ACCOUNT_INDEX from environment: ${LEDGER_ACCOUNT_INDEX_VAL}"
fi


# ---  Final Confirmation ---
echo ""
echo "🚨 ========================================================= 🚨"
echo "🚨               EMERGENCY POOL FEATURE TOGGLE               🚨"
echo "🚨 ========================================================= 🚨"
echo "You are about to run a script to DISABLE ALL FEATURES for ALL pools."
echo ""
echo "   RPC Endpoint:                 ${RPC_ENDPOINT_VAL}"
echo "   Pool Manager Contract:        ${POOL_MANAGER_CONTRACT_ADDRESS_VAL}"
echo "   Ledger Account Index:         ${LEDGER_ACCOUNT_INDEX_VAL}"
echo ""
echo "   Node.js Script to execute:    ${NODE_SCRIPT_PATH}"
echo ""
echo "⚠️  PLEASE VERIFY THESE PARAMETERS CAREFULLY! ⚠️"
echo "⚠️  Ensure your Ledger is connected, unlocked, and the correct app is open. ⚠️"
echo ""
read -r -p "Type 'PROCEED' in all caps to continue, or anything else to abort: " confirmation

if [ "$confirmation" != "PROCEED" ]; then
    echo "🛑 Aborted by user."
    exit 1
fi

echo ""
echo "🚀 Executing Node.js script..."
echo ""

node "${NODE_SCRIPT_PATH}" \
    "${RPC_ENDPOINT_VAL}" \
    "${POOL_MANAGER_CONTRACT_ADDRESS_VAL}" \
    "${LEDGER_ACCOUNT_INDEX_VAL}"

EXIT_CODE=$?

echo ""
if [ $EXIT_CODE -eq 0 ]; then
    echo "✅ Node.js script execution finished successfully. All pools features have been disabled."
else
    echo "🔴 Node.js script execution failed with exit code $EXIT_CODE."
fi

exit $EXIT_CODE