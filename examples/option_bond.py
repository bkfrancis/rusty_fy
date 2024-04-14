# Example of calculating the price of a callable bond


from rusty_fy import fixed_income


notional = 1000
option_type = "call"    # or put
option_rate = 0.02
forward_curve = [0.01, 0.02, 0.03]
interest_vol = 0.3


bond = fixed_income.OptionEmbeddedBond(
    notional, option_type, option_rate, forward_curve, interest_vol
)

print("Current price:", bond.binomial_tree[0].prices[0])
for level in bond.binomial_tree:
    print(level.prices)
    print(level.rates)
