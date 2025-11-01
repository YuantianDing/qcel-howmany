use std::collections::{HashMap, HashSet, VecDeque};

use itertools::Itertools;
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pymethods};
use rayon::iter::ParallelIterator;
use sorted_vec2::SortedSet;

use crate::identity::{
    circuit::Circ,
    idcircuit::{IdentityCirc, IdentitySubcircuit},
};

#[gen_stub_pyclass]
#[pyo3::pyclass]
#[derive(Debug, Clone)]
pub struct IdentitySet {
    identities: HashSet<IdentityCirc>,
    rules: Vec<HashMap<Circ, SortedSet<Circ>>>,
    max_identity_size: usize,
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
                IdentitySubcircuit::subcircuit_splits_n(identity, i).flat_map(move |(c1, c2)| {
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
    pub fn new(max_identity_size: usize) -> Self {
        Self {
            identities: HashSet::new(),
            rules: Vec::new(),
            max_identity_size
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
                    if identity.len() / 2 < self.max_identity_size / 2 {
                        for identity in visited.into_iter() {
                            self.track_identity(&identity);
                        }
                    }
                    return None;
                } else if nstep + 1 < max_step {
                    search_queue.push_back((new_id, nstep + 1));
                }
            }
        };
        if let Some(id) = cont {
            let result = self.add_identity(id, max_step);
            if identity.len() / 2 < self.max_identity_size / 2 {
                for identity in visited.into_iter() {
                    self.track_identity(&identity);
                }
            }

            result
        } else {
            self.save_identity(&identity);
            Some(identity)
        }
    }
    pub fn test_identity(&self, identity: IdentityCirc, max_step: usize) -> Option<IdentityCirc> {
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
            self.test_identity(id, max_step)
        } else {
            Some(identity)
        }
    }
    fn save_identity(&mut self, identity: &IdentityCirc) {
        self.identities.insert(identity.clone());

        
        let total_size = identity.len();
        if total_size % 2 == 0 { 
            self.add_identity_rules_on_size(identity, total_size / 2);
        } 
        self.add_identity_rules_on_size(identity, total_size / 2 + 1);
    }
    fn track_identity(&mut self, identity: &IdentityCirc) {
        // if self.par_apply_rules(&identity).into_iter().any(|id| id.len() < identity.len()) { return; }
        
        let total_size = identity.len();
        let size = total_size / 2 + 1;
        self.add_identity_rules_on_size(identity, size);
    }
    fn add_identity_rules_on_size(&mut self, identity: &IdentityCirc, size: usize) {
        assert!(size >= identity.len() / 2);
        for (c1, c2) in IdentitySubcircuit::subcircuit_splits_n(identity, size) {
            let (c1_0, perm) = c1.inverse().representative_with_perm();
            let c2_0 = c2.permut(perm).reorder_instrs().compact_qubits();

            if c1_0 == c2_0 {
                continue   
            }
            while c1_0.len() >= self.rules.len() {
                self.rules.push(HashMap::new());
            }
            self.insert_rule(c1_0.clone(), c2_0.clone());

            let c1_len = c1.len();
            let (c1_1, perm) = c1.representative_with_perm();
            let c2_1 = c2.inverse().permut(perm).reorder_instrs().compact_qubits();
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
    fn insert_rule(&mut self, lhs: Circ, rhs: Circ) -> bool {
        self.rules[lhs.len()]
            .entry(lhs)
            .or_default()
            .find_or_insert(rhs).is_found()
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