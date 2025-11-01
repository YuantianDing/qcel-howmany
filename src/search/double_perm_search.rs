use std::{cmp::Ordering, collections::{hash_map::Entry, BTreeSet, HashMap, VecDeque}, iter};

use derive_more::{Deref, Debug, Display};
use itertools::Itertools;
use nohash_hasher::BuildNoHashHasher;
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pymethods};
use rand::rngs::ThreadRng;
use smallvec::{smallvec, SmallVec};


use crate::{
    circ::{gates::SWAP, Gate, Instr, Instruction, InstructionSliceExt}, groups::permutation::Permut32, search::ECC, state::StateVec, utils::{AliasList, JoinOptionIter}
};
use linear_map::set::LinearSet;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[debug("{} {} {}", self.front_perm, self.circ.clone().collect_vec().iter().rev().join_option(" ", "", ""), self.back_perm)]
pub struct CircTriple{
    pub front_perm: Permut32,
    pub circ: AliasList<Instr>,
    pub back_perm: Permut32
}

impl CircTriple {
    pub fn simplify(&self) -> (Vec<Instr>, Permut32) {
        let front_perm_inv = self.front_perm.inv();
        let mut instrs = self.circ.iter().map(|a| a.apply_permutation(front_perm_inv)).collect::<Vec<_>>();
        instrs.reverse();

        (instrs, self.back_perm * self.front_perm)
    }
}

impl std::fmt::Display for CircTriple {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (v, p) = self.simplify();
        write!(f, "{} {p}", v.iter().join_option(" ", "", ""))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CircuitECC {
    pub size: usize,
    pub front_gates: LinearSet<Instr>,
    pub back_gates: LinearSet<Instr>,
    pub circuits: Vec<CircTriple>,
}

impl CircuitECC {
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

    pub fn simplify(&self) -> super::ECC {
        let result = self.circuits.iter().map(|triple| triple.simplify()).collect_vec();
        let set: BTreeSet<_> = result[0].0.iter().flat_map(|a| a.1.iter().cloned()).collect();
        let uniform_inv = Permut32::from_iter_with_ext(result[0].1.len(), set.into_iter());
        let uniform = uniform_inv.inv();
        result.into_iter().map(|(instrs, perm)| (
                instrs.into_iter().map(|a| a.apply_permutation(uniform)).collect(),
                uniform * perm * uniform_inv
        )).collect_vec().into()
    }
}

#[gen_stub_pyclass]
#[pyo3::pyclass(eq)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Evaluator {
    pub initial_state: StateVec,
    pub backtrack_state: StateVec,
}

impl Evaluator {
    pub fn from_random(nqubits: usize, rng: &mut ThreadRng) -> Self {
        let initial_state = StateVec::from_random_symmetric(rng, nqubits as u32);

        let mut backtrack_state = StateVec::from_random(rng, nqubits as u32);
        backtrack_state.normalize_arg();
        backtrack_state.apply_permutation(backtrack_state.get_permutation().inv());

        Self { initial_state, backtrack_state }
    }
    pub fn evaluate(&self, instrs: &[Instr]) -> (StateVec, Permut32, Permut32) {
        let mut state = self.initial_state.clone();
        let mut mask = 0u8;
        for instr in instrs.iter() {
            state.apply(&instr.1, instr.0);
            mask |= instr.arg_mask();
        }
        state.normalize_arg();
        let back_perm_inv = state.get_permutation();
        let back_perm = back_perm_inv.inv();

        let mut backstate = self.backtrack_state.clone();
        backstate.apply_permutation(back_perm_inv);
        for Instr(gate, idx) in instrs.iter().rev() {
            backstate.apply(idx, gate.adjoint());
        }
        backstate.normalize_arg();
        let front_perm = backstate.get_permutation();
        let front_perm_inv = front_perm.inv();
        backstate.apply_permutation(front_perm_inv);

        (backstate, front_perm, back_perm)
    }
    fn evaluate_multiple(&self, instrs: &[Instr]) -> SmallVec<[(StateVec, Permut32, Permut32); 1]> {
        let mut state = self.initial_state.clone();
        let mut mask = 0u8;
        for instr in instrs.iter() {
            state.apply(&instr.1, instr.0);
            mask |= instr.arg_mask();
        }
        
        state.normalize_arg();
        let (back_perm_inv, mut eq_mask) = state.get_permutation_with_eq();
        let back_perm = back_perm_inv.inv();

        let mut backstate = self.backtrack_state.clone();
        backstate.apply_permutation(back_perm_inv);
        for Instr(gate, idx) in instrs.iter().rev() {
            backstate.apply(idx, gate.adjoint());
        }
        backstate.normalize_arg();
        let front_perm = backstate.get_permutation();
        let front_perm_inv = front_perm.inv();
        backstate.apply_permutation(front_perm_inv);

        let mut result = smallvec![(backstate, front_perm, back_perm)];

        // Add Isomorphism Points
        eq_mask &= mask;
        if eq_mask > 0 {
            let mask = back_perm.permut_bv(mask);
            state.apply_permutation(back_perm);
            for i in 0u8..(state.nqubits() - 1) as u8 {
                if (mask >> i) & 1 != 0 &&
                (mask >> (i + 1)) & 1 != 0 && 
                state.compare_qubits(i, i+1) == Ordering::Equal &&
                state.qubit_equiv(i, i+1) {
                    
                    let mut real_back_perm = back_perm.clone();
                    real_back_perm.swap_inputs(back_perm_inv.at(i), back_perm_inv.at(i+1));
                    
                    let mut backstate = self.backtrack_state.clone();
                    backstate.apply_permutation(real_back_perm.inv());
                    for Instr(gate, idx) in instrs.iter().rev() {
                        backstate.apply(idx, gate.adjoint());
                    }
                    backstate.normalize_arg();
                    let front_perm = backstate.get_permutation();
                    let front_perm_inv = front_perm.inv();
                    backstate.apply_permutation(front_perm_inv);

                    result.push((backstate, front_perm, real_back_perm))
                }
            }
        }
        result
    }
}

#[gen_stub_pymethods]
#[pyo3::pymethods]
impl Evaluator {
    #[new]
    fn random_py(nqubits: usize) -> Self {
        let mut rng = rand::rng();
        Self::from_random(nqubits, &mut rng)
    }
    pub fn nqubits(&self) -> usize {
        self.initial_state.nqubits()
    }

    #[pyo3(name="evaluate")]
    pub fn evaluate_py(&self, instrs: Vec<Instr>) -> (StateVec, Permut32, Permut32) {
        self.evaluate(&instrs)
    }
}

#[gen_stub_pyclass]
#[pyo3::pyclass(name="RawECCs")]
#[derive(Clone, Deref)]
pub struct RawECCs {
    #[deref]
    inner: HashMap<u64, CircuitECC, BuildNoHashHasher<u64>>,
    pub nqubits: usize,
}


impl std::ops::Drop for RawECCs {
    fn drop(&mut self) {
        for (_, instrs) in self.inner.iter_mut() {
            for triple in instrs.circuits.iter_mut() {
                unsafe {
                    triple.circ.delete();
                }
            }
        }
    }
}

fn circuit_get_surface_gates<'a>(circ: impl Iterator<Item = &'a Instr> + Clone) -> impl Iterator<Item = &'a Instr> + Clone {
    let mut mask = 0;
    circ.filter(move |Instr(_, qubs)| qubs.iter().all(|&qubit| {
        let qubit_mask = 1 << qubit;
        if mask & qubit_mask == 0 {
            mask |= qubit_mask;
            true
        } else {
            false
        }
    })) 
}

impl RawECCs {
    fn add_entry(&mut self, hash_value: u64, front_perm: Permut32, back_perm: Permut32, circ: &AliasList<Instr>, instr_vec: &[Instr]) -> Option<AliasList<Instr>> {
        let instr = instr_vec.last().unwrap().clone();
        let front_gates_iter = circuit_get_surface_gates(instr_vec.iter())
            .map(|instr| instr.apply_permutation(front_perm.inv()));
        let back_gates_iter = circuit_get_surface_gates(instr_vec.iter().rev())
            .map(|instr| instr.apply_permutation(back_perm));

        match self.inner.entry(hash_value) {
            Entry::Vacant(v) => {
                // println!("\t{instr}: {front_perm}, {back_perm} new value");
                let new_point = circ.cons(instr);
                v.insert(CircuitECC {
                    size: instr_vec.len(),
                    front_gates: front_gates_iter.collect(),
                    back_gates: back_gates_iter.collect(),
                    circuits: vec![CircTriple{front_perm, circ: new_point.clone(), back_perm}],
                });
                Some(new_point)
            }
            Entry::Occupied(mut o) => {
                
                    // println!("\t{instr}: {front_perm}, {back_perm} equal -> {}", entry.circuits[0].circ);
                
                // } else {
                    // println!("\t{instr}: {front_perm}, {back_perm} skip -> {}", o.get().circuits[0].circ);
                // }
                let entry = o.get_mut();
                let front_unique = front_gates_iter.clone().all(|instr| !entry.front_gates.contains(&instr));
                let back_unique = back_gates_iter.clone().all(|instr| !entry.back_gates.contains(&instr));
                if front_unique && back_unique {
                    for instr in front_gates_iter { entry.front_gates.insert(instr); }
                    for instr in back_gates_iter { entry.back_gates.insert(instr); }
                    let new_point = circ.cons(instr);
                    let triple = CircTriple { front_perm, circ: new_point.clone(), back_perm};
                    entry.circuits.push(triple);
                }
                None
            }
        }
    }
    pub fn find_equivalents(&self, evaluator: &Evaluator, instrs: &[Instr]) -> Option<ECC> {
        let (backstate, front_perm, back_perm) = evaluator.evaluate(instrs);

        self.inner.get(&backstate.hash_value()).map(|ecc| {
            ECC(ecc.circuits
                .iter()
                .map(|triple| CircTriple{
                    front_perm: triple.front_perm * front_perm.inv(),
                    back_perm: back_perm.inv() * triple.back_perm,
                    circ: triple.circ.clone(),
                }.simplify())
                .filter(|(c, p)| c != instrs || *p != Permut32::identity(evaluator.nqubits() as u8))
                .collect_vec())
        }).and_then(|ecc| if ecc.0.is_empty() { None } else { Some(ecc) })
    }
}

#[gen_stub_pymethods]
#[pyo3::pymethods]
impl RawECCs {
    #[new]
    pub fn new(evaluator: &Evaluator) -> Self {
        let mut map = RawECCs {
            inner: Default::default(),
            nqubits: evaluator.nqubits(),
        };
        map.inner.insert(evaluator.backtrack_state.hash_value(), CircuitECC::root(evaluator.nqubits()));
        map
    }
    #[staticmethod]
    fn search_naive(evaluator: &Evaluator, instrs: Vec<Instr>, max_size: usize) -> (RawECCs, [usize; 3]) {
        let mut map = RawECCs::new(evaluator);
        let nqubits = evaluator.nqubits() as u8;
        
        let mut queue: VecDeque<AliasList<Instr>> = VecDeque::new();
        queue.push_back(AliasList::nil());

        let mut instr_vec = Vec::new();
        let mut counters = [0; 3];
        while let Some(circ) = queue.pop_front() {
            instr_vec.clear();
            instr_vec.extend(circ.iter().cloned());
            instr_vec.reverse();
            // let mask = instr_vec.iter().fold(0u8, |a, i| a | i.arg_mask());
            
            if counters[0] % 400 == 0 {
                println!("#{} Exploring {} ({} queued, {} ECCs, {} circs, {} circs perm)", counters[0], instr_vec.iter().join_option(" ", "", ""), queue.len(), map.len(), counters[1], counters[2]);
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
        (map, counters)
    }
    #[staticmethod]
    fn search(evaluator: &Evaluator, instrs: Vec<Instr>, max_size: usize) -> (RawECCs, [usize; 3]) {
        let mut map = RawECCs::new(evaluator);
        
        let mut queue: VecDeque<AliasList<Instr>> = VecDeque::new();
        queue.push_back(AliasList::nil());

        let mut instr_vec = Vec::new();
        let mut counters = [0; 3];
        while let Some(circ) = queue.pop_front() {
            instr_vec.clear();
            instr_vec.extend(circ.iter().cloned());
            instr_vec.reverse();
            let mask = instr_vec.iter().fold(0u8, |a, i| a | i.arg_mask());
            
            if counters[0] % 400 == 0 {
                println!("#{} Exploring {} ({} queued, {} ECCs, {} circs, {} circs perm)", counters[0], instr_vec.iter().join_option(" ", "", ""), queue.len(), map.len(), counters[1], counters[2]);
            }
            
            for instr in instrs.iter() {
                if instr.pass_mask(mask).is_none() { continue; }
                
                instr_vec.push(instr.clone());
                let vec = evaluator.evaluate_multiple(&instr_vec[..]);
                for (backstate, front_perm, back_perm) in vec {
                    if let Some(new_point) = map.add_entry(backstate.hash_value(), front_perm, back_perm, &circ, &instr_vec) {
                        if instr_vec.len() < max_size { queue.push_back(new_point.clone()); }
                    }
                    counters[2] += 1;
                }
                instr_vec.pop();
                counters[1] += 1;
            }
            counters[0] += 1;
        }
        (map, counters)
    }

    pub fn simplify(&self) -> super::ECCs {
        super::ECCs {
            eccs: self.inner.values().map(|a| a.simplify()).collect(),
            nqubits: self.nqubits,
        }
    }
    #[staticmethod]
    pub fn generate(
        evaluator: &Evaluator,
        mut gates: Vec<Gate>,
        max_size: usize,
    ) -> (RawECCs, [usize; 3]) {
        let adjoint_gates = gates.iter().map(|g| g.adjoint()).collect_vec();

        for g in adjoint_gates {
            if !gates.contains(&g) {
                gates.push(g);
            }
        }
        
        gates.sort_by_key(|g| (g.nqargs(), g.name()));
        

        let mut instructions: Vec<Instr> = Vec::new();

        for instr in gates {
            match instr.nqargs() {
                1 => instructions.extend((0..evaluator.nqubits()).map(|i| Instr(instr.clone(), smallvec::smallvec![i as u8]))),
                2 => instructions.extend((0..evaluator.nqubits()).flat_map(|i| {
                    (0..evaluator.nqubits())
                        .filter(move |j| *j != i)
                        .map(move |j| Instr(instr.clone(), smallvec::smallvec![i as u8, j as u8]))
                })),
                _ => panic!("Only 1 and 2 qubit instructions are supported"),
            }
        }

        instructions.sort_by_key(|a| a.largest_qubit());
        RawECCs::search(&evaluator, instructions, max_size)
    }
    #[staticmethod]
    pub fn generate_naive(
        evaluator: &Evaluator,
        mut gates: Vec<Gate>,
        max_size: usize,
    ) -> (RawECCs, [usize; 3]) {
        let adjoint_gates = gates.iter().map(|g| g.adjoint()).collect_vec();

        for g in adjoint_gates {
            if !gates.contains(&g) {
                gates.push(g);
            }
        }
        
        gates.sort_by_key(|g| (g.nqargs(), g.name()));

        let mut instructions: Vec<Instr> = Vec::new();

        for instr in gates {
            match instr.nqargs() {
                1 => instructions.extend((0..evaluator.nqubits()).map(|i| Instr(instr.clone(), smallvec::smallvec![i as u8]))),
                2 => instructions.extend((0..evaluator.nqubits()).flat_map(|i| {
                    (0..evaluator.nqubits())
                        .filter(move |j| *j != i)
                        .map(move |j| Instr(instr.clone(), smallvec::smallvec![i as u8, j as u8]))
                })),
                _ => panic!("Only 1 and 2 qubit instructions are supported"),
            }
        }

        instructions.sort_by_key(|a| a.largest_qubit());
        RawECCs::search_naive(&evaluator, instructions, max_size)
    }

    #[pyo3(name="find_equivalents")]
    fn find_equivalents_py(&self, evaluator: &Evaluator, instrs: Vec<Instr>) -> Option<ECC> {
        self.find_equivalents(evaluator, &instrs)
    }

    
    // for i in 0..instrs.len() {
    //     for j in (i+1)..=instrs.len() {
    //         let (backstate, front_perm, back_perm) = evaluator.evaluate(&instrs[i..j]);
    //         println!("{i}..{j} {front_perm} {} {back_perm}", instrs[i..j].iter().join_option(" ", "", ""));
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
