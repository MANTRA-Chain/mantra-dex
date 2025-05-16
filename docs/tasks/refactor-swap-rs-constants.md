## Refactor `swap.rs` Integration Tests: Constants Extraction

This document outlines the tasks for refactoring the integration tests in `contracts/pool-manager/src/tests/integration/swap.rs` to extract hardcoded values into constants.
The constants should be extracted to the top of the file.

- [x] **`swap_large_digits_stable_18_digits`**: Extract numeric literals and string denoms. (DONE - 2023-10-27)
- [x] **`swap_3pool_same_decimals`**: Extract numeric literals, string denoms, and pool parameters. (DONE - 2023-10-27)
- [x] **`swap_3pool_different_decimals`**: Extract numeric literals, string denoms, and pool parameters. Also review associated helper `setup_3pool_different_decimals`. (DONE)
- [x] **`swap_4pool_different_decimals`**: Extract numeric literals, string denoms, and pool parameters. Also review associated helper `setup_4pool_different_decimals`. (DONE)
- [x] **`simulation_vs_reverse_simulation_3pool`**: Extract numeric literals, string denoms, and test case parameters. (DONE)
- [x] **`belief_price_works_decimals_independent`**: Extract numeric literals, string denoms, and amounts. (DONE)
- [x] **`compute_offer_amount_floor`**: Extract numeric literals, string denoms, and amounts. (DONE)
- [x] **`providing_skewed_liquidity_on_stableswap_gets_punished_same_decimals`**: Extract numeric literals, string denoms, amounts, and decimals. (DONE)
- [x] **basic_swapping_test**: Extract numeric literals, string denoms, and amounts. (DONE)
- [x] **basic_swapping_pool_reserves_event_test**: Extract numeric literals, string denoms, and amounts. (DONE)
- [x] **`basic_swapping_test_stable_swap_two_assets`**: Extract numeric literals, string denoms, and amounts. (DONE)
- [x] **`swap_with_fees`**: Extract numeric literals, string denoms, amounts, and fee parameters. (DONE)
- [x] **`swap_large_digits_xyk`**: Extract numeric literals, string denoms, and amounts. (DONE)
- [x] **`swap_large_digits_stable`**: Extract numeric literals, string denoms, and amounts. (DONE)

## Progress Tracking
- Percentage Complete: 100% (14/14 tasks)

## Notes
All constants have been successfully extracted and organized at the top of the file. Each test now uses constants instead of hardcoded values, making the code more maintainable and easier to read.