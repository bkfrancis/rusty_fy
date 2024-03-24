use pyo3::prelude::*;



#[derive(Clone)]
#[pyclass]
pub enum BondOptionType {
    Call,
    Put
}


#[pyclass]
pub struct OptionEmbeddedBond {
    #[pyo3(get, set)]
    par_value: f64,
    #[pyo3(get,set)]
    forward_curve: Vec<f64>,
    #[pyo3(get, set)]
    bond_option: BondOptionType,
    #[pyo3(get,set)]
    option_rate: f64,
    #[pyo3(get, set)]
    interest_vol: f64,
    #[pyo3(get)]
    binomial_tree: Vec<BinomialTreeLevel>,
}


// [TODO] Implement a coupon structure
#[pymethods]
impl OptionEmbeddedBond {
    #[new]
    fn new(
        par_value: f64, bond_option: BondOptionType, option_rate: f64, 
        forward_curve: Vec<f64>, interest_vol: f64
    ) -> Self {
        let mut bond = OptionEmbeddedBond {
            par_value,
            bond_option,
            option_rate,
            forward_curve,
            interest_vol,
            binomial_tree: Vec::new(),
        };
        bond.init();
        bond
    }

    // Initialize the interest rate tree and value the bond
    pub fn init(&mut self) {
        for n in 0..self.forward_curve.len() {
            let nodes = n;
            let level = BinomialTreeLevel::new(
                nodes,
                &self.forward_curve[n],
                &self.interest_vol 
            );
            self.binomial_tree.push(level);
        }

        self.calculate_tree();
    }

    // Calibrate tree & calculate price
    fn calculate_tree(&mut self) {        
        let n = self.binomial_tree.len() - 1;
        self.binomial_tree[n].calculate_last_layer_call(&self.par_value, &self.option_rate);
        
        match self.bond_option {
            BondOptionType::Call => { 
                let mut left_i = self.binomial_tree.len() - 1;
                
                while left_i > 0 {
                    let (left, right) = self.binomial_tree.split_at_mut(left_i);
                    left[left.len()-1].calculate_leaf_node_call(
                        &self.par_value, &self.option_rate, &right[0].prices
                    );
                    
                    left_i -= 1;
                }
            }
            BondOptionType::Put => {
                let mut left_i = self.binomial_tree.len() - 1;
                
                while left_i > 0 {
                    let (left, right) = self.binomial_tree.split_at_mut(left_i);
                    left[left.len()-1].calculate_leaf_node_put(
                        &self.par_value, &self.option_rate, &right[0].prices
                    );
                    
                    left_i -= 1;
                }
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
}


impl BinomialTreeLevel {
    pub fn new(
        n: usize, rate: &f64, interest_vol: &f64
    ) -> Self { 
        let mut level = BinomialTreeLevel {
            prices: vec![0.0; n + 1],   // init vector of prices with zero
            rates: vec![*rate; n + 1],   // init with base interest rate, map vol
        }; 
        level.calibrate_interest_paths(interest_vol);

        level
    }

    // Calculate interest rate paths from interest rate volatility
    fn calibrate_interest_paths(&mut self, interest_vol: &f64) {
        let n = self.rates.len() as f64;
        self.rates.iter_mut().enumerate().for_each(|(index, value)| {
            *value = *value 
                * (*interest_vol * (-1.0 * (index as f64) + (n - (index as f64) - 1.0))).exp();
        });
    }

    // Calculate the PV's at last layer
    pub fn calculate_last_layer_call(&mut self, par_value: &f64, option_rate: &f64) {
        let n = self.prices.len();
        for i in 0..n {
            let rate = self.rates[i];
            if rate <= *option_rate {
                self.prices[i] = *par_value;
            } else {
                self.prices[i] = par_value * (-1.0 * rate).exp();
            }
        }
    }

    // Calculate probablilty weighted price with interest rate limit
    pub fn calculate_leaf_node_call(
        &mut self, par_value: &f64, option_rate: &f64, prices: &Vec<f64>
    ) {
        let n = self.prices.len();
        for (index, price) in self.prices.iter_mut().enumerate() {
            // If last level, dont apply option
            if n == 1 {
                *price = (prices[index] * 0.5 + prices[index + 1] * 0.5)
                    * (-1.0 * self.rates[index]).exp();
            } else if self.rates[index] <= *option_rate {
                *price = *par_value;
            } else {
                *price = (prices[index] * 0.5 + prices[index + 1] * 0.5) 
                    * (-1.0 * self.rates[index]).exp();
            }
        }
    }
    
    // Calculate probability weighted prices with interest rate cap
    pub fn calculate_leaf_node_put(
        &mut self, par_value: &f64, option_rate: &f64, prices: &Vec<f64>
    ) {
        let n = self.prices.len();
        for (index, price) in self.prices.iter_mut().enumerate() {
            // If last level, dont apply option
            if n == 1 {
                *price = (prices[index] * 0.5 + prices[index + 1] * 0.5)
                    * (-1.0 * self.rates[index]).exp();
            } else if self.rates[index] >= *option_rate {
                *price = *par_value;
            } else {
                *price = (prices[index] * 0.5 + prices[index + 1] * 0.5) 
                    * (-1.0 * self.rates[index]).exp();
            }
        }
    }
}

