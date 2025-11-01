use derive_more::Deref;
use pyo3::{Bound, PyAny};
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pymethods};
use rand;
use rayon::iter::{IntoParallelRefIterator, ParallelBridge, ParallelIterator};
use serde_json::Value;
use std::fmt;

use crate::circ::Gate;
// use crate::identity::circuit::CircGraph;
use crate::{circ::Instr, identity::circuit::Circ, state::StateVec};
use colored::Colorize;
use itertools::Itertools;
use std::collections::{HashSet, VecDeque};
// use petgraph::graph::{DiGraph, NodeIndex};

#[gen_stub_pyclass]
#[pyo3::pyclass(eq, ord, frozen, sequence, dict)]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Deref, serde::Serialize, serde::Deserialize)]
pub struct IdentityCirc(Circ);

impl IdentityCirc {
    pub fn into_inner(self) -> Circ {
        self.0
    }
    // pub fn to_identity_petgraph(&self) -> CircGraph {
    //     let mut nodes = vec![Vec::<NodeIndex>::new(); self.len()];
    //     let mut graph = CircGraph::new();
    //     for (instr_id, Instr(g, _)) in self.instrs.iter().enumerate() {
    //         let n = graph.add_node((*g, 0));
    //         assert!(n.index() == instr_id);
    //         nodes[instr_id].push(n);
    //     }
    //     for (instr_id, Instr(g, qargs)) in self.instrs.iter().enumerate() {
    //         for (qarg_d, _)  in qargs.iter().enumerate().skip(1) {
    //             let n = graph.add_node((*g, qarg_d as u8));
    //             graph.add_edge(*nodes[instr_id].last().unwrap(), n, true);
    //             nodes[instr_id].push(n);
    //         }
    //     }

    //     for instr_id in 0..self.len() {
    //         for (qarg_id, fw_instr_id) in self.qargs_forward(instr_id).into_iter().enumerate() {
    //             let mut qarg = self.0[instr_id].1[qarg_id];
    //             if fw_instr_id <= instr_id {
    //                 qarg = self.perm.at(qarg);
    //             }
    //             graph.add_edge(
    //                 nodes[instr_id][qarg_id],
    //                 nodes[fw_instr_id][self[fw_instr_id].position_of_qubit(qarg).unwrap()],
    //                 false,
    //             );
    //         }
    //     }

    //     graph
    // }
}

#[gen_stub_pymethods]
#[pyo3::pymethods]
impl IdentityCirc {
    #[new]
    pub fn unchecked(circuit: Circ) -> Self {
        let result = Self(circuit);
        assert!(result.check(), "IdentityCirc::new: circuit is not identity {}", result);
        result
    }
    #[staticmethod]
    pub fn from_python(obj: &Bound<'_, PyAny>) -> pythonize::Result<Self> {
        pythonize::depythonize(obj)
    }
    pub fn nqubits(&self) -> u8 {
        self.0.nqubits()
    }

    pub fn qargs_forward(&self, gate_id: usize) -> Vec<usize> {
        let qargs = &self.0[gate_id].1;
        let mut qargs_coverage: Vec<usize> = vec![usize::MAX; qargs.len()];
        let num_instrs = self.0.len();

        let rotated_circ = self.0.rotate((gate_id + 1) as isize);
        assert!(rotated_circ.len() == self.0.len());
        assert!(rotated_circ.instrs[self.0.len() - 1].0 == self.0[gate_id].0);
        assert!(rotated_circ.instrs[0].0 == self.0[(gate_id + 1) % self.0.len()].0);

        for (i, instr) in rotated_circ.instrs.iter().enumerate() {
            if qargs_coverage.iter().all(|&x| x != usize::MAX) {
                break;
            }
            for &q_in_instr in &instr.1 {
                if let Some(qarg_idx) = qargs.iter().position(|&q| q == q_in_instr) {
                    if qargs_coverage[qarg_idx] == usize::MAX {
                        qargs_coverage[qarg_idx] = (gate_id + 1 + i) % num_instrs;
                    }
                }
            }
        }
        assert!(!qargs_coverage.iter().any(|&x| x == usize::MAX), "{self} {qargs_coverage:?}");
        qargs_coverage
    }

    pub fn qargs_backward(&self, gate_id: usize) -> Vec<usize> {
        let qargs = &self.0[gate_id].1;
        let mut qargs_coverage: Vec<usize> = vec![usize::MAX; qargs.len()];
        let num_instrs = self.0.len();

        let rotated_circ = self.0.rotate(gate_id as isize - num_instrs as isize);
        assert!(rotated_circ.len() == self.0.len());
        assert!(rotated_circ.instrs[0].0 == self.0[gate_id].0);
        assert!(rotated_circ.instrs[self.0.len() - 1].0 == self.0[(gate_id + self.0.len() - 1) % self.0.len()].0);
        
        for (i, instr) in rotated_circ.instrs.iter().rev().enumerate() {
            if qargs_coverage.iter().all(|&x| x != usize::MAX) {
                break;
            }
            for &q_in_instr in &instr.1 {
                if let Some(qarg_idx) = qargs.iter().position(|&q| q == q_in_instr) {
                    if qargs_coverage[qarg_idx] == usize::MAX {
                        qargs_coverage[qarg_idx] = (gate_id + num_instrs - 1 - i) % num_instrs;
                    }
                }
            }
        }
        assert!(!qargs_coverage.iter().any(|&x| x == usize::MAX), "a {self} {rotated_circ:?} {gate_id}");
        qargs_coverage
    }
    pub fn check(&self) -> bool {
        let mut state1 = StateVec::from_random(&mut rand::rng(), self.nqubits() as u32);
        let mut state2 = state1.clone();

        for Instr(g, qargs) in self.0.instrs.iter() {
            state1.apply(&qargs, *g);
        }
        state1.apply_permutation(self.perm);
        state1.normalize_arg();
        
        // assert!(state1.approx_eq(&state2, 1e-8), "IdentityCirc::check: circuit is not identity {:?} {:?}", state1, state2);
        state1.approx_eq(&state2, 1e-8)
    }

    pub fn __str__(&self) -> String {
        format!("{}", self)
    }
    pub fn __repr__(&self) -> String {
        format!("{:?}", self)
    }
    pub fn __hash__(&self) -> u64 {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }
    #[getter]
    pub fn inner(&self) -> Circ {
        self.0.clone()
    }
    pub fn __len__(&self) -> usize {
        self.len()
    }
    pub fn __getitem__(&self, idx: isize) -> Instr {
        self.0[idx.rem_euclid(self.len() as isize) as usize].clone()
    }
    pub fn pythonize<'py>(&self, py: pyo3::Python<'py>) -> pythonize::Result<Bound<'py, pyo3::PyAny>> {
        pythonize::pythonize(py, self)
    }

}

impl fmt::Display for IdentityCirc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Identity {{ ")?;
        for instr in &self.0.instrs {
            write!(f, "{} ", instr)?;
        }
        write!(f, "{} }}", self.0.perm)
    }
}


#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Deserialize, serde::Serialize)]
pub struct Port {
    pub qubit: u8,
    pub instr_id: usize,
    pub qargs_idx: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IdentitySubcircuit<'a> {
    pub circuit: &'a IdentityCirc,
    pub gates: Vec<bool>,
}

impl<'a> fmt::Display for IdentitySubcircuit<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (instr, g) in self.circuit.instrs.iter().zip(&self.gates) {
            if *g {
                write!(f, "{} ", instr.to_string().white().bold())?;
            } else {
                write!(f, "{} ", instr.to_string().truecolor(128, 128, 128))?;
            }
        }
        write!(f, "{}", self.circuit.perm)
    }
}

impl<'a> IdentitySubcircuit<'a> {
    pub fn inputs(&self) -> impl Iterator<Item = Port> + '_ {
        (0..self.gates.len())
            .filter(move |&instr_id| self.gates[instr_id])
            .flat_map(move |instr_id| {
                self.circuit.qargs_backward(instr_id)
                    .into_iter()
                    .enumerate()
                    .filter_map(move |(qarg_id, i2)| {
                        if i2 == instr_id || !self.gates[i2] {
                            Some(Port {
                                qubit: self.circuit[instr_id].1[qarg_id],
                                instr_id,
                                qargs_idx: qarg_id,
                            })
                        } else {
                            None
                        }
                    })
            })
    }

    pub fn outputs(&self) -> impl Iterator<Item = Port> + '_ {
        (0..self.gates.len())
            .filter(move |&instr_id| self.gates[instr_id])
            .flat_map(move |instr_id| {
                self.circuit.qargs_forward(instr_id)
                    .into_iter()
                    .enumerate()
                    .filter_map(move |(qarg_id, i2)| {
                    if i2 == instr_id || !self.gates[i2] {
                        Some(Port {
                            qubit: self.circuit[instr_id].1[qarg_id],
                            instr_id,
                            qargs_idx: qarg_id,
                        })
                    } else {
                        None
                    }
                })
            })
    }

    pub fn is_convex(&self) -> bool {
        self.inputs().map(|p| p.qubit).all_unique() && self.outputs().map(|p| p.qubit).all_unique()
    }

    pub fn is_connected(&self) -> bool {
        let first_gate = match self.gates.iter().position(|&g| g) {
            Some(i) => i,
            None => return true,
        };

        let mut visited = HashSet::new();
        visited.insert(first_gate);
        let mut boundary_instrs = VecDeque::new();
        boundary_instrs.push_back(first_gate);

        while let Some(instr_id) = boundary_instrs.pop_front() {
            for i2 in self.circuit.qargs_forward(instr_id) {
                assert!(i2 != usize::MAX);
                if self.gates[i2] && visited.insert(i2) {
                    boundary_instrs.push_back(i2);
                }
            }
            for i2 in self.circuit.qargs_backward(instr_id) {
                assert!(i2 != usize::MAX);
                if self.gates[i2] && visited.insert(i2) {
                    boundary_instrs.push_back(i2);
                }
            }
        }

        self.gates
            .iter()
            .enumerate()
            .all(|(i, &g)| g == visited.contains(&i))
    }

    pub fn split(&self) -> Option<(Circ, Circ)> {
        use crate::circ::Instr;
        use crate::groups::permutation::Permut32;

        let a = self.gates.iter().position(|&g| g)?;
        let mut circuit = self.circuit.0.rotate(a as isize);
        let mut gates = self.gates.clone();
        gates.rotate_left(a);

        let mut new_circuit_instrs: VecDeque<Instr> = VecDeque::new();

        while gates.iter().any(|&g| g) {
            let a = gates.iter().position(|&g| g).unwrap();
            if a == 0 {
                new_circuit_instrs.push_back(circuit.instrs[0].clone());
                circuit.instrs.remove(0);
                gates.remove(0);
            } else if circuit.instrs[a - 1].disjoint(&circuit.instrs[a]) {
                circuit.instrs.swap(a - 1, a);
                gates.swap(a - 1, a);
            } else if circuit.instrs[..a]
                .iter()
                .all(|instr| new_circuit_instrs.iter().all(|new_instr| instr.disjoint(new_instr)))
            {
                circuit = circuit.rotate(a as isize);
                gates.rotate_left(a);
            } else {
                break;
            }
        }

        while gates.iter().any(|&g| g) {
            let j = gates.iter().rposition(|&g| g).unwrap();
            if j == gates.len() - 1 {
                new_circuit_instrs
                    .push_front(circuit.instrs.last().unwrap().permut(circuit.perm));
                circuit.instrs.pop();
                gates.pop();
            } else if circuit.instrs[j].disjoint(&circuit.instrs[j + 1]) {
                circuit.instrs.swap(j, j + 1);
                gates.swap(j, j + 1);
            } else if circuit.instrs[j + 1..].iter().all(|instr| {
                new_circuit_instrs
                    .iter()
                    .all(|new_instr| instr.permut(circuit.perm).disjoint(new_instr))
            }) {
                let rot = -(gates.len() as isize - 1 - j as isize);
                circuit = circuit.rotate(rot);
                gates.rotate_right((-rot) as usize);
            } else {
                return None;
            }
        }

        let sub_circ = Circ::new(
            new_circuit_instrs.into(),
            Permut32::identity(self.circuit.nqubits()),
        );
        Some((sub_circ, circuit))
    }

    pub fn subcircuits(
        circuit: &'a IdentityCirc,
        n: usize,
    ) -> impl Iterator<Item = IdentitySubcircuit<'a>> {
        (0..circuit.len())
            .combinations(n)
            .map(move |gate_indices| {
                let mut gates = vec![false; circuit.len()];
                for i in gate_indices {
                    gates[i] = true;
                }
                IdentitySubcircuit { circuit, gates }
            })
            .filter(|subcirc| {
                // println!("Checking subcircuit: {:?} {:?}", subcirc.inputs().map(move |a| (a.instr_id, a.qargs_idx)).collect_vec(), subcirc.outputs().map(move |a| (a.instr_id, a.qargs_idx)).collect_vec());
                // println!("Checking subcircuit: {} {} {} {}", subcirc, subcirc.is_connected(), subcirc.is_convex(), subcirc.split().is_some());
                subcirc.is_convex() && subcirc.is_connected() && subcirc.split().is_some()
            })
    }

    pub fn subcircuit_splits_n(
        circuit: &'a IdentityCirc,
        n: usize,
    ) -> impl Iterator<Item = (Circ, Circ)> + 'a {
        (0..circuit.len())
            .combinations(n)
            .filter_map(move |gate_indices| {
                let mut gates = vec![false; circuit.len()];
                for i in gate_indices {
                    gates[i] = true;
                }
                let subcirc = IdentitySubcircuit { circuit, gates };
                if subcirc.is_convex() && subcirc.is_connected() {
                    subcirc.split()
                } else {
                    None
                }
            })
    }
    pub fn subcircuit_splits(
        circuit: &'a IdentityCirc
    ) -> impl Iterator<Item = (Circ, Circ)> + 'a {
        COMBINATIONS[circuit.len()]
            .iter()
            .filter_map(move |gate_indices| {
                let mut gates = vec![false; circuit.len()];
                for i in gate_indices {
                    gates[*i as usize] = true;
                }
                let subcirc = IdentitySubcircuit { circuit, gates };
                if subcirc.is_convex() && subcirc.is_connected() {
                    subcirc.split()
                } else {
                    None
                }
            })
    }
    pub fn par_subcircuit_splits(
        circuit: &'a IdentityCirc
    ) -> impl ParallelIterator<Item = (Circ, Circ)> + 'a {
        COMBINATIONS[circuit.len()]
            .par_iter()
            .filter_map(move |gate_indices| {
                let mut gates = vec![false; circuit.len()];
                for i in gate_indices {
                    gates[*i as usize] = true;
                }
                let subcirc = IdentitySubcircuit { circuit, gates };
                if subcirc.is_connected() && subcirc.is_convex() {
                    subcirc.split()
                } else {
                    None
                }
            })
    }
    pub fn from_index_iter(
        circuit: &'a IdentityCirc,
        gate_indices: impl Iterator<Item = usize>,
    ) -> Self {
        let mut gates = vec![false; circuit.len()];
        for i in gate_indices {
            gates[i] = true;
        }
        IdentitySubcircuit { circuit, gates }
    }
}

lazy_static::lazy_static!(
    static ref COMBINATIONS : Vec<Vec<Vec<u8>>> = {
        let mut combinations: Vec<Vec<Vec<u8>>> = Vec::new();
        for n in 0u8..=18 {
            combinations.push((0u8..=(n / 2 + 1)).flat_map(|k| (0u8..n).combinations(k as usize)).collect_vec());
        }
        combinations
    };
);

#[cfg(test)]
mod tests {
    use crate::{circ::{gates, Instr}, groups::permutation::Permut32};

    use super::*;
    use smallvec::smallvec;
    #[test]
    fn test_subcircuit_split() {
        let instrs = vec![
            gates::T.instr([0u8]),
            gates::CX.instr([1u8, 0]),
            gates::CX.instr([0u8, 2]),
            gates::CX.instr([0u8, 1]),
            gates::TDG.instr([1u8]),
            gates::CX.instr([0u8, 1]),
            gates::CX.instr([2u8, 0]),
            gates::CX.instr([0u8, 2]),
            gates::CX.instr([1u8, 2]),
        ];
        let perm = Permut32::from_iter([2, 1, 0].into_iter());
        let circuit = Circ::new(instrs, perm);
        let id_circ = IdentityCirc::unchecked(circuit);
        
        println!("{} {:?} {:?}", id_circ, id_circ.qargs_forward(0), id_circ.qargs_backward(0));
        println!();

        for subcirc in IdentitySubcircuit::subcircuits(&id_circ, 2) {
            println!("{}", subcirc);
            if let Some((c1, c2)) = subcirc.split() {
                println!("{} {}", c1, c2);
                let (c1_rep, perm) = c1.representative_with_perm();
                let c2_permuted = c2.permut(perm);
                println!("{} {}", c1_rep, c2_permuted);
            }
            println!();
        }
    }
}