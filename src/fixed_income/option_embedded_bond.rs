/*
Calculates price of an option embedded bond using a binomial tree 
- All rates stated as continously compounded 

[TODO]
- add coupon payment scheme to structure
*/

use pyo3::prelude::*;
use pyo3::exceptions::PyValueError;


#[pyclass]
pub struct OptionEmbeddedBond {
    #[pyo3(get, set)]
    notional: f64,
    #[pyo3(get,set)]
    forward_curve: Vec<f64>,
    #[pyo3(get, set)]
    bond_option: String,
    #[pyo3(get,set)]
    option_price: f64,
    #[pyo3(get, set)]
    interest_vol: f64,
    #[pyo3(get, set)]
    coupons: Vec<f64>,
    #[pyo3(get)]
    binomial_tree: Vec<BinomialTreeLevel>,
}

#[pymethods]
impl OptionEmbeddedBond {
    #[new]
    fn new(
        notional: f64, bond_option: String, option_price: f64, 
        forward_curve: Vec<f64>, interest_vol: f64, coupons: Option<Vec<f64>>
    ) -> Result<Self, PyErr> {
        // Check if coupons or create a vec of 0 coupons
        let coupons = match coupons {
            Some(vec) => {
                if vec.len() != forward_curve.len() {
                    return Err(PyValueError::new_err(
                        "Coupon vector length does not match forward curve length"
                    ));
                } else {
                    vec
                }
            }
            None => vec![0.0; forward_curve.len()]
        };

        let mut bond = OptionEmbeddedBond {
            notional,
            bond_option: bond_option.to_string(),
            option_price,
            forward_curve,
            interest_vol,
            coupons,
        binomial_tree: Vec::new(),
        }; 
        bond.init()?;

        Ok(bond)
    }

    // Initialize the interest rate tree and value the bond
    pub fn init(&mut self) -> Result<(), PyErr> {
        for n in 0..self.forward_curve.len() {
            let nodes = n;
            let level = BinomialTreeLevel::new(
                nodes,
                self.forward_curve[n],
                self.interest_vol,
                self.coupons[n],
            );
            self.binomial_tree.push(level);
        }
        self.calculate_tree()?;

        Ok(())
    }

    fn calculate_tree(&mut self) -> Result<(), PyErr> {        
        match self.bond_option.as_str() {
            "call" => {
                let n = self.binomial_tree.len() - 1;
                self.binomial_tree[n].calculate_last_layer_call(self.notional, self.option_price);
                
                let mut left_i = self.binomial_tree.len() - 1;
                while left_i > 0 {
                    let (left, right) = self.binomial_tree.split_at_mut(left_i);
                    left[left.len()-1].calculate_branch_node_call(
                        self.option_price, &right[0].prices
                    );
                    
                    left_i -= 1;
                }

                Ok(())
            }
            "put" => {
                let n = self.binomial_tree.len() - 1;
                self.binomial_tree[n].calculate_last_layer_put(self.notional, self.option_price);
                
                let mut left_i = self.binomial_tree.len() - 1;
                while left_i > 0 {
                    let (left, right) = self.binomial_tree.split_at_mut(left_i);
                    left[left.len()-1].calculate_branch_node_put(
                        self.option_price, &right[0].prices
                    );
                    
                    left_i -= 1;
                }
                
                Ok(())
            }
            _ => {
                Err(PyValueError::new_err("Invalid option type parameter. Use call or put"))
            }
        } 
    }
}


#[derive(Clone)]
#[pyclass]
struct BinomialTreeLevel {
    #[pyo3(get)]
    prices: Vec<f64>,
    #[pyo3(get)]
    rates: Vec<f64>,
    #[pyo3(get)]
    coupon: f64,
}

impl BinomialTreeLevel {
    pub fn new(
        n: usize, rate: f64, interest_vol: f64, coupon: f64,
    ) -> Self { 
        let mut level = BinomialTreeLevel {
            prices: vec![0.0; n + 1],   // init vector of prices with zero
            rates: vec![rate; n + 1],   // init with base interest rate, map vol
            coupon,
        }; 
        level.calibrate_interest_paths(interest_vol);

        level
    }

    // Calculate interest rate paths from interest rate volatility
    fn calibrate_interest_paths(&mut self, interest_vol: f64) {
        let n = self.rates.len() as f64;
        self.rates.iter_mut().enumerate().for_each(|(index, value)| {
            let i = index as f64;
            *value = *value 
                * (interest_vol * (-2.0 * i + n - 1.0)).exp();
        });
    }

    // Calculate the PV's at last layer
    fn calculate_last_layer_call(&mut self, notional: f64, option_price: f64) {
        let n = self.prices.len();
        for i in 0..n {
            let price = (notional + self.coupon) * (-1.0 * self.rates[i]).exp();
            if price >= option_price {
                self.prices[i] = option_price;
            } else {
                self.prices[i] = price;
            }
        }
    }

    // Calculate probablilty weighted price with interest rate limit
    pub fn calculate_branch_node_call(
        &mut self, option_price: f64, prices: &Vec<f64>
    ) {
        let n = self.prices.len();
        for (index, price) in self.prices.iter_mut().enumerate() {
            let p = (prices[index] * 0.5 + prices[index + 1] * 0.5 + self.coupon)
                * (-1.0 * self.rates[index]).exp();
            
            // If last level, dont apply option
            if n == 1 {
                *price = p;
            } else {
                if p >= option_price {
                    *price = option_price;
                } else {
                    *price = p;
                }
            }
        }
    }
    
    // Calculate the PV's at last layer
    fn calculate_last_layer_put(&mut self, notional: f64, option_price: f64) {
        let n = self.prices.len();
        for i in 0..n {
            let p = (notional + self.coupon) * (-1.0 * self.rates[i]).exp();
            if p <= option_price {
                self.prices[i] = option_price;
            } else {
                self.prices[i] = p;
            }
        }
    }

    // Calculate probability weighted prices with interest rate cap
    pub fn calculate_branch_node_put(
        &mut self, option_price: f64, prices: &Vec<f64>
    ) {
        let n = self.prices.len();
        for (index, price) in self.prices.iter_mut().enumerate() {
            let p = (prices[index] * 0.5 + prices[index + 1] * 0.5 + self.coupon)
                * (-1.0 * self.rates[index]).exp();
            
            // If last level, dont apply option
            if n == 1 {
                *price = p;
            } else {
                if p <= option_price {
                    *price = option_price;
                } else {
                    *price = p;
                }
            }
        }
    }
}

