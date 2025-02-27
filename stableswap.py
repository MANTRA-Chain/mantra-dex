N_COINS = 3
xp_ = [100, 100, 100]

def get_D(xp, amp):
    S: uint256 = 0
    for _x in xp:
        S += _x
    if S == 0:
        return 0

    Dprev = 0
    D = S
    Ann = amp * N_COINS
    for _i in range(32):
        D_P = D
        for _x in xp:
            D_P = D_P * D / (_x * N_COINS)
        Dprev = D
        D = (Ann * S + D_P * N_COINS) * D / ((Ann - 1) * D + (N_COINS + 1) * D_P)
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

    amp = 100
    D = get_D(xp_, amp)
    print("D:: ", D)
    c = D
    S_ = 0
    Ann = amp * N_COINS

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
    for _i in range(32):
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

print("Swap amount: ", xp_[2] - get_y(0, 2, 120, xp_))