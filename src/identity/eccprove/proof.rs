//! Proof tracking and exported proof representation.

use std::{arch::x86_64::_CMP_FALSE_OS, cmp::Reverse, collections::{BinaryHeap, HashMap, HashSet, VecDeque}, future::Future, ops::IndexMut, task::Waker};

use clap::Id;
use derive_more::{Deref, DerefMut, From, Index, Into};
use itertools::Itertools;
use rayon::iter::ParallelIterator;
use spin::Mutex;

use crate::{identity::{circuit::Circ, eccprove::IdentityProver, idcircuit::{IdentityCirc, IdentitySubcircuit}}, utils::HashTable64};

#[derive(Debug, Clone)]
pub enum ProofFuture {
    Result(Option<(u64, u64)>),
    Waiting {
        expanded: bool,
        notify: Vec<(u64, u64)>
    }
}

impl Default for ProofFuture {
    fn default() -> Self {
        ProofFuture::Waiting {
            expanded: false,
            notify: Vec::new(),
        }
    }
}

pub struct ProofTracker(HashTable64<IdentityCirc, ProofFuture>);

impl ProofTracker {

    pub fn new() -> Self {
        ProofTracker(HashTable64::new())
    }

    pub fn prove(&mut self, identity: &IdentityCirc, prover: &IdentityProver, additional_size: usize, count_limit: usize, filter_set: &HashSet<IdentityCirc>) -> bool {
        if prover.assume.contains(identity) {
            self.0.insert(identity.clone(), ProofFuture::Result(None));
            return true;
        }
        let mut search_queue = BinaryHeap::<Reverse<(usize, u64)>>::new();
        let max_size = additional_size + identity.len();

        let initial = identity.hash_value();
        search_queue.push(Reverse((identity.len(), initial)));

        self.0.insert(identity.clone(), Default::default());
        let mut counter = 0;
        while let Some(Reverse((len, key))) = search_queue.pop() {
            if self.0.len() > count_limit {
                println!("Proof search exceeded limit of {count_limit} states.");
                break;
            }
            if self.prune(key, initial) {
                if counter % 50 == 0 {
                    println!("Exploring {key:x} {} {} ({} queued)", len, self.0.address(key).unwrap().0, search_queue.len());
                }
                self.expand(key, prover, max_size, &mut search_queue, filter_set);
                counter += 1;
            }

            if let ProofFuture::Result(_) = self.0.address(initial).unwrap().1 {
                return true;
            }
        }
        false
    }

    fn expand(&mut self, key: u64, prover: &IdentityProver, max_size: usize, que: &mut BinaryHeap<Reverse<(usize, u64)>>, filter_set: &HashSet<IdentityCirc>) -> bool {
        let a = if let (id0, ProofFuture::Waiting{expanded, ..}) = self.0.address_mut(key).unwrap() {
            if !*expanded {
                *expanded = true;
                prover.par_transition_pairs(&id0, max_size - id0.len())
            }
            else { return false; }
        } else { return false; };

        for (id1, id2) in a {
            if !prover.proved.contains_key(&id1) && !filter_set.contains(&id1) {
                continue;
            }

            if !prover.proved.contains_key(&id2) && !filter_set.contains(&id2) {
                continue;
            }

            let id1_len = id1.len();
            let id2_len = id2.len();
            let id1hash = id1.hash_value();
            let id2hash = id2.hash_value();

            if prover.assume.contains(&id1) {
                self.0.insert(id1, ProofFuture::Result(None));
            } else if self.wait(id1, (id2hash, key)) {
                assert!(id1hash != key);
                que.push(Reverse((id1_len, id1hash)));
            }
            
            if prover.assume.contains(&id2) {
                self.0.insert(id2,  ProofFuture::Result(None));
            } else if self.wait(id2, (id1hash, key)) {
                assert!(id2hash != key);
                que.push(Reverse((id2_len, id2hash)));
            }

            if self.notify(id1hash, id2hash, key) {
                return true;
            }
        }

        false
    }

    fn wait(&mut self, identity: IdentityCirc, notify: (u64, u64)) -> bool {
        let hash = identity.hash_value();
        if let Some((_, ProofFuture::Waiting{notify: n, expanded})) = self.0.address_or(hash, (identity, Default::default())) {
            n.push(notify);
            !*expanded
        } else {
            false
        }
    }
    fn notify(&mut self, id0: u64, id1: u64, id2: u64) -> bool {
        let Some((_, ProofFuture::Result(_))) = self.0.address_mut(id0) else { return false; };
        let Some((_, ProofFuture::Result(_))) = self.0.address_mut(id1) else { return false; };
        let Some((_, fut2)) = self.0.address_mut(id2) else { return false; };
        // assert!(!matches!(self.0.address(id2).unwrap().1, ProofFuture::Result(_)));
        if let ProofFuture::Result(_) = fut2 { return false; }

        let f = std::mem::replace(fut2, ProofFuture::Result(Some((id0, id1))));
        if let ProofFuture::Waiting{notify, ..} = f {
            for (n1, n2) in notify {
                self.notify(id2, n1, n2);
            }
        }

        assert!(matches!(self.0.address(id2).unwrap().1, ProofFuture::Result(_)));
        let i2 = self.0.address_mut(id2).unwrap().0.clone();
        let i0 = self.0.address_mut(id0).unwrap().0.clone();
        let i1 = self.0.address_mut(id1).unwrap().0.clone();
        // println!("Found proof for {id2:x} {i2} <- {i0}, {i1}");
        true
    }
    fn prune(&mut self, key: u64, initial: u64) -> bool {
        let mut set = HashSet::new();
        set.insert(key);
        self.check_reach(key, initial, &mut set) 
    }

    fn check_reach(&self, key: u64, initial: u64, set: &mut HashSet<u64>) -> bool {
        if key == initial {
            return true;
        }
        if let Some((_, ProofFuture::Waiting{notify, ..})) = &self.0.address(key) {
            for (_, n2) in notify {
                if !set.insert(*n2) { continue; }
                if self.check_reach(*n2, initial, set) {
                    return true;
                }
            }
        }
        false
    }
    pub fn export(&self, key: u64) -> Vec<(IdentityCirc, Option<(usize, usize)>)> {
        let mut map = HashMap::<u64, usize>::new();
        self.export_inner(key, &mut map);
        // println!("Exporting Map {:x?}", map);
        
        let mut res = vec![(IdentityCirc::new(), None); map.len()];

        for (k, v) in map.iter() {
            let (id, ProofFuture::Result(opt)) = &self.0.address(*k).unwrap() else {
                panic!("{:?}", self.0.address(*k).unwrap());
            };
            res[*v] = (id.clone(), opt.map(|(a, b)| (map[&a], map[&b])));
        }

        res
    }

    fn export_inner(&self, key: u64, map: &mut HashMap::<u64, usize>) {
        map.insert(key, map.len());
        let ProofFuture::Result(Some((a, b))) = &self.0.address(key).unwrap().1 else { return; };
        // println!("Exporting {:x} {:x?}", a, map);
        if !map.contains_key(a) {
            self.export_inner(*a, map);
        }
        // println!("Exporting {:x} {:x?}", b, map);
        if !map.contains_key(b) {
            self.export_inner(*b, map);
        }
    }
}

#[pyo3_stub_gen::derive::gen_stub_pyclass]
#[pyo3::pyclass(eq, ord)]
#[derive(Debug, Clone, Deref, DerefMut, Default, PartialEq, Eq, Hash, PartialOrd, Ord, serde::Serialize, serde::Deserialize, Index, From, Into)]
/// Exported proof DAG.
///
/// Each entry stores:
/// - an identity circuit,
/// - either `None` (assumption) or `(lhs_idx, rhs_idx)` showing how it is derived.
pub struct Proof(pub Vec<(IdentityCirc, Option<(usize,usize)>)>);

impl std::fmt::Display for Proof {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, (id, opt)) in self.0.iter().enumerate() {
            if let Some((a, b)) = opt {
                writeln!(f, "{i}: {} <- {a}, {b}", id)?;
            } else {
                writeln!(f, "{i}: {} <- ASSUME", id)?;
            }
        }
        Ok(())
    }
}
#[pyo3_stub_gen::derive::gen_stub_pymethods]
#[pyo3::pymethods]
impl Proof {
    #[new]
    /// Creates a proof from raw `(identity, dependency)` entries.
    fn new_py(data: Vec<(IdentityCirc, Option<(usize,usize)>)>) -> Self {
        Proof(data)
    }

    #[getter]
    /// Returns raw proof entries.
    fn raw(&self) -> Vec<(IdentityCirc, Option<(usize,usize)>)> {
        self.0.clone()
    }

    fn __str__(&self) -> String {
        format!("{}", self)
    }

    fn __repr__(&self) -> String {
        format!("{:?}", self.0.len())
    }
    
}
