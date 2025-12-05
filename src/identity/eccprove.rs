
use std::collections::{HashMap, HashSet, VecDeque};

use itertools::Itertools;
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pymethods};
use rayon::iter::ParallelIterator;
use indicatif::ProgressIterator;

use crate::{identity::{circuit::Circ, eccprove::proof::{Proof, ProofTracker}, idcircuit::{IdentityCirc, IdentitySubcircuit}}, search::{ECC, ECCs}, utils::FmtJoinIter};

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
            prover.add_identity(id, 3, 50000);
        }
        
        // eprintln!("Removing redundant assumptions");
        // while let Some(redundant_id) = prover.get_redundant_assumption() {
        //     // panic!();
        //     prover.assume.remove(redundant_id);
        // }

        prover
    }
    pub fn into_assumed_identities(self) -> Vec<IdentityCirc> {
        self.assume
    }
    pub fn assumed_identities(&self) -> &Vec<IdentityCirc> {
        &self.assume
    }
    fn _get_redundant_assumption(&mut self) -> Option<usize> {
        for id in 0..self.assume.len() {
            let mut prover = Self {
                assume: Vec::new(),
                proved: HashMap::new(),
                circ_map: HashMap::new(),
            };
            for (i, assumption) in self.assume.iter().enumerate() {
                if i == id { continue; }
                prover.assume.push(assumption.clone());
                prover.proved.insert(assumption.clone(), ());
                IdentitySubcircuit::subcircuit_splits(assumption).for_each(|(c1, c2)| {
                    let (c1_rep, perm) = c1.representative_with_perm();
                    let c2_permuted = c2.permut(perm);
                    
                    prover.circ_map.entry(c1_rep).or_default().push(c2_permuted.inverse());
                });
            }
            if prover.prove_identity(self.assume[id].clone(), 11, 50000).is_none() {
                return Some(id);
            }
        }
        return None;
    }
    pub fn par_transition_pairs<'a>(
        self: &IdentityProver,
        identity: &'a IdentityCirc,
        additional_size: usize,
    ) -> Vec<(IdentityCirc, IdentityCirc)> {
        IdentitySubcircuit::par_subcircuit_splits(identity).flat_map(move |(c1, c2)| {
            let (c1_rep, perm) = c1.representative_with_perm();
            let c1_reversed = c1_rep.inverse();
            let c2_permuted = c2.permut(perm);
            if let Some(c) = self.circ_map.get(&c1_rep) {
                c.iter().filter_map(move |c| {
                    if c.len() > additional_size + c1_rep.len() {
                        return None
                    }
                    let id = (c + &c1_reversed).rotate_representative();
                    let id2 = (c + &c2_permuted).rotate_representative();
                    Some((id2, id))
                }).collect_vec()
            } else { Vec::new() }
        }).collect()
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
        let (result, _) = self.prove_identity_with_visited(identity, additional_size, count_limit);
        result
    }

    pub fn prove_identity_with_visited(&self, identity: IdentityCirc, additional_size: usize, count_limit: usize) -> (Option<IdentityCirc>, HashSet<IdentityCirc>) {
        let mut search_queue = vec![VecDeque::<IdentityCirc>::new(); additional_size + 1];
        let mut visited = HashSet::new();

        let min_size = identity.len();
        let max_size = additional_size + identity.len();

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
                    let (result, mut visited2) = self.prove_identity_with_visited(new_id, additional_size, count_limit);
                    visited.extend(visited2.drain());
                    return (result, visited);
                } else if self.proved.contains_key(&new_id) {
                    return (None, visited);
                }
                search_queue[new_id.len() - min_size].push_back(new_id);

                if visited.len() > count_limit {
                    return (Some(identity), visited);
                }
            }
        }
        (Some(identity), visited)
    }

    pub fn export_proof(&self, identity: IdentityCirc, additional_size: usize, count_limit: usize) -> Option<Proof> {
        let (result, visited) = self.prove_identity_with_visited(identity.clone(), additional_size, count_limit);
        if result.is_some() { return None; }
        let mut tracker = ProofTracker::new();
        if tracker.prove(&identity, &self, additional_size, count_limit, &visited){
            Some(Proof(tracker.export(identity.hash_value())))
        } else {
            panic!("Failed to export proof for proved identity");
        }
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

pub mod proof;

#[cfg(test)]
mod test {
    use rand::SeedableRng;

    use crate::{circ::gates::*, identity::{circuit::Circ, eccprove::IdentityProver}, search::double_perm_search::{Evaluator, RawECCs}};

    #[test]
    fn test() {
        let nqubits = 5;
        let ngates = 3;
        let evaluator1 = Evaluator::from_random(nqubits, &mut rand::rngs::StdRng::from_seed([1; 32]));
        let (ecc1, _) = RawECCs::generate(&evaluator1, vec![*H, *X, *Y, *Z, *CX, *CY, *CZ, *S, *SDG, *T, *TDG], ngates);
        let eccs = ecc1.simplify().filter_single();
        let prover =  IdentityProver::build(eccs.as_slice());
        // let prover: IdentityProver = postcard::from_io((
        //     std::io::BufReader::new(std::fs::File::open(".cache/prover-common-clifford-t-3-5.prover").unwrap()),
        //     &mut [0; 8192],
        // )).unwrap().0;

        for id in prover.assumed_identities() {
            println!("{}", id);
        }

        let proof = prover.export_proof(
            Circ::new_no_perm(vec![cx(0, 1), cy(0, 2), cx(0, 1), cy(0, 2)]).rotate_representative(), 5, 50000);

        println!("{:?}", proof)
    }
}