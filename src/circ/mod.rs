use std::hash::Hasher;

use derive_more::{Debug, Display};
use either::Either;
use extension_traits::extension;

mod gate;
pub use gate::*;
use itertools::Itertools;
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pymethods};
use smallvec::SmallVec;

use crate::{groups::permutation::Permut32, utils::JoinOptionIter};
pub mod param;

#[gen_stub_pyclass]
#[pyo3::pyclass(eq, str)]
#[derive(Debug, Clone, Display, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[display("{regid}[{index}]")]
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
#[gen_stub_pyclass]
#[pyo3::pyclass(eq, str)]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Instruction {
    #[pyo3(get, set)]
    pub gate: Gate,
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
    pub fn new(
        gate: Gate,
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
    fn schedule_quantum(&self) -> (Vec<Argument>, Vec<Instr>) {
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

            let indices: SmallVec<[u8; 2]> = gate
                .qargs
                .iter()
                .map(|arg| qargs.iter().position(|a| a == arg).unwrap() as u8)
                .collect();
            results.push(Instr(gate.gate, indices));
        }

        (qargs, results)
    }
}

impl Instruction {
    pub fn from_quantum_schedule(
        arguments: Vec<Argument>,
        instructions: Vec<Instr>,
    ) -> Vec<Self> {
        let mut results = Vec::new();

        for Instr(gate, indices) in instructions {
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
#[debug("{}({})", self.0, self.1.iter().join_option(", ", "", ""))]
#[display("{}({})", self.0, self.1.iter().join_option(", ", "", ""))]
pub struct Instr(pub Gate, pub SmallVec<[u8; 2]>);


impl PartialOrd for Instr {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.cmp(other).into()
    }
}

impl Ord for Instr {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.1.cmp(&other.1).then_with(|| self.0.cmp(&other.0))
    }
}

#[pyo3_stub_gen::derive::gen_stub_pymethods]
#[pyo3::pymethods]
impl Instr {
    #[new]
    pub fn new(gate: Gate, qargs: Vec<u8>) -> Self {
        assert!(
            qargs.len() == gate.nqargs(),
            "Number of qubit arguments does not match gate arity"
        );
        let qargs = SmallVec::from_vec(qargs);
        assert!(qargs.iter().all_unique(), "Qubit arguments must be unique");
        Instr(gate, qargs)
    }
    #[getter]
    pub fn gate(&self) -> Gate {
        self.0
    }
    #[getter]
    pub fn qargs(&self) -> Vec<u8> {
        self.1.iter().cloned().collect()
    }
    pub fn apply_permutation(&self, perm: Permut32) -> Self {
        Instr(self.0, self.1.iter().map(|&qubit| perm.at(qubit)).collect())
    }
    pub fn arg_mask(&self) -> u8 {
        self.1.iter().fold(0, |acc, &q| acc | (1 << q))
    }
    pub fn pass_mask(&self, mut mask: u8) -> Option<u8> {
        for &q in &self.1 {
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
    pub fn largest_qubit(&self) -> u8 {
        *self.1.iter().max().unwrap_or(&0)
    }
    pub fn permut(&self, perm: Permut32) -> Self {
        Instr(self.0, self.1.iter().map(|&q| perm.at(q)).collect())
    }
    pub fn disjoint(&self, other: &Instr) -> bool {
        self.arg_mask() & other.arg_mask() == 0
    }
    pub fn adjoint(&self) -> Self {
        Instr(self.0.adjoint(), self.1.clone())
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

