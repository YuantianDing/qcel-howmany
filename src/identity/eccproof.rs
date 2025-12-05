
use std::collections::{HashMap, HashSet, VecDeque};

use itertools::Itertools;
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pymethods};
use rayon::iter::ParallelIterator;
use indicatif::ProgressIterator;

use crate::{identity::{circuit::Circ, idcircuit::{IdentityCirc, IdentitySubcircuit}}, search::{ECCs, ECC}};

#[gen_stub_pyclass]
#[pyo3::pyclass]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct IdentityProver {
    assume: Vec<IdentityCirc>,
    proved: HashMap<IdentityCirc, ()>,
    circ_map: HashMap<Circ, Vec<Circ>>,
}

impl IdentityProver {
    pub fn build(
        eccs: &[ECC],
    ) -> Self {
        let mut prover = Self {
            assume: Vec::new(),
            proved: HashMap::new(),
            circ_map: HashMap::new(),
        };
        let mut identities = Vec::new();
        eprintln!("Building identity prover from {} ECCs", eccs.len());
        for ecc in eccs.iter().progress() {
            let ecc = ecc.clone().simplify_permute();
            for (c, p) in ecc.iter() {
                let (c_rep, perm) = Circ::new_no_perm(c.clone()).representative_with_perm();
                // let min_len = ecc.iter().map(|(c0, _)| c0.len()).min().unwrap();
                let circs = ecc.iter()
                    .filter(|(c0, _)| c0 != c)
                    .map(|(c0, p0)| Circ::new(c0.clone(), p.inverse() * (*p0)).permut(perm))
                    .collect_vec();
                prover.circ_map.insert(c_rep, circs);
            }
        
            let initial = Circ::new(ecc[0].0.clone(), ecc[0].1).inverse();
            for (c, p) in ecc.iter().skip(1) {
                let c = Circ::new(c.clone(), *p);
                identities.push((&initial + &c).rotate_representative());
            }
        }
        
        identities.sort();
        identities.dedup();
        eprintln!("Proving Identities");
        for id in identities.into_iter().progress() {
            prover.add_identity(id, 2, 50000);
        }

        prover
    }
    pub fn into_assumed_identities(self) -> Vec<IdentityCirc> {
        self.assume
    }
    pub fn assumed_identities(&self) -> &Vec<IdentityCirc> {
        &self.assume
    }
}

#[gen_stub_pymethods]
#[pyo3::pymethods]
impl IdentityProver {
    #[staticmethod]
    pub fn build_from_eccs(
        eccs: ECCs,
    ) -> Self {
        Self::build(eccs.as_slice())
    }

    pub fn par_apply_rules<'a>(
        &'a self,
        identity: &'a IdentityCirc,
        additional_size: usize,
    ) -> Vec<IdentityCirc> {
        IdentitySubcircuit::par_subcircuit_splits(identity).flat_map(move |(c1, c2)| {
            let (c1_rep, perm) = c1.representative_with_perm();
            let c1_reversed = c1_rep.inverse();
            let c2_permuted = c2.permut(perm);
            if let Some(c) = self.circ_map.get(&c1_rep) {
                c.iter().filter_map(move |c| {
                    if c.len() > additional_size + c1_rep.len() {
                        None
                    } else if !self.proved.contains_key(&(c + &c1_reversed).rotate_representative()) {
                        None
                    } else {
                        Some((c + &c2_permuted).rotate_representative())
                    }
                }).collect_vec()
            } else { Vec::new() }
        }).collect()
    }

    fn add_identity(&mut self, identity: IdentityCirc, additional_size: usize, count_limit: usize) -> bool {
        if identity.is_empty() || self.proved.contains_key(&identity) {
            return false;
        }

        let (visited, result) = self.add_identity_search(identity, additional_size, count_limit);
        
        for id in visited {
            self.proved.insert(id, ());
        }

        result
    }
    
    fn add_identity_search(&mut self, identity: IdentityCirc, additional_size: usize, count_limit: usize) -> (HashSet<IdentityCirc>, bool) {
        let mut search_queue = vec![VecDeque::<IdentityCirc>::new(); additional_size + 1];
        let mut visited = HashSet::new();

        let min_size = identity.len();
        let max_size = additional_size + identity.len();
        let mut counter = 0;
        search_queue[0].push_back(identity.clone());
        visited.insert(identity.clone());
        while let Some(id0) = search_queue
            .iter_mut()
            .find(|x| !x.is_empty())
            .and_then(|a| a.pop_front()) {
            
            for new_id in self.par_apply_rules(&id0, max_size - id0.len()) {
                assert!(new_id.len() <= max_size);
                if visited.contains(&new_id) {
                    continue;
                }
                visited.insert(new_id.clone());
                if new_id.len() < min_size {
                    return (visited, self.add_identity(new_id, additional_size, count_limit));
                } else if self.proved.contains_key(&new_id) {
                    return (visited, false);
                }
                search_queue[new_id.len() - min_size].push_back(new_id);

                counter += 1;
                if counter > count_limit {
                    self.assume.push(identity);
                    return (visited, true);
                }
            }
        }
        self.assume.push(identity);
        return (visited, true);
    }

    pub fn prove_identity(&self, identity: IdentityCirc, additional_size: usize, count_limit: usize) -> Option<IdentityCirc> {
        let mut search_queue = vec![VecDeque::<IdentityCirc>::new(); additional_size + 1];
        let mut visited = HashSet::new();

        let min_size = identity.len();
        let max_size = additional_size + identity.len();
        let mut counter = 0;

        search_queue[0].push_back(identity.clone());
        visited.insert(identity.clone());
        while let Some(id0) = search_queue
            .iter_mut()
            .find(|x| !x.is_empty())
            .and_then(|a| a.pop_front()) {

            for new_id in self.par_apply_rules(&id0, max_size - id0.len()) {
                assert!(new_id.len() <= max_size);
                if visited.contains(&new_id) {
                    continue;
                }
                visited.insert(new_id.clone());
                if new_id.len() < min_size {
                    return self.prove_identity(new_id, additional_size, count_limit);
                } else if self.proved.contains_key(&new_id) {
                    return None;
                }
                search_queue[new_id.len() - min_size].push_back(new_id);

                counter += 1;
                if counter > count_limit {
                    return Some(identity);
                }
            }
        }
        Some(identity)
    }
    fn get_assumed(&self) -> Vec<IdentityCirc> {
        self.assumed_identities().clone()
    }
    #[staticmethod]
    fn from_postcard(filepath: String) -> pyo3::PyResult<Self> {
        use std::fs::File;
        use std::io::BufReader;

        let file = File::open(filepath)
            .map_err(|e| pyo3::PyErr::new::<pyo3::exceptions::PyIOError, _>(format!("Failed to open file: {}", e)))?;
        let reader = BufReader::new(file);
        let mut buffer : [u8; 8192] = [0; 8192];
        let eccs: Self = postcard::from_io((reader, &mut buffer)).map(|a| a.0)
            .map_err(|e| pyo3::PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Failed to read postcard data: {}", e)))?;
        Ok(eccs)
    }

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
}


fn available_system_memory() -> usize {
  let contents=std::fs::read_to_string("/proc/meminfo").expect("Could not read /proc/meminfo");
  let mem_info = contents.lines().find(|line| line.starts_with("MemAvailable")).expect("Could not find MemAvailable line");
  let size = mem_info.split(" ").nth(3).expect("Found the size");
  let available_mem: usize = size.parse().unwrap();
  available_mem  // in kilobytes KB
}

fn total_system_memory() -> usize {
  let contents=std::fs::read_to_string("/proc/meminfo").expect("Could not read /proc/meminfo");
  let mem_info = contents.lines().find(|line| line.starts_with("MemTotal")).expect("Could not find MemAvailable line");
  let size = mem_info.split(" ").nth(7).expect("Found the size");
  let total_mem: usize = size.parse().unwrap();
  total_mem      // in kilobytes KB
}

fn memory_usage_rate() -> usize {
    available_system_memory() * 100 / total_system_memory()
}