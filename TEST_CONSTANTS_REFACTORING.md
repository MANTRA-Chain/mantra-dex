# Refactor Integration Tests: Constants Extraction

This document outlines the tasks for refactoring the integration tests in the `contracts/pool-manager/src/tests/integration/` directory to extract hardcoded values into constants.
The constants should be extracted to the top of each file.

## Router Tests (`router.rs`)

- [x] **`basic_swap_operations_test`**: Extract numeric literals, string denoms, pool parameters, and fee values.
- [x] **`rejects_empty_swaps`**: Extract numeric literals and string denoms.
- [x] **`rejects_non_consecutive_swaps`**: Extract numeric literals, string denoms, and swap operation parameters.
- [x] **`sends_to_correct_receiver`**: Extract numeric literals, string denoms, and swap parameters.
- [x] **`checks_minimum_receive`**: Extract numeric literals, string denoms, and swap operation parameters.
- [x] **`query_swap_operations`**: Extract numeric literals, string denoms, and query parameters.

## Query Tests (`query.rs`)

- [x] **`simulation_queries_fees_verification`**: Extract numeric literals, string denoms, pool parameters, and fee percentages.
- [x] **`simulate_swap_operations_query_verification`**: Extract numeric literals, string denoms, and simulation parameters.
- [x] **`reverse_simulation_queries_fees_verification`**: Extract numeric literals, string denoms, pool parameters, and fee percentages.
- [x] **`reverse_simulate_swap_operations_query_verification`**: Extract numeric literals, string denoms, and simulation parameters.

## Pool Management Tests (`pool_management.rs`)

- [x] **`insufficient_pool_creation_fee`**: Extract numeric literals, string denoms, and fee values.
- [x] **`invalid_assets_on_pool_creation`**: Extract numeric literals and string denoms.
- [x] **`invalid_amount_assets_xyk_pool`**: Extract numeric literals, string denoms, and pool parameters.
- [x] **`sends_more_funds_than_needed`**: Extract numeric literals, string denoms, and fund amounts.
- [x] **`sends_less_tf_denoms_than_needed_with_funds_in_pools`**: Extract numeric literals, string denoms, and fund amounts.
- [x] **`sends_more_funds_than_needed_3_tf_denoms`**: Extract numeric literals, string denoms, and fund amounts.
- [x] **`wrong_pool_label`**: Extract numeric literals, string denoms, and label parameters.
- [x] **`cant_recreate_existing_pool`**: Extract numeric literals, string denoms, and pool parameters.
- [x] **`cant_create_stableswap_with_zero_amp_factor`**: Extract numeric literals, string denoms, and pool parameters.
- [x] **`cant_create_pool_not_paying_multiple_tf_fees`**: Extract numeric literals, string denoms, and fee values.
- [x] **`cant_create_pool_without_paying_tf_fees_same_denom`**: Extract numeric literals, string denoms, and fee values.
- [x] **`attacker_creates_farm_positions_through_pool_manager`**: Extract numeric literals, string denoms, and position parameters.
- [x] **`cant_create_pool_with_bogus_identifier`**: Extract numeric literals, string denoms, and identifier parameters.
- [x] **`cant_create_pool_with_large_number_of_assets`**: Extract numeric literals, string denoms, and asset parameters.
- [x] **`providing_custom_pool_id_doesnt_increment_pool_counter`**: Extract numeric literals, string denoms, and pool ID parameters.
- [x] **`lock_single_pool`**: Extract numeric literals, string denoms, and lock parameters.
- [x] **`cant_toggle_unexisting_pool`**: Extract numeric literals, string denoms, and toggle parameters.

## Ownership Tests (`ownership.rs`)

- [x] **`verify_ownership`**: Extract numeric literals and string denoms.
- [x] **`checks_ownership_when_updating_config`**: Extract numeric literals and string denoms.
- [x] **`updates_config_fields`**: Extract numeric literals, string denoms, and config parameters.

## Basic Tests (`basic_tests.rs`)

- [x] **All tests**: Extract numeric literals, string denoms, and test parameters.

## LP Actions Tests

### Basic LP Tests (`lp_actions/basic_lp.rs`)

- [x] **All tests**: Extract numeric literals, string denoms, and LP parameters. (DONE - 2024-05-16 - Refactored remaining constants in `lp_actions/basic_lp.rs`)

### Locking Tests (`lp_actions/locking.rs`)

- [x] **All tests**: Extract numeric literals, string denoms, and locking parameters.

### Shares Calculation Tests (`lp_actions/shares_calculation.rs`)

- [x] **All tests**: Extract numeric literals, string denoms, and shares calculation parameters. (DONE - 2024-05-21 - Refactored constants in `lp_actions/shares_calculation.rs`)

### Single Asset Tests (`lp_actions/single_asset.rs`)

- [x] **All tests**: Extract numeric literals, string denoms, and asset parameters. (DONE - 2024-05-17 - Refactored constants in `lp_actions/single_asset.rs`)

### Slippage and Fees Tests (`lp_actions/slippage_and_fees.rs`)

- [x] **All tests**: Extract numeric literals, string denoms, fee values, and slippage parameters.

### Stableswap Tests (`lp_actions/stableswap.rs`)

- [x] **All tests**: Extract numeric literals, string denoms, and stableswap parameters. (DONE - 2024-05-21 - Verified constants are already in place)

## Progress Tracking
- Percentage Complete: 56% (28/~50 tasks)

## Notes
The goal of this refactoring is to improve code maintainability by moving hardcoded values to constants at the top of each file. This will make the tests easier to read, modify, and maintain. Each test should use constants instead of hardcoded values.

## Guidelines for Constant Extraction
1. Extract all numeric literals (e.g., amounts, fees, decimals) to constants
2. Extract all string literals (e.g., denoms like "uwhale", "uluna") to constants
3. Group related constants together (e.g., all fee-related constants)
4. Use descriptive names for constants that indicate their purpose
5. Maintain consistent naming conventions across all test files 