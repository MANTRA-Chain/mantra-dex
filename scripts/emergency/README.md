# Emergency Script: Toggle Pool Features OFF

**⚠️ EXTREME CAUTION REQUIRED ⚠️**

This script is designed for **EMERGENCY USE ONLY**. It allows an authorized account (**via Ledger only**) to iterate through all pools in a specified Pool Manager contract and send transactions to **DISABLE** key features for each pool (withdrawals, deposits, and swaps) by setting their respective flags to `false` via the `update_config` message with a `feature_toggle`.

**IMPLICATIONS:**

- Running this script will HALT normal operations (deposits, withdrawals, swaps) for ALL pools managed by the target Pool Manager.
- This is a powerful action and should only be taken in critical situations by authorized personnel who fully understand the consequences.
- **ALWAYS TEST THOROUGHLY ON A TESTNET BEFORE EVEN CONSIDERING MAINNET USE.**

## 1. Purpose

To provide a mechanism to quickly disable core functionalities (deposits, withdrawals, swaps) across all liquidity pools managed by a given Pool Manager contract. This is achieved by sending an `update_config` message with a `feature_toggle` for each pool, setting `withdrawals_enabled: false`, `deposits_enabled: false`, and `swaps_enabled: false`.

## 2. Prerequisites

Before running this script, ensure the following are in place:

### 2.1. Software & Environment:

- **Node.js and npm:** The script is written in JavaScript and requires Node.js to run. npm (Node Package Manager) is needed to install dependencies. Download from [nodejs.org](https://nodejs.org/).
- **Project Dependencies:** Navigate to the root of your project in the terminal and install the required libraries:
  ```bash
  npm install @cosmjs/cosmwasm-stargate @cosmjs/amino @cosmjs/ledger-amino @ledgerhq/hw-transport-node-hid @cosmjs/stargate @cosmjs/encoding
  ```
  (Ensure these versions are compatible with your project or update as needed.)

### 2.2. Hardware & Ledger Setup:

- **Ledger Hardware Wallet:** A Ledger Nano S, Nano X, or compatible device.
- **Ledger Live:** Ensure Ledger Live is up-to-date for firmware updates if needed, but **CLOSE Ledger Live before running the script** to avoid conflicts.
- **Cosmos App on Ledger:** The official Cosmos (or chain-specific, e.g., Mantra) application must be installed on your Ledger device via Ledger Live's Manager.
- **Ledger Device State:**
  1.  Connect your Ledger device to your computer.
  2.  Unlock the Ledger by entering your PIN.
  3.  Navigate to and **open the Cosmos (or chain-specific) app** on the Ledger. It must remain open during the script's execution.

### 2.3. (Linux Users Only) udev Rules:

- Linux systems may require udev rules to allow non-root users to access USB hardware like the Ledger. Search "Ledger udev rules Linux" for setup instructions if you encounter connection issues.

### 2.4. `just` Command Runner (Optional, for `just` recipe usage)

- If you intend to use the `just` recipe, ensure `just` is installed. See [just GitHub page](https://github.com/casey/just) for installation instructions.
- The project must have a `justfile` at its root containing the `emergency-toggle-pool-features-off` recipe.

## 3. Configuration (Within the Script / Command Line / Environment Variables)

The script requires the following information:

- **RPC Endpoint:** The URL of a public or private RPC node for the target blockchain.
  - _Provided as a command-line argument._
- **Pool Manager Contract Address:** The on-chain address of the Pool Manager smart contract you want to interact with.
  - _Provided as a command-line argument._
- **Ledger Account Index (Optional):** The derivation path index for your Ledger account (e.g., `0` for `m/44'/118'/0'/0/0`, `1` for `m/44'/118'/0'/0/1`).
  - _Provided as an optional command-line argument. Defaults to `0`._
- **Gas Price & Denomination (Hardcoded in Node.js script):**
  - The Node.js script (`toggle_pool_features.js`) has a `GAS_PRICE_STRING` constant (e.g., `"0.025uom"`).
  - `uom` (or similar like `uphoton`, `aconst`) is the fee denomination for your chain.
  - `0.025` is the price per unit of gas in that denomination.
  - **Verify and adjust this value within the Node.js script if necessary to match your target chain's fee requirements.**

## 4. How to Run the Script

There are two primary ways to run this script:

### Method 1: Using the `just` Recipe (Recommended for safety and ease)

This method uses a `just` command defined in the project's `justfile`. It typically wraps the execution in a shell script (`scripts/emergency/run_toggle_pool_features.sh`) that provides additional safety checks, input prompts, and confirmations.

1.  **Navigate to Project Root:** Open your terminal and change to the root directory of your project (e.g., `/path/to/your/project/`).
2.  **Ensure Ledger is Ready:** Connect Ledger, unlock, open Cosmos (or chain-specific) app. Close Ledger Live.
3.  **Execute the `just` Command:**
    - **Interactive mode (prompts for inputs if not set as environment variables):**
      ```bash
      just emergency-toggle-pool-features-off
      ```
      The script will prompt for "RPC Endpoint URL" and "Pool Manager Contract Address" if they are not already set as environment variables.
    - **With environment variables (to skip prompts):**
      ```bash
      RPC_ENDPOINT="https://rpc.mantra.finance" \
      POOL_MANAGER_CONTRACT_ADDRESS="mantra1eadexmanagercontractaddressxxxx" \
      just emergency-toggle-pool-features-off
      ```
      You can also set `LEDGER_ACCOUNT_INDEX` (e.g., `LEDGER_ACCOUNT_INDEX=1`) if you don't want to use the default (0).
4.  **Follow Prompts:** The script will display the parameters and ask for a final confirmation (typing "PROCEED") before executing the underlying Node.js script.

### Method 2: Direct Node.js Script Execution

This method involves calling the Node.js script directly. The `just` recipe and its wrapper shell script add safety layers, so direct execution should be done with even greater care.

1.  **Navigate to Project Root:** Open your terminal and change to the root directory of your project.
2.  **Ensure Ledger is Ready:** Connect Ledger, unlock, open Cosmos (or chain-specific) app. Close Ledger Live.
3.  **Execute the Command:**

    ```bash
    node scripts/emergency/toggle_pool_features.js "<YOUR_RPC_ENDPOINT>" "<YOUR_POOL_MANAGER_CONTRACT_ADDRESS>" [ACCOUNT_INDEX]
    ```

    **Replace placeholders:**

    - `<YOUR_RPC_ENDPOINT>`: e.g., `"http://localhost:26657"`, `"https://rpc.mainnet.yourchain.io"`
    - `<YOUR_POOL_MANAGER_CONTRACT_ADDRESS>`: e.g., `"mantra1xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"`
    - `[ACCOUNT_INDEX]`: (Optional) e.g., `0`, `1`, `2`. Defaults to `0`.

    **Example (using default account index 0):**

    ```bash
    node scripts/emergency/toggle_pool_features.js "https://rpc.mantra.finance" "mantra1eadexmanagercontractaddressxxxx"
    ```

    **Example FOR MAINNET:**

    ```bash
    node scripts/emergency/toggle_pool_features.js "https://rpc.mantrachain.io:443" "mantra1466nf3zuxpya8q9emxukd7vftaf6h4psr0a07srl5zw74zh84yjqagspfm"
    ```

## 5. Execution Flow & User Interaction (Applies to both methods)

1.  **Ledger Connection:** The script (Node.js or via shell wrapper) will attempt to connect to your Ledger.
2.  **RPC Connection:** Connects to the specified blockchain RPC endpoint.
3.  **Querying Pools:** Fetches all pool identifiers from the Pool Manager contract.
4.  **Message Generation:** Creates `update_config` messages for each pool.
5.  **JSON Preview File:**
    - A JSON file (e.g., `emergency_toggle_off_features_tx_preview_xxxxxxxxxxxxx.json`) will be saved.
    - **CRITICAL STEP:** Open and **carefully review this file**.
6.  **Terminal Confirmation:**
    - The script (or wrapper) will ask for explicit confirmation before signing and broadcasting.
7.  **Ledger Device Approval:**
    - You **MUST approve the transaction(s) on your Ledger device**.
8.  **Transaction Broadcast & Output:**
    - The script will output the transaction hash or errors.

## 6. Troubleshooting Common Issues

- **Ledger Connection Failed:**
  - Is Ledger plugged in and unlocked?
  - Is the correct app (Cosmos/Mantra) open on Ledger?
  - Is Ledger Live or another app closed (to avoid conflict)?
  - (Linux) Are udev rules configured?
- **"No fee coin provided" / Fee Errors:**
  - Ensure `GAS_PRICE_STRING` in the script is correct for your chain (both value and denomination like "uom").
  - The script now calculates the fee amount. If issues persist, the `gasPerMessage` estimate might be too low, leading to an overall fee the chain rejects.
- **Transaction Failed (Contract Error):**
  - Examine the `Log:` output from the script. This often contains smart contract-specific error messages.
  - Verify the `poolManagerAddress` is correct and that the Ledger account used (`senderAddress`) has the necessary permissions on the contract to perform `update_config`.
- **Transaction Failed (e.g., "out of gas"):**
  - The `gasPerMessage` constant in the script (e.g., `250000`) might be too low for the complexity of the `update_config` call on your chain. Increase it and try again (on a testnet first).

## 7. Safety Checklist & Best Practices

- ✅ **TESTNET, TESTNET, TESTNET!** Before running on mainnet, always perform multiple successful dry runs on a reliable testnet using a test Ledger account and test contracts.
- ✅ **Verify Contract Addresses:** Double-check the RPC endpoint and especially the Pool Manager contract address. A typo can have disastrous consequences.
- ✅ **Review Generated JSON:** Do not skip reviewing the `_tx_preview_...json` file. Understand every message being sent.
- ✅ **Understand Script Logic:** Familiarize yourself with what the script is doing at each step.
- ✅ **Secure Environment:** Run the script from a trusted and secure computer.
- ✅ **Ledger App Version:** Ensure your Ledger's Cosmos (or chain-specific) app is up to date.
- ✅ **Small Batches (if necessary):** While this script processes all pools in one transaction, if there were an _extremely_ large number of pools, consider if modifications for batching would be safer (though this script is designed for a single, comprehensive emergency action). This script currently does not support batching transactions.
- ✅ **Authorized Personnel Only:** This script should only be handled by individuals with the authority and technical understanding to use it.

---

By using this script, you acknowledge the risks involved and take full responsibility for its execution and consequences.
