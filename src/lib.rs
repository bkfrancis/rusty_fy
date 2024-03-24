use pyo3::prelude::*;

mod fixed_income;

 
#[pymodule]
fn rusty_fy(py: Python<'_>, m: &PyModule) -> PyResult<()> {
    fixed_income::register_fixed_income(py, m)?;
    Ok(())
}
