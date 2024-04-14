/*
[TODO]
*/


use pyo3::prelude::*;
use pyo3::exceptions::PyValueError;
use nalgebra::DMatrix;


#[derive(Clone)]
#[pyclass]
pub struct CubicFn {
    #[pyo3(get)]
    x_lower: f64,
    #[pyo3(get)]
    x_upper: f64,
    #[pyo3(get)]
    a: f64,
    #[pyo3(get)]
    b: f64,
    #[pyo3(get)]
    c: f64,
    #[pyo3(get)]
    d: f64,
}


#[pyclass]
pub struct CubicSpline {
    #[pyo3(get, set)]
    x: Vec<f64>,
    y: Vec<f64>,
    #[pyo3(get)]
    params: Vec<CubicFn>,
    m_matrix: DMatrix<f64>,
    y_matrix: DMatrix<f64>,
    b_matrix: DMatrix<f64>,
}


#[pymethods]
impl CubicSpline {
    #[new]
    pub fn new(x: Vec<f64>, y: Vec<f64>) -> Self {
        let n_size = 4 * (x.len() - 1);
        let mut cubic_spline = CubicSpline {
            x,
            y,
            params: Vec::new(),
            m_matrix: DMatrix::from_element(n_size, n_size, 0.0),
            y_matrix: DMatrix::from_element(n_size, 1, 0.0),
            b_matrix: DMatrix::from_element(n_size, 1, 0.0),
        };

        cubic_spline.calculate_coeff();
        cubic_spline.set_params();

        cubic_spline
    }

    fn calculate_coeff(&mut self) {
        let (rows, cols) = self.m_matrix.shape();
        
        // Natural spline bounds
        // f0''(x0) = 0
        self.m_matrix[(0, 1)] = 2.0;

        // fi(xn) = yn
        self.m_matrix[(rows - 1, cols - 4)] = 6.0 * (self.x[self.x.len() - 1] 
            - self.x[self.x.len() - 2]);
        self.m_matrix[(rows - 1, cols - 3)] = 2.0;


        // Calibrate inner matrix
        for i in 0..(self.x.len() - 2) {
            
            // f(x0) = y0
            self.m_matrix[((i * 4) + 1, (i * 4) + 0)] = self.x[i].powi(3);
            self.m_matrix[((i * 4) + 1, (i * 4) + 1)] = self.x[i].powi(2);
            self.m_matrix[((i * 4) + 1, (i * 4) + 2)] = self.x[i];
            self.m_matrix[((i * 4) + 1, (i * 4) + 3)] = 1.0;
            self.y_matrix[((i * 4) + 1, 0)] = self.y[i];

            // fi(x1) = y1
            self.m_matrix[((i * 4) + 2, (i * 4) + 0)] = (self.x[i + 1]).powi(3);
            self.m_matrix[((i * 4) + 2, (i * 4) + 1)] = (self.x[i + 1]).powi(2);
            self.m_matrix[((i * 4) + 2, (i * 4) + 2)] = self.x[i + 1];
            self.m_matrix[((i * 4) + 2, (i * 4) + 3)] = 1.0;
            self.y_matrix[((i * 4) + 2, 0)] = self.y[i + 1];

            // fi'(x1) = fi+1'(x1)
            self.m_matrix[((i * 4) + 3, (i * 4) + 0)] = 3.0 * (self.x[i + 1]).powi(2);
            self.m_matrix[((i * 4) + 3, (i * 4) + 1)] = 2.0 * (self.x[i + 1]);
            self.m_matrix[((i * 4) + 3, (i * 4) + 2)] = 1.0;
            self.m_matrix[((i * 4) + 3, (i * 4) + 4)] = -3.0 * (self.x[i + 1]).powi(2);
            self.m_matrix[((i * 4) + 3, (i * 4) + 5)] = -2.0 * (self.x[i + 1]);
            self.m_matrix[((i * 4) + 3, (i * 4) + 6)] = -1.0;

            // fi''(x1) = fi+1''(x1)
            self.m_matrix[((i * 4) + 4, (i * 4) + 0)] = 6.0 * (self.x[i + 1]);
            self.m_matrix[((i * 4) + 4, (i * 4) + 1)] = 2.0;
            self.m_matrix[((i * 4) + 4, (i * 4) + 4)] = -6.0 * (self.x[i + 1]);
            self.m_matrix[((i * 4) + 4, (i * 4) + 5)] = -2.0;

        }
        
        // Last 2 conditions
        let i = self.x.len() - 1;
        
        // fi-1(xn-1) = yn-1
        self.m_matrix[(rows - 3, cols - 4)] = self.x[i - 1].powi(3);
        self.m_matrix[(rows - 3, cols - 3)] = self.x[i - 1].powi(2);
        self.m_matrix[(rows - 3, cols - 2)] = self.x[i - 1];
        self.m_matrix[(rows - 3, cols - 1)] = 1.0;
        self.y_matrix[(rows - 3, 0)] = self.y[i - 1];

        // fi(xn) = yn
        self.m_matrix[(rows - 2, cols - 4)] = (self.x[i]).powi(3);
        self.m_matrix[(rows - 2, cols - 3)] = (self.x[i]).powi(2);
        self.m_matrix[(rows - 2, cols - 2)] = self.x[i];
        self.m_matrix[(rows - 2, cols - 1)] = 1.0;
        self.y_matrix[(rows - 2, 0)] = self.y[i];

        // Invert Matix
        let m_inv = self.m_matrix.clone().try_inverse().unwrap();
        
        self.b_matrix = m_inv * &self.y_matrix;
    }

    // Setup the params params
    fn set_params(&mut self) {
        let (rows, _) = self.b_matrix.shape();
        
        for i in 0..(rows / 4) {
            let cubic_fn = CubicFn {
                x_lower: self.x[i],
                x_upper: self.x[i + 1],
                a: self.b_matrix[i * 4],
                b: self.b_matrix[i * 4 + 1],
                c: self.b_matrix[i * 4 + 2],
                d: self.b_matrix[i * 4 + 3],
            };

            self.params.push(cubic_fn)
        }
    }


    // Calculate y's given a vec of x's
    fn get_values(&mut self, x_input: Vec<f64>) -> PyResult<Vec<f64>> {
        let mut spline_values = Vec::new();


        for value in x_input.iter() {
            // Binary search of fns to calc y values
            let mut low = 0;
            let mut high = self.params.len() - 1;
             
            if (value < &self.params[0].x_lower) || 
                (value > &self.params[self.params.len() - 1].x_upper) 
            {
                return Err(PyValueError::new_err("Value not in spline range"));
            }
            
            while low <= high {
                let mid = (high + low) / 2;
                
                if (value >= &self.params[mid].x_lower) && 
                    (value <= &self.params[mid].x_upper) 
                {
                    let x = value;
                    let y = self.params[mid].a * x.powi(3)
                        + self.params[mid].b * x.powi(2)
                        + self.params[mid].c * x
                        + &self.params[mid].d;

                    spline_values.push(y);
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

    // Convert the matrix into a vec<vec<>>
    fn get_matrix(&mut self) -> PyResult<Vec<Vec<f64>>> {
        let mut vectors: Vec<Vec<f64>> = Vec::new();
        for row in self.m_matrix.row_iter() {
            vectors.push(row.iter().cloned().collect());
        }

        Ok(vectors)
    }

    fn get_b_matrix(&mut self) -> PyResult<Vec<Vec<f64>>> {
        let mut vectors: Vec<Vec<f64>> = Vec::new();
        for row in self.b_matrix.row_iter() {
            vectors.push(row.iter().cloned().collect());
        }
        
        Ok(vectors)
    }

    fn get_y_matrix(&mut self) -> PyResult<Vec<Vec<f64>>> {
        let mut vectors: Vec<Vec<f64>> = Vec::new();
        for row in self.y_matrix.row_iter() {
            vectors.push(row.iter().cloned().collect());
        }
        
        Ok(vectors)
    }

}
