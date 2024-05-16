// Interpolation Module

use pyo3::prelude::*;

mod linear_spline;
mod cubic_spline;


#[pymodule]
pub fn register_interpolate(py: Python, parent_m: &PyModule) -> PyResult<()> {
    let interpolate = PyModule::new(py, "interpolate")?;
    interpolate.add_class::<linear_spline::LinearFn>()?;
    interpolate.add_class::<linear_spline::LinearSpline>()?;
    interpolate.add_class::<cubic_spline::CubicFn>()?;
    interpolate.add_class::<cubic_spline::CubicSpline>()?;
    parent_m.add_submodule(interpolate)?;

    Ok(())
}
