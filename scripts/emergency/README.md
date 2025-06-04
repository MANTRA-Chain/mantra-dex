# Emergency Scripts

This directory contains scripts for **EMERGENCY USE ONLY**. These scripts allow authorized accounts (via Ledger) to perform critical, widespread actions on smart contracts.

**⚠️ EXTREME CAUTION REQUIRED FOR ALL SCRIPTS IN THIS DIRECTORY ⚠️**
*   Running these scripts can have significant and immediate impact on contract operations and user funds/positions.
*   These are powerful tools and should only be used in critical situations by authorized personnel who fully understand the consequences.
*   **ALWAYS TEST THOROUGHLY ON A TESTNET BEFORE EVEN CONSIDERING MAINNET USE.**
*   **VERIFY ALL PARAMETERS AND REVIEW GENERATED JSON PREVIEWS CAREFULLY BEFORE BROADCASTING ANY TRANSACTION.**

---

## Common Prerequisites & Setup (For All Emergency Scripts)

### 1. Software & Environment:
*   **Node.js and npm:** Required to run the JavaScript files. Download from [nodejs.org](https://nodejs.org/).
*   **Project Dependencies:** From the project root, install necessary libraries:
    ```bash
    npm install @cosmjs/cosmwasm-stargate @cosmjs/amino @cosmjs/ledger-amino @ledgerhq/hw-transport-node-hid @cosmjs/stargate @cosmjs/encoding
    ```
*   **`just` Command Runner (Optional, for `just` recipe usage):**
    *   If using `just` recipes, ensure `just` is installed. See [just GitHub page](https://github.com/casey/just).
    *   The project's root `justfile` must contain the relevant recipes.

### 2. Hardware & Ledger Setup:
*   **Ledger Hardware Wallet:** Nano S/X or compatible.
*   **Ledger Live:** Ensure it's up-to-date for firmware, but **CLOSE Ledger Live before running scripts** to avoid conflicts.
*   **Cosmos App on Ledger:** The official Cosmos (or chain-specific, e.g., Mantra) app must be installed and **open** on your Ledger device.
*   **Ledger Device State:** Connected to computer and unlocked (PIN entered).

### 3. (Linux Users Only) udev Rules:
*   May be required for Ledger device access. Search "Ledger udev rules Linux".

### 4. Gas Price Configuration (In `.js` scripts):
*   Each `.js` script (e.g., `toggle_pool_features.js`, `close_farms.js`) contains a `GAS_PRICE_STRING` constant (e.g., `"0.025uom"`).
*   **Verify and adjust this value within EACH Node.js script if necessary to match your target chain's fee requirements (denomination and price).**

---

## Script 1: Toggle Pool Features OFF (`toggle_pool_features.js`)

### Purpose
To DISABLE key features (withdrawals, deposits, and swaps) for ALL pools in a specified Pool Manager contract. It sends an `update_config` message with a `feature_toggle` for each pool.

### How to Run

#### Method A: Using the `just` Recipe (Recommended)
Uses the `emergency-toggle-pool-features-off` recipe in the project's `justfile`, which calls `scripts/emergency/run_toggle_pool_features.sh`.

1.  **Navigate to Project Root.**
2.  **Ensure Ledger is Ready** (Connected, Unlocked, App Open, Ledger Live Closed).
3.  **Execute `just` Command:**
    *   Interactive (prompts if env vars not set):
        ```bash
        just emergency-toggle-pool-features-off
        ```
    *   With environment variables:
        ```bash
        RPC_ENDPOINT="<RPC_URL>" \
        POOL_MANAGER_CONTRACT_ADDRESS="<POOL_MANAGER_ADDR>" \
        just emergency-toggle-pool-features-off
        ```
        (Optional: `LEDGER_ACCOUNT_INDEX=<INDEX>`)
4.  **Follow Prompts:** The wrapper script will ask for confirmation (typing "PROCEED").

#### Method B: Direct Node.js Script Execution
1.  **Navigate to Project Root.**
2.  **Ensure Ledger is Ready.**
3.  **Execute Command:**
    ```bash
    node scripts/emergency/toggle_pool_features.js "<RPC_URL>" "<POOL_MANAGER_ADDR>" [LEDGER_ACCOUNT_INDEX]
    ```

### Execution Flow (Common for both methods)
1.  Connects to Ledger & RPC.
2.  Queries all pool identifiers from the Pool Manager.
3.  Generates `update_config` messages.
4.  Saves a `emergency_toggle_off_features_tx_preview_...json` file. **REVIEW THIS FILE.**
5.  Asks for terminal confirmation.
6.  Requires **Ledger device approval** for the transaction.
7.  Broadcasts transaction and outputs result.

---

## Script 2: Close ALL Farms (`close_farms.js`)

### Purpose
To attempt to CLOSE ALL farms listed in a specified Farm Manager contract. It iterates through all farms, and for each, sends a `ManageFarm` execute message with the `Close` action. **Note: This script, as per `close_farms.js`, attempts to close ALL farms returned by the query, not just "active" ones.**

### How to Run

#### Method A: Using the `just` Recipe (Recommended)
Uses the `emergency-close-all-farms` (or similar name you've defined) recipe in the project's `justfile`, which calls `scripts/emergency/close_farms.sh`.

1.  **Navigate to Project Root.**
2.  **Ensure Ledger is Ready** (Connected, Unlocked, App Open, Ledger Live Closed).
3.  **Execute `just` Command:**
    *   Interactive (prompts if env vars not set):
        ```bash
        just emergency-close-all-farms 
        ``` 
        (Adjust recipe name if different in your `justfile`)
    *   With environment variables:
        ```bash
        RPC_ENDPOINT="<RPC_URL>" \
        FARM_MANAGER_CONTRACT_ADDRESS="<FARM_MANAGER_ADDR>" \
        just emergency-close-all-farms
        ```
        (Optional: `LEDGER_ACCOUNT_INDEX=<INDEX>`)
4.  **Follow Prompts:** The wrapper script (`close_farms.sh`) will ask for confirmation (e.g., typing "PROCEED").

#### Method B: Direct Node.js Script Execution
1.  **Navigate to Project Root.**
2.  **Ensure Ledger is Ready.**
3.  **Execute Command:**
    ```bash
    node scripts/emergency/close_farms.js "<RPC_URL>" "<FARM_MANAGER_ADDR>" [LEDGER_ACCOUNT_INDEX]
    ```

### Execution Flow (Common for both methods)
1.  Connects to Ledger & RPC.
2.  Queries all farm identifiers from the Farm Manager.
3.  Generates `ManageFarm` (Close action) messages for **all** queried farms.
4.  Saves a `emergency_close_all_farms_tx_preview_...json` file. **REVIEW THIS FILE.**
5.  Asks for terminal confirmation.
6.  Requires **Ledger device approval** for the transaction.
7.  Broadcasts transaction and outputs result.

---

## General Troubleshooting (For All Scripts)

*   **Ledger Connection Failed:** Check physical connection, ensure Ledger is unlocked, correct app is open, and Ledger Live (or other conflicting apps) is closed. (Linux) Verify udev rules.
*   **Fee Errors / "No fee coin provided":**
    *   Verify `GAS_PRICE_STRING` (value and denomination) in the relevant `.js` script.
    *   If "out of gas," increase `gasPerMessage` in the `.js` script.
*   **Transaction Failed (Contract Error):** Examine `Log:` output from the script for contract-specific errors. Verify contract addresses and permissions of the sending Ledger account.
*   **Node.js Script Not Found / Cannot find module:** Ensure you are in the project root directory when running commands and that `npm install` has completed successfully.

## Safety Checklist & Best Practices (For All Scripts)

*   ✅ **TESTNET, TESTNET, TESTNET!** Crucial before mainnet.
*   ✅ **Verify Contract Addresses & RPC URL:** Typos can be disastrous.
*   ✅ **Review Generated JSON Preview Files:** Understand every message.
*   ✅ **Understand Script Logic:** Know what each script does.
*   ✅ **Secure Environment:** Run from a trusted computer.
*   ✅ **Ledger App Version:** Keep Ledger apps updated.
*   ✅ **Authorized Personnel Only.**

---

By using these scripts, you acknowledge the risks involved and take full responsibility for their execution and consequences.
