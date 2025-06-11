#!/usr/bin/env bash
set -euo pipefail

# Root directory (workspace root)
ROOT_DIR="$(pwd)"
REPORT_MD="$ROOT_DIR/TOP_14_TEST_RESULTS.md"

# Fresh report header
cat > "$REPORT_MD" <<'EOF'
# Top 14 Critical Tests – Execution Report

This report captures console output (including the new balance snapshots) for each of the protocol's 14 most critical tests, including 4 important stableswap operations, executed with `--nocapture` so all `println!` statements appear.
EOF

echo "Generating report at $REPORT_MD"

# Array of test identifiers (exact substrings accepted by `cargo test`)
# Top 10 original critical tests + 4 stableswap tests
TESTS=(
  test_emergency_withdrawal
  position_fill_attack_is_not_possible
  emergency_withdrawal_shares_penalty_with_active_farm_owners
  attacker_creates_farm_positions_through_pool_manager
  test_emergency_withdrawal_with_proportional_penalty
  change_contract_ownership
  basic_swapping_test
  test_manage_position
  swap_with_fees
  create_farms
  swap_large_digits_stable
  cant_create_stableswap_with_zero_amp_factor
  basic_swapping_test_stable_swap_two_assets
  providing_skewed_liquidity_on_stableswap_gets_punished_same_decimals
)

# Build once to save time
cargo test --no-run >/dev/null

echo "Starting individual test execution …"

for test_name in "${TESTS[@]}"; do
  echo "Running $test_name …"
  echo -e "\n## $test_name\n" >> "$REPORT_MD"
  
  # Run single test and filter out unwanted output while preserving actor actions
  if cargo test "$test_name" -- --nocapture 2>&1 | \
    grep -v "warning: method .* is never used" | \
    grep -v "warning: .* generated .* warning" | \
    grep -v "warning: unused variable:" | \
    grep -v "warning: function .* is never used" | \
    grep -v "warning: field .* is never used" | \
    grep -v "warning: struct .* is never used" | \
    grep -v "warning: enum .* is never used" | \
    grep -v "warning: variant .* is never used" | \
    grep -v "^\s*[0-9]* | " | \
    grep -v "^\s*| " | \
    grep -v "^\s*help: " | \
    grep -v "^\s*\.\.\." | \
    grep -v "Finished .* profile \[.*\] target(s) in" | \
    grep -v "Running unittests" | \
    grep -v "Running tests/" | \
    grep -v "^running 0 tests$" | \
    grep -v "^running [0-9]* tests$" | \
    grep -v "^test .* \.\.\. ok$" | \
    grep -v "^test .* \.\.\. FAILED$" | \
    grep -v "^test result: ok\." | \
    grep -v "^test result: FAILED\." | \
    grep -v "^Event { ty:" | \
    grep -v "^EVENT Event { ty:" | \
    grep -v "^\s*$" | \
    grep -v "^\s*|\s*$" | \
    grep -v "^\s*-->" | \
    grep -v "^\s*=" \
    >> "$REPORT_MD"; then
    echo "✅ $test_name passed" | tee -a "$REPORT_MD"
  else
    echo "❌ $test_name failed" | tee -a "$REPORT_MD"
  fi
  echo -e "\n---\n" >> "$REPORT_MD"
  echo "$test_name completed."
done

echo "All tests executed. Markdown report ready: $REPORT_MD" 