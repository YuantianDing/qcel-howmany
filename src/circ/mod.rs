use std::hash::Hasher;

use derive_more::Display;
use either::Either;
use extension_traits::extension;

mod gate;
pub use gate::*;
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pymethods};
pub mod param;

#[gen_stub_pyclass]
#[pyo3::pyclass]
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

    pub fn __eq__(&self, other: &Self) -> bool {
        self == other
    }
    pub fn __hash__(&self) -> usize {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        std::hash::Hash::hash(&self, &mut hasher);
        hasher.finish() as usize
    }
    pub fn __repr__(&self) -> String {
        format!("{:?}", self)
    }
    pub fn __str__(&self) -> String {
        format!("{}", self)
    }
}
#[gen_stub_pyclass]
#[pyo3::pyclass]
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
    pub fn __eq__(&self, other: &Self) -> bool {
        self == other
    }
    pub fn __hash__(&self) -> usize {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        std::hash::Hash::hash(&self, &mut hasher);
        hasher.finish() as usize
    }
    pub fn __repr__(&self) -> String {
        format!("{:?}", self)
    }
    pub fn __str__(&self) -> String {
        format!("{}", self)
    }
}

#[extension(pub trait InstructionSliceExt)]
impl [Instruction] {
    fn schedule_quantum(&self) -> (Vec<Argument>, Vec<(Gate, Vec<usize>)>) {
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

            let indices: Vec<usize> = gate
                .qargs
                .iter()
                .map(|arg| qargs.iter().position(|a| a == arg).unwrap())
                .collect();
            results.push((gate.gate, indices));
        }

        (qargs, results)
    }
}

impl Instruction {
    pub fn from_quantum_schedule(
        arguments: Vec<Argument>,
        instructions: Vec<(Gate, Vec<usize>)>,
    ) -> Vec<Self> {
        let mut results = Vec::new();

        for (gate, indices) in instructions {
            let qargs: Vec<Argument> = indices.iter().map(|&i| arguments[i].clone()).collect();
            results.push(Instruction {
                gate,
                qargs,
                cargs: Vec::new(),
            });
        }

        results
    }
}
