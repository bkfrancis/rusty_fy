use pyo3::prelude::*;


#[pyclass]
pub struct SimpleBond {
    #[pyo3(get, set)]
    notional: f64,
    #[pyo3(get, set)]
    n_period: i32,
    #[pyo3(get, set)]
    coupon_amount: f64,  
    #[pyo3(get, set)]
    coupon_freq: f64,
    #[pyo3(get)]
    interest_rate: f64,
    #[pyo3(get)]
    price: f64,
    #[pyo3(get)]
    mod_duration: f64,
    #[pyo3(get)]
    mac_duration: f64,
    #[pyo3(get)]
    convexity: f64,
}


#[pymethods]
impl SimpleBond {
    #[new]
    pub fn new (
        notional: f64, n_period: i32, coupon_amount: f64, coupon_freq: f64, interest_rate: f64
    ) -> Self {
        let mut simple_bond = SimpleBond {
            notional,
            n_period,
            coupon_amount,
            coupon_freq,
            interest_rate,
            price: 0.0,
            mod_duration: 0.0,
            mac_duration: 0.0,
            convexity: 0.0,
        };

        simple_bond.calculate();
        simple_bond
    }

    pub fn calculate(&mut self) {
        self.calc_price();
        self.calc_dur();
        self.calc_convex();
    }

    // Simple bond pricing formula assuming a flat interest rate curve
    fn calc_price(&mut self) {
        let n_coupons: f64 = (self.n_period as f64) * self.coupon_freq;
        let eff_rate: f64 = (1.0 + self.interest_rate).powf(1.0 / self.coupon_freq) - 1.0;
        let pv_coupons = self.coupon_amount * (1.0 - (1.0 + eff_rate).powf(-n_coupons)) / eff_rate;
        
        let pv_notional = self.notional / (1.0 + self.interest_rate).powi(self.n_period);
        self.price = pv_coupons + pv_notional;
    }

    // Calculate the mac duration and derive the modified duration
    fn calc_dur(&mut self) {
        let mut numerator: f64 = 0.0;
        let n_coupons = (self.n_period as f64) * self.coupon_freq;
        let eff_rate: f64 = (1.0 + self.interest_rate).powf(1.0 / self.coupon_freq) - 1.0;
        for n in 1..=(n_coupons as i32) {
            let pv_c = (n as f64) / (self.coupon_freq as f64) 
                * self.coupon_amount / (1.0 + eff_rate).powi(n);
            numerator += pv_c;
        }

        numerator += ((self.n_period as f64) * self.notional) / 
            (1.0 + self.interest_rate).powi(self.n_period);
        
        self.mac_duration = numerator / self.price;
        self.mod_duration = 
            self.mac_duration / (1.0 + (self.interest_rate / (self.coupon_freq as f64)));
    }

    fn calc_convex(&mut self) {
        let notional_term = 1.0 / (self.price * (1.0 + self.interest_rate).powi(2));
        let mut cf_term = 0.0;
        for t in 1..self.n_period {
            let cf = self.coupon_amount * ((t as f64).powi(2) + (t as f64)) 
                / (1.0 + self.interest_rate).powi(t);
            cf_term += cf;
        }
        self.convexity = notional_term * cf_term;
    }

    // Create vector of bond prices by interest rates
    fn plot_price_range(&self) -> PyResult<(Vec<f64>, Vec<f64>)> {
        // Create linear space of interest rate range      
        let n_rates = (self.interest_rate* 2.0 * 100.0 / 0.1).round() as i32;
        let int_rates_range: Vec<_> = (0..=n_rates).map(|x| (x as f64) * 0.001).collect();

        let mut price_range: Vec<f64> = Vec::new();
        for i in &int_rates_range {
            let n_coupons: f64 = (self.n_period as f64) * self.coupon_freq;
            let eff_rate: f64 = (1.0 + i).powf(1.0 / self.coupon_freq) - 1.0;
            let pv_coupons = self.coupon_amount 
                * (1.0 - (1.0 + eff_rate).powf(-n_coupons)) / eff_rate;
        
            let pv_notional = self.notional / (1.0 + i).powi(self.n_period);
            let price = pv_coupons + pv_notional;

            price_range.push(price);
        }

        Ok((int_rates_range, price_range))
    }
}

