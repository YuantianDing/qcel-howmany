//! Core circuit-level data structures shared by Rust and Python APIs.

use std::hash::Hasher;

use derive_more::{Debug, Display};
use either::Either;
use extension_traits::extension;

mod gate;
pub use gate::*;
use itertools::Itertools;
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pymethods};

use crate::{groups::permutation::Permut32, utils::FmtJoinIter};
pub mod param;

#[gen_stub_pyclass]
#[pyo3::pyclass(eq, str)]
#[derive(Debug, Clone, Display, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[display("{regid}[{index}]")]
/// A named register argument (e.g., `q[0]`, `c[2]`) used by high-level instructions.
pub struct Argument {
    #[pyo3(get, set)]
    pub regid: String,
    #[pyo3(get, set)]
    pub index: usize,
}

#[gen_stub_pymethods]
#[pyo3::pymethods]
impl Argument {
    #[new]
    /// Creates a register argument like `("q", 0)`.
    pub fn new(regid: String, index: usize) -> Self {
        Self { regid, index }
    }

    pub fn __hash__(&self) -> usize {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        std::hash::Hash::hash(&self, &mut hasher);
        hasher.finish() as usize
    }
    pub fn __repr__(&self) -> String {
        format!("{:?}", self)
    }
}
pub mod qargs;
#[gen_stub_pyclass]
#[pyo3::pyclass(eq, str)]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// A high-level instruction with gate + explicit quantum/classical register arguments.
pub struct Instruction {
    #[pyo3(get, set)]
    pub gate: Gate16,
    #[pyo3(get, set)]
    pub qargs: Vec<Argument>,
    #[pyo3(get, set)]
    pub cargs: Vec<Argument>,
}

impl std::fmt::Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {};",
            self.gate,
            fmttools::join(self.qargs.iter().chain(self.cargs.iter()), ", "),
        )
    }
}
#[gen_stub_pymethods]
#[pyo3::pymethods]
impl Instruction {
    #[new]
    #[pyo3(signature = (gate, qargs, cargs = Vec::new()))]
    /// Creates a high-level instruction with explicit quantum/classical arguments.
    pub fn new(
        gate: Gate16,
        qargs: Vec<Either<Argument, (String, usize)>>,
        cargs: Vec<Either<Argument, (String, usize)>>,
    ) -> Self {
        Self {
            gate,
            qargs: qargs
                .into_iter()
                .map(|arg| match arg {
                    Either::Left(a) => a,
                    Either::Right((regid, index)) => Argument::new(regid, index),
                })
                .collect(),
            cargs: cargs
                .into_iter()
                .map(|arg| match arg {
                    Either::Left(a) => a,
                    Either::Right((regid, index)) => Argument::new(regid, index),
                })
                .collect(),
        }
    }
    pub fn __hash__(&self) -> usize {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        std::hash::Hash::hash(&self, &mut hasher);
        hasher.finish() as usize
    }
    pub fn __repr__(&self) -> String {
        format!("{:?}", self)
    }
}

#[extension(pub trait InstructionSliceExt)]
impl [Instruction] {
    fn schedule_quantum(&self) -> (Vec<Argument>, Vec<Instr32>) {
        let mut qargs = Vec::new();
        let mut results = Vec::new();

        for gate in self {
            for arg in &gate.qargs {
                if !qargs.contains(arg) {
                    qargs.push(arg.clone());
                }
            }
            assert!(
                gate.cargs.is_empty(),
                "Classical arguments are not supported in scheduling"
            );

            results.push(Instr32(gate.gate, gate
                .qargs
                .iter()
                .map(|arg| qargs.iter().position(|a| a == arg).unwrap() as u8)
                .collect()));
        }

        (qargs, results)
    }
}

impl Instruction {
    pub fn from_quantum_schedule(
        arguments: Vec<Argument>,
        instructions: Vec<Instr32>,
    ) -> Vec<Self> {
        let mut results = Vec::new();

        for Instr32(gate, indices) in instructions {
            let qargs: Vec<Argument> = indices.iter().map(|&i| arguments[i as usize].clone()).collect();
            results.push(Instruction {
                gate,
                qargs,
                cargs: Vec::new(),
            });
        }

        results
    }
}

#[pyo3_stub_gen::derive::gen_stub_pyclass]
#[pyo3::pyclass]
#[derive(Debug, Display, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[debug("{}({})", self.0, self.1.iter().fjoin(", "))]
#[display("{}({})", self.0, self.1.iter().fjoin(", "))]
#[pyo3(name = "Instr")]
/// A compact, index-only quantum instruction used internally by the search/prover.
pub struct Instr32(pub Gate16, pub qargs::QArgs16);


impl PartialOrd for Instr32 {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.cmp(other).into()
    }
}

impl Ord for Instr32 {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.1.cmp(&other.1).then_with(|| self.0.cmp(&other.0))
    }
}

impl Instr32 {
    pub fn reindex(&self, map: &mut crate::utils::DenseIndexMap) -> Self {
        Instr32(
            self.0,
            self.1.iter().map(|&q| map.get_or_insert(q as usize) as u8).collect(),
        )
    }
}

#[pyo3_stub_gen::derive::gen_stub_pymethods]
#[pyo3::pymethods]
impl Instr32 {
    #[new]
    /// Creates a compact instruction from a gate and integer qubit arguments.
    pub fn new(gate: Gate16, qargs: Vec<u8>) -> Self {
        assert!(
            qargs.len() == gate.nqargs(),
            "Number of qubit arguments does not match gate arity"
        );
        assert!(qargs.iter().all_unique(), "Qubit arguments must be unique");
        Instr32(gate, qargs.into_iter().collect())
    }
    #[getter]
    /// Underlying gate.
    pub fn gate(&self) -> Gate16 {
        self.0
    }
    #[getter]
    /// Qubit arguments as dense integer indices.
    pub fn qargs(&self) -> Vec<u8> {
        self.1.iter().cloned().collect()
    }
    /// Applies a qubit permutation.
    pub fn apply_permutation(&self, perm: Permut32) -> Self {
        Instr32(self.0, self.1.iter().map(|&qubit| perm.at(qubit)).collect())
    }
    /// Bitmask of qubits touched by this instruction.
    pub fn arg_mask(&self) -> u8 {
        self.1.iter().fold(0, |acc, &q| acc | (1 << q))
    }
    /// Updates a frontier mask for left-to-right scheduling checks.
    pub fn pass_mask(&self, mut mask: u8) -> Option<u8> {
        for &q in self.1.iter() {
            if (mask >> q) & 1 != 0 {
                continue;
            } else if q as u32 == mask.trailing_ones() {
                mask |= 1 << q;
            } else {
                return None;
            }
        }
        Some(mask)
    }
    /// Largest qubit index touched by this instruction.
    pub fn largest_qubit(&self) -> u8 {
        *self.1.iter().max().unwrap_or(&0)
    }
    /// Alias of `apply_permutation`.
    pub fn permut(&self, perm: Permut32) -> Self {
        Instr32(self.0, self.1.iter().map(|&q| perm.at(q)).collect())
    }
    /// Returns `true` when this and `other` touch disjoint qubits.
    pub fn disjoint(&self, other: &Instr32) -> bool {
        self.arg_mask() & other.arg_mask() == 0
    }
    /// Returns the adjoint instruction.
    pub fn adjoint(&self) -> Self {
        Instr32(self.0.adjoint(), self.1.clone())
    }
    pub fn __str__(&self) -> String {
        format!("{}", self)
    }
    pub fn __repr__(&self) -> String {
        format!("{:?}", self)
    }
    pub fn __hash__(&self) -> u64 {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }
    /// Returns position of `qubit` in argument list, if present.
    pub fn position_of_qubit(&self, qubit: u8) -> Option<usize> {
        self.1.iter().position(|&q| q == qubit)
    }
}

#[macro_export]
macro_rules! instr_vec {
    ($($gate:tt $($n:literal),*;)*) => {
        vec![
            $(
                $crate::circ::Instr(*$gate, smallvec::smallvec![$($n as u8),*])
            ),*
        ]
    };
}
