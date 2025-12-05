use std::collections::HashSet;

use itertools::Itertools;
use serde_json::{json, Value};

use crate::{circ::{gates::SWAP}, groups::permutation::Permut32, search::{ECCs, Instr32}};

impl ECCs {
    pub fn as_quartz(&self) -> serde_json::Value {
        let mut eccs = serde_json::Map::<String, Value>::new();

        for (k, ecc) in self.iter().enumerate() {
            if ecc.len() <= 1 { continue; }
            for (i, ecc) in generate_ecc_with_swap(&ecc).enumerate() {
                eccs.insert(format!("{:x}_{i}", k), ecc_to_json(ecc).into());
            }
        }

        return json!([[[0], [0]], eccs]);
    }
    pub fn as_quartz_no_perm_variants(&self) -> serde_json::Value {
        let mut eccs = serde_json::Map::<String, Value>::new();

        for (k, ecc) in self.iter().enumerate() {
            if ecc.len() <= 1 { continue; }
            let ecc = generate_ecc_with_swap(&ecc).next().unwrap();
            eccs.insert(format!("{k:x}"), ecc_to_json(ecc).into());
        }

        return json!([[[0], [0]], eccs]);
    }
}

fn ecc_to_json(ecc: Vec<Vec<Instr32>>) -> Vec<Value> {
    let ecc = ecc.iter().map(|list| {
        // println!("   {}", list.iter().fjoin(" "));
        let mut largest_qubit = 0;
        let instrs = list.iter().map(|Instr32(g, inds)| {
            let qs = inds.iter().map(|q| format!("Q{q}")).collect_vec();
            largest_qubit = largest_qubit.max(*inds.iter().max().unwrap_or(&0));
            json!([format!("{}", g), qs, qs])
        }).collect_vec();
        json!([
            [largest_qubit+1, instrs.len()],
            instrs
        ])
    }).collect_vec();
    // println!("}}\n");
    ecc
}


fn generate_ecc_with_swap<'a>(ecc: &'a [(Vec<Instr32>, Permut32)]) -> impl Iterator<Item=Vec<Vec<Instr32>>> + 'a {
    let mut covered = HashSet::<usize>::new();
    (0..ecc.len()).into_iter().filter_map(move |i| {
        if covered.contains(&i) { return None; }
        let perm = ecc[i].1.inv();
        Some(ecc.iter().enumerate().map(|(i, (instrs, p))| {
            let pp = perm * *p;
            if pp.is_identity() { covered.insert(i); }
            instrs.iter().cloned().chain(
                pp.generate_swaps().map(|(a, b)| Instr32(*SWAP, [a, b].into_iter().collect()))
            ).collect_vec()
        }).collect_vec())
    })
}

