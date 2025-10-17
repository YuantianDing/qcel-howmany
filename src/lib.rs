pub(crate) mod defs;

pub mod circ;
pub mod search;
pub mod state;
pub mod utils;
pub mod groups;
pub mod identity;
use pyo3::prelude::*;
use pyo3_stub_gen::define_stub_info_gatherer;

use crate::identity::{circuit::Circ, idcircuit::IdentityCirc, idset::IdentitySet};

#[pyo3::pymodule]
fn quclif(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<circ::Argument>()?;
    m.add_class::<circ::Instruction>()?;
    m.add_class::<circ::Gate>()?;
    m.add_class::<circ::Instr>()?;
    m.add_class::<state::StateVec>()?;
    m.add_class::<search::ECC>()?;
    m.add_class::<search::ECCs>()?;
    m.add_class::<Circ>()?;
    m.add_class::<IdentityCirc>()?;
    m.add_class::<IdentitySet>()?;
    Ok(())
}

define_stub_info_gatherer!(stub_info);