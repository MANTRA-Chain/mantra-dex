## Refactor `swap.rs` Integration Tests: Constants Extraction

This document outlines the tasks for refactoring the integration tests in `contracts/pool-manager/src/tests/integration/swap.rs` to extract hardcoded values into constants.

- [x] **`swap_large_digits_stable_18_digits`**: Extract numeric literals and string denoms. (DONE - 2023-10-27)
- [x] **`swap_3pool_same_decimals`**: Extract numeric literals, string denoms, and pool parameters. (DONE - 2023-10-27)
- [ ] **`swap_3pool_different_decimals`**: Extract numeric literals, string denoms, and pool parameters. Also review associated helper `setup_3pool_different_decimals`. (TODO)
- [ ] **`swap_4pool_different_decimals`**: Extract numeric literals, string denoms, and pool parameters. Also review associated helper `setup_4pool_different_decimals`. (TODO)
- [ ] **`simulation_vs_reverse_simulation_3pool`**: Extract numeric literals, string denoms, and test case parameters. (TODO)
- [ ] **`belief_price_works_decimals_independent`**: Extract numeric literals, string denoms, and amounts. (TODO)
- [ ] **`compute_offer_amount_floor`**: Extract numeric literals, string denoms, and amounts. (TODO)
- [ ] **`providing_skewed_liquidity_on_stableswap_gets_punished_same_decimals`**: Extract numeric literals, string denoms, amounts, and decimals. (TODO)

## Progress Tracking
- Percentage Complete: 25% (2/8 tasks)

## Notes
- Ensure all tests pass after each refactoring step (`cargo test -p pool-manager`).
- Add new constants to the top of the `swap.rs` file.
- Reuse existing constants where applicable.
- Follow existing naming conventions for new constants. 