#!/usr/bin/env bash
set -euo pipefail

# --- Emergency Script Runner: Close all Farms ---
# This script provides a safety wrapper for executing the Node.js script
# that closes all farms in a Farm Manager contract.
#
# ⚠️ EXTREME CAUTION: This is a highly disruptive emergency action. ⚠️
# ALWAYS test on a testnet first. Verify all parameters.

# --- Configuration & Paths ---
NODE_SCRIPT_PATH="$(dirname "$0")/close_farms.js"
PROJECT_ROOT_GUESS="$(dirname "$0")/../../"

# --- Prerequisites Check ---
if ! command -v node &>/dev/null; then
	echo "🔴 Error: Node.js (node) command could not be found. Please install Node.js."
	exit 1
fi

if [ ! -f "${NODE_SCRIPT_PATH}" ]; then
	echo "🔴 Error: Node.js script not found at ${NODE_SCRIPT_PATH}"
	exit 1
fi

echo "ℹ️  Checking for npm dependencies installation (basic check)..."
if [ ! -d "${PROJECT_ROOT_GUESS}node_modules/@cosmjs" ]; then
	echo "🤔 Warning: '@cosmjs' dependencies might not be installed in '${PROJECT_ROOT_GUESS}node_modules'."
	echo "   If the script fails, try running 'npm install' in the project root: ${PROJECT_ROOT_GUESS}"
fi

# --- Gather Required Inputs (Environment variables override prompts) ---
# RPC_ENDPOINT
if [ -z "${RPC_ENDPOINT:-}" ]; then
	read -r -p "Enter RPC Endpoint URL for Farm Manager: " rpc_url_input
	if [ -z "$rpc_url_input" ]; then
		echo "🔴 Error: RPC Endpoint URL cannot be empty."
		exit 1
	fi
	RPC_ENDPOINT_VAL="$rpc_url_input"
else
	RPC_ENDPOINT_VAL="${RPC_ENDPOINT}"
	echo "ℹ️  Using RPC_ENDPOINT from environment: ${RPC_ENDPOINT_VAL}"
fi

# FARM_MANAGER_CONTRACT_ADDRESS
if [ -z "${FARM_MANAGER_CONTRACT_ADDRESS:-}" ]; then
	read -r -p "Enter Farm Manager Contract Address: " contract_address_input
	if [ -z "$contract_address_input" ]; then
		echo "🔴 Error: Farm Manager Contract Address cannot be empty."
		exit 1
	fi
	FARM_MANAGER_CONTRACT_ADDRESS_VAL="$contract_address_input"
else
	FARM_MANAGER_CONTRACT_ADDRESS_VAL="${FARM_MANAGER_CONTRACT_ADDRESS}"
	echo "ℹ️  Using FARM_MANAGER_CONTRACT_ADDRESS from environment: ${FARM_MANAGER_CONTRACT_ADDRESS_VAL}"
fi

# --- Gather Optional Inputs ---
LEDGER_ACCOUNT_INDEX_VAL="${LEDGER_ACCOUNT_INDEX:-0}"
if [[ -n "${LEDGER_ACCOUNT_INDEX:-}" ]]; then
	echo "ℹ️  Using LEDGER_ACCOUNT_INDEX from environment: ${LEDGER_ACCOUNT_INDEX_VAL}"
fi

# --- Final Confirmation ---
echo ""
echo "🚨 ========================================================= 🚨"
echo "🚨               EMERGENCY CLOSE ALL FARMS                🚨"
echo "🚨 ========================================================= 🚨"
echo "You are about to run a script to CLOSE ALL FARMS."
echo ""
echo "   RPC Endpoint:                 ${RPC_ENDPOINT_VAL}"
echo "   Farm Manager Contract:        ${FARM_MANAGER_CONTRACT_ADDRESS_VAL}"
echo "   Ledger Account Index:         ${LEDGER_ACCOUNT_INDEX_VAL}"
echo ""
echo "   Node.js Script to execute:    ${NODE_SCRIPT_PATH}"
echo ""
echo "⚠️  PLEASE VERIFY THESE PARAMETERS CAREFULLY! ⚠️"
echo "⚠️  This will attempt to close all farms. ⚠️"
echo "⚠️  Ensure your Ledger is connected, unlocked, and the correct app is open. ⚠️"
echo ""
read -r -p "Type 'PROCEED' in all caps to continue, or anything else to abort: " confirmation

if [ "$confirmation" != "PROCEED" ]; then
	echo "🛑 Aborted by user."
	exit 1
fi

echo ""
echo "🚀 Executing Node.js script to close farms..."
echo ""

node "${NODE_SCRIPT_PATH}" \
	"${RPC_ENDPOINT_VAL}" \
	"${FARM_MANAGER_CONTRACT_ADDRESS_VAL}" \
	"${LEDGER_ACCOUNT_INDEX_VAL}"

EXIT_CODE=$?
echo ""
if [ $EXIT_CODE -eq 0 ]; then
	echo "✅ Node.js script execution finished successfully."
else
	echo "🔴 Node.js script execution failed with exit code $EXIT_CODE."
fi

exit $EXIT_CODE
