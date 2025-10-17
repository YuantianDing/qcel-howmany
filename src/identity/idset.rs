use std::collections::{HashMap, HashSet, VecDeque};

use itertools::Itertools;
use linear_map::set::LinearSet;
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pymethods};
use rayon::iter::ParallelIterator;

use crate::identity::{
    circuit::Circ,
    idcircuit::{IdentityCirc, IdentitySubcircuit},
};

#[gen_stub_pyclass]
#[pyo3::pyclass]
#[derive(Debug, Clone)]
pub struct IdentitySet {
    identities: HashSet<IdentityCirc>,
    rules: Vec<HashMap<Circ, LinearSet<Circ>>>,
}

impl IdentitySet {
    pub fn apply_rules<'a>(
        &'a self,
        identity: &'a IdentityCirc,
    ) -> impl Iterator<Item = IdentityCirc> + 'a {
        self.rules
            .iter()
            .enumerate()
            .flat_map(move |(i, circmap): (_, &'a _)| {
                IdentitySubcircuit::subcircuit_splits(identity, i).flat_map(move |(c1, c2)| {
                    let (c1_rep, perm) = c1.representative_with_perm();
                    circmap.get(&c1_rep).into_iter().flat_map(move |a| {
                        let c2_permuted = c2.permut(perm);
                        a.iter().map(move |c| {
                            (c + &c2_permuted).rotate_representative()
                        })
                    })
                })
            })
    }
    pub fn par_apply_rules<'a>(
        &'a self,
        identity: &'a IdentityCirc,
    ) -> Vec<IdentityCirc> {
        IdentitySubcircuit::par_subcircuit_splits(identity).flat_map(move |(c1, c2)| {
            let (c1_rep, perm) = c1.representative_with_perm();
            self.rules.get(c1_rep.len()).and_then(|a| a.get(&c1_rep)).into_iter().flat_map(move |a| {
                let c2_permuted = c2.permut(perm);
                a.iter().map(move |c| {
                    (c + &c2_permuted).rotate_representative()
                })
            }).collect_vec()
        }).collect()
    }
    pub fn identities(self) -> HashSet<IdentityCirc> {
        self.identities
    }
}
#[gen_stub_pymethods]
#[pyo3::pymethods]
impl IdentitySet {
    #[new]
    pub fn new() -> Self {
        Self {
            identities: HashSet::new(),
            rules: Vec::new(),
        }
    }

    pub fn add_identity(&mut self, identity: IdentityCirc, max_step: usize) -> Option<IdentityCirc> {
        let mut search_queue = VecDeque::new();
        let mut visited = HashSet::new();

        if identity.is_empty() || self.identities.contains(&identity) {
            return None;
        }

        let total_size = identity.len();
        search_queue.push_back((identity.clone(), 0));
        visited.insert(identity.clone());
        let cont: Option<IdentityCirc> = 'poploop: loop {
            let Some((id0, nstep)) = search_queue.pop_front() else {
                break None;
            };
            for new_id in self.par_apply_rules(&id0) {
                if visited.contains(&new_id) {
                    continue;
                }
                visited.insert(new_id.clone());
                if new_id.len() < total_size {
                    break 'poploop Some(new_id);
                } else if self.identities.contains(&new_id) {
                    return None;
                } else if nstep + 1 < max_step {
                    search_queue.push_back((new_id, nstep + 1));
                }
            }
        };
        if let Some(id) = cont {
            return self.add_identity(id, max_step);
        }
        // let identity = visited.into_iter().min().unwrap();
        self.save_identity(identity.clone());
        Some(identity)
    }
    fn save_identity(&mut self, identity: IdentityCirc) {
        let total_size = identity.len();
        assert!(!self.identities.contains(&identity));
        self.identities.insert(identity.clone());

        let sizes = if total_size % 2 == 0 {
            vec![total_size / 2, total_size / 2 + 1]
        } else {
            vec![total_size / 2 + 1]
        };

        for i in sizes {
            if i == 0 || i > total_size { continue; }
            for (c1, c2) in IdentitySubcircuit::subcircuit_splits(&identity, i) {
                let (c1_0, perm) = c1.inverse().representative_with_perm();
                let c2_0 = c2.permut(perm).reorder_instrs().compact_circuit();

                if c1_0 == c2_0 {
                    continue   
                }
                while c1_0.len() >= self.rules.len() {
                    self.rules.push(HashMap::new());
                }
                self.insert_rule(c1_0.clone(), c2_0.clone());

                let c1_len = c1.len();
                let (c1_1, perm) = c1.representative_with_perm();
                let c2_1 = c2.inverse().permut(perm).reorder_instrs().compact_circuit();
                if c1_1 == c2_1 {
                    continue
                }
                self.insert_rule(c1_1.clone(), c2_1.clone());

                if c2.is_empty() {
                    for j in 1.. c1_len {
                        let c1_0_rot_rep = c1_0.rotate(j as isize).representative();
                        self.insert_rule(c1_0_rot_rep, c2_0.clone());

                        let c1_1_rot_rep = c1_1.rotate(j as isize).representative();
                        self.insert_rule(c1_1_rot_rep, c2_1.clone());
                    }
                }
            }
        }
    }
    fn insert_rule(&mut self, lhs: Circ, rhs: Circ) {
        self.rules[lhs.len()]
            .entry(lhs)
            .or_default()
            .insert(rhs);
    }
    #[pyo3(name = "as_list")]
    pub fn as_list_py(&self) -> Vec<IdentityCirc> {
        let mut idvec: Vec<IdentityCirc> = self.identities.iter().cloned().collect();
        idvec.sort_by(|a, b| a.len().cmp(&b.len()).then_with(|| a.cmp(b)));
        idvec
    }
    pub fn count_rules(&self) -> usize {
        self.rules.iter().map(|m| m.len()).sum()
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test() {
        
    }
}