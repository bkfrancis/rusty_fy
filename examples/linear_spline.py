# Example of using linear spline interpolation


from rusty_fy import interpolate
import matplotlib.pyplot as plt
import numpy as np


data = {
    "x": [0, 1, 2, 3, 5, 10],
    "y": [0.00, 0.01, 0.005, 0.03, 0.04, 0.10]
}

cs = interpolate.LinearSpline(data["x"], data["y"])

x_linspace = np.linspace(0, 10, 100)

plt.plot(x_linspace, cs.get_values(x_linspace))
plt.show()
