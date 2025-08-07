use std::collections::HashSet;

use itertools::Itertools;
use serde_json::{json, Value};
use smallvec::smallvec;

use crate::{circ::{gates::SWAP}, groups::permutation::Permut32, search::{CircuitECCs, Instr}, utils::JoinOptionIter};

impl CircuitECCs {
    pub fn as_quartz(&self) -> serde_json::Value {
        let mut eccs = serde_json::Map::<String, Value>::new();

        for (k, ecc) in self.inner.iter() {
            if ecc.circuits.len() <= 1 { continue; }
            for (i, ecc) in generate_ecc_with_swap(&ecc.simplify()).enumerate() {
                let eccname = format!("{:x}_{i}", k);
                // println!("ECC {eccname} {{");
                let ecc = ecc.iter().map(|list| {
                    // println!("   {}", list.iter().join_option(" ", "", ""));
                    let mut largest_qubit = 0;
                    let mut instrs = list.iter().map(|Instr(g, inds)| {
                        let qs = inds.iter().map(|q| format!("Q{q}")).collect_vec();
                        largest_qubit = largest_qubit.max(*inds.iter().max().unwrap_or(&0));
                        json!([g.name(), qs, qs])
                    }).collect_vec();
                    json!([
                        [largest_qubit+1, instrs.len()],
                        instrs
                    ])
                }).collect_vec();
                // println!("}}\n");
                eccs.insert(format!("{:x}_{i}", k), ecc.into());
            }
        }

        return json!([[[0], [0]], eccs]);
    }
}


fn generate_ecc_with_swap<'a>(ecc: &'a [(Vec<Instr>, Permut32)]) -> impl Iterator<Item=Vec<Vec<Instr>>> + 'a {
    let mut covered = HashSet::<usize>::new();
    (0..ecc.len()).into_iter().filter_map(move |i| {
        if covered.contains(&i) { return None; }
        let perm = ecc[i].1.inv();
        Some(ecc.iter().enumerate().map(|(i, (instrs, p))| {
            let pp = perm * *p;
            if pp.is_identity() { covered.insert(i); }
            instrs.iter().cloned().chain(
                pp.generate_swaps().map(|(a, b)| Instr(*SWAP, smallvec![a, b]))
            ).collect_vec()
        }).collect_vec())
    })
}

