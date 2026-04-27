
use derive_more::{Debug, Deref, DerefMut, From, Index, Into};
use itertools::Itertools;
use rand::{SeedableRng, rngs::StdRng};

use crate::{circ::{Instr32, gates::SWAP}, groups::permutation::Permut32, identity::{circuit::Circ, idcircuit::IdentityCirc}, state::StateVec, utils::{DenseIndexMap, FmtJoinIter}};



mod quartz;
pub mod double_perm_search;

#[pyo3_stub_gen::derive::gen_stub_pyclass]
#[pyo3::pyclass(eq, str)]
#[derive(Debug, Deref, DerefMut, Index, From, Into, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize, PartialOrd, Ord)]
/// One equivalence class of circuits.
pub struct ECC(pub Vec<(Vec<Instr32>, Permut32)>);

impl ECC {
    pub fn circuits(&self) -> impl Iterator<Item=Vec<Instr32>> + '_ {
        let unit = self[0].1.inv();
        self.iter().map(move |(instrs, p)| {
            instrs.iter().cloned().chain(
                (unit * *p).generate_swaps().map(|(a,b)| Instr32(*SWAP, [a, b].into_iter().collect()))
            ).collect_vec()
        })
    }
    pub fn simplify_permute(self) -> Self {
        let unit = self[0].1.inv();
        ECC(self.0.into_iter().map(|(instrs, p)| {
            (instrs, unit * p)
        }).collect())
    }
    pub fn simplify(self) -> Self {
        let mut map = DenseIndexMap::new();
        self.simplify_permute().0.into_iter().map(|(instrs, p)| {
            let instrs = instrs.into_iter().map(|instr| instr.reindex(&mut map)).collect();
            (instrs, p.reindex(&mut map))
        }).collect::<Vec<_>>().into()
    }
}

impl std::fmt::Display for ECC {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ECC {{\n")?;
        for circ in self.circuits() {
            write!(f, "\t{},\n", circ.iter().fjoin(" "))?;
        }
        write!(f, "}}")
    }
}

#[pyo3_stub_gen::derive::gen_stub_pymethods]
#[pyo3::pymethods]
impl ECC {
    #[pyo3(name="circuits")]
    /// Returns all circuits in this class with swap instructions materialized.
    fn circuits_py(&self) -> Vec<Vec<Instr32>> {
        self.circuits().collect()
    }
}

#[pyo3_stub_gen::derive::gen_stub_pyclass]
#[pyo3::pyclass(eq, str)]
#[derive(Debug, Deref, DerefMut, Index, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize, PartialOrd, Ord)]
/// Collection of equivalence classes for a fixed number of qubits.
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
        for (i, ecc) in self.iter().filter(|ecc| ecc.len() > 1).enumerate() {
            write!(f, "{}{ecc}", if i > 0 { " & " } else {""})?;
        }
        Ok(())
    }
}

impl ECCs {
    pub fn check<'a>(&'a self) -> impl Iterator<Item=&'a ECC> + 'a {
        let state = StateVec::from_random(&mut StdRng::from_os_rng(), self.nqubits as u32);
        self.eccs.iter().filter(move |ecc| !ecc.circuits().into_iter().map(|circ| {
                let mut s = state.clone();
                for Instr32(g, idx) in circ {
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
    // #[staticmethod]
    // pub fn generate(
    //     nqubits: usize,
    //     gates: Vec<Gate>,
    //     max_size: usize,
    // ) -> ECCs {
    //     let evaluator = Evaluator::from_random(nqubits, &mut rand::rng());
    //     RawECCs::generate(&evaluator, gates, max_size).simplify()
    // }
    /// Exports ECCs to Quartz JSON format.
    pub fn dump_quartz(&self, filepath: String) -> pyo3::PyResult<()> {
        use std::fs::File;

        let quartz_data = self.as_quartz();
        let file = File::create(filepath)
            .map_err(|e| pyo3::PyErr::new::<pyo3::exceptions::PyIOError, _>(format!("Failed to create file: {}", e)))?;

        serde_json::to_writer(file, &quartz_data)
            .map_err(|e| pyo3::PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Failed to write JSON: {}", e)))?;
        
        Ok(())
    }

    #[staticmethod]
    /// Loads ECCs from a postcard file.
    fn from_postcard(filepath: String) -> pyo3::PyResult<Self> {
        use std::fs::File;
        use std::io::BufReader;

        let file = File::open(filepath)
            .map_err(|e| pyo3::PyErr::new::<pyo3::exceptions::PyIOError, _>(format!("Failed to open file: {}", e)))?;
        let reader = BufReader::new(file);
        let mut buffer : [u8; 8192] = [0; 8192];
        let eccs: ECCs = postcard::from_io((reader, &mut buffer)).map(|a| a.0)
            .map_err(|e| pyo3::PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Failed to read postcard data: {}", e)))?;
        Ok(eccs)
    }

    /// Saves ECCs as postcard.
    fn dump_postcard(&self, filepath: String) -> pyo3::PyResult<()> {
        use std::fs::File;
        use std::io::BufWriter;

        let file = File::create(filepath)
            .map_err(|e| pyo3::PyErr::new::<pyo3::exceptions::PyIOError, _>(format!("Failed to create file: {}", e)))?;
        let writer = BufWriter::new(file);

        postcard::to_io(self, writer)
            .map_err(|e| pyo3::PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Failed to write postcard data: {}", e)))?;
        Ok(())
    }
    #[pyo3(name="check")]
    /// Returns classes that fail randomized equivalence checks.
    pub fn check_py(&self) -> Vec<ECC> {
        self.check().cloned().collect()
    }
    
    pub fn __len__(&self) -> usize {
        self.eccs.len()
    }
    
    #[pyo3(name="to_list")]
    /// Returns all classes as a Python list.
    pub fn to_list_py(&self) -> Vec<ECC> {
        self.eccs.clone()
    }
    /// Removes classes with only one circuit.
    pub fn filter_single(&self) -> ECCs {
        ECCs {
            eccs: self.eccs.iter().filter(|ecc| ecc.len() > 1).cloned().collect(),
            nqubits: self.nqubits,
        }
    }
    /// Converts classes into canonical identities plus witness circuit pairs.
    pub fn to_identity_circuits(&self) -> Vec<(IdentityCirc, Circ, Circ)> {
        let mut identities = Vec::new();
        for ecc in self.iter() {
            let initial = Circ::new(ecc[0].0.clone(), ecc[0].1).inverse();
            for (c, p) in ecc.iter().skip(1) {
                let c = Circ::new(c.clone(), *p);
                identities.push(((&initial + &c).rotate_representative(), initial.clone(), c));
            }
        }
        identities.sort();
        // identities.dedup();
        return identities;
    }
}

