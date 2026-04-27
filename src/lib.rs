
//! Core library and Python bindings for the CAV'26 quantum-identity artifact.
//!
//! The crate exposes:
//! - Rust modules for circuit search, identity generation, and proving.
//! - A PyO3 module (`qcel_howmany`) used by the Python API in `python/qcel_howmany`.

#![cfg_attr(feature = "f128", feature(f128))]

#[cfg(not(feature = "f128"))]
mod qreal_f64;
/// Real-number backend used by the simulator/search pipeline (default: `f64`).
#[cfg(not(feature = "f128"))]
pub type Qreal = qreal_f64::Qreal;
/// Complex-number type built on top of [`Qreal`] (default: `Complex<f64>`-backed).
#[cfg(not(feature = "f128"))]
pub type Qcplx = qreal_f64::Qcplx;

#[cfg(feature = "f128")]
mod qreal_f128;
/// Real-number backend used when compiling with the `f128` feature.
#[cfg(feature = "f128")]
pub type Qreal = qreal_f128::Qreal;
/// Complex-number type built on top of [`Qreal`] for the `f128` backend.
#[cfg(feature = "f128")]
pub type Qcplx = qreal_f128::Qcplx;


/// Circuit and gate data structures.
pub mod circ;
/// ECC generation/search algorithms.
pub mod search;
/// State-vector simulation backend.
pub mod state;
/// Shared utility data structures/helpers.
pub mod utils;
/// Group-theory helpers (e.g., permutations).
pub mod groups;
/// Identity representations and proving.
pub mod identity;
use pyo3::prelude::*;
use pyo3_stub_gen::define_stub_info_gatherer;

use crate::identity::{circuit::Circ, eccprove::{IdentityProver, proof::Proof}, idcircuit::IdentityCirc};

#[pyo3::pymodule]
/// Python extension module entrypoint.
///
/// This registers Rust classes that are re-exported and wrapped by
/// `python/qcel_howmany/__init__.py`.
fn qcel_howmany(m: &Bound<'_, PyModule>) -> PyResult<()> {
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
