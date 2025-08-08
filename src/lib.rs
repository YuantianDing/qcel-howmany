pub(crate) mod defs;

pub mod circ;
pub mod search;
pub mod state;
pub mod utils;
pub mod groups;
use pyo3::prelude::*;
use pyo3_stub_gen::define_stub_info_gatherer;

#[pyo3::pymodule]
fn quclif(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<circ::Argument>()?;
    m.add_class::<circ::Instruction>()?;
    m.add_class::<circ::Gate>()?;
    m.add_class::<state::StateVec>()?;
    m.add_class::<search::Instr>()?;
    m.add_class::<search::ECC>()?;
    m.add_class::<search::ECCs>()?;
    Ok(())
}

define_stub_info_gatherer!(stub_info);