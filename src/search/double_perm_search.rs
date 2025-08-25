use std::{cmp::Ordering, collections::{hash_map::Entry, BTreeSet, HashMap, VecDeque}};

use derive_more::{Deref, Debug, Display};
use itertools::Itertools;
use nohash_hasher::BuildNoHashHasher;
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pymethods};
use rand::rngs::ThreadRng;
use smallvec::{smallvec, SmallVec};


use crate::{
    circ::{gates::SWAP, Gate, Instruction, InstructionSliceExt}, groups::permutation::Permut32, circ::Instr, state::StateVec, utils::{AliasList, JoinOptionIter}
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
    pub fn nqubits(&self) -> usize {
        self.initial_state.nqubits()
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
#[derive(Clone, Deref)]
pub struct CircuitECCs {
    #[deref]
    inner: HashMap<u64, CircuitECC, BuildNoHashHasher<u64>>,
    pub nqubits: usize,
}


impl std::ops::Drop for CircuitECCs {
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

impl CircuitECCs {
    pub fn new(evaluator: &Evaluator) -> Self {
        let mut map = CircuitECCs {
            inner: Default::default(),
            nqubits: evaluator.nqubits(),
        };
        map.inner.insert(evaluator.backtrack_state.hash_value(), CircuitECC::root(evaluator.nqubits()));
        map
    }
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
                if entry.size == instr_vec.len() {
                    let new_point = circ.cons(instr);
                    let triple = CircTriple { front_perm, circ: new_point.clone(), back_perm};
                    entry.circuits.push(triple);
                    Some(new_point)
                } else {
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
    }
    fn search(evaluator: &Evaluator, instrs: Vec<Instr>, max_size: usize) -> CircuitECCs {
        let mut map = CircuitECCs::new(evaluator);
        
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
        map
    }

    pub fn simplify(self) -> super::ECCs {
        super::ECCs {
            eccs: self.inner.values().map(|a| a.simplify()).collect(),
            nqubits: self.nqubits,
        }
    }
    pub fn generate(
        evaluator: &Evaluator,
        gates: Vec<Gate>,
        max_size: usize,
    ) -> CircuitECCs {
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
        CircuitECCs::search(&evaluator, instructions, max_size)
    }
    
}
