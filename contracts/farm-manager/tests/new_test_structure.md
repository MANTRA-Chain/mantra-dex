
```
contracts/farm-manager/tests/
├── integration/
│   ├── instantiation.rs         // Tests for contract instantiation
│   ├── farm_management.rs       // Tests for creating, expanding, closing farms
│   ├── position_management.rs   // Tests for creating, expanding, closing positions
│   ├── reward_claiming.rs       // Tests related to claiming rewards
│   ├── emergency_withdrawal.rs  // Tests for emergency withdrawal scenarios
│   ├── queries.rs               // Tests for various contract queries
│   ├── ownership_and_config.rs  // Tests for ownership and configuration
│   ├── error_handling.rs        // Tests for specific error conditions and edge cases
│   ├── expiration.rs            // Tests related to farm and position expiration
│   └── common.rs                // Keep your existing common test utilities here
└── integration.rs               // This file would be removed or become a mod.rs
```

**Mapping of current functions to new files:**

*   **`instantiation.rs`**:
    *   `instantiate_farm_manager`
*   **`farm_management.rs`**:
    *   `create_farms`
    *   `expand_farms`
    *   `cant_expand_farm_too_late`
    *   `close_farms`
    *   `close_farms_wont_fail_with_malicious_tf_token`
    *   `test_farm_helper` (if primarily about farm setup/helpers)
    *   `fails_to_create_farm_if_more_tokens_than_needed_were_sent`
    *   `fails_to_create_farm_if_start_epoch_is_zero`
    *   `overriding_farm_with_bogus_id_not_possible`
    *   `providing_custom_farm_id_doesnt_increment_farm_counter`
    *   `farm_cant_be_created_in_the_past`
*   **`position_management.rs`**:
    *   `test_manage_position`
    *   `test_withdrawing_open_positions_updates_weight`
    *   `test_expand_position_unsuccessfully`
    *   `cant_create_position_with_overlapping_identifier`
    *   `test_fill_closed_position`
    *   `test_refill_position_uses_current_position_unlocking_period`
    *   `position_fill_attack_is_not_possible`
    *   `positions_can_handled_by_pool_manager_for_the_user`
    *   `test_positions_limits`
    *   `test_overwriting_position_is_not_possible`
    *   `providing_custom_position_id_doesnt_increment_position_counter`
    *   `test_managing_positions_close_and_emergency_withdraw` (Could also fit in emergency_withdrawal or be split)
*   **`reward_claiming.rs`**:
    *   `claim_expired_farm_returns_nothing`
    *   `claiming_rewards_with_multiple_positions_arent_inflated`
    *   `user_can_claim_expired_epochs`
    *   `farm_owners_get_penalty_fees`
    *   `test_claim_rewards_divide_by_zero_mitigated`
    *   `test_claim_until_epoch`
    *   `test_claim_until_epoch_closing_positions`
    *   `test_claiming_while_expanding_farm`
*   **`emergency_withdrawal.rs`**:
    *   `test_emergency_withdrawal`
    *   `test_emergency_withdrawal_penalty_only_to_active_farms`
    *   `test_emergency_withdrawal_with_proportional_penalty`
    *   `test_emergency_withdrawal_with_pending_rewards_are_lost`
    *   `emergency_withdrawal_shares_penalty_with_active_farm_owners`
    *   `can_emergency_withdraw_an_lp_without_farm`
*   **`queries.rs`**:
    *   `test_rewards_query_overlapping_farms`
    *   `test_positions_query_filters_and_pagination`
    *   `test_query_rewards_divide_by_zero`
    *   `test_query_rewards_divide_by_zero_mitigated`
*   **`ownership_and_config.rs`**:
    *   `verify_ownership`
    *   `update_config`
*   **`error_handling.rs`**:
    *   `test_farm_and_position_id_validation` (Could also be split between farm/position management)
    *   (Many tests already have error checks within them, but if there are specific complex error scenarios, they could go here)
*   **`expiration.rs`**:
    *   `test_close_expired_farms`
    *   `expand_expired_farm`
    *   `test_farm_expired`
    *   `closing_expired_farm_wont_pay_penalty`
*   **Could be in multiple or a general `complex_scenarios.rs`**:
    *   `test_multiple_farms_and_positions`
