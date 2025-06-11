# Top 14 Critical Tests – Execution Report

This report captures console output (including the new balance snapshots) for each of the protocol's 14 most critical tests, including 4 important stableswap operations, executed with `--nocapture` so all `println!` statements appear.

## test_emergency_withdrawal

(other - create_farm)
(other - create_position)
[BEFORE WITHDRAW] other LP balance: 999999000
[BEFORE WITHDRAW] fee_collector LP balance: 0
(other - emergency_withdrawal)
[AFTER WITHDRAW] other LP balance: 999999950
[AFTER WITHDRAW] fee_collector LP balance: 50
✅ test_emergency_withdrawal passed

---


## position_fill_attack_is_not_possible

running 1 test
✅ position_fill_attack_is_not_possible passed

---


## emergency_withdrawal_shares_penalty_with_active_farm_owners

running 1 test
[BEFORE WITHDRAW] other LP balance: 1000000000
[BEFORE WITHDRAW] fee_collector LP balance: 0
[BEFORE WITHDRAW] alice LP balance: 1000000000
[AFTER WITHDRAW] other LP balance: 1000150000
[AFTER WITHDRAW] fee_collector LP balance: 300000
[AFTER WITHDRAW] alice LP balance: 1000150000
✅ emergency_withdrawal_shares_penalty_with_active_farm_owners passed

---


## attacker_creates_farm_positions_through_pool_manager

running 1 test
✅ attacker_creates_farm_positions_through_pool_manager passed

---


## test_emergency_withdrawal_with_proportional_penalty

running 1 test
✅ test_emergency_withdrawal_with_proportional_penalty passed

---


## change_contract_ownership

running 1 test
✅ change_contract_ownership passed

---


## basic_swapping_test

(creator - create_pool)
[BEFORE_LIQ] mantra15n2dapfyf7mzz70y0srycnduw5skp0s9u9g74e uwhale: 1000000001
[BEFORE_LIQ] mantra15n2dapfyf7mzz70y0srycnduw5skp0s9u9g74e uluna: 1000000000
(creator - provide_liquidity)
[AFTER_LIQ] mantra15n2dapfyf7mzz70y0srycnduw5skp0s9u9g74e uwhale: 999000001
[AFTER_LIQ] mantra15n2dapfyf7mzz70y0srycnduw5skp0s9u9g74e uluna: 999000000
[POOL_RESERVES] Pool assets:
  uwhale - 1000000
  uluna - 1000000
  Total LP shares: 1000000
(creator - swap: 1000 uwhale → uluna, expecting ~999 uluna)
  SWAP RESULT: Offered 1000 uwhale, Received 999 uluna
[POOL_RESERVES] After first swap:
  uwhale - 1001000
  uluna - 999001
(creator - reverse_swap: 1000 uluna → uwhale, expecting 1000 uwhale)
  SWAP RESULT: Offered 1000 uluna, Received 1000 uwhale
[POOL_RESERVES] After reverse swap:
  uwhale - 1000000
  uluna - 1000001
✅ basic_swapping_test passed

---


## test_manage_position

running 1 test
(creator - create_farm)
(creator - create_position)
thread 'integration::position_management::test_manage_position' panicked at contracts/farm-manager/tests/integration/position_management.rs:183:24:
called `Result::unwrap()` on an `Err` value: Error executing WasmMsg:
  sender: mantra15n2dapfyf7mzz70y0srycnduw5skp0s9u9g74e
  Execute { contract_addr: "mantra1qg5ega6dykkxc307y25pecuufrjkxkaggkkxh7nad0vhyhtuhw3sylwkvw", msg: {"manage_position":{"action":{"create":{"identifier":"creator_position","unlocking_duration":86400,"receiver":null}}}}, funds: [Coin { 1000 "factory/mantra1uw7lmx5muaymunqxu93f3rn35cyg50r8tz9qtun2tllq7z5dv4hqktw042/LP" }] }
Caused by:
    The position with the identifier u-creator_position already exists
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
failures:
failures:
    integration::position_management::test_manage_position
error: test failed, to rerun pass `-p farm-manager --test mod`
❌ test_manage_position failed

---


## swap_with_fees

running 1 test
(creator - create_pool)
(creator - provide_liquidity)
(creator - swap)
✅ swap_with_fees passed

---


## create_farms

running 1 test
(other - create_farm)
(other - create_farm)
✅ create_farms passed

---


## swap_large_digits_stable

❌ swap_large_digits_stable failed

---


## cant_create_stableswap_with_zero_amp_factor

running 1 test
✅ cant_create_stableswap_with_zero_amp_factor passed

---


## basic_swapping_test_stable_swap_two_assets

running 1 test
✅ basic_swapping_test_stable_swap_two_assets passed

---


## providing_skewed_liquidity_on_stableswap_gets_punished_same_decimals

running 1 test
TEST providing_skewed_liquidity_on_stableswap_gets_punished_same_decimals ===============================================================
Alice USDC balance change:
difference:       508547
initial_balance:  300000000000000000000000000000000
coin.amount:      300000000000000000000000000508547
Alice USDT balance change:
difference:       18866502
initial_balance:  300000000000000000000000000000000
coin.amount:      300000000000000000000000018866502
Alice nominal difference:  19375049
Bob USDC balance change:
difference:           -509048
initial balance:      300000000000000000000000000000000
current amount:       299999999999999999999999999490952
Bob USDT balance change:
difference:           -18867012
initial balance:      300000000000000000000000000000000
current amount:       299999999999999999999999981132988
Bob nominal difference: -19376060
amount:       501
denom:        uusdc
amount:       510
denom:        uusdt
✅ providing_skewed_liquidity_on_stableswap_gets_punished_same_decimals passed

---

