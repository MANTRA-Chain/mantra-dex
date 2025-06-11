# Top 10 Critical Tests – Execution Report

This report captures console output (including the new balance snapshots) for each of the protocol's 10 most critical tests, executed with `--nocapture` so all `println!` statements appear.

## test_emergency_withdrawal

480 | impl TestingSuite {
    | ----------------- method in this implementation
...
609 |     pub(crate) fn debug_balance(&mut self, label: &str, address: &Addr, denom: &str) -> &mut Self {
    |                   ^^^^^^^^^^^^^
warning: unused variable: `i`
278 |             for (i, asset) in res.pools[0].pool_info.assets.iter().enumerate() {
    |                  ^ help: if this is intentional, prefix it with an underscore: `_i`
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

480 | impl TestingSuite {
    | ----------------- method in this implementation
...
609 |     pub(crate) fn debug_balance(&mut self, label: &str, address: &Addr, denom: &str) -> &mut Self {
    |                   ^^^^^^^^^^^^^
warning: unused variable: `i`
278 |             for (i, asset) in res.pools[0].pool_info.assets.iter().enumerate() {
    |                  ^ help: if this is intentional, prefix it with an underscore: `_i`
running 1 test
✅ position_fill_attack_is_not_possible passed

---


## emergency_withdrawal_shares_penalty_with_active_farm_owners

480 | impl TestingSuite {
    | ----------------- method in this implementation
...
609 |     pub(crate) fn debug_balance(&mut self, label: &str, address: &Addr, denom: &str) -> &mut Self {
    |                   ^^^^^^^^^^^^^
warning: unused variable: `i`
278 |             for (i, asset) in res.pools[0].pool_info.assets.iter().enumerate() {
    |                  ^ help: if this is intentional, prefix it with an underscore: `_i`
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

480 | impl TestingSuite {
    | ----------------- method in this implementation
...
609 |     pub(crate) fn debug_balance(&mut self, label: &str, address: &Addr, denom: &str) -> &mut Self {
    |                   ^^^^^^^^^^^^^
warning: unused variable: `i`
278 |             for (i, asset) in res.pools[0].pool_info.assets.iter().enumerate() {
    |                  ^ help: if this is intentional, prefix it with an underscore: `_i`
running 1 test
✅ attacker_creates_farm_positions_through_pool_manager passed

---


## test_emergency_withdrawal_with_proportional_penalty

480 | impl TestingSuite {
    | ----------------- method in this implementation
...
609 |     pub(crate) fn debug_balance(&mut self, label: &str, address: &Addr, denom: &str) -> &mut Self {
    |                   ^^^^^^^^^^^^^
warning: unused variable: `i`
278 |             for (i, asset) in res.pools[0].pool_info.assets.iter().enumerate() {
    |                  ^ help: if this is intentional, prefix it with an underscore: `_i`
running 1 test
✅ test_emergency_withdrawal_with_proportional_penalty passed

---


## change_contract_ownership

480 | impl TestingSuite {
    | ----------------- method in this implementation
...
609 |     pub(crate) fn debug_balance(&mut self, label: &str, address: &Addr, denom: &str) -> &mut Self {
    |                   ^^^^^^^^^^^^^
warning: unused variable: `i`
278 |             for (i, asset) in res.pools[0].pool_info.assets.iter().enumerate() {
    |                  ^ help: if this is intentional, prefix it with an underscore: `_i`
running 1 test
✅ change_contract_ownership passed

---


## basic_swapping_test

480 | impl TestingSuite {
    | ----------------- method in this implementation
...
609 |     pub(crate) fn debug_balance(&mut self, label: &str, address: &Addr, denom: &str) -> &mut Self {
    |                   ^^^^^^^^^^^^^
warning: unused variable: `i`
278 |             for (i, asset) in res.pools[0].pool_info.assets.iter().enumerate() {
    |                  ^ help: if this is intentional, prefix it with an underscore: `_i`
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

480 | impl TestingSuite {
    | ----------------- method in this implementation
...
609 |     pub(crate) fn debug_balance(&mut self, label: &str, address: &Addr, denom: &str) -> &mut Self {
    |                   ^^^^^^^^^^^^^
warning: unused variable: `i`
278 |             for (i, asset) in res.pools[0].pool_info.assets.iter().enumerate() {
    |                  ^ help: if this is intentional, prefix it with an underscore: `_i`
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

480 | impl TestingSuite {
    | ----------------- method in this implementation
...
609 |     pub(crate) fn debug_balance(&mut self, label: &str, address: &Addr, denom: &str) -> &mut Self {
    |                   ^^^^^^^^^^^^^
warning: unused variable: `i`
278 |             for (i, asset) in res.pools[0].pool_info.assets.iter().enumerate() {
    |                  ^ help: if this is intentional, prefix it with an underscore: `_i`
running 1 test
(creator - create_pool)
(creator - provide_liquidity)
(creator - swap)
✅ swap_with_fees passed

---


## create_farms

480 | impl TestingSuite {
    | ----------------- method in this implementation
...
609 |     pub(crate) fn debug_balance(&mut self, label: &str, address: &Addr, denom: &str) -> &mut Self {
    |                   ^^^^^^^^^^^^^
warning: unused variable: `i`
278 |             for (i, asset) in res.pools[0].pool_info.assets.iter().enumerate() {
    |                  ^ help: if this is intentional, prefix it with an underscore: `_i`
running 1 test
(other - create_farm)
(other - create_farm)
✅ create_farms passed

---

