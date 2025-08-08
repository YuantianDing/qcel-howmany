use derive_more::{Debug, Deref, DerefMut, Display, From, Index, Into};
use itertools::Itertools;
use smallvec::SmallVec;

use crate::{circ::{gates::SWAP, Gate}, groups::permutation::Permut32, search::double_perm_search::CircuitECCs, state::StateVec, utils::JoinOptionIter};



mod quartz;
pub mod double_perm_search;


#[pyo3_stub_gen::derive::gen_stub_pyclass]
#[pyo3::pyclass]
#[derive(Debug, Display, Clone, PartialEq, Eq, Hash)]
#[debug("{}({})", self.0, self.1.iter().join_option(", ", "", ""))]
#[display("{}({})", self.0, self.1.iter().join_option(", ", "", ""))]
pub struct Instr(pub Gate, pub SmallVec<[u8; 2]>);

impl Instr {
    pub fn apply_permutation(&self, perm: Permut32) -> Self {
        Instr(self.0, self.1.iter().map(|&qubit| perm.at(qubit)).collect())
    }
}

#[pyo3_stub_gen::derive::gen_stub_pymethods]
#[pyo3::pymethods]
impl Instr {
    #[new]
    pub fn new_py(gate: Gate, qubits: Vec<u8>) -> Self {
        Instr(gate, SmallVec::from_vec(qubits))
    }
    #[getter(gate)]
    pub fn gate_py(&self) -> Gate {
        self.0
    }
    
    #[getter(qubits)]
    pub fn qubits_py(&self) -> Vec<u8> {
        self.1.iter().cloned().collect()
    }
}

#[pyo3_stub_gen::derive::gen_stub_pyclass]
#[pyo3::pyclass(eq, str)]
#[derive(Debug, Deref, DerefMut, Index, From, Into, Clone, PartialEq, Eq, Hash)]
pub struct ECC(Vec<(Vec<Instr>, Permut32)>);

impl ECC {
    pub fn circuits(&self) -> impl Iterator<Item=Vec<Instr>> + '_ {
        let unit = self[0].1.inv();
        self.iter().map(move |(instrs, p)| {
            instrs.iter().cloned().chain(
                (unit * *p).generate_swaps().map(|(a,b)| Instr(*SWAP, smallvec::smallvec![a, b]))
            ).collect_vec()
        })
    }
}

impl std::fmt::Display for ECC {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ECC {{\n")?;
        for circ in self.circuits() {
            write!(f, "\t{},\n", circ.iter().join_option(" ", "", ""))?;
        }
        write!(f, "}}")
    }
}

#[pyo3_stub_gen::derive::gen_stub_pymethods]
#[pyo3::pymethods]
impl ECC {
    #[pyo3(name="circuits")]
    fn circuits_py(&self) -> Vec<Vec<Instr>> {
        self.circuits().collect()
    }
}

#[pyo3_stub_gen::derive::gen_stub_pyclass]
#[pyo3::pyclass(eq, str)]
#[derive(Debug, Deref, DerefMut, Index, Clone, PartialEq, Eq, Hash)]
pub struct ECCs {
    #[deref]
    #[deref_mut]
    #[index]
    pub eccs: Vec<ECC>,
    #[pyo3(get)]
    pub nqubits: usize,
}

impl std::fmt::Display for ECCs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, ecc) in self.iter().enumerate() {
            if ecc.len() > 1 {
                write!(f, "{}{ecc}", if i > 0 { " + " } else {""})?;
            }
        }
        Ok(())
    }
}

impl ECCs {
    pub fn check<'a>(&'a self) -> impl Iterator<Item=&'a ECC> + 'a {
        let state = StateVec::from_random(&mut rand::rng(), self.nqubits as u32);
        self.eccs.iter().filter(move |ecc| !ecc.circuits().into_iter().map(|circ| {
                let mut s = state.clone();
                for Instr(g, idx) in circ {
                    s.apply(&idx, g);
                }
                s.normalize_arg();
                s.hash_value()
            }).all_equal()
        )
    }
}

#[pyo3_stub_gen::derive::gen_stub_pymethods]
#[pyo3::pymethods]
impl ECCs {
    #[staticmethod]
    pub fn generate(
        nqubits: usize,
        gates: Vec<Gate>,
        max_size: usize,
    ) -> ECCs {
        CircuitECCs::generate(nqubits, gates, max_size, &mut rand::rng()).simplify()
    }
    pub fn dump_quartz(&self, filepath: String) -> pyo3::PyResult<()> {
        use std::fs::File;

        let quartz_data = self.as_quartz();
        let file = File::create(filepath)
            .map_err(|e| pyo3::PyErr::new::<pyo3::exceptions::PyIOError, _>(format!("Failed to create file: {}", e)))?;

        serde_json::to_writer(file, &quartz_data)
            .map_err(|e| pyo3::PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Failed to write JSON: {}", e)))?;
        
        Ok(())
    }
    #[pyo3(name="check")]
    pub fn check_py(&self) -> Vec<ECC> {
        self.check().cloned().collect()
    }
}


