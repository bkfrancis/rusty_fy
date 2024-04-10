use pyo3::prelude::*;
use pyo3::exceptions::PyValueError;



#[derive(Clone)]
#[pyclass]
pub struct LinearFn {
    #[pyo3(get)]
    x_lower: f64,
    #[pyo3(get)]
    x_upper: f64,
    #[pyo3(get)]
    b0: f64,
    #[pyo3(get)]
    b1: f64,
}


#[pyclass]
pub struct LinearSpline {
    #[pyo3(get, set)]
    x: Vec<f64>,
    #[pyo3(get, set)]
    y: Vec<f64>,
    #[pyo3(get)]
    params: Vec<LinearFn>,
}


#[pymethods]
impl LinearSpline {
    #[new]
    pub fn new(x: Vec<f64>, y: Vec<f64>) -> Result<Self, PyErr> {
        if x.len() != y.len() {
            return Err(PyValueError::new_err("Vectors x and y must have the same length"))
        }

        let mut linear_spline = LinearSpline {
            x,
            y,
            params: Vec::new(),
        };
        linear_spline.calibrate();

        Ok(linear_spline)
    }

    fn calibrate(&mut self) {
        let left_iter = self.x.iter().zip(self.y.iter());
        let right_iter = self.x.iter().skip(1).zip(self.y.iter().skip(1));

        for ((left_x, left_y), (right_x, right_y)) in left_iter.zip(right_iter) {
            let x_lower = *left_x;
            let x_upper = *right_x;
            let b1 = (*right_y - *left_y) / (*right_x - *left_x);
            let b0 = *left_y;

            self.params.push(LinearFn {x_lower, x_upper, b0, b1});
        }
    }

    pub fn get_values(&mut self, x_input: Vec<f64>) -> PyResult<Vec<f64>> {
        let mut spline_values: Vec<f64> = Vec::new();

        for value in x_input.iter() {
            // Binary search of coefficients 
            let mut low: usize = 0;
            let mut high: usize = self.params.len() - 1;
            
            if (value < &self.params[0].x_lower) || 
                (value > &self.params[self.params.len() - 1].x_upper) 
            {
                return Err(PyValueError::new_err("Value not in spline range"));
            }
            
            while low <= high {
                let mid: usize = (high + low) / 2;
                
                if (value >= &self.params[mid].x_lower) &&
                    (value <= &self.params[mid].x_upper) 
                {
                    spline_values.push(self.params[mid].b0 + (
                        (*value - self.params[mid].x_lower) * self.params[mid].b1));
                    break
                } else if value < &self.params[mid].x_lower {
                    high = mid - 1;
                } else if value > &self.params[mid].x_upper {
                    low = mid + 1;
                } else {    
                    return Err(PyValueError::new_err("Value error"));
                }
            }
        }

        Ok(spline_values)
    }
}
