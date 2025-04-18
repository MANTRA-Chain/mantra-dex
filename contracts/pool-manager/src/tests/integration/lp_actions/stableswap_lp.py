import math

# Constants matching Vyper contract (where relevant)
PRECISION = 10**18
A_PRECISION = 100
MAX_COINS = 3 # For our specific scenario

# --- Core Math Functions (Simplified Python versions) ---

def get_D(xp, A):
    """
    D invariant calculation in non-overflowing integer operations iteratively.
    Python version of the Vyper get_D function.
    A should be the raw A value (e.g., 100 * A_PRECISION).
    """
    S = sum(xp)
    if S == 0:
        return 0

    N_COINS = len(xp)
    Ann = A * N_COINS

    D = S
    for _ in range(255): # Max iterations
        D_P = D
        for x in xp:
            if x == 0: # Prevent division by zero if a balance is zero
                 # In reality, this might revert or require special handling,
                 # but for simulation, we can treat D_P contribution as zero.
                 # However, a more robust approach might be needed if this happens often.
                 # For now, let's return the current D if we hit a zero balance,
                 # as the invariant is ill-defined. Or maybe just skip this term?
                 # Let's skip the multiplication for this term.
                 continue
            # Hacky fix: Need to avoid division by zero if a balance is 0.
            # Let's just continue if x is 0, effectively skipping this term's contribution
            # to D_P. A better approach might be needed for a full simulation.
            try:
                 D_P = D_P * D // (x * N_COINS)
            except ZeroDivisionError:
                 print(f"Warning: Division by zero avoided in get_D loop for x={x}. Check balances.")
                 # Handle this case, perhaps return current D or raise a specific error
                 return D # Or maybe continue? Let's return D for now.

        Dprev = D

        # Original Vyper: D = ((Ann * S / A_PRECISION + D_P * N_COINS) * D / ((Ann - A_PRECISION) * D / A_PRECISION + (N_COINS + 1) * D_P))
        # Python equivalent with integer division:
        numerator = (Ann * S // A_PRECISION + D_P * N_COINS) * D
        denominator = (Ann - A_PRECISION) * D // A_PRECISION + (N_COINS + 1) * D_P

        if denominator == 0:
             # Avoid division by zero if denominator becomes 0
             print("Warning: Denominator zero in get_D update. Returning previous D.")
             # This might indicate an issue with inputs or the algorithm state
             return Dprev # Or handle differently

        D = numerator // denominator

        # Check for convergence
        if abs(D - Dprev) <= 1:
            return D

    # If it doesn't converge (shouldn't happen in normal conditions)
    print("Warning: D calculation did not converge")
    return D

# --- Pool Simulation Class ---

class StableSwapPool:
    def __init__(self, A, rates):
        """
        Initializes the pool simulation.
        A: Amplification parameter (e.g., 100)
        rates: List of rate multipliers (10**(18 - decimals)) for each coin.
        """
        self.A = A * A_PRECISION # Store A multiplied by A_PRECISION
        self.rates = rates
        self.N_COINS = len(rates)
        if self.N_COINS > MAX_COINS:
             raise ValueError(f"Number of coins exceeds MAX_COINS ({MAX_COINS})")
        # Initialize balances with zeros matching the number of rates
        self.balances = [0] * self.N_COINS
        self.total_supply = 0

    def _xp_mem(self, balances):
        """ Calculates normalized balances """
        xp = []
        for i in range(self.N_COINS):
             # Check if rates[i] or balances[i] are zero to avoid issues
             if self.rates[i] == 0 or balances[i] == 0:
                  xp.append(0)
             else:
                  xp.append(self.rates[i] * balances[i] // PRECISION)
        # Pad with zeros if N_COINS < MAX_COINS (not strictly needed for Python lists usually)
        # xp.extend([0] * (MAX_COINS - self.N_COINS))
        return xp


    def add_liquidity(self, deposit_amounts):
        """
        Simulates adding liquidity and returns the minted LP amount.
        deposit_amounts: List of raw amounts to deposit for each coin.
        """
        if len(deposit_amounts) != self.N_COINS:
            raise ValueError("Deposit amounts list length must match number of coins")

        # Calculate D before deposit
        xp_old = self._xp_mem(self.balances)
        D0 = get_D(xp_old, self.A)

        # Update balances (create new list to avoid modifying original during calculation)
        new_balances = list(self.balances)
        for i in range(self.N_COINS):
            new_balances[i] += deposit_amounts[i]

        # Calculate D after deposit
        xp_new = self._xp_mem(new_balances)
        D1 = get_D(xp_new, self.A)

        # Calculate minted LP tokens
        mint_amount = 0
        if self.total_supply == 0:
            # Initial deposit
            mint_amount = D1
        else:
            # Subsequent deposit
            if D0 == 0: # Avoid division by zero if initial D was 0
                 print("Warning: D0 is zero during subsequent deposit calculation.")
                 # Handle this edge case, perhaps mint_amount should be based only on D1?
                 # Or maybe it implies an error state. Let's default to 0 or raise an error.
                 mint_amount = 0 # Or raise ValueError("Cannot calculate mint amount with D0=0")
            else:
                 mint_amount = (D1 - D0) * self.total_supply // D0

        # Update pool state *after* calculations
        self.balances = new_balances
        self.total_supply += mint_amount

        return mint_amount

# --- Simulation Setup ---

# Assets and Decimals from Rust tests
asset_names = ["uluna", "uusd", "uweth"]
decimals = [6, 6, 18]

# Calculate rates (10**(18 - decimals))
rates = [(10**(18 - d)) for d in decimals]

# Amplification Factor
A = 100

# --- Run Simulation ---

print("--- Initial Liquidity Provision ---")
pool = StableSwapPool(A, rates)
initial_deposit = [
    10 * 10**6,  # uluna
    10 * 10**6,  # uusd
    10 * 10**18, # uweth
]
print(f"Initial Balances: {pool.balances}")
print(f"Initial Total Supply: {pool.total_supply}")
print(f"Depositing: {initial_deposit}")

initial_lp_minted = pool.add_liquidity(initial_deposit)

print(f"New Balances: {pool.balances}")
print(f"New Total Supply (LP Tokens): {pool.total_supply}")
print(f"LP Minted: {initial_lp_minted}")
print("-" * 30)

# Store state after initial deposit for Test Case 2
initial_state_balances = list(pool.balances)
initial_state_total_supply = pool.total_supply


print("--- Test Case 1: Deposit uluna (6 dec) + uweth (18 dec) ---")
# Use the pool state *after* initial deposit
deposit_case1 = [
    2 * 10**6,  # uluna
    0,          # uusd
    2 * 10**18, # uweth
]
print(f"Current Balances: {pool.balances}")
print(f"Current Total Supply: {pool.total_supply}")
print(f"Depositing: {deposit_case1}")

lp_minted_case1 = pool.add_liquidity(deposit_case1)

print(f"New Balances: {pool.balances}")
print(f"New Total Supply: {pool.total_supply}")
print(f"LP Minted in Case 1: {lp_minted_case1}")
print("-" * 30)


print("--- Test Case 2: Deposit uluna (6 dec) + uusd (6 dec) ---")
# Reset pool state to *after* the initial deposit
pool_case2 = StableSwapPool(A, rates)
pool_case2.balances = initial_state_balances
pool_case2.total_supply = initial_state_total_supply

deposit_case2 = [
    2 * 10**6, # uluna
    2 * 10**6, # uusd
    0,         # uweth
]
print(f"Current Balances: {pool_case2.balances}")
print(f"Current Total Supply: {pool_case2.total_supply}")
print(f"Depositing: {deposit_case2}")

lp_minted_case2 = pool_case2.add_liquidity(deposit_case2)

print(f"New Balances: {pool_case2.balances}")
print(f"New Total Supply: {pool_case2.total_supply}")
print(f"LP Minted in Case 2: {lp_minted_case2}")
print("-" * 30)


print("--- Summary ---")
print(f"Initial LP Minted: {initial_lp_minted}")
print(f"Case 1 LP Minted (uluna + uweth): {lp_minted_case1}")
print(f"Case 2 LP Minted (uluna + uusd): {lp_minted_case2}")

# Compare with Rust assertion values (approximate check)
print(f"\nComparison with Rust Test Assertions:")
print(f"Case 1 Rust Assertion: ~13,901,163,096,216")
print(f"Case 2 Rust Assertion: ~ 9,054,673,799,013")
