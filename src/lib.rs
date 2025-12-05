

mod qreal_fixed64;
mod qreal_f64;


#[cfg(not(feature = "fixed64"))]
pub(crate) type Qreal = qreal_f64::Qreal;
#[cfg(feature = "fixed64")]
pub(crate) type Qreal = qreal_fixed64::Qreal;
#[cfg(not(feature = "fixed64"))]
pub(crate) type Qcplx = qreal_f64::Qcplx;
#[cfg(feature = "fixed64")]
pub(crate) type Qcplx = qreal_f64::Qcplx;


pub mod circ;
pub mod search;
pub mod state;
pub mod utils;
pub mod groups;
pub mod identity;
use pyo3::prelude::*;
use pyo3_stub_gen::define_stub_info_gatherer;

use crate::identity::{circuit::Circ, eccproof::IdentityProver, idcircuit::IdentityCirc};

#[pyo3::pymodule]
fn quclif(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<circ::Argument>()?;
    m.add_class::<circ::Instruction>()?;
    m.add_class::<circ::Gate16>()?;
    m.add_class::<circ::Instr32>()?;
    m.add_class::<state::StateVec>()?;
    m.add_class::<search::ECC>()?;
    m.add_class::<search::ECCs>()?;
    m.add_class::<search::double_perm_search::RawECCs>()?;
    m.add_class::<search::double_perm_search::Evaluator>()?;
    m.add_class::<Circ>()?;
    m.add_class::<IdentityCirc>()?;
    m.add_class::<IdentityProver>()?;
    Ok(())
}

define_stub_info_gatherer!(stub_info);