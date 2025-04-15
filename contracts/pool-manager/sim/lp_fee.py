N_COINS = 2
xp_ = [1000, 1000]
fee = 0.05

def get_D(xp, amp):
    S = 0
    for _x in xp:
        S += _x
    if S == 0:
        return 0

    print("sum_pools:: ", S)

    Dprev = 0
    D = S
    Ann = amp * N_COINS
    for _i in range(225):
        print("get_D_i", _i)
        D_P = D
        for _x in xp:
            D_P = D_P * D / (_x * N_COINS)
        Dprev = D

        print("Dprev", Dprev)
        D = (Ann * S + D_P * N_COINS) * D / ((Ann - 1) * D + (N_COINS + 1) * D_P)
        print("D", D)
        print("---")

        # Equality with the precision of 1
        if D > Dprev:
            if D - Dprev <= 1:
                break
        else:
            if Dprev - D <= 1:
                break
    return D

def get_y(i, j, x, xp_):
    # i the index of token in
    # j the index of token out
    # x the new resrves of token in (xp[i]+dx)
    # xp_ array that contains pool assets

    # x in the input is converted to the same price/precision

    assert i != j       # dev: same coin
    assert j >= 0       # dev: j below zero
    assert j < N_COINS  # dev: j above N_COINS

    # should be unreachable, but good for safety
    assert i >= 0
    assert i < N_COINS

    amp = 85
    D = get_D(xp_, amp)
    print("D:: ", D)
    c = D
    S_ = 0
    Ann = amp * N_COINS
    print("Ann:: ", Ann)
    _x = 0
    for _i in range(N_COINS):
        print("-----")
        print("_i:: ", _i)
        if _i == i:
            _x = x

        elif _i != j:
            _x = xp_[_i]
        else:
            continue
        S_ += _x
        print("_x:: ", _x)
        print("S_:: ", S_)
        c = c * D / (_x * N_COINS)
        print("c:: ", c)

    print("S_ total :: ", S_)

    c = c * D / (Ann * N_COINS)
    print("final c:: ", c)
    b = S_ + D / Ann  # - D
    print("b:: ", b)
    y_prev = 0
    y = D
    for _i in range(255):
        y_prev = y
        y = (y*y + c) / (2 * y + b - D)
        print(f'y{_i}:: {y}')
        # Equality with the precision of 1
        if y > y_prev:
            if y - y_prev <= 1:
                break
        else:
            if y_prev - y <= 1:
                break
    return y

PRECISION = 1

def add_liquidity(_amounts):
    amp = 85
    old_balances = xp_

    # Initial invarian
    D0 = get_D(old_balances, amp)
    print(">>>>>>>> D0, ", D0)
    total_supply = 2000
    new_balances = [0, 0]

    # -------------------------- Do Transfers In -----------------------------

    for i in range(N_COINS):
        if _amounts[i] > 0:
            new_balances[i] += (_amounts[i] + old_balances[i])

    # ------------------------------------------------------------------------

    print(f'old_balances: {old_balances}')
    print(f'new_balances: {new_balances}')
    # Invariant after change
    D1 = get_D(new_balances, amp)
    print(">>>>>>>> D1, ", D1)
    assert D1 > D0

    # We need to recalculate the invariant accounting for fees
    # to calculate fair user's share
    fees = []
    mint_amount = 0

    if total_supply > 0:
        ideal_balance: uint256 = 0
        difference: uint256 = 0
        new_balance: uint256 = 0

        ys: uint256 = (D0 + D1) / N_COINS
        xs: uint256 = 0
        _dynamic_fee_i: uint256 = 0
        print("ys: ", ys)
        print("N_COINS: ", N_COINS)
        # Only account for fees if we are not the first to deposit
        base_fee: uint256 = fee * N_COINS / 4 * (N_COINS - 1)

        for i in range(N_COINS):
            print("~~~~~~~~~~~~~~~~~~~~~~")
            ideal_balance = D1 * old_balances[i] / D0
            difference = 0
            new_balance = new_balances[i]

            if ideal_balance > new_balance:
                difference = ideal_balance - new_balance
            else:
                difference = new_balance - ideal_balance

            print("old_balances[i]", old_balances[i])
            print("new_balance", new_balance)

            # fee[i] = _dynamic_fee(i, j) * difference / FEE_DENOMINATOR
            xs = (old_balances[i] + new_balance) / PRECISION
            _dynamic_fee_i = _dynamic_fee(xs, ys, base_fee)
            fees.append((_dynamic_fee_i * difference) / FEE_DENOMINATOR)
            new_balances[i] -= fees[i]
            print("ideal_balance: ", ideal_balance)
            print("difference: ", difference)
            print("xs: ", xs)
            print("dynamic_fee_i: ", _dynamic_fee_i)
            print("fees[i]: ", fees[i])

        print("fees: ", fees)
        print("adjusted_new_pool_assets: ", new_balances)
        D1 = get_D(new_balances, amp)
        print(">>>>>>>> adjusted_d_1, ", D1)
        print("total_supply", total_supply)
        mint_amount = (total_supply * (D1 - D0)) / D0

    return mint_amount

FEE_DENOMINATOR = 1

def _dynamic_fee(xpi, xpj, _fee):
    print("**** dynamic_fee ****")
    _offpeg_fee_multiplier: uint256 = 2
    if _offpeg_fee_multiplier <= FEE_DENOMINATOR:
        return _fee

    xps2: uint256 = (xpi + xpj) ** 2
    print("xpi: ", xpi)
    print("xpj: ", xpj)
    print("xps2: ", xps2)
    print("_fee: ", _fee)

    numerator = _offpeg_fee_multiplier * _fee
    denominator = (_offpeg_fee_multiplier - FEE_DENOMINATOR) * 4 * xpi * xpj / xps2
    print("numerator: ", numerator)
    print("denominator: ", denominator)

    print("**** dynamic_fee ****")
    return FEE_DENOMINATOR + ( numerator / denominator)


#print("Swap amount: ", xp_[2] - get_y(0, 2, 1200, xp_))

print("add liquidity: ", add_liquidity([90,110]))