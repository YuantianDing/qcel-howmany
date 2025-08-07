use std::collections::{hash_map::Entry, BTreeSet, HashMap, VecDeque};

use derive_more::{Deref, Debug, Display};
use itertools::Itertools;
use nohash_hasher::BuildNoHashHasher;
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pymethods};
use rand::rngs::ThreadRng;
use smallvec::SmallVec;


use crate::{
    circ::{gates::SWAP, Gate, Instruction, InstructionSliceExt}, groups::permutation::Permut32, state::StateVec, utils::{AliasList, JoinOptionIter}
};
use linear_map::set::LinearSet;

mod quartz;

#[derive(Debug, Display, Clone, PartialEq, Eq, Hash)]
#[debug("{}({})", self.0, self.1.iter().join_option(", ", "", ""))]
#[display("{}({})", self.0, self.1.iter().join_option(", ", "", ""))]
pub struct Instr(Gate, SmallVec<[u8; 2]>);

impl Instr {
    pub fn apply_permutation(&self, perm: Permut32) -> Self {
        Instr(self.0, self.1.iter().map(|&qubit| perm.at(qubit)).collect())
    }
}

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
#[debug("{front_gates:?} {} {back_gates:?}", self.circuits.iter().join_option(", ", "[", "]"))]
pub struct ECC {
    pub front_gates: LinearSet<Instr>,
    pub back_gates: LinearSet<Instr>,
    pub circuits: Vec<CircTriple>,
}

impl ECC {
    pub fn simplify(&self) -> Vec<(Vec<Instr>, Permut32)> {
        let result = self.circuits.iter().map(|triple| triple.simplify()).collect_vec();
        let set: BTreeSet<_> = result[0].0.iter().flat_map(|a| a.1.iter().cloned()).collect();
        let uniform_inv = Permut32::from_iter_with_ext(result[0].1.len(), set.into_iter());
        let uniform = uniform_inv.inv();
        result.into_iter().map(|(instrs, perm)| (
                instrs.into_iter().map(|a| a.apply_permutation(uniform)).collect(),
                uniform * perm * uniform_inv
        )).collect()
    }
    pub fn simply_circuits(&self) -> Vec<Vec<Instr>> {
        let simplified = self.simplify();
        let unit = simplified[0].1.inv();
        simplified.into_iter().map(|(instrs, p)| {
            instrs.iter().cloned().chain(
                (unit * p).generate_swaps().map(|(a,b)| Instr(*SWAP, smallvec::smallvec![a, b]))
            ).collect_vec()
        }).collect()
    }
}

impl std::fmt::Display for ECC {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ECC {{\n")?;
        for circ in self.simply_circuits() {
            write!(f, "\t{},\n", circ.iter().join_option(" ", "", ""))?;
        }
        write!(f, "}}")
    }
}



#[gen_stub_pyclass]
#[pyo3::pyclass]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CircuitECCs {
    inner: HashMap<u64, ECC, BuildNoHashHasher<u64>>,
    initial_state: StateVec,
    backtrack_state: StateVec,
}

impl std::fmt::Display for CircuitECCs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, ecc) in self.inner.values().enumerate() {
            if ecc.circuits.len() > 1 {
                write!(f, "{}{ecc}", if i > 0 { " + " } else {""})?;
            }
        }
        Ok(())
    }
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

                            v.insert(ECC {
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

    pub fn check1(&self) -> impl Iterator<Item= Vec<Vec<Instr>>> + '_ {
        let state = StateVec::from_random(&mut rand::rng(), self.initial_state.nqubits() as u32);
        self.inner.values().filter(move |ecc|
            ecc.circuits.len() > 1 && !ecc.simply_circuits().iter().map(|circ| {
                let mut s = state.clone();
                for Instr(g, idx) in circ {
                    s.apply(&idx, *g);
                }
                s.normalize_arg();
                s.hash_value()
            }).all_equal()
        ).map(|a| a.simply_circuits())
    }
    pub fn check(&self) -> impl Iterator<Item= Vec<Vec<Instr>>> + '_ {
        let state = StateVec::from_random(&mut rand::rng(), self.initial_state.nqubits() as u32);
        self.inner.values().filter_map(|ecc| {
            if ecc.circuits.len() <= 1 { return None; }
            Some(ecc.simply_circuits())
        }).filter(move |circ| !circ.iter().map(|circ| {
                let mut s = state.clone();
                for Instr(g, idx) in circ {
                    s.apply(idx, *g);
                }
                s.normalize_arg();
                s.hash_value()
            }).all_equal()
        )
    }
}
#[gen_stub_pymethods]
#[pyo3::pymethods]
impl CircuitECCs {
    #[staticmethod]
    pub fn generate(
        nqubits: usize,
        gates: Vec<Gate>,
        max_size: usize,
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
        
        CircuitECCs::search(nqubits, instructions, max_size, &mut rand::rng())
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
}
