

#![cfg_attr(feature = "f128", feature(f128))]

#[cfg(not(feature = "f128"))]
mod qreal_f64;
#[cfg(not(feature = "f128"))]
pub type Qreal = qreal_f64::Qreal;
#[cfg(not(feature = "f128"))]
pub type Qcplx = qreal_f64::Qcplx;

#[cfg(feature = "f128")]
mod qreal_f128;
#[cfg(feature = "f128")]
pub type Qreal = qreal_f128::Qreal;
#[cfg(feature = "f128")]
pub type Qcplx = qreal_f128::Qcplx;


pub mod circ;
pub mod search;
pub mod state;
pub mod utils;
pub mod groups;
pub mod identity;
use pyo3::prelude::*;
use pyo3_stub_gen::define_stub_info_gatherer;

use crate::identity::{circuit::Circ, eccprove::{IdentityProver, proof::Proof}, idcircuit::IdentityCirc};

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
    m.add_class::<Proof>()?;
    Ok(())
}

define_stub_info_gatherer!(stub_info);