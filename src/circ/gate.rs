use std::{cell::RefCell, collections::HashMap, hash::Hasher, sync::LazyLock};

use derive_more::Display;
use either::Either;
use nalgebra::DMatrix;
use nohash_hasher::BuildNoHashHasher;
use num_complex::Complex64;
use numpy::{PyArray2, PyArrayLike2, ToPyArray};
use pyo3::PyResult;
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pymethods};
use serde::{Deserialize, Serialize, ser::SerializeStruct};
use serde_json::Value;
use spin::RwLock;

use crate::{
    circ::{Argument, Instr32, Instruction, gates::initial_gates, param::{NumericError, evaluate_with_pi}},
    defs::cmplx64mat_to_fixpoint,
    utils::JoinOptionIter,
};

#[derive(Debug, Clone, Display)]
#[display("{}{}", self.name, self.params.iter().join_option(", ", "(", ")"))]
pub struct GateData {
    pub name: String,
    pub params: Vec<String>,
    pub matrix: DMatrix<Complex64>,
    pub adjoint: Option<Gate16>,
}

impl PartialEq for GateData {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && cmplx64mat_to_fixpoint(&self.matrix) == cmplx64mat_to_fixpoint(&other.matrix)
    }
}

impl Eq for GateData {}

impl std::hash::Hash for GateData {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        if self.matrix.nrows() == 0 || self.matrix.ncols() == 0 {
            self.name.hash(state);
        } else {
            cmplx64mat_to_fixpoint(&self.matrix).hash(state);
        }
    }
}

impl GateData {
    pub fn new(name: String, params: Vec<String>, matrix: DMatrix<Complex64>) -> Self {
        Self {
            name,
            params,
            matrix,
            adjoint: None,
        }
    }
    pub fn hash_value(&self) -> u64 {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        std::hash::Hash::hash(&self, &mut hasher);
        hasher.finish()
    }
}

static INSTRUCTION_SET: LazyLock<spin::RwLock<Vec<GateData>>> = LazyLock::new(|| RwLock::new(initial_gates()));

#[gen_stub_pyclass]
#[pyo3::pyclass(eq, frozen, hash, str)]
#[derive(
    derive_more::Debug, Clone, Copy, derive_more::Display, PartialEq, Eq, Ord, PartialOrd, Hash
)]
#[debug("Gate({} -> {}{})", self.0, self.name(), self.params().iter().join_option(", ", "(", ")"))]
#[display("{}{}", self.name(), self.params().iter().join_option(", ", "(", ")"))]
#[pyo3(name = "Gate")]
pub struct Gate16(u16);

impl Serialize for Gate16 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        format!("{}", self).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Gate16 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(Gate16::from_name(&String::deserialize(deserializer)?).expect("Failed to deserialize Gate8"))
    }
}


impl Gate16 {
    pub fn new(name: String, params: Vec<String>, matrix: DMatrix<Complex64>) -> Self {
        assert!(
            matrix.nrows().is_power_of_two(),
            "Matrix must have a number of rows that is a power of two"
        );
        assert!(matrix.nrows() == matrix.ncols(), "Matrix must be squared");
        let gate_data = GateData {
            name,
            params,
            matrix,
            adjoint: None,
        };
        let hash_value = gate_data.hash_value();

        let mut set = INSTRUCTION_SET.write();
        if let Some(idx) = set.iter() .position(|g| g.hash_value() == hash_value) {
            Gate16(idx as u16)
        } else {
            set.push(gate_data);
            Gate16((set.len() - 1) as u16)
        }
    }

    pub fn name(&self) -> String {
        INSTRUCTION_SET.read()[self.0 as usize].name.clone()
    }
    pub fn params(&self) -> Vec<String> {
        INSTRUCTION_SET.read()[self.0 as usize].params.clone()
    }

    pub fn matrix<T>(&self, f: impl FnOnce(&DMatrix<Complex64>) -> T) -> T {
        f(&INSTRUCTION_SET.read()[self.0 as usize].matrix)
    }
    pub fn data<T>(&self, f: impl FnOnce(&GateData) -> T) -> T {
        f(&INSTRUCTION_SET.read()[self.0 as usize])
    }

    pub fn nqargs(&self) -> usize {
        INSTRUCTION_SET.read()[self.0 as usize].matrix.nrows().trailing_zeros() as usize
    }

    pub fn adjoint(&self) -> Gate16 {
        let gate = { INSTRUCTION_SET.read()[self.0 as usize].adjoint.clone() };
        match gate {
            Some(adjoint) => adjoint,
            None => {
                let gate = Gate16::new(
                    self.name() + "†",
                    self.params(),
                    self.matrix(|m| m.adjoint())
                );
                INSTRUCTION_SET.write()[self.0 as usize].adjoint = Some(gate);
                gate
            }
        }
    }
    pub fn instr(&self, qargs: impl IntoIterator<Item=u8>) -> Instr32 {
        Instr32(self.clone(), qargs.into_iter().collect())
    }
}

impl Default for Gate16 {
    fn default() -> Self {
        gates::I.clone()
    }
}

pub mod gates;

#[gen_stub_pymethods]
#[pyo3::pymethods]
impl Gate16 {
    #[gen_stub(skip)]
    #[new]
    pub fn new_py(name: String, params: Vec<String>, matrix: PyArrayLike2<Complex64>) -> Self {
        Self::new(name, params, matrix.as_matrix().into())
    }

    #[staticmethod]
    pub fn from_name(name: &str) -> Option<Self> {
        INSTRUCTION_SET.read().iter().position(|g| name.starts_with(&g.name) && name == format!("{}", g)).map(|idx| Gate16(idx as u16))
    }

    #[getter(name)]
    pub fn name_py(&self) -> String {
        self.name()
    }

    #[getter(params)]
    pub fn params_py(&self) -> Vec<String> {
        self.params()
    }

    #[getter(params_f)]
    pub fn params_f_py(&self) -> PyResult<Vec<f64>> {
        let result: Result<Vec<f64>, NumericError> = self
            .params()
            .into_iter()
            .map(|p| evaluate_with_pi(&p))
            .collect();

        result.map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))
    }

    #[getter(matrix)]
    pub fn matrix_py<'py>(&self, py: pyo3::Python<'py>) -> pyo3::Bound<'py, PyArray2<Complex64>> {
        self.matrix(|m| <DMatrix<Complex64> as ToPyArray>::to_pyarray(m, py).to_owned())
    }
    
    #[getter(nqargs)]
    pub fn nqargs_py(&self) -> usize {
        self.nqargs()
    }
    
    #[pyo3(name="adjoint")]
    pub fn adjoin_py(&self) -> Gate16 {
        self.adjoint()
    }

    #[pyo3(signature=(*args))]
    pub fn __call__(&self, args: Either<Vec<u8>, Vec<Argument>>) -> Either<Instr32, Instruction> {
        match args {
            Either::Left(args) => {
                assert!(args.len() == self.nqargs(), "Number of arguments not match for gate {}", self);
                Either::Left(Instr32(self.clone(), args.into_iter().collect()))
            }
            Either::Right(args) => Either::Right(Instruction {
                gate: self.clone(),
                qargs: args[..self.nqargs()].into(),
                cargs: args[self.nqargs()..].into(),
            }),
        }
    }
}
