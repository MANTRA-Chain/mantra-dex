# Constant Usage Analysis Report

Analysis of directory: ../../mantra/mantra-dex/contracts/pool-manager/src/tests
Total constants found: 449
Total files scanned: 12

## Constants Used Only Once or Never (Likely Unused) - Grouped by File

### File: `../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/basic_tests.rs`

- [x] **DENOM_TEST** (used 1 times) - Removed successfully, inlined as "utest"
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/basic_tests.rs:18
- [x] **INITIAL_TEST_BALANCE** (used 1 times) - Removed successfully, inlined as 1000
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/basic_tests.rs:18

### File: `../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/lp_actions/basic_lp.rs`

- [x] **POOL_LABEL_WHALE_LUNA** (used 1 times) - Removed successfully, inlined as "whale.uluna"
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/lp_actions/basic_lp.rs:80

### File: `../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/lp_actions/locking.rs`

- [x] **POOL_CREATION_FEE_UOM_AMOUNT** (used 0 times) - Removed successfully, was commented out
- [x] **POSITION_IDENTIFIER_2** (used 1 times) - Removed successfully, was not actually used in code
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/lp_actions/locking.rs:258

### File: `../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/lp_actions/shares_calculation.rs`

- [!] **EXPECTED_LP_SHARES_RIGHT_EMISSION** (used 1 times) - Analysis error: constant was already removed or never existed
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/lp_actions/shares_calculation.rs:261
- [!] **LIQUIDITY_9K** (used 1 times) - Analysis error: constant is actually used in code
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/lp_actions/shares_calculation.rs:131
- [!] **SWAP_FEE_PERMILLE** (used 1 times) - Analysis error: constant is actually used in code
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/lp_actions/shares_calculation.rs:203
- [!] **UOM_LIQUIDITY_AMOUNT** (used 1 times) - Analysis error: constant is actually used in code
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/lp_actions/shares_calculation.rs:244
- [!] **USDC_LIQUIDITY_AMOUNT** (used 1 times) - Analysis error: constant is actually used in code
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/lp_actions/shares_calculation.rs:248

### File: `../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/lp_actions/single_asset.rs`

- [x] **CONTRACT_DUST_ULUNA** (used 1 times) - Removed successfully, inlined as 991u128
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/lp_actions/single_asset.rs:546
- [x] **CONTRACT_DUST_UWHALE** (used 1 times) - Removed successfully, inlined as 1_011u128
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/lp_actions/single_asset.rs:550
- [x] **CREATOR_REMAINING_ULUNA** (used 1 times) - Removed successfully, inlined as 9_000_000u128
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/lp_actions/single_asset.rs:381
- [x] **CREATOR_REMAINING_UWHALE** (used 1 times) - Removed successfully, inlined as 9_000_000u128
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/lp_actions/single_asset.rs:397
- [x] **CREATOR_ULUNA_AFTER_WITHDRAW** (used 1 times) - Removed successfully, inlined as 9_989_208u128
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/lp_actions/single_asset.rs:420
- [x] **CREATOR_UWHALE_AFTER_WITHDRAW** (used 1 times) - Removed successfully, was already inlined as 10_009_092u128
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/lp_actions/single_asset.rs:436
- [!] **EDGE_CASE_SINGLE_ASSET_DEPOSIT_SLIPPAGE_FAIL** (used 1 times) - Analysis error: constant does not exist in code
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/lp_actions/single_asset.rs:812
- [!] **EDGE_CASE_SINGLE_ASSET_DEPOSIT_SLIPPAGE_FAIL_LARGE** (used 1 times) - Analysis error: constant does not exist in code
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/lp_actions/single_asset.rs:832
- [!] **EDGE_CASE_SINGLE_ASSET_DEPOSIT_SUCCESS** (used 1 times) - Analysis error: constant is actually used in code
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/lp_actions/single_asset.rs:852
- [!] **FEE_COLLECTOR_ULUNA_FEES** (used 1 times) - Analysis error: constant is actually used in code
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/lp_actions/single_asset.rs:525
- [x] **FIFTEEN_PERCENT_FEE** (used 1 times) - Removed successfully, inlined as Decimal::percent(15)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/lp_actions/single_asset.rs:741
- [x] **FIVE_PERCENT_FEE** (used 1 times) - Removed successfully, inlined as Decimal::percent(5)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/lp_actions/single_asset.rs:744
- [!] **LP_TOKENS_FOR_ANOTHER_USER** (used 1 times) - Analysis error: constant is actually used in code
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/lp_actions/single_asset.rs:681
- [!] **OTHER_REMAINING_UWHALE** (used 1 times) - Analysis error: constant is actually used in code
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/lp_actions/single_asset.rs:472
- [!] **OTHER_ULUNA_AFTER_WITHDRAW** (used 1 times) - Analysis error: constant is actually used in code
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/lp_actions/single_asset.rs:495
- [!] **OTHER_UWHALE_AFTER_WITHDRAW** (used 1 times) - Analysis error: constant is actually used in code
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/lp_actions/single_asset.rs:511
- [!] **TOTAL_LP_SUPPLY_AFTER_SINGLE_ASSET_DEPOSIT** (used 1 times) - Analysis error: constant is actually used in code
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/lp_actions/single_asset.rs:324

### File: `../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/lp_actions/slippage_and_fees.rs`

- [x] **FEE_COLLECTOR_UUSD_FINAL** (used 1 times) - Removed successfully, inlined as 3_000u128 + 299u128 + 396u128
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/lp_actions/slippage_and_fees.rs:934
- [x] **FEE_PERCENT_10** (used 1 times) - Removed successfully, inlined as 10
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/lp_actions/slippage_and_fees.rs:198
- [x] **FEE_PERCENT_15** (used 1 times) - Removed successfully, inlined as 15
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/lp_actions/slippage_and_fees.rs:214
- [x] **FEE_PERCENT_3** (used 1 times) - Removed successfully, inlined as 3
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/lp_actions/slippage_and_fees.rs:204
- [x] **FEE_PERCENT_5** (used 1 times) - Removed successfully, inlined as 5
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/lp_actions/slippage_and_fees.rs:217
- [x] **FEE_PERCENT_7** (used 1 times) - Removed successfully, inlined as 7
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/lp_actions/slippage_and_fees.rs:201
- [x] **LIQUIDITY_200K** (used 1 times) - Removed successfully, inlined as 200_000u128
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/lp_actions/slippage_and_fees.rs:168
- [x] **PM_ULUNA_BAL_AFTER_ROUTER** (used 1 times) - Removed successfully, inlined as 3_003_570u128
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/lp_actions/slippage_and_fees.rs:962
- [x] **PM_UUSD_BAL_AFTER_ROUTER** (used 1 times) - Removed successfully, inlined as 995_035u128
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/lp_actions/slippage_and_fees.rs:966
- [x] **PM_UWHALE_BAL_AFTER_ROUTER** (used 1 times) - Removed successfully, inlined as 2_003_440u128
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/lp_actions/slippage_and_fees.rs:970
- [x] **POOL_1_LUNA_AFTER_SWAP_1** (used 1 times) - Removed successfully, inlined as 999_070u128
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/lp_actions/slippage_and_fees.rs:450
- [x] **POOL_1_WHALE_AFTER_SWAP_1** (used 1 times) - Removed successfully, inlined as 1_001_000u128
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/lp_actions/slippage_and_fees.rs:449
- [ ] **POOL_2_LUNA_AFTER_ROUTER_SWAP** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/lp_actions/slippage_and_fees.rs:879
- [ ] **POOL_2_LUNA_AFTER_SWAP_1** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/lp_actions/slippage_and_fees.rs:568
- [ ] **POOL_2_LUNA_AFTER_SWAP_2** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/lp_actions/slippage_and_fees.rs:616
- [ ] **POOL_2_WHALE_AFTER_ROUTER_SWAP** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/lp_actions/slippage_and_fees.rs:878
- [ ] **POOL_2_WHALE_AFTER_SWAP_1** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/lp_actions/slippage_and_fees.rs:567
- [ ] **POOL_2_WHALE_AFTER_SWAP_2** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/lp_actions/slippage_and_fees.rs:615
- [ ] **POOL_3_LUNA_AFTER_ROUTER_SWAP** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/lp_actions/slippage_and_fees.rs:909
- [ ] **POOL_3_LUNA_AFTER_SWAP_1** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/lp_actions/slippage_and_fees.rs:680
- [ ] **POOL_3_LUNA_AFTER_SWAP_2** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/lp_actions/slippage_and_fees.rs:736
- [ ] **POOL_3_UUSD_AFTER_ROUTER_SWAP** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/lp_actions/slippage_and_fees.rs:910
- [ ] **POOL_3_UUSD_AFTER_SWAP_1** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/lp_actions/slippage_and_fees.rs:681
- [ ] **POOL_3_UUSD_AFTER_SWAP_2** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/lp_actions/slippage_and_fees.rs:737
- [ ] **POOL_MANAGER_ULUNA_BALANCE_3M** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/lp_actions/slippage_and_fees.rs:397
- [ ] **POOL_MANAGER_ULUNA_FINAL** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/lp_actions/slippage_and_fees.rs:790
- [ ] **POOL_MANAGER_UUSD_BALANCE_1M** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/lp_actions/slippage_and_fees.rs:401
- [ ] **POOL_MANAGER_UUSD_FINAL** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/lp_actions/slippage_and_fees.rs:794
- [ ] **POOL_MANAGER_UWHALE_BALANCE_2M** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/lp_actions/slippage_and_fees.rs:405
- [ ] **POOL_MANAGER_UWHALE_FINAL** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/lp_actions/slippage_and_fees.rs:798
- [x] **SLIPPAGE_PERCENT_20** (used 1 times) - Removed successfully, inlined as 20
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/lp_actions/slippage_and_fees.rs:417
- [x] **SLIPPAGE_PERCENT_60** (used 1 times) - Removed successfully, inlined as 60
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/lp_actions/slippage_and_fees.rs:158
- [x] **SWAP_AMOUNT_1_5K** (used 1 times) - Removed successfully, inlined as 1_500u128
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/lp_actions/slippage_and_fees.rs:707
- [x] **SWAP_AMOUNT_3K** (used 1 times) - Removed successfully, inlined as 3_000u128
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/lp_actions/slippage_and_fees.rs:651
- [x] **SWAP_AMOUNT_5K** (used 1 times) - Removed successfully, inlined as 5_000u128
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/lp_actions/slippage_and_fees.rs:825
- [ ] **ULUNA_UUSD_POOL_1_LABEL** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/lp_actions/slippage_and_fees.rs:262
- [ ] **WHALE_ULUNA_POOL_1_LABEL** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/lp_actions/slippage_and_fees.rs:232
- [ ] **WHALE_ULUNA_POOL_2_LABEL** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/lp_actions/slippage_and_fees.rs:247

### File: `../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/lp_actions/stableswap.rs`

- [x] **EXPECTED_LP_AMOUNT_FIRST** (used 1 times) - Removed successfully, inlined as calculation
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/lp_actions/stableswap.rs:430
- [x] **EXPECTED_LP_AMOUNT_SECOND** (used 1 times) - Removed successfully, inlined as calculation
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/lp_actions/stableswap.rs:466
- [x] **UUSDC_UUSDT_UUSDY_POOL_LABEL** (used 1 times) - Removed successfully, inlined as "uusdc.uusdt.uusdy"
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/lp_actions/stableswap.rs:386
- [x] **WHALE_ULUNA_UUSD_POOL_LABEL** (used 1 times) - Removed successfully, inlined as "whale.uluna.uusd"
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/lp_actions/stableswap.rs:93

### File: `../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/ownership.rs`

- [x] **POOL_CREATION_FEE_INCREMENT** (used 1 times) - Removed successfully, inlined as 1u32
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/ownership.rs:114

### File: `../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/pool_management.rs`

- [ ] **CUSTOM_POOL_ID_1** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/pool_management.rs:1784
- [ ] **CUSTOM_POOL_ID_2** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/pool_management.rs:1799
- [ ] **CUSTOM_POOL_PREFIX_1** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/pool_management.rs:1828
- [ ] **CUSTOM_POOL_PREFIX_2** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/pool_management.rs:1832
- [ ] **DENOM_FACTORY** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/pool_management.rs:812
- [ ] **INSUFFICIENT_POOL_CREATION_AMOUNT** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/pool_management.rs:1247
- [ ] **INSUFFICIENT_POOL_CREATION_FEE** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/pool_management.rs:139
- [ ] **INVALID_POOL_IDENTIFIER_DASH** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/pool_management.rs:847
- [ ] **INVALID_POOL_IDENTIFIER_LONG** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/pool_management.rs:869
- [ ] **LEGIT_POSITION_ID** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/pool_management.rs:1449
- [ ] **LIQUIDITY_AMOUNT_UOM** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/pool_management.rs:527
- [ ] **LIQUIDITY_AMOUNT_UUSD** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/pool_management.rs:528
- [ ] **LOCK_POOL_ID_1** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/pool_management.rs:1877
- [ ] **LOCK_POOL_ID_2** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/pool_management.rs:1892
- [ ] **LOCK_POOL_SWAP_AMOUNT_2** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/pool_management.rs:2109
- [ ] **MOCK_AMOUNT_ULUNA** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/pool_management.rs:1179
- [ ] **POOL_ID_PREFIX_O** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/pool_management.rs:1558
- [ ] **SINGLE_SIDED_LP_PERCENT** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/pool_management.rs:1430
- [ ] **TOGGLE_INVALID_POOL_ID** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/pool_management.rs:2182
- [ ] **TOGGLE_POOL_ID** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/pool_management.rs:2166
- [ ] **WHALE_LUNA_POOL_RAW_ID** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/pool_management.rs:1355

### File: `../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/query.rs`

- [ ] **DESIRED_OUTPUT_AMOUNT** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/query.rs:996
- [ ] **EXPECTED_BURN_FEE_USDC** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/query.rs:527
- [ ] **EXPECTED_BURN_FEE_USDC_REV** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/query.rs:1044
- [ ] **EXPECTED_BURN_FEE_USDT** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/query.rs:528
- [ ] **EXPECTED_BURN_FEE_USDT_REV** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/query.rs:1045
- [ ] **EXPECTED_EXTRA_FEE_USDC** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/query.rs:534
- [ ] **EXPECTED_EXTRA_FEE_USDC_REV** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/query.rs:1051
- [ ] **EXPECTED_EXTRA_FEE_USDT** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/query.rs:535
- [ ] **EXPECTED_EXTRA_FEE_USDT_REV** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/query.rs:1052
- [ ] **EXPECTED_OFFER_AMOUNT_TOLERANCE** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/query.rs:1018
- [ ] **EXPECTED_PROTOCOL_FEE_USDC** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/query.rs:520
- [ ] **EXPECTED_PROTOCOL_FEE_USDC_REV** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/query.rs:1037
- [ ] **EXPECTED_PROTOCOL_FEE_USDT** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/query.rs:521
- [ ] **EXPECTED_PROTOCOL_FEE_USDT_REV** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/query.rs:1038
- [ ] **EXPECTED_RETURN_AMOUNT** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/query.rs:501
- [ ] **EXPECTED_RETURN_AMOUNT_TOLERANCE** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/query.rs:1100
- [ ] **EXPECTED_SLIPPAGE_AMOUNT** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/query.rs:297
- [ ] **EXPECTED_SLIPPAGE_USDC** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/query.rs:506
- [ ] **EXPECTED_SLIPPAGE_USDC_REV** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/query.rs:1023
- [ ] **EXPECTED_SLIPPAGE_USDT** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/query.rs:507
- [ ] **EXPECTED_SLIPPAGE_USDT_REV** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/query.rs:1024
- [ ] **EXPECTED_SWAP_FEE_USDC** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/query.rs:513
- [ ] **EXPECTED_SWAP_FEE_USDC_REV** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/query.rs:1030
- [ ] **EXPECTED_SWAP_FEE_USDT** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/query.rs:514
- [ ] **EXPECTED_SWAP_FEE_USDT_REV** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/query.rs:1031
- [ ] **REVERSE_SIMULATION_SLIPPAGE_PERCENT** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/query.rs:755

### File: `../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/router.rs`

- [x] **INITIAL_SIMULATION_TOLERANCE** (used 1 times) - Removed successfully, inlined as "0.006"
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/router.rs:1039
- [x] **LARGE_SLIPPAGE_TOLERANCE** (used 1 times) - Removed successfully, inlined as 5
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/router.rs:1068
- [x] **REVERSE_SIMULATION_EXPECTED_AMOUNT** (used 1 times) - Removed successfully, inlined as 1_007
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/router.rs:1083
- [x] **REVERSE_SIMULATION_TOLERANCE** (used 1 times) - Removed successfully, inlined as "0.1"
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/router.rs:1084
- [x] **SIMULATED_RESULT_AFTER_PRICE_CHANGE** (used 1 times) - Removed successfully, inlined as 935
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/router.rs:1097
- [x] **SLIPPAGE_TOLERANCE** (used 1 times) - Removed successfully, inlined as 2
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/router.rs:205

### File: `../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/swap.rs`

- [ ] **BALANCE_UOM_150T_STABLE** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/swap.rs:1405
- [ ] **BALANCE_UUSDC_100T_XYK** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/swap.rs:1213
- [ ] **BASIC_SWAP_INITIAL_ULUNA_BALANCE** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/swap.rs:302
- [ ] **BASIC_SWAP_INITIAL_UOM_BALANCE** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/swap.rs:309
- [ ] **BASIC_SWAP_INITIAL_UUSD_BALANCE** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/swap.rs:306
- [ ] **BASIC_SWAP_INITIAL_UWHALE_BALANCE** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/swap.rs:298
- [ ] **BOB_INITIAL_LIQ_USDC_BASE_SKEWED_TEST** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/swap.rs:3367
- [ ] **BOB_INITIAL_LIQ_USDT_BASE_SKEWED_TEST** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/swap.rs:3365
- [ ] **DECIMAL_PERCENT_20** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/swap.rs:1746
- [ ] **DEFAULT_AMP_3POOL** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/swap.rs:3046
- [ ] **DEFAULT_INITIAL_BALANCE_BASE_3POOL** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/swap.rs:3051
- [ ] **DEFAULT_LIQUIDITY_VALUE_1_3POOL** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/swap.rs:3063
- [ ] **DEFAULT_LIQUIDITY_VALUE_2_3POOL** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/swap.rs:3064
- [ ] **DEFAULT_LIQUIDITY_VALUE_3_3POOL** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/swap.rs:3065
- [ ] **EVENT_KEY_ADDED_SHARES** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/swap.rs:391
- [!] **EVENT_KEY_OFFER_AMOUNT** (used 0 times) - Analysis error: constant is actually used in code
- [!] **EVENT_KEY_RETURN_AMOUNT** (used 0 times) - Analysis error: constant is actually used in code
- [ ] **EXPECTED_FEE_COLLECTOR_BALANCE_ULUNA_U128** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/swap.rs:1186
- [ ] **EXPECTED_LP_SHARES_XYK** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/swap.rs:1295
- [ ] **INITIAL_BALANCE_ULUNA_COMPUTE_OFFER** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/swap.rs:2914
- [ ] **INITIAL_BALANCE_UOM_BELIEF_PRICE** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/swap.rs:2771
- [ ] **INITIAL_BALANCE_UOM_COMPUTE_OFFER** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/swap.rs:2922
- [ ] **INITIAL_BALANCE_UUSD_BELIEF_PRICE** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/swap.rs:2763
- [ ] **INITIAL_BALANCE_UUSD_COMPUTE_OFFER** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/swap.rs:2918
- [ ] **INITIAL_BALANCE_UWETH_BELIEF_PRICE** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/swap.rs:2767
- [ ] **LIQUIDITY_AUSDY_100Q_XYK** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/swap.rs:1279
- [ ] **LIQUIDITY_ULUNA_COMPUTE_OFFER** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/swap.rs:2975
- [ ] **LIQUIDITY_UOM_150T_XYK** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/swap.rs:1275
- [ ] **LIQUIDITY_UUSD_BELIEF_PRICE** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/swap.rs:2823
- [ ] **LIQUIDITY_UUSD_COMPUTE_OFFER** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/swap.rs:2979
- [ ] **LIQUIDITY_UWETH_BELIEF_PRICE** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/swap.rs:2827
- [ ] **NEEDED_UUSD_COMPUTE_OFFER** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/swap.rs:2988
- [ ] **OFFER_10M_UWHALE_U128** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/swap.rs:1148
- [ ] **ONE_BPS_DECIMAL_TOLERANCE** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/swap.rs:2718
- [ ] **POOL_RESERVES_INITIAL_ULUNA_BALANCE** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/swap.rs:541
- [ ] **POOL_RESERVES_INITIAL_UOM_BALANCE** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/swap.rs:549
- [ ] **POOL_RESERVES_INITIAL_UUSD_BALANCE** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/swap.rs:545
- [ ] **POOL_RESERVES_INITIAL_UWHALE_BALANCE** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/swap.rs:537
- [!] **POOL_RESERVES_KEY_POOL_IDENTIFIER** (used 0 times) - Analysis error: constant is actually used in code
- [!] **POOL_RESERVES_KEY_POOL_RESERVES** (used 0 times) - Analysis error: constant is actually used in code
- [ ] **SIM_TEST_AMOUNT_UUSDC_AS_OFFER_FOR_UUSDT** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/swap.rs:2648
- [ ] **SIM_TEST_AMOUNT_UUSDT_AS_OFFER_FOR_UUSD** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/swap.rs:2654
- [ ] **SIM_TEST_AMOUNT_UUSD_AS_OFFER_FOR_UUSDC** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/swap.rs:2643
- [ ] **SIM_TEST_AMOUNT_UUSD_AS_OFFER_FOR_UUSDT** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/swap.rs:2660
- [ ] **STABLE_SWAP_AMP_85** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/swap.rs:1815
- [ ] **STABLE_SWAP_PROTOCOL_FEE_SHARE** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/swap.rs:868
- [ ] **STABLE_SWAP_SWAP_FEE_SHARE** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/swap.rs:871
- [ ] **STABLE_SWAP_TWO_ASSETS_INITIAL_ULUNA_BALANCE** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/swap.rs:841
- [ ] **STABLE_SWAP_TWO_ASSETS_INITIAL_UOM_BALANCE** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/swap.rs:849
- [ ] **STABLE_SWAP_TWO_ASSETS_INITIAL_UUSD_BALANCE** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/swap.rs:845
- [ ] **STABLE_SWAP_TWO_ASSETS_INITIAL_UWHALE_BALANCE** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/swap.rs:837
- [ ] **SWAP_200T_PICO_UUSDC_3POOL_DIFFERENT_DECIMALS** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/swap.rs:1913
- [ ] **SWAP_4POOL_VAL1** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/swap.rs:2096
- [ ] **SWAP_4POOL_VAL2** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/swap.rs:2276
- [ ] **SWAP_4POOL_VAL3** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/swap.rs:2327
- [ ] **SWAP_4POOL_VAL4** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/swap.rs:2378
- [ ] **SWAP_4POOL_VAL5** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/swap.rs:2429
- [ ] **SWAP_4POOL_VAL6** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/swap.rs:2480
- [ ] **SWAP_4POOL_VAL7** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/swap.rs:2531
- [ ] **SWAP_4POOL_VAL8** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/swap.rs:2582
- [ ] **SWAP_FEE_PERCENT_5_SKEWED_LIQ_TEST** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/swap.rs:3305
- [ ] **SWAP_FEE_PERMILLE_30** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/swap.rs:1234
- [ ] **SWAP_FEE_SHARE_FOR_FEES_TEST** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/swap.rs:1076
- [ ] **SWAP_LARGE_DIGITS_STABLE_MAX_SLIPPAGE_1** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/swap.rs:1522
- [ ] **SWAP_LARGE_DIGITS_STABLE_MAX_SLIPPAGE_2** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/swap.rs:1569
- [ ] **SWAP_LARGE_DIGITS_XYK_MAX_SLIPPAGE** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/swap.rs:1325
- [ ] **SWAP_LARGE_DIGITS_XYK_MAX_SLIPPAGE_2** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/swap.rs:1372
- [ ] **SWAP_OFFER_1000_UUSD_U128** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/swap.rs:666
- [ ] **SWAP_WITH_FEES_ASSERT_APPROX_TOLERANCE** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/swap.rs:1173
- [ ] **SWAP_WITH_FEES_INITIAL_ULUNA_BALANCE** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/swap.rs:1046
- [ ] **SWAP_WITH_FEES_INITIAL_UOM_BALANCE** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/swap.rs:1054
- [ ] **SWAP_WITH_FEES_INITIAL_UUSD_BALANCE** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/swap.rs:1050
- [ ] **SWAP_WITH_FEES_INITIAL_UWHALE_BALANCE** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/swap.rs:1042
- [ ] **UOM_LIQUIDITY_RESERVES** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/swap.rs:620
- [ ] **UOM_UUSD_POOL_RAW** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/swap.rs:585
- [ ] **UUSDC_UUSDT_POOL_RAW** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/swap.rs:3322
- [ ] **UUSD_LIQUIDITY_RESERVES_UOM_POOL** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/swap.rs:624
- [ ] **UUSD_UWETH_POOL_RAW** (used 1 times)
  - Usage locations: ../../mantra/mantra-dex/contracts/pool-manager/src/tests/integration/swap.rs:2803

## Summary Statistics

Usage distribution:

- Used 0 times: 5 constants
- Used 1 times: 196 constants
- Used 2 times: 73 constants
- Used 3 times: 35 constants
- Used 4 times: 33 constants
- Used 5 times: 18 constants
- Used 6 times: 26 constants
- Used 7 times: 6 constants
- Used 8 times: 5 constants
- Used 9 times: 11 constants
- Used 10 times: 7 constants
- Used 11 times: 2 constants
- Used 12 times: 3 constants
- Used 13 times: 2 constants
- Used 14 times: 2 constants
- Used 15 times: 2 constants
- Used 16 times: 2 constants
- Used 17 times: 1 constants
- Used 20 times: 1 constants
- Used 21 times: 1 constants
- Used 22 times: 1 constants
- Used 24 times: 1 constants
- Used 25 times: 1 constants
- Used 28 times: 1 constants
- Used 34 times: 1 constants
- Used 39 times: 1 constants
- Used 42 times: 1 constants
- Used 47 times: 1 constants
- Used 48 times: 1 constants
- Used 49 times: 1 constants
- Used 56 times: 1 constants
- Used 59 times: 1 constants
- Used 65 times: 1 constants
- Used 80 times: 1 constants
- Used 81 times: 1 constants
- Used 82 times: 1 constants
- Used 99 times: 1 constants
- Used 113 times: 1 constants

## Instructions for AI Coder Agent

To track progress when removing constants:

1. Check the box `[ ]` → `[x]` when you've successfully removed a constant
2. Add notes after the checkbox if needed (e.g., `[x] Removed - was only used in tests`)
3. If a constant cannot be removed, change to `[!]` and add explanation
4. Use `[?]` if you need human review for a particular constant

Example:

- [x] **SOME_CONSTANT** (used 1 times) - Removed successfully
- [!] **OTHER_CONSTANT** (used 0 times) - Cannot remove, used in macro
- [?] **THIRD_CONSTANT** (used 1 times) - Needs human review
