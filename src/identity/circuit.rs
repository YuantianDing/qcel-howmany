use derive_more::Index;
// use petgraph::graph::{DiGraph, NodeIndex};
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pymethods};

use crate::identity::idcircuit::IdentityCirc;
use crate::utils::DenseIndexMap;
use crate::{circ::Instr32, groups::permutation::Permut32};
use std::cmp::Ordering;
use std::fmt;
use std::ops::{Add, AddAssign};

// pub type CircGraph = petgraph::graph::DiGraph<(Gate, u8), bool>;

#[gen_stub_pyclass]
#[pyo3::pyclass(eq, ord, sequence)]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Index, serde::Serialize, serde::Deserialize)]
pub struct Circ {
    #[index]
    #[pyo3(get)]
    pub instrs: Vec<Instr32>,
    #[pyo3(get)]
    pub perm: Permut32,
}

impl PartialOrd for Circ {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Circ {
    fn cmp(&self, other: &Self) -> Ordering {
        self.instrs.len().cmp(&other.instrs.len()).then_with(||
            self.instrs.cmp(&other.instrs)
                .then_with(|| self.perm.cmp(&other.perm)))
    }
}

impl Circ {
    pub fn reindex_qubits(&mut self, map: &mut DenseIndexMap) {
        for instr in &mut self.instrs {
            for q in instr.1.iter_mut() {
                *q = map.get_or_insert(*q as usize) as u8;
            }
        }
        self.perm = self.perm.reindex(map);
    }
    pub fn affected_qubits(&self) -> impl Iterator<Item = u8> + '_ {
        let instrs_qubits = self.instrs.iter().flat_map(|i| i.1.iter().copied());
        let perm_qubits = (0u8..self.nqubits()).filter(move |&i| self.perm.at(i) != i);
        instrs_qubits.chain(perm_qubits)
    }
    fn _rotate_representative(&self) -> impl Iterator<Item = Circ> + '_ {
        (0..self.instrs.len())
            .filter(move |&i| self.instrs[i].1.contains(&0))
            .map(move |i| {
                self.rotate(i as isize).reorder_instrs().compact_qubits()
            })
    }
    pub fn compact_qubits(mut self) -> Self {
        if self.instrs.is_empty() {
            return self;
        }
        if let Some(max_q) = self.affected_qubits().max() {
            if max_q + 1 < self.nqubits() {
                let n = max_q + 1;
                let perm_vec = (0u8..n).map(|i| self.perm.at(i));
                self.perm = Permut32::from_iter(perm_vec);
                return self;
            }
        }
        self
    }

    pub fn reorder_instrs(mut self) -> Self {
        for i in 1..self.instrs.len() {
            for j in (1..=i).rev() {
                if self.instrs[j].disjoint(&self.instrs[j - 1])
                    && self.instrs[j] < self.instrs[j - 1]
                {
                    self.instrs.swap(j, j - 1);
                } else {
                    break;
                }
            }
        }
        self
    }

    pub fn rotate_representative(self) -> IdentityCirc {
        let circ = self.compact_qubits().remove_swaps();
        if circ.len() == 0 {
            return IdentityCirc::unchecked(circ);
        }
        IdentityCirc::unchecked(
            Permut32::all(circ.nqubits()).iter().cloned()
                .flat_map(|p| circ.permut(p)._rotate_representative().min())
                .min()
                .unwrap())
    }

    pub fn representative(self) -> Self {
        let circ = self.compact_qubits().remove_swaps();
        if circ.is_empty() {
            return circ;
        }

        Permut32::all(circ.nqubits())
            .iter()
            .cloned()
            .map(|p| circ.permut(p).reorder_instrs().compact_qubits())
            .min()
            .unwrap()
    }

    pub fn representative_with_perm(self) -> (Self, Permut32) {
        let circ = self.compact_qubits().remove_swaps();
        if circ.is_empty() {
            return (circ, Permut32::identity(0));
        }

        Permut32::all(circ.nqubits())
            .iter()
            .cloned()
            .map(|p| (circ.permut(p).reorder_instrs().compact_qubits(), p))
            .min()
            .unwrap()
    }
    pub fn new(instrs: Vec<Instr32>, perm: Permut32) -> Self {
        if let Some(max_q) = instrs.iter().flat_map(|i| i.1.iter().copied()).max() {
            assert!(
                perm.len() > max_q,
                "Permutation length must be greater than the max qubit index."
            );
        }
        Self { instrs, perm }
    }

    pub fn new_no_perm(instrs: Vec<Instr32>) -> Self {
        let n = instrs
            .iter()
            .flat_map(|i| i.1.iter().copied())
            .max()
            .map_or(0, |q| q + 1);
        let perm = Permut32::identity(n as u8);
        Self { instrs, perm }
    }
    // pub fn to_circuit_petgraph(&self) -> CircGraph {
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
    //         assert!(nodes[instr_id].len() == self[instr_id].1.len());
    //         for (qarg_id, qarg) in self[instr_id].1.iter().enumerate() {
    //             if let Some(fw_instr_id) = self[(instr_id+1)..].iter().position(|Instr(_, qargs)| qargs.contains(qarg)) {
    //                 let fw_instr_id = fw_instr_id + instr_id + 1;
    //                 graph.add_edge(
    //                     nodes[instr_id][qarg_id],
    //                     nodes[fw_instr_id][self[fw_instr_id].position_of_qubit(*qarg).unwrap()],
    //                     false,
    //                 );
    //             }
    //         }
    //     }
        
    //     graph
    // }
}

#[gen_stub_pymethods]
#[pyo3::pymethods]
impl Circ {
    #[new]
    #[pyo3(signature = (instrs, perm=None))]
    fn new_py(instrs: Vec<Instr32>, perm: Option<Permut32>) -> Self {
        let Some(p) = perm else {
            return Circ::new_no_perm(instrs);
        };
        Circ::new(instrs, p)
    }


    pub fn nqubits(&self) -> u8 {
        self.perm.len()
    }
    #[pyo3(name = "rotate_representative")]
    fn rotate_representative_py(&self) -> IdentityCirc {
        self.clone().rotate_representative()
    }
    #[pyo3(name = "representative")]
    fn representative_py(&self) -> Circ {
        self.clone().representative()
    }
    #[pyo3(name = "representative_with_perm")]
    fn representative_with_perm_py(&self) -> (Circ, Permut32) {
        self.clone().representative_with_perm()
    }

    pub fn permut(&self, perm: Permut32) -> Self {
        let (self_perm, other_perm) = self.perm.coordinate_permute(perm);
        let new_instrs = self
            .instrs
            .iter()
            .map(|instr| instr.permut(other_perm))
            .collect();
        let new_perm = other_perm.clone() * self_perm * other_perm.inv();
        Circ::new(new_instrs, new_perm)
    }

    pub fn remove_swaps(&self) -> Self {
        let mut qregs_map: Vec<u8> = (0..self.nqubits()).collect();
        let mut new_instrs = Vec::new();
        for instr in &self.instrs {
            if instr.0 == *crate::circ::gates::SWAP {
                let q1 = instr.1[0] as usize;
                let q2 = instr.1[1] as usize;
                qregs_map.swap(q1, q2);
            } else {
                let new_qargs = instr.1.iter().map(|&q| qregs_map[q as usize]).collect();
                new_instrs.push(Instr32(instr.0.clone(), new_qargs));
            }
        }
        let map_perm = Permut32::from_iter_unchecked(qregs_map.into_iter());
        let last_perm = self.perm.clone() * map_perm.inv();
        Circ::new(new_instrs, last_perm)
    }



    pub fn inverse(&self) -> Self {
        let new_instrs = self
            .instrs
            .iter()
            .rev()
            .map(|instr| instr.adjoint().permut(self.perm))
            .collect();
        Circ::new(new_instrs, self.perm.inv())
    }

    pub fn rotate(&self, n: isize) -> Self {
        if n == 0 { return self.clone(); }
        let len = self.instrs.len();
        
        let new_instrs = if n > 0 {
            let n = n as usize;
            let perm_inv = self.perm.inv();
            self.instrs[n..]
                .iter()
                .cloned()
                .chain(self.instrs[..n].iter().map(|instr| instr.permut(perm_inv)))
                .collect()
        } else {
            let n = (-n) as usize;
            self.instrs[len - n..]
                .iter()
                .map(|instr| instr.permut(self.perm))
                .chain(self.instrs[..len - n].iter().cloned())
                .collect()
        };
        Circ::new(new_instrs, self.perm.clone())
    }

    pub fn len(&self) -> usize {
        self.instrs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.instrs.is_empty()
    }

    pub fn instrs_with_swaps(&self) -> Vec<Instr32> {
        let mut instrs = self.instrs.clone();
        instrs.extend(
            self.perm
                .generate_swaps()
                .map(|(a, b)| Instr32(*crate::circ::gates::SWAP, [a, b].into_iter().collect())),
        );
        instrs
    }
    pub fn __add__(&self, other: &Circ) -> Circ {
        self + other
    }
    pub fn __len__(&self) -> usize {
        self.len()
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
    pub fn __getitem__(&self, idx: isize) -> Instr32 {
        self[idx.rem_euclid(self.len() as isize) as usize].clone()
    }

}

impl Add for &Circ {
    type Output = Circ;

    fn add(self, other: Self) -> Circ {
        let (self_perm, other_perm) = self.perm.coordinate_permute(other.perm);
        let perm_inv = self_perm.inv();
        let mut new_instrs = self.instrs.clone();
        new_instrs.extend(other.instrs.iter().map(|instr| instr.permut(perm_inv)));
        let new_perm = other_perm * self_perm;
        Circ::new(new_instrs, new_perm)
    }
}

impl AddAssign<&Circ> for Circ {
    fn add_assign(&mut self, other: &Circ) {
        let (self_perm, other_perm) = self.perm.coordinate_permute(other.perm);
        let perm_inv = self_perm.inv();
        self.instrs
            .extend(other.instrs.iter().map(|instr| instr.permut(perm_inv)));
        self.perm = other_perm * self_perm;
    }
}

impl fmt::Display for Circ {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Circuit {{ ")?;
        for instr in &self.instrs {
            write!(f, "{} ", instr)?;
        }
        write!(f, "{} }}", self.perm)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::circ::gates::{CX, H, S, SWAP, T, tdg};
    use smallvec::smallvec;

    fn h(q: u8) -> Instr32 {
        Instr32(*H, [q].into_iter().collect())
    }

    fn s(q: u8) -> Instr32 {
        Instr32(*S, [q].into_iter().collect())
    }

    fn t(q: u8) -> Instr32 {
        Instr32(*T, [q].into_iter().collect())
    }

    fn cx(c: u8, t: u8) -> Instr32 {
        Instr32(*CX, [c, t].into_iter().collect())
    }

    fn swap(q1: u8, q2: u8) -> Instr32 {
        Instr32(*SWAP, [q1, q2].into_iter().collect())
    }

    #[test]
    fn test_rotate_representative_simple_rotation() {
        let circ1 = Circ::new_no_perm(vec![h(0), t(0)]);
        let rep1 = circ1.rotate_representative();

        let circ2 = Circ::new_no_perm(vec![tdg(0), h(0)]);
        let rep2 = circ2.rotate_representative();

        assert_eq!(rep1, rep2);
    }

    #[test]
    fn test_rotate_representative_with_permutation() {
        // H(0) CX(0,1)
        let circ1 = Circ::new_no_perm(vec![h(0), cx(0, 1)]);
        let rep1 = circ1.rotate_representative();

        // H(1) CX(1,0) -> permute(0,1) -> H(0) CX(0,1)
        let circ2 = Circ::new_no_perm(vec![h(1), cx(1, 0)]);
        let rep2 = circ2.rotate_representative();

        assert_eq!(rep1, rep2);
    }

    #[test]
    fn test_rotate_representative_with_rotation_and_permutation() {
        // CX(0,1) H(0) -> rotate -> H(0) CX(0,1)
        let circ1 = Circ::new_no_perm(vec![cx(0, 1), h(0)]);
        let rep1 = circ1.rotate_representative();

        // CX(1,0) H(1) -> permute(0,1) -> CX(0,1) H(0) -> rotate -> H(0) CX(0,1)
        let circ2 = Circ::new_no_perm(vec![cx(1, 0), h(1)]);
        let rep2 = circ2.rotate_representative();

        assert_eq!(rep1, rep2);
    }

    #[test]
    fn test_rotate_representative_with_reordering() {
        // H(0) S(1) is disjoint and H < S, so this is canonical
        let circ1 = Circ::new_no_perm(vec![h(0), s(1)]);
        let rep1 = circ1.rotate_representative();

        // S(1) H(0) should be reordered to H(0) S(1)
        let circ2 = Circ::new_no_perm(vec![s(1), h(0)]);
        let rep2 = circ2.rotate_representative();

        assert_eq!(rep1, rep2);
    }

    #[test]
    fn test_rotate_representative_complex() {
        // A more complex case involving rotation, permutation, and reordering
        // circ1: S(1) H(0) CX(0,1)
        // One rotation brings H(0) to front: H(0) CX(0,1) S(1)
        // Reordering S(1) gives: H(0) S(1) CX(0,1)
        let circ1 = Circ::new_no_perm(vec![s(1), h(0), cx(0, 1)]);
        let rep1 = circ1.rotate_representative();

        // circ2: permute(0,1) of the canonical form
        // H(1) S(0) CX(1,0)
        let circ2 = Circ::new_no_perm(vec![h(1), s(0), cx(1, 0)]);
        let rep2 = circ2.rotate_representative();

        assert_eq!(rep1, rep2);
    }
}
