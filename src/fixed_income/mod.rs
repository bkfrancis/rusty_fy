// Fixed Income Module

use pyo3::prelude::*;

mod option_embedded_bond;
mod simple_bond;

#[pymodule]
pub fn register_fixed_income(py: Python, parent_m: &PyModule) -> PyResult<()> {
    let fixed_income = PyModule::new(py, "fixed_income")?;
    fixed_income.add_class::<simple_bond::SimpleBond>()?;
    fixed_income.add_class::<option_embedded_bond::OptionEmbeddedBond>()?;
    parent_m.add_submodule(fixed_income)?;

    Ok(())
}
