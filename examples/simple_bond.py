# Example of calculating the price of a coupon paying bond


from rusty_fy import fixed_income
import matplotlib.pyplot as plt


notional = 1000
n_periods = 5
coupon_amount = 20
coupon_freq = 1
interest_rate = 0.05

bond = fixed_income.SimpleBond(notional, n_periods, coupon_amount, coupon_freq, interest_rate)

print("Current price:", bond.price)
print("Modified duration:", bond.mod_duration)
print("Macaulay duration:", bond.mac_duration)
print("Convexity:", bond.convexity)

plt.xlabel("Interest Rate")
plt.ylabel("Price")
plt.plot(*bond.plot_price_range())
plt.show()
