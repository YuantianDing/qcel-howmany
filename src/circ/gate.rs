use std::{cell::RefCell, collections::HashMap, hash::Hasher};

use nalgebra::DMatrix;
use nohash_hasher::BuildNoHashHasher;
use num_complex::Complex64;
use numpy::{PyArray2, PyArrayLike2, ToPyArray};
use pyo3::PyResult;
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pymethods};

use crate::{
    circ::param::{evaluate_with_pi, NumericError},
    defs::cmplx64mat_to_fixpoint,
    utils::JoinOptionIter,
};

#[derive(Debug, Clone)]
pub struct GateData {
    pub name: String,
    pub params: Vec<String>,
    pub matrix: DMatrix<Complex64>,
    pub adjoint: Option<Gate>,
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
    pub fn hash_value(&self) -> u64 {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        std::hash::Hash::hash(&self, &mut hasher);
        hasher.finish()
    }
}

thread_local! {
    static INSTRUCTION_SET: RefCell<HashMap<u64, GateData, BuildNoHashHasher<u64>>> = RefCell::new(Default::default());
}

#[gen_stub_pyclass]
#[pyo3::pyclass]
#[derive(
    derive_more::Debug, Clone, Copy, derive_more::Display, PartialEq, Eq, Hash, PartialOrd, Ord,
)]
#[debug("Gate({} -> {}{})", self.0, self.name(), self.params().iter().join_option(", ", "(", ")"))]
#[display("{}{}", self.name(), self.params().iter().join_option(", ", "(", ")"))]
pub struct Gate(u64);

impl Gate {
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

        INSTRUCTION_SET.with(|set| {
            let mut set = set.borrow_mut();
            match set.entry(hash_value) {
                std::collections::hash_map::Entry::Occupied(_) => (),
                std::collections::hash_map::Entry::Vacant(vacant_entry) => {
                    vacant_entry.insert(gate_data);
                }
            }
        });

        Gate(hash_value)
    }

    pub fn name(&self) -> String {
        INSTRUCTION_SET.with(|set| set.borrow()[&self.0].name.clone())
    }
    pub fn params(&self) -> Vec<String> {
        INSTRUCTION_SET.with(|set| set.borrow()[&self.0].params.clone())
    }

    pub fn matrix<T>(&self, f: impl FnOnce(&DMatrix<Complex64>) -> T) -> T {
        INSTRUCTION_SET.with(|set| f(&set.borrow()[&self.0].matrix))
    }

    pub fn nqargs(&self) -> usize {
        INSTRUCTION_SET.with(|set| set.borrow()[&self.0].matrix.nrows().trailing_zeros() as usize)
    }

    pub fn adjoint(&self) -> Gate {
        match INSTRUCTION_SET.with(|set| set.borrow()[&self.0].adjoint.clone()) {
            Some(adjoint) => adjoint,
            None => {
                let gate = Gate::new(
                    self.name() + "†",
                    self.params(),
                    self.matrix(|m| m.adjoint())
                );
                INSTRUCTION_SET.with(|set| set.borrow_mut().get_mut(&self.0).unwrap().adjoint = Some(gate));
                gate
            }
        }
    }
}

pub mod gates;

#[gen_stub_pymethods]
#[pyo3::pymethods]
impl Gate {
    #[gen_stub(skip)]
    #[new]
    pub fn new_py(name: String, params: Vec<String>, matrix: PyArrayLike2<Complex64>) -> Self {
        Self::new(name, params, matrix.as_matrix().into())
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
        self.matrix(|m| m.to_pyarray(py))
    }
    
    #[getter(nqargs)]
    pub fn nqargs_py(&self) -> usize {
        self.nqargs()
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
