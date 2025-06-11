# Top 10 Critical Tests for Mantra DEX Protocol

This document outlines the 10 most critical tests for the Mantra DEX protocol, ranked by security and stability importance (95-100). Each test includes implementation details, parameters, and replication instructions for automated testing.

---

## 🔴 CRITICAL SECURITY TESTS

### 1. Emergency Withdrawal Test
- [ ] **Test Name**: `test_emergency_withdrawal`
- **Location**: `contracts/farm-manager/tests/integration/emergency_withdrawal.rs:63`
- **Importance**: 100/100
- **Category**: Fund Safety & Crisis Management

**Description**: Tests the emergency withdrawal mechanism that allows users to immediately withdraw their funds during crisis situations, bypassing normal withdrawal procedures.

**Parameters**:
- User account with active farming positions
- Farm with locked liquidity
- Emergency trigger conditions
- Minimum withdrawal amount thresholds

**Numeric Values & Denominations**:
- Farm funding: **4,000 `uUSDY`** sent during `FarmAction::Fill`
- Farm fee deposit: **1,000 `uOM`** (covers contract fee)
- LP locked per position: **1,000 `LP` tokens** (`factory/{addr}/{LP_SYMBOL}`)
- Penalty rate: **10 %** → **100 LP** deducted
- Penalty split: **50 LP** returned to farm owner, **50 LP** to `fee_collector`

**Test Steps**:
1. Initialize farm manager contract
2. Create active farm with reward distribution
3. User deposits LP tokens and creates farming position
4. Trigger emergency withdrawal condition
5. Execute emergency withdrawal
6. Verify immediate fund release to user
7. Validate remaining farm state consistency

**Expected Outcomes**:
- User receives their proportional share immediately
- Farm state updates correctly
- No funds are lost or locked
- Emergency withdrawal event is emitted

**State Snapshot (Before → After)**
| Entity | LP Balance | Farm Status | Note |
|--------|------------|-------------|------|
| `other` (user) | 1 000 000 000 → 999 999 000 (after lock) → **999 999 950** (after emergency withdraw) | — | Receives 900 LP back + 50 LP penalty share |
| `fee_collector` | 0 → **50 LP** | — | Receives half of 10 % penalty |
| Farm `farm` | N/A | **Active** → remains active | 1 000 LP withdrawn, reward pot unchanged |

**AI Agent Prompt**: 
*"Implement an emergency withdrawal test that simulates a crisis scenario where users need immediate access to their farmed LP tokens. The test should validate that the emergency mechanism works correctly, funds are released immediately, and the farm state remains consistent after emergency withdrawals."*

---

### 2. Position Fill Attack Prevention
- [ ] **Test Name**: `position_fill_attack_is_not_possible`
- **Location**: `contracts/farm-manager/tests/integration/position_management.rs:1903`
- **Importance**: 99/100
- **Category**: Attack Prevention & Security

**Description**: Prevents sophisticated attacks where malicious actors attempt to manipulate position filling mechanisms to drain rewards or manipulate farming calculations.

**Parameters**:
- Attacker account with initial funds
- Target farm with available positions
- Normal user accounts for comparison
- Various attack vectors (rapid position creation/deletion, timing attacks)

**Numeric Values & Denominations**:
- Farm reward deposit: **8,000 `uUSDY`** (creator funds farm)
- Legitimate user stake: **5,000 `LP` tokens**
- Attacker attempts: **100× positions** each with **1 LP** (total 100 LP)
- Victim unlocking period: **86,400 s (1 day)**
- Attacker unlocking period: **31,556,926 s (~1 year)**

**Test Steps**:
1. Setup farm with reward distribution
2. Create legitimate user positions as baseline
3. Simulate attacker attempting rapid position manipulation
4. Test timing-based attacks on position filling
5. Attempt to overflow position calculations
6. Verify attack prevention mechanisms activate
7. Confirm legitimate users remain unaffected

**Expected Outcomes**:
- All attack attempts fail gracefully
- Position calculations remain accurate
- Reward distribution is not manipulated
- Legitimate users are protected

**State Snapshot (Before → After)**
| Entity | LP Positions | LP Balance Impact |
|--------|--------------|-------------------|
| `victim` | 1 position @ 5 000 LP | unchanged |
| `attacker` | 0 legitimate → 0 (all 100 attempts rejected) | -100 LP attempted payment refunded |
| Farm | Funded with 8 000 uUSDY | unaffected |

**AI Agent Prompt**: 
*"Create a comprehensive attack simulation test that attempts various position manipulation strategies including rapid position creation/deletion, timing attacks, and calculation overflow attempts. Ensure all attack vectors are properly blocked while legitimate position operations continue to work correctly."*

---

### 3. Emergency Withdrawal Penalty Distribution
- [ ] **Test Name**: `emergency_withdrawal_shares_penalty_with_active_farm_owners`
- **Location**: `contracts/farm-manager/tests/integration/emergency_withdrawal.rs:284`
- **Importance**: 98/100
- **Category**: Economic Security & Fair Distribution

**Description**: Tests the penalty mechanism during emergency withdrawals, ensuring penalties are fairly distributed among remaining active farm participants.

**Parameters**:
- Multiple farm owners with varying stake sizes
- Emergency withdrawal penalty percentage
- Active vs inactive farm positions
- Penalty distribution calculations

**Numeric Values & Denominations**:
- Bob's locked amount: **6,000,000 `LP` tokens**
- Penalty rate: **10 %** → **600,000 LP** deducted
  • **300,000 LP** (50 %) to `fee_collector`
  • **150,000 LP** to each active farm owner (`other`, `alice`)
- Each farm funded with: **4,000 `uUSDY`** + **1,000 `uOM`** fee

**Test Steps**:
1. Setup farm with multiple active owners
2. Create varying position sizes for different users
3. Trigger emergency withdrawal by one user
4. Calculate expected penalty amounts
5. Execute emergency withdrawal with penalty
6. Verify penalty distribution among remaining owners
7. Validate proportional penalty calculations

**Expected Outcomes**:
- Penalties are distributed proportionally
- Active farm owners receive appropriate compensation
- Inactive positions don't receive unearned penalties
- Mathematical precision is maintained

**State Snapshot (Before → After)**
| Entity | LP Balance | Note |
|--------|------------|------|
| `bob` (withdrawing) | 1 000 000 000 → 1 000 000 000 – 6 000 000 = 994 000 000 (lock) → **0** (position closed) |
| `alice` | 1 000 000 000 → **1 000 150 000** (+150 000 share) |
| `other` | 1 000 000 000 → **1 000 150 000** (+150 000 share) |
| `fee_collector` | 0 → **300 000 LP** |
| Farm Manager escrow | 6 000 000 → **0** |

**AI Agent Prompt**: 
*"Implement a test that validates the penalty distribution mechanism during emergency withdrawals. The test should ensure penalties are calculated correctly and distributed fairly among active farm participants, preventing any economic exploits or unfair advantage scenarios."*

---

### 4. Unauthorized Farm Position Creation Prevention
- [ ] **Test Name**: `attacker_creates_farm_positions_through_pool_manager`
- **Location**: `contracts/pool-manager/src/tests/integration/pool_management.rs:1307`
- **Importance**: 98/100
- **Category**: Access Control & Authorization

**Description**: Prevents attackers from bypassing normal authorization to create farm positions through the pool manager, which could lead to unauthorized reward claims.

**Parameters**:
- Unauthorized user account
- Valid pool manager contract
- Farm creation permissions
- Authorization bypass attempts

**Numeric Values & Denominations**:
- Pool creation fee: **1,000 `uUSD`** + **8,888 `uOM`** (token-factory fee)
- Initial liquidity provided by creator: **1,000,000 `uWHALE`** + **1,000,000 `uLUNA`**
- Attacker's unauthorized liquidity attempt: **1,000,000 `uWHALE`** + **1,000,000 `uLUNA`**
- Denoms in scope: `uWHALE`, `uLUNA`, `uUSD`, `uOM`

**Test Steps**:
1. Deploy pool manager and farm manager contracts
2. Setup proper authorization chains
3. Attempt unauthorized farm position creation via pool manager
4. Test various bypass methods (direct calls, proxy contracts)
5. Verify authorization checks are enforced
6. Confirm only authorized entities can create positions
7. Validate error messages and access denials

**Expected Outcomes**:
- All unauthorized attempts are rejected
- Proper error messages are returned
- Authorization system remains intact
- Legitimate authorized operations still work

**State Snapshot (Before → After)**
| Entity | Positions | LP Balance |
|--------|-----------|------------|
| `attacker` | tries 2 unauthorized + 1 legit | −1 000 000 (legit provide) |
| `victim` | 0 → 0 | no unwanted positions |
| Pool `o.whale.uluna` total LP | 999 000 (initial) → 1 998 000 (after creator & attacker legit adds) |

**AI Agent Prompt**: 
*"Create a security test that attempts to bypass authorization controls to create farm positions through the pool manager. Test various attack vectors including direct unauthorized calls and proxy contract attempts, ensuring all unauthorized access is properly blocked."*

---

### 5. Proportional Penalty Emergency Withdrawal
- [ ] **Test Name**: `test_emergency_withdrawal_with_proportional_penalty`
- **Location**: `contracts/farm-manager/tests/integration/emergency_withdrawal.rs:426`
- **Importance**: 97/100
- **Category**: Economic Integrity & Fair Penalties

**Description**: Validates that proportional penalty calculations during emergency withdrawals are mathematically correct and economically fair.

**Parameters**:
- User positions of varying sizes
- Emergency withdrawal penalty rates
- Proportional calculation parameters
- Minimum penalty thresholds

**Numeric Values & Denominations**:
- Each farm funded with: **4,000 `uUSDY`** + **1,000 `uOM`** fee
- Locked per position: **1,000 LP** (two separate LP denoms)
- Early withdrawal A: **10 % penalty** → **100 LP** (50 to user as farm owner, 50 to fee collector)
- Early withdrawal B (max-penalty case): **90 % penalty** → **900 LP** (all to fee collector due to inactive farm)

**Test Steps**:
1. Create farm with multiple user positions of different sizes
2. Configure emergency withdrawal penalty parameters
3. Trigger emergency withdrawal for medium-sized position
4. Calculate expected proportional penalty
5. Execute withdrawal and measure actual penalty
6. Verify proportional distribution to remaining users
7. Test edge cases (very small/large positions)

**Expected Outcomes**:
- Penalty calculations are mathematically accurate
- Proportional distribution is fair and precise
- Edge cases are handled correctly
- No precision loss in calculations

**State Snapshot (Before → After)**
| Step | `other` LP Balance | `creator` LP Balance | `fee_collector` | Comment |
|------|-------------------|----------------------|-----------------|---------|
| After lock (2×1 000 LP) | 999 999 000 (lp1) / 999 999 000 (lp2) | 1 000 000 000 | 0 | two positions open |
| Early withdraw (closed pos half-unlock) | **999 999 950** | 1 000 000 000 | 50 LP | 10 % penalty, farm active |
| Max-penalty withdraw | **999 999 100** | 1 000 000 000 | +900 LP (total 950) | 90 % penalty (inactive farm) |

**AI Agent Prompt**: 
*"Implement a test that validates proportional penalty calculations during emergency withdrawals. Ensure mathematical precision, test edge cases with very small and large positions, and verify that the proportional distribution mechanism is fair and accurate across different scenarios."*

---

## 🟠 HIGH PRIORITY CORE FUNCTIONALITY

### 6. Contract Ownership Management
- [ ] **Test Name**: `change_contract_ownership`
- **Location**: `contracts/fee-collector/tests/integration.rs:17`
- **Importance**: 96/100
- **Category**: Access Control & Governance

**Description**: Tests the secure transfer of contract ownership, which is fundamental to protocol governance and security.

**Parameters**:
- Current owner account
- New owner candidate account
- Ownership transfer process
- Permission validation checks

**Numeric Values & Denominations**:
- No token transfers; operation involves **owner addresses only** (`admin` ➜ `alice`)

**Test Steps**:
1. Deploy contract with initial owner
2. Verify current owner permissions
3. Initiate ownership transfer to new account
4. Validate transfer process (pending/confirmed states)
5. Confirm new owner has full permissions
6. Verify old owner permissions are revoked
7. Test unauthorized ownership transfer attempts

**Expected Outcomes**:
- Ownership transfer completes successfully
- New owner has full contract control
- Old owner loses all privileged access
- Unauthorized transfers are rejected

**State Snapshot (Before → After)**
| Contract | Owner |
|----------|-------|
| Fee Collector | `admin` → `alice` |

**AI Agent Prompt**: 
*"Create a comprehensive ownership transfer test that validates the secure handover of contract control. Test both successful transfers and unauthorized attempts, ensuring the ownership mechanism is secure and foolproof."*

---

### 7. Basic DEX Swapping Functionality
- [ ] **Test Name**: `basic_swapping_test`
- **Location**: `contracts/pool-manager/src/tests/integration/swap.rs:172`
- **Importance**: 94/100
- **Category**: Core DEX Functionality

**Description**: Tests the fundamental swapping mechanism that forms the core of the DEX protocol.

**Parameters**:
- Asset pairs for swapping
- Swap amounts (input/output)
- Pool liquidity levels
- Slippage tolerances

**Numeric Values & Denominations**:
- Liquidity added: **1,000,000 `uWHALE`** + **1,000,000 `uLUNA`** (6-decimals)
- Swap offer: **1,000 `uWHALE`** ➜ expected return ≈ **1,000 `uLUNA`** minus spread/fees
- Protocol fee: **0.01 %**, Swap fee: **0.02 %**, Burn fee: **0 %**

**Test Steps**:
1. Initialize pool with two assets and liquidity
2. Setup user account with input tokens
3. Execute swap transaction with specified parameters
4. Verify token balances before and after swap
5. Validate pool reserve updates
6. Check swap rate calculations
7. Confirm slippage is within acceptable bounds

**Expected Outcomes**:
- Swap executes successfully
- Token balances update correctly
- Pool reserves reflect the swap
- Swap rates are calculated accurately

**State Snapshot (Before → After)**
| Pool Reserves (`uWHALE`,`uLUNA`) | LP Supply |
|---------------------------------|-----------|
| 1 000 000 / 1 000 000 → 999 000 / 1 000 999 | 1 000 000 (unchanged) |
| `creator` balances | 1 000 000 001 → 999 999 001 `uWHALE`  | +≈1 000 `uLUNA` − fees |

**AI Agent Prompt**: 
*"Implement a basic swap test that validates the core DEX functionality. Test token swapping between asset pairs, verify balance updates, pool reserve changes, and ensure swap rate calculations are accurate and within expected slippage bounds."*

---

### 8. Position Management Core
- [ ] **Test Name**: `test_manage_position`
- **Location**: `contracts/farm-manager/tests/integration/position_management.rs:59`
- **Importance**: 93/100
- **Category**: Asset Management & User Experience

**Description**: Tests the core position management functionality that allows users to create, modify, and close their farming positions.

**Parameters**:
- User account with LP tokens
- Farm with available capacity
- Position size parameters
- Management operation types (create/modify/close)

**Numeric Values & Denominations**:
- Initial stake per position: **1,000 `LP` tokens**
- LP tokens sent to pool-manager setup: **100,000 LP**
- Farm reward pool: **8,000 `uUSDY`** + **1,000 `uOM`** fee
- Standard unlocking duration: **86,400 s (1 day)**

**Test Steps**:
1. Setup farm manager with active farm
2. User creates initial farming position
3. Verify position is recorded correctly
4. Modify position (increase/decrease size)
5. Test position closure process
6. Validate reward calculations throughout
7. Confirm position state consistency

**Expected Outcomes**:
- Positions are created and managed correctly
- State updates are accurate
- Reward calculations are precise
- User operations complete successfully

**State Snapshot (Before → After)**
| Entity | Positions (#) | LP Locked | Note |
|--------|---------------|-----------|------|
| `creator` | 0 → 1 → various edits → 0 | 0 → 1 000 → adjust | Tests create/expand/close logic |
| Fee Collector | collects 10 % emergency penalties where applicable | |

**AI Agent Prompt**: 
*"Create a comprehensive position management test that covers the full lifecycle of farming positions including creation, modification, and closure. Ensure state consistency, accurate reward calculations, and proper user experience throughout all operations."*

---

### 9. Swap Fee Collection
- [ ] **Test Name**: `swap_with_fees`
- **Location**: `contracts/pool-manager/src/tests/integration/swap.rs:883`
- **Importance**: 93/100
- **Category**: Revenue Generation & Protocol Sustainability

**Description**: Tests the fee collection mechanism during swaps, which is critical for protocol revenue and sustainability.

**Parameters**:
- Swap fee percentages
- Fee collection addresses
- Swap amounts of various sizes
- Fee distribution mechanisms

**Numeric Values & Denominations**:
- Liquidity added: **1,000,000,000 `uWHALE`** + **1,000,000,000 `uLUNA`**
- Swap offer amount: **10,000,000 `uWHALE`** (≈ 10 tokens)
- Fee configuration: Protocol **0.001 %**, Swap **0.002 %**, Burn **0 %**
- Expected protocol fee collected: **99 `uLUNA`**

**Test Steps**:
1. Configure pool with specific fee parameters
2. Execute swaps of different sizes
3. Calculate expected fee amounts
4. Verify fees are collected correctly
5. Check fee distribution to appropriate recipients
6. Validate remaining swap amounts are accurate
7. Test fee collection across multiple swaps

**Expected Outcomes**:
- Fees are calculated and collected accurately
- Fee distribution works correctly
- Swap amounts are reduced by appropriate fees
- Protocol revenue is properly tracked

**State Snapshot (Before → After)**
| Pool Reserves | Fee Collector `uLUNA` |
|---------------|----------------------|
| 1 000 000 000 / 1 000 000 000 → 990 099 307 / 1 010 000 000 (approx) | **99** |
| Protocol fee mint | included in swap event | |

**AI Agent Prompt**: 
*"Implement a swap fee testing mechanism that validates accurate fee calculation, collection, and distribution during token swaps. Test various swap sizes and ensure fee mechanisms work correctly across different scenarios while maintaining protocol revenue integrity."*

---

### 10. Farm Creation Functionality
- [ ] **Test Name**: `create_farms`
- **Location**: `contracts/farm-manager/tests/integration/farm_management.rs:42`
- **Importance**: 92/100
- **Category**: Yield Generation & Protocol Growth

**Description**: Tests the farm creation process, which establishes yield-generating opportunities for users and is fundamental to the protocol's value proposition.

**Parameters**:
- Farm owner account
- Reward token specifications
- Farm duration and parameters
- Liquidity pool requirements

**Numeric Values & Denominations**:
- Standard farm creation deposit: **4,000 `uUSDY`** reward budget
- Farm fee payment: **1,000 `uOM`** (required)
- Invalid scenarios tested with: **2,000 `uUSDY`**, **5,000 `uUSDY`**, **8,000 `uOM`** etc.
- Epoch window examples: **start at 25**, **end at 28** (blocks scheduling)

**Test Steps**:
1. Setup farm manager contract
2. Prepare reward tokens for distribution
3. Create new farm with specified parameters
4. Verify farm configuration is correct
5. Test farm activation process
6. Validate reward distribution setup
7. Confirm farm is ready for user participation

**Expected Outcomes**:
- Farm is created successfully
- Configuration parameters are set correctly
- Reward distribution is properly initialized
- Farm is available for user participation

**State Snapshot (Before → After)**
| Scenario | Farm State | Creator Balance |
|----------|------------|-----------------|
| Valid creation | `Pending` → `Active` at epoch 25 | −4 000 uUSDY −1 000 uOM |
| Misconfig (0 reward) | rejected, no state change | |
| Misconfig (wrong denom) | rejected | |

**AI Agent Prompt**: 
*"Create a comprehensive farm creation test that validates the complete setup process for yield farms. Test parameter configuration, reward distribution setup, and ensure farms are properly initialized and ready for user participation with accurate reward mechanisms."*

---

## Testing Execution Guide

### Prerequisites
- Rust testing environment
- CosmWasm test framework
- Mock dependencies for contract interactions
- Test data for various scenarios

### Execution Order
1. Run security tests (1-5) first - these are critical for fund safety
2. Execute core functionality tests (6-10) - these validate basic protocol operations
3. Use parallel execution where possible to optimize testing time
4. Maintain isolated test environments to prevent interference

### Success Criteria
- All security tests must pass with 100% reliability
- Core functionality tests must demonstrate consistent behavior
- Edge cases and error conditions should be handled gracefully
- Performance should meet specified benchmarks

### Automation Integration
These tests should be integrated into CI/CD pipelines with:
- Automated execution on every code change
- Performance regression detection
- Security vulnerability scanning
- Comprehensive reporting and alerting 