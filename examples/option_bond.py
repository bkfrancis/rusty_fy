# Example of calculating the price of a callable bond
# All rates stated as continously compounded


from rusty_fy import fixed_income


notional = 100
option_price = 100
forward_curve = [0.01, 0.02, 0.03]
interest_vol = 0.3
coupons = [3, 3, 3]


# Callable no coupon
bond = fixed_income.OptionEmbeddedBond(
    notional, "call", option_price, forward_curve, interest_vol
)

print("Callable no coupon bond price:", bond.binomial_tree[0].prices[0])
for level in bond.binomial_tree:
    print(level.prices)
    print(level.rates)


# Callable with coupon
bond = fixed_income.OptionEmbeddedBond(
    notional, "call", option_price, forward_curve, interest_vol, coupons
)

print()
print("Callable coupon bond price:", bond.binomial_tree[0].prices[0])
for level in bond.binomial_tree:
    print(level.prices)
    print(level.rates)


# Putable no coupon
bond = fixed_income.OptionEmbeddedBond(
    notional, "put", option_price, forward_curve, interest_vol
)

print()
print("Putable bond price:", bond.binomial_tree[0].prices[0])
for level in bond.binomial_tree:
    print(level.prices)
    print(level.rates)


# Putable with coupon
bond = fixed_income.OptionEmbeddedBond(
    notional, "put", option_price, forward_curve, interest_vol, coupons
)

print()
print("Putable coupon bond price:", bond.binomial_tree[0].prices[0])
for level in bond.binomial_tree:
    print(level.prices)
    print(level.rates)
