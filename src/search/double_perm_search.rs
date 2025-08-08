use std::collections::{hash_map::Entry, BTreeSet, HashMap, VecDeque};

use derive_more::{Deref, Debug, Display};
use itertools::Itertools;
use nohash_hasher::BuildNoHashHasher;
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pymethods};
use rand::rngs::ThreadRng;
use smallvec::SmallVec;


use crate::{
    circ::{gates::SWAP, Gate, Instruction, InstructionSliceExt}, groups::permutation::Permut32, search::Instr, state::StateVec, utils::{AliasList, JoinOptionIter}
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

pub struct CircuitECC {
    pub front_gates: LinearSet<Instr>,
    pub back_gates: LinearSet<Instr>,
    pub circuits: Vec<CircTriple>,
}

impl CircuitECC {
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
#[pyo3::pyclass]
pub struct CircuitECCs {
    inner: HashMap<u64, CircuitECC, BuildNoHashHasher<u64>>,
    initial_state: StateVec,
    backtrack_state: StateVec,
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

fn circuit_get_surface_gates<'a>(circ: impl Iterator<Item = &'a Instr>) -> impl Iterator<Item = &'a Instr> {
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
    pub fn search(nqubits: usize, instrs: Vec<Instr>, max_size: usize, rng: &mut ThreadRng) -> CircuitECCs {
        let mut backtrack_state = StateVec::from_random(rng, nqubits as u32);

        backtrack_state.apply_permutation(backtrack_state.get_permutation().inv());
        let mut map = CircuitECCs {
            inner: Default::default(),
            initial_state: StateVec::from_random_symmetric(rng, nqubits as u32),
            backtrack_state,
        };
        let mut queue: VecDeque<AliasList<Instr>> = VecDeque::new();
        let mut instr_vec = Vec::new();
        queue.push_back(AliasList::nil());
        let mut counter = 0;
        while let Some(circ) = queue.pop_front() {
            instr_vec.clear();
            instr_vec.extend(circ.iter().cloned());
            instr_vec.reverse();

            if counter % 200 == 0 {
                println!("#{counter} Exploring {} ({} queued)", instr_vec.iter().join_option(" ", "", ""), queue.len());
            }
            
            for instr in instrs.iter() {
                instr_vec.push(instr.clone());
                let mut state = map.initial_state.clone();
                
                for Instr(gate, idx) in instr_vec.iter() {
                    state.apply(idx, *gate);
                }
                state.normalize_arg();
                let back_perm_inv = state.get_permutation();
                let back_perm = back_perm_inv.inv();
                state.apply_permutation(back_perm);
                let mut backstate = map.backtrack_state.clone();
                backstate.apply_permutation(back_perm_inv);
                for Instr(gate, idx) in instr_vec.iter().rev() {
                    backstate.apply(idx, gate.adjoint());
                }
                backstate.normalize_arg();
                let front_perm = backstate.get_permutation();
                let front_perm_inv = front_perm.inv();
                backstate.apply_permutation(front_perm_inv);
                {
                    let mut front_gates_iter = circuit_get_surface_gates(instr_vec.iter())
                        .map(|instr| instr.apply_permutation(front_perm_inv));
                    let mut back_gates_iter = circuit_get_surface_gates(instr_vec.iter().rev())
                        .map(|instr| instr.apply_permutation(back_perm));

                    match map.inner.entry(backstate.hash_value()) {
                        Entry::Vacant(v) => {
                            let new_point = circ.cons(instr.clone());
                            if instr_vec.len() < max_size { queue.push_back(new_point.clone()); }

                            v.insert(CircuitECC {
                                front_gates: front_gates_iter.collect(),
                                back_gates: back_gates_iter.collect(),
                                circuits: vec![CircTriple{front_perm, circ: new_point, back_perm}],
                            });
                        }
                        Entry::Occupied(mut o) => {
                            if front_gates_iter.all(|instr| o.get_mut().front_gates.insert(instr)) &&
                                back_gates_iter.all(|instr| o.get_mut().back_gates.insert(instr)) {
                                
                                let new_point = circ.cons(instr.clone());
                                let triple = CircTriple { front_perm, circ: new_point, back_perm};
                                o.get_mut().circuits.push(triple);
                            }
                        }
                    }
                }
                instr_vec.pop();
            }
            counter += 1;
        }
        map
    }

    pub fn simplify(self) -> super::ECCs {
        super::ECCs {
            eccs: self.inner.values().map(|a| a.simplify()).collect(),
            nqubits: self.initial_state.nqubits(),
        }
    }
    pub fn generate(
        nqubits: usize,
        gates: Vec<Gate>,
        max_size: usize,
        rng: &mut ThreadRng,
    ) -> CircuitECCs {
        let mut instructions: Vec<Instr> = Vec::new();

        for instr in gates {
            match instr.nqargs() {
                1 => instructions.extend((0..nqubits).map(|i| Instr(instr.clone(), smallvec::smallvec![i as u8]))),
                2 => instructions.extend((0..nqubits).flat_map(|i| {
                    (0..nqubits)
                        .filter(move |j| *j != i)
                        .map(move |j| Instr(instr.clone(), smallvec::smallvec![i as u8, j as u8]))
                })),
                _ => panic!("Only 1 and 2 qubit instructions are supported"),
            }
        }
        
        CircuitECCs::search(nqubits, instructions, max_size, rng)
    }
}
