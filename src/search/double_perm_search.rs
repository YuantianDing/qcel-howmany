//! Symmetry-aware ECC search.
//!
//! This module builds raw equivalence classes of circuits (ECCs) by hashing
//! evaluated states and tracking representative circuits with front/back
//! permutations.

use std::{cell::Cell, cmp::Ordering, collections::{BTreeSet, HashMap, VecDeque, hash_map::Entry}};

use derive_more::{Deref, Debug};
use indicatif::ProgressIterator;
use itertools::Itertools;
use nohash_hasher::BuildNoHashHasher;
use postcard::fixint::le;
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pymethods};
use rand::{SeedableRng, rngs::StdRng};
use rayon::iter::{IntoParallelRefIterator, ParallelBridge, ParallelIterator};
use smallvec::SmallVec;


use crate::{
    circ::{Gate16, Instr32}, groups::permutation::Permut32, identity::circuit::Circ, search::ECC, state::StateVec, utils::{AliasList, FmtJoinIter}
};
use linear_map::{LinearMap, set::LinearSet};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[debug("{} {} {}", self.front_perm, self.circ.clone().collect_vec().iter().rev().fjoin(" "), self.back_perm)]
/// A circuit stored with canonicalization permutations on both ends.
///
/// `circ` is kept in reverse (linked-list style) for efficient BFS extension.
pub struct CircTriple{
    pub front_perm: Permut32,
    pub circ: AliasList<Instr32>,
    pub back_perm: Permut32
}

impl CircTriple {
    /// Converts the internal reverse representation into forward instruction order,
    /// and combines front/back permutations into one output permutation.
    pub fn simplify(&self) -> (Vec<Instr32>, Permut32) {
        let front_perm_inv = self.front_perm.inv();
        let mut instrs = self.circ.iter().map(|a| a.apply_permutation(front_perm_inv)).collect::<Vec<_>>();
        instrs.reverse();

        (instrs, self.back_perm * self.front_perm)
    }
}

impl std::fmt::Display for CircTriple {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (v, p) = self.simplify();
        write!(f, "{} {p}", v.iter().fjoin(" "))
    }
}

#[derive(Debug, derive_more::Display, Clone, PartialEq, Eq)]
#[display("CircuitECC {{{}}}", self.circuits.iter().fjoin(", "))]
/// Raw equivalence class bucket keyed by evaluated state hash.
pub struct CircuitECC {
    pub size: usize,
    pub front_gates: LinearSet<Instr32>,
    pub back_gates: LinearSet<Instr32>,
    pub circuits: Vec<CircTriple>,
}

impl CircuitECC {
    /// Creates the root ECC containing only the empty circuit.
    pub fn root(nqubits: usize) -> CircuitECC {
        CircuitECC {
            size: 0,
            front_gates: LinearSet::new(),
            back_gates: LinearSet::new(),
            circuits: vec![
                CircTriple {
                    front_perm: Permut32::identity(nqubits as u8),
                    circ: AliasList::nil(),
                    back_perm: Permut32::identity(nqubits as u8),
                }
            ],
        }
    }

    /// Converts this raw bucket to a normalized public [`ECC`] representation.
    pub fn simplify(&self) -> super::ECC {
        let mut result = self.circuits.iter().map(|triple| triple.simplify()).collect_vec();
        result.sort_by(|a, b| (a.0.len(), a).cmp(&(b.0.len(), b)));
        let set: BTreeSet<_> = result[0].0.iter().flat_map(|a| a.1.iter().cloned()).collect();
        let uniform_inv = Permut32::from_iter_with_ext(result[0].1.len(), set.into_iter());
        let uniform = uniform_inv.inv();
        result.into_iter().map(|(instrs, perm)| (
                instrs.into_iter().map(|a| a.apply_permutation(uniform)).collect(),
                uniform * perm * uniform_inv
        )).collect_vec().into()
    }

    /// Checks whether a candidate's front/back surface gates are new for this bucket.
    pub fn test_filter(&self, front_perm: Permut32, instr_vec: &[Instr32], back_perm: Permut32) -> bool {
        let front_gates_iter = circuit_get_surface_gates(instr_vec.iter())
            .map(|instr| instr.apply_permutation(front_perm.inv()));
        let back_gates_iter = circuit_get_surface_gates(instr_vec.iter().rev())
            .map(|instr| instr.apply_permutation(back_perm));
        let front_unique = front_gates_iter.clone().all(|instr| !self.front_gates.contains(&instr));
        let back_unique = back_gates_iter.clone().all(|instr| !self.back_gates.contains(&instr));

        front_unique && back_unique
    }

    /// Produces a concrete pair of equivalent circuits for diagnostics/export.
    pub fn export_equivalence(&self, front_perm: Permut32, instr_vec: &[Instr32], back_perm: Permut32) -> (Vec<Instr32>, Permut32, Vec<Instr32>, Permut32) {
        let instrs20 = instr_vec.iter().map(|a| a.apply_permutation(front_perm.inv())).collect::<Vec<_>>();
        let perm2 = back_perm * front_perm;

        for circ in self.circuits.iter() {
            let mut instrs1 = circ.circ.iter().map(|a| a.apply_permutation(circ.front_perm.inv())).collect::<Vec<_>>();
            instrs1.reverse();
            let perm1 = circ.back_perm * circ.front_perm;
            let mut instrs2 = instrs20.clone();
            if instrs1 == instrs2 && perm1 == perm2 {
                return (vec![], perm1, vec![], perm2);
            }

            if reduce_equivalence(
                (&mut instrs1, perm1),
                (&mut instrs2, perm2),
            ) {
                return (instrs1, perm1, instrs2, perm2);
            }
        }
        panic!("No equivalence found during export_equivalence: {} = {} {}", 
            self.circuits.iter().fjoin(", "), 
            instrs20.iter().fjoin(", "), perm2);
    }
}

#[gen_stub_pyclass]
#[pyo3::pyclass(eq)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// Randomized evaluator used to hash circuits and recover permutation metadata.
pub struct Evaluator {
    pub initial_state: StateVec,
    pub backtrack_state: StateVec,
}

impl Evaluator {
    /// Builds a deterministic evaluator from a seeded RNG.
    pub fn from_random(nqubits: usize, rng: &mut StdRng) -> Self {
        let initial_state = StateVec::from_random_symmetric(rng, nqubits as u32);

        let mut backtrack_state = StateVec::from_random(rng, nqubits as u32);
        backtrack_state.normalize_arg();
        backtrack_state.apply_permutation(backtrack_state.get_permutation().inv());

        Self { initial_state, backtrack_state }
    }
    // pub fn evaluate_detailed(&self, instrs: &[Instr32]) -> (StateVec, StateVec, Permut32, Permut32, u8) {
    //     let mut state = self.initial_state.clone();
    //     let mut mask = 0u8;
    //     for instr in instrs.iter() {
    //         state.apply(&instr.1, instr.0);
    //         mask |= instr.arg_mask();
    //     }
        
    //     state.normalize_arg();
    //     let (back_perm_inv, mut eq_mask) = state.get_orderinfo().as_bits();
    //     let back_perm = back_perm_inv.inv();

    //     let mut backstate = self.backtrack_state.clone();
    //     backstate.apply_permutation(back_perm_inv);
    //     for Instr32(gate, idx) in instrs.iter().rev() {
    //         backstate.apply(idx, gate.adjoint());
    //     }
    //     backstate.normalize_arg();
    //     let front_perm = backstate.get_permutation();
    //     let front_perm_inv = front_perm.inv();
    //     backstate.apply_permutation(front_perm_inv);

    //     (state, backstate, front_perm, back_perm, eq_mask & mask)
    // }
    /// Evaluates one circuit and returns:
    /// - canonical backtracked state,
    /// - front permutation,
    /// - back permutation,
    /// - forward-evolved state.
    pub fn evaluate(&self, instrs: &[Instr32]) -> (StateVec, Permut32, Permut32, StateVec) {
        let (backstate, vec, oi) = self.evaluate_multiple(instrs);

        (backstate, vec[0].0, vec[0].1, oi)
    }
    /// Evaluates one circuit and returns all equivalent `(front_perm, back_perm)` pairs
    /// that map to the same minimal backtracked state.
    pub fn evaluate_multiple(&self, instrs: &[Instr32]) -> (StateVec, SmallVec<[(Permut32, Permut32); 1]>, StateVec) {
        let mut state = self.initial_state.clone();
        // let mut mask = 0u8;
        for instr in instrs.iter() {
            state.apply(&instr.1, instr.0);
            // mask |= instr.arg_mask();
        }
        
        state.normalize_arg();
        let oi = state.get_orderinfo();
        let mut true_back_state = None;
        let mut perms = SmallVec::new();
        for back_perm_inv in oi.as_perms() {
            let back_perm = back_perm_inv.inv();

            let mut backstate = self.backtrack_state.clone();
            backstate.apply_permutation(back_perm_inv);
            for Instr32(gate, idx) in instrs.iter().rev() {
                backstate.apply(idx, gate.adjoint());
            }
            backstate.normalize_arg();
            let (front_perm, front_perm_eq) = backstate.get_orderinfo().as_bits();
            assert!(front_perm_eq == 0, "Non-deterministic front permutation detected during evaluation");
            let front_perm_inv = front_perm.inv();
            backstate.apply_permutation(front_perm_inv);
            if true_back_state.is_none() {
                true_back_state = Some(backstate);
                perms.push((front_perm, back_perm));
            } else if let Some(ref tb) = true_back_state {
                match backstate.cmp(&tb) {
                    Ordering::Less => {
                        true_back_state = Some(backstate);
                        perms.clear();
                        perms.push((front_perm, back_perm));
                    },
                    Ordering::Equal => {
                        perms.push((front_perm, back_perm));
                    },
                    _ => ()  
                } 
            }
        }

        (true_back_state.unwrap(), perms, state)
    }
}

#[gen_stub_pymethods]
#[pyo3::pymethods]
impl Evaluator {
    #[new]
    /// Creates a random evaluator for `nqubits`.
    fn random_py(nqubits: usize) -> Self {
        Self::from_random(nqubits, &mut rand::rngs::StdRng::from_os_rng())
    }
    /// Returns evaluator qubit count.
    pub fn nqubits(&self) -> usize {
        self.initial_state.nqubits()
    }

    #[pyo3(name="evaluate")]
    /// Python wrapper for circuit evaluation.
    pub fn evaluate_py(&self, instrs: Vec<Instr32>) -> (StateVec, Permut32, Permut32, StateVec) {
        self.evaluate(&instrs)
    }

    /// Hash key for the empty circuit under this evaluator.
    fn initial_key(&self) -> u64 {
        self.evaluate(&[]).0.hash_value()
    }
}

#[gen_stub_pyclass]
#[pyo3::pyclass(name="RawECCs")]
#[derive(Clone, Deref)]
/// Mutable ECC map keyed by evaluator hash.
///
/// This is the direct output of the search algorithm before final simplification.
pub struct RawECCs {
    #[deref]
    inner: HashMap<u64, CircuitECC, BuildNoHashHasher<u64>>,
    pub nqubits: usize,
    pub drop: bool,
}


impl std::ops::Drop for RawECCs {
    fn drop(&mut self) {
        if self.drop {
            for (_, instrs) in self.inner.iter_mut() {
                for triple in instrs.circuits.iter_mut() {
                    unsafe {
                        triple.circ.delete();
                    }
                }
            }
        }
    }
}

fn circuit_get_surface_gates<'a>(circ: impl Iterator<Item = &'a Instr32> + Clone) -> impl Iterator<Item = &'a Instr32> + Clone {
    let mut mask = 0;
    circ.filter(move |Instr32(_, qubs)| qubs.iter().filter(|&qubit| {
        let qubit_mask = 1 << qubit;
        if mask & qubit_mask == 0 {
            mask |= qubit_mask;
            true
        } else {
            false
        }
    }).count() == qubs.len()) 
}
fn circuit_get_surface_gates_hashmap<'a>(circ: &[Instr32], filtering_mask: u64) -> LinearMap<Instr32, usize> {
    let mut mask = 0;
    circ.iter().cloned().enumerate().filter(move |(i, Instr32(_, qubs))| 
        (filtering_mask & (1 << i)) != 0 &&
        qubs.iter().filter(|&qubit| {
            let qubit_mask = 1 << qubit;
            if mask & qubit_mask == 0 {
                mask |= qubit_mask;
                true
            } else {
                false
            }
        }).count() == qubs.len()
    ).map(|(a,b)| (b, a)).collect()
}


impl RawECCs {
    fn add_entry(&mut self, hash_value: u64, front_perm: Permut32, back_perm: Permut32, circ: &AliasList<Instr32>, instr_vec: &[Instr32]) -> Option<AliasList<Instr32>> {
        let instr = instr_vec.last().unwrap().clone();
        let front_gates_iter = circuit_get_surface_gates(instr_vec.iter())
            .map(|instr| instr.apply_permutation(front_perm.inv()));
        let back_gates_iter = circuit_get_surface_gates(instr_vec.iter().rev())
            .map(|instr| instr.apply_permutation(back_perm));

        match self.inner.entry(hash_value) {
            Entry::Vacant(v) => {
                // println!("\t{instr}: {front_perm}, {back_perm} new value");
                let new_point = circ.cons(instr);
                assert!(instr_vec.len() > 0);
                v.insert(CircuitECC {
                    size: instr_vec.len(),
                    front_gates: front_gates_iter.collect(),
                    back_gates: back_gates_iter.collect(),
                    circuits: vec![CircTriple{front_perm, circ: new_point.clone(), back_perm}],
                });
                Some(new_point)
            }
            Entry::Occupied(mut o) => {
                let entry = o.get_mut();
                let front_unique = front_gates_iter.clone().all(|instr| !entry.front_gates.contains(&instr));
                let back_unique = back_gates_iter.clone().all(|instr| !entry.back_gates.contains(&instr));
                if front_unique && back_unique {
                    for instr in front_gates_iter { entry.front_gates.insert(instr); }
                    for instr in back_gates_iter { entry.back_gates.insert(instr); }
                    // print!("\t{instr}: ");
                    let new_point = circ.cons(instr);
                    let triple = CircTriple { front_perm, circ: new_point.clone(), back_perm};
                    // println!("{} equal -> {} {}", triple, entry, hash_value);
                    entry.circuits.push(triple);
                } else {
                    // print!("\t{instr}: ");
                    // let new_point = circ.cons(instr);
                    // let triple = CircTriple { front_perm, circ: new_point.clone(), back_perm};
                    // let cause = front_gates_iter.clone().filter(|instr| entry.front_gates.contains(&instr)).chain(
                    //     back_gates_iter.clone().filter(|instr| entry.back_gates.contains(&instr))
                    // ).collect_vec();
                    // println!("{} skip -> {} {} {} {} cause: {}", triple, entry, 
                    //     entry.front_gates.iter().fjoin(", "),
                    //     entry.back_gates.iter().fjoin(", "),
                    //     hash_value,
                    //     cause.iter().join_option(", ", "{", "}"));
                }
                None
            }
        }
    }
    /// Finds all known circuits equivalent to `instrs` under the given evaluator.
    pub fn find_equivalents(&self, evaluator: &Evaluator, instrs: &[Instr32]) -> Option<ECC> {
        let (backstate, front_perm, back_perm, _) = evaluator.evaluate(instrs);

        self.inner.get(&backstate.hash_value()).map(|ecc| {
            ECC(ecc.circuits
                .iter()
                .map(|triple| CircTriple{
                    front_perm: triple.front_perm * front_perm.inv(),
                    back_perm: back_perm.inv() * triple.back_perm,
                    circ: triple.circ.clone(),
                }.simplify())
                // .filter(|(c, p)| c != instrs || *p != Permut32::identity(evaluator.nqubits() as u8))
                .collect_vec())
        }).and_then(|ecc| if ecc.0.is_empty() { None } else { Some(ecc) })
    }
    /// Rewrites a circuit recursively to the canonical representative within this ECC map.
    ///
    /// Used as an internal consistency check when comparing two search strategies.
    pub fn checked_equivalent(&self, instrs: &[Instr32], perm: Permut32, evaluator: &Evaluator) -> (Vec<Instr32>, Permut32) {
        let mut vec = Vec::<Instr32>::new();
        let mut permut = perm;
        for instr in instrs.iter() {
            vec.push(instr.apply_permutation(permut.inv()));
            let (backstate, f, b, _) = evaluator.evaluate(&vec);
            if let Some(a) = self.inner.get(&backstate.hash_value()) {
                let (instrs1, perm1, instrs2, perm2) = a.export_equivalence(f, &vec, b);
                if instrs1.len() + instrs2.len() > 0 {
                    let (instrs10, perm10) = self.checked_equivalent(&instrs1, perm1, evaluator);
                    let (instrs20, perm20) = self.checked_equivalent(&instrs2, perm2, evaluator);
                    assert!(instrs10 == instrs20 && perm10 == perm20,
                        "Non-deterministic equivalence detected during checked equivalence: \n {} vs {} \n {} {} vs {} {} \n {} {} vs {} {}",
                            a,
                            vec.iter().map(|a| a.apply_permutation(f.inv())).fjoin(" "),
                            instrs1.iter().fjoin(", "), perm1,
                            instrs2.iter().fjoin(", "), perm2,
                            instrs10.iter().fjoin(", "), perm10,
                            instrs20.iter().fjoin(", "), perm20,
                        );
                }

                let CircTriple { front_perm, circ, back_perm } = &a.circuits[0];
                let fperm = *front_perm * f.inv();
                vec = circ.iter().map(|a| a.apply_permutation(fperm.inv())).collect_vec();
                vec.reverse();
                permut = permut * b.inv() * *back_perm * fperm;
            }
        }

        let (bv, vec, _) = &evaluator.evaluate_multiple(&instrs);
        vec.into_iter().map(|(f, b)|
            self.inner[&bv.hash_value()].circuits.iter().map(|triple| {
                let fperm = triple.front_perm * f.inv();
                let mut v = triple.circ.iter().map(|a| a.apply_permutation(fperm.inv())).collect_vec();
                v.reverse();
                let p = perm * b.inv() * triple.back_perm * fperm;
                (v, p)
            }).min().unwrap()
        ).min().unwrap()
    }
}

#[gen_stub_pymethods]
#[pyo3::pymethods]
impl RawECCs {
    /// Initializes the raw map with the empty circuit class.
    #[new]
    pub fn new(evaluator: &Evaluator) -> Self {
        let mut map = RawECCs {
            inner: Default::default(),
            nqubits: evaluator.nqubits(),
            drop: false,
        };
        map.inner.insert(evaluator.evaluate(&[]).0.hash_value(), CircuitECC::root(evaluator.nqubits()));
        map
    }
    /// Exhaustive baseline search without permutation-based multiplicity reduction.
    #[staticmethod]
    fn search_naive(evaluator: &Evaluator, instrs: Vec<Instr32>, max_size: usize) -> (RawECCs, [usize; 3]) {
        let mut map = RawECCs::new(evaluator);
        let nqubits = evaluator.nqubits() as u8;
        
        let mut queue: VecDeque<AliasList<Instr32>> = VecDeque::new();
        queue.push_back(AliasList::nil());

        let mut instr_vec = Vec::new();
        let mut counters = [0; 3];
        while let Some(circ) = queue.pop_front() {
            instr_vec.clear();
            instr_vec.extend(circ.iter().cloned());
            instr_vec.reverse();
            // let mask = instr_vec.iter().fold(0u8, |a, i| a | i.arg_mask());
            
            if counters[0] % 400 == 0 {
                println!("#{} Exploring {} ({} queued, {} ECCs, {} circs, {} circs perm)", counters[0], instr_vec.iter().fjoin(" "), queue.len(), map.len(), counters[1], counters[2]);
            }
            
            for instr in instrs.iter() {
                // if instr.pass_mask(mask).is_none() { continue; }
                
                instr_vec.push(instr.clone());
                let mut state = evaluator.backtrack_state.clone();
                for instr in instr_vec.iter() {
                    state.apply(&instr.1, instr.0);
                }
                state.normalize_arg();
                
                if let Some(new_point) = map.add_entry(state.hash_value(), Permut32::identity(nqubits), Permut32::identity(nqubits), &circ, &instr_vec) {
                    if instr_vec.len() < max_size { queue.push_back(new_point.clone()); }
                }
                counters[2] += 1;

                instr_vec.pop();
                counters[1] += 1;
            }
            counters[0] += 1;
        }
        map.drop = true;
        (map, counters)
    }
    /// Optimized search using permutation-equivalent placements per circuit.
    #[staticmethod]
    fn search(evaluator: &Evaluator, instrs: Vec<Instr32>, max_size: usize) -> (RawECCs, [usize; 3]) {
        let mut map = RawECCs::new(evaluator);
        
        let mut queue: VecDeque<AliasList<Instr32>> = VecDeque::new();
        queue.push_back(AliasList::nil());

        let mut instr_vec = Vec::new();
        let mut counters = [0; 3];
        while let Some(circ) = queue.pop_front() {
            instr_vec.clear();
            instr_vec.extend(circ.iter().cloned());
            instr_vec.reverse();
            // let mask = instr_vec.iter().fold(0u8, |a, i| a | i.arg_mask());
            
            if counters[0] % 400 == 0 {
                println!("#{} Exploring {} ({} queued, {} ECCs, {} circs, {} circs perm)", counters[0], instr_vec.iter().fjoin(" "), queue.len(), map.len(), counters[1], counters[2]);
            }
            
            for instr in instrs.iter() {
                // if instr.pass_mask(mask).is_none() { continue; }
                
                instr_vec.push(instr.clone());
                let (backstate, vec, _) = evaluator.evaluate_multiple(&instr_vec[..]);
                let hash_value = backstate.hash_value();
                for (front_perm, back_perm) in vec {
                    if let Some(new_point) = map.add_entry(hash_value, front_perm, back_perm, &circ, &instr_vec) {
                        if instr_vec.len() < max_size { queue.push_back(new_point.clone()); }
                        // println!("\t{}: {}, {} new value {}", instr, front_perm, back_perm, hash_value);
                    }
                    counters[2] += 1;
                }
                instr_vec.pop();
                counters[1] += 1;
            }
            counters[0] += 1;
        }
        map.drop = true;
        (map, counters)
    }

    /// Converts raw buckets into sorted, normalized ECCs.
    pub fn simplify(&self) -> super::ECCs {
        let mut eccs: Vec<_> = self.inner.values().map(|a| a.simplify()).collect();
        eccs.sort();
        super::ECCs {
            eccs,
            nqubits: self.nqubits,
        }
    }
    /// Generates ECCs from a gate set using the optimized search.
    ///
    /// Adjoint gates are added automatically if missing.
    #[staticmethod]
    pub fn generate(
        evaluator: &Evaluator,
        mut gates: Vec<Gate16>,
        max_size: usize,
    ) -> (RawECCs, [usize; 3]) {
        let adjoint_gates = gates.iter().map(|g| g.adjoint()).collect_vec();

        for g in adjoint_gates {
            if !gates.contains(&g) {
                gates.push(g);
            }
        }
        
        gates.sort_by_key(|g| (g.nqargs(), *g));
        

        let mut instructions: Vec<Instr32> = Vec::new();

        for instr in gates {
            match instr.nqargs() {
                1 => instructions.extend((0..evaluator.nqubits()).map(|i| Instr32(instr.clone(), [i as u8].into_iter().collect()))),
                2 => instructions.extend((0..evaluator.nqubits()).flat_map(|i| {
                    (0..evaluator.nqubits())
                        .filter(move |j| *j != i)
                        .map(move |j| Instr32(instr.clone(), [i as u8, j as u8].into_iter().collect()))
                })),
                _ => panic!("Only 1 and 2 qubit instructions are supported"),
            }
        }

        instructions.sort_by_key(|a| a.largest_qubit());
        RawECCs::search(&evaluator, instructions, max_size)
    }
    /// Generates ECCs from a gate set using the naive search.
    #[staticmethod]
    pub fn generate_naive(
        evaluator: &Evaluator,
        mut gates: Vec<Gate16>,
        max_size: usize,
    ) -> (RawECCs, [usize; 3]) {
        let adjoint_gates = gates.iter().map(|g| g.adjoint()).collect_vec();

        for g in adjoint_gates {
            if !gates.contains(&g) {
                gates.push(g);
            }
        }
        
        gates.sort_by_key(|g| (g.nqargs(), *g));

        let mut instructions: Vec<Instr32> = Vec::new();

        for instr in gates {
            match instr.nqargs() {
                1 => instructions.extend((0..evaluator.nqubits()).map(|i| Instr32(instr.clone(), [i as u8].into_iter().collect()))),
                2 => instructions.extend((0..evaluator.nqubits()).flat_map(|i| {
                    (0..evaluator.nqubits())
                        .filter(move |j| *j != i)
                        .map(move |j| Instr32(instr.clone(), [i as u8, j as u8].into_iter().collect()))
                })),
                _ => panic!("Only 1 and 2 qubit instructions are supported"),
            }
        }

        instructions.sort_by_key(|a| a.largest_qubit());
        RawECCs::search_naive(&evaluator, instructions, max_size)
    }

    #[pyo3(name="find_equivalents")]
    /// Python wrapper returning equivalent circuits for a candidate program.
    fn find_equivalents_py(&self, evaluator: &Evaluator, instrs: Vec<Instr32>) -> Option<ECC> {
        self.find_equivalents(evaluator, &instrs)
    }

    /// Computes the next reachable ECC hash after appending one instruction.
    ///
    /// Returns `None` when the resulting state was not discovered in this map.
    pub fn compute_next_key(&self, evaluator: &Evaluator, current_key: u64, instr: Instr32) -> Option<u64> {
        self.inner.get(&current_key).and_then(|ecc| {
            let triple = ecc.circuits.first().unwrap();
            let mut instrs = triple.circ.iter().cloned().collect_vec();
            instrs.push(instr.clone());
            let (backstate, _, _, _) = evaluator.evaluate(&instrs);
            let hash_value = backstate.hash_value();
            if self.inner.contains_key(&hash_value) {
                Some(hash_value)
            } else { None }
        })
    }

    /// Re-evaluates all stored circuits under a new evaluator.
    ///
    /// Useful for cross-checking evaluator-independence of discovered classes.
    pub fn switch_evaluator(&self, new_evaluator: &Evaluator) -> RawECCs {
        let mut new_map = RawECCs::new(new_evaluator);
        for ecc in self.inner.values() {
            let instrs = ecc.circuits[0].circ.iter().cloned().collect_vec();
            let (backstate, _, _, _) = new_evaluator.evaluate(&instrs);
            let hash = backstate.hash_value();
            let mut result = CircuitECC {
                size: instrs.len(),
                front_gates: LinearSet::new(),
                back_gates: LinearSet::new(),
                circuits: vec![],
            };

            for triple in ecc.circuits.iter() {
                let instrs = triple.circ.iter().cloned().collect_vec();
                let (backstate, front_perm, back_perm, _) = new_evaluator.evaluate(&instrs);
                assert!(backstate.hash_value() == hash, "Inconsistent ECC detected during evaluator switch");

                result.circuits.push(CircTriple { front_perm, circ: triple.circ.clone(), back_perm });
            }
            if let Some(a) = new_map.inner.insert(hash, result) {
                if a.size > 0 {
                    panic!("Hash collision detected during evaluator switch: {:?} vs {:?}", a, ecc);
                }
            }
        }
        new_map
    }

    /// Checks that every identity in `self` is representable in `ecc1`.
    pub fn check_identity_subset(&self, ecc1: &RawECCs, evaluator: &Evaluator) {
        for chunk in &self.values().chunks(4096) {
            chunk.collect_vec().par_iter().for_each(|ecc| {
                for (instrs, p) in &ecc.simplify().0 {
                    let (bv, _, _, _) = evaluator.evaluate(instrs);
                    assert!(ecc1.contains_key(&bv.hash_value()));
                    let _ = ecc1.checked_equivalent(instrs, *p, &evaluator);
                }
            })
        }
    }
    
    // for i in 0..instrs.len() {
    //     for j in (i+1)..=instrs.len() {
    //         let (backstate, front_perm, back_perm) = evaluator.evaluate(&instrs[i..j]);
    //         println!("{i}..{j} {front_perm} {} {back_perm}", instrs[i..j].iter().fjoin(" "));
    //         eccs.get(&backstate.hash_value()).map(|ecc| {
    //             for c in ecc.circuits.iter() {
    //                 println!("\t{}", c);
    //             }
    //             println!("\t{}", CircTriple {
    //                 circ: instrs[i..j].iter().cloned().collect(),
    //                 front_perm,
    //                 back_perm,
    //             });
    //         });
    //     }
    // }

}

fn reduce_equivalence_front(
    nqubits: usize,
    circ1: &[Instr32],
    circ2: &[Instr32],
) -> (u64, u64) {
    let mut mask1 = (1 << circ1.len()) - 1;
    let mut mask2 = (1 << circ2.len()) - 1;

    while {
        let front_gates1 = circuit_get_surface_gates_hashmap(circ1, mask1);
        let front_gates2 = circuit_get_surface_gates_hashmap(circ2, mask2);

        front_gates1.iter()
            .filter_map(|(instr, i)| front_gates2.get(instr).map(|j| (*i, *j)))
            .map(|(i, j)| {
                mask1 &= !(1 << i);
                mask2 &= !(1 << j);
            }).count() > 0
    } {}
    


    (mask1, mask2)
}

fn reduce_equivalence(
    circ1: (&mut Vec<Instr32>, Permut32),
    circ2: (&mut Vec<Instr32>, Permut32),
) -> bool {
    let nqubits = circ1.1.len() as usize;
    let (mask1f, mask2f) = reduce_equivalence_front(nqubits, &*circ1.0, &*circ2.0);
    let (mask1b, mask2b) = reduce_equivalence_front(nqubits,
        &circ1.0.iter().rev().map(|a| a.apply_permutation(circ1.1))
            .collect::<Vec<_>>(),
        &circ2.0.iter().rev().map(|a| a.apply_permutation(circ2.1))
            .collect::<Vec<_>>(),
    );

    let changed1 = retain_by_masks(circ1.0, mask1f, mask1b);
    let changed2 = retain_by_masks(circ2.0, mask2f, mask2b);
    assert!(changed1 == changed2, "mask1f: {:b}, mask1b: {:b}, mask2f: {:b}, mask2b: {:b}", mask1f, mask1b, mask2f, mask2b);
    
    changed1 > 0
}

fn retain_by_masks(circ1: &mut Vec<Instr32>, mask1f: u64, mask1b: u64) -> usize {
    let mut index = 0;
    let mut changed = 0;
    let len1 = circ1.len();
    circ1.retain(|_| {
        index += 1;
        if (mask1f & (1 << (index - 1))) != 0 && (mask1b & (1 << (len1 - index))) != 0 {
            true
        } else {
            changed += 1;
            false
        }
    });
    changed
}

#[cfg(test)]
mod test {
    use rand::{SeedableRng};

    use crate::{circ::gates::{CX, H, T, TDG, X, Y, cx, h, x, y}, groups::permutation::Permut32, identity::{eccprove::IdentityProver, idcircuit::IdentityCirc}, instr_vec, search::{ECCs, double_perm_search::{CircTriple, Evaluator, RawECCs, circuit_get_surface_gates, reduce_equivalence}}, utils::FmtJoinIter};

    #[test]
    fn test_eval() {
        let evaluator1 = Evaluator::from_random(3, &mut rand::rngs::StdRng::from_seed([0; 32]));

        let h1 = evaluator1.evaluate(&[h(0)]).0.hash_value();
        let h2 = evaluator1.evaluate(&[h(1)]).0.hash_value();

        assert_eq!(h1, h2);
    }

    #[test]
    fn test_naive_equivalence() {
        let nqubits = 5;
        let ngates = 3;
        let evaluator = Evaluator::from_random(nqubits, &mut rand::rngs::StdRng::from_seed([0; 32]));
        let (ecc1, _) = RawECCs::generate(&evaluator, vec![*Y, *X, *CX], ngates);
        let (ecc2, _) = RawECCs::generate_naive(&evaluator, vec![*Y, *X, *CX], ngates);

        ecc2.check_identity_subset(&ecc1, &evaluator);
    }
    #[test]
    fn test_same_result() {
        let nqubits = 5;
        let ngates = 6;
        let evaluator1 = Evaluator::from_random(nqubits, &mut rand::rngs::StdRng::from_seed([0; 32]));
        let (ecc1, _) = RawECCs::generate(&evaluator1, vec![*H, *X, *TDG, *T, *CX], ngates);
        let evaluator2 = Evaluator::from_random(nqubits, &mut rand::rngs::StdRng::from_seed([1; 32]));
        let (ecc2, _) = RawECCs::generate(&evaluator2, vec![*H, *X, *TDG, *T, *CX], ngates);
        assert_eq!(ecc1.len(), ecc2.len());

        println!("{}", ecc1.switch_evaluator(&evaluator2).len());
        println!("{}", ecc2.switch_evaluator(&evaluator1).len());
    }
    #[test]
    fn test1() {
        let circ1 = vec![cx(3, 0), cx(2, 3), cx(4, 2)];
        let circ2 = vec![cx(3, 2), cx(3, 0), cx(4, 3)];
        let p = Permut32::identity(5);
        
        let evaluator = Evaluator::from_random(5, &mut rand::rngs::StdRng::from_seed([0; 32]));
        let (ecc1, _) = RawECCs::generate(&evaluator, vec![*CX], 4);

        let (backstate1, front_perm1, back_perm1, _) = evaluator.evaluate(&circ1);
        let (backstate2, front_perm2, back_perm2, _) = evaluator.evaluate(&circ2);

        assert!(backstate1 == backstate2, "{} vs {}", backstate1, backstate2);

        let (instrs1, perm1) = ecc1.checked_equivalent(&circ1, p, &evaluator);
        let (instrs2, perm2) = ecc1.checked_equivalent(&circ2, p, &evaluator);
        assert!(instrs1 == instrs2, "{} vs {}", instrs1.iter().fjoin(" "), instrs2.iter().fjoin(" "));
    }
    #[test]
    fn test2() {
        let instrs1 = vec![x(0), cx(0, 3), y(3)];

        assert!(circuit_get_surface_gates(instrs1.iter()).count() == 1);

        let mut v1 = vec![cx(2, 1), cx(1, 4), cx(3, 0), cx(2, 3), cx(4, 2)];
        let mut v2 = vec![cx(2, 1), cx(1, 4), cx(3, 2), cx(3, 0), cx(4, 3), cx(2, 3)];

        let changed = reduce_equivalence(
            (&mut v1, Permut32::identity(5)),
            (&mut v2, Permut32::identity(5).swap_inputs(2, 3)),
        );

        // assert!(!changed, "Should not change {} vs {}", v1.iter().fjoin(", "), v2.iter().fjoin(", "));
    }

}
