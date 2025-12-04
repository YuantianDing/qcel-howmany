use std::{collections::HashSet, fs::File, io::{BufReader, BufWriter}};

use indicatif::ProgressIterator;
use itertools::Itertools;
use quclif::{circ::{gates::*, Instr32}, identity::{eccproof::IdentityProver, idcircuit::IdentityCirc}, instr_vec, search::{double_perm_search::{CircTriple, Evaluator, RawECCs}, ECCs}, utils::JoinOptionIter};
use rand::SeedableRng;

fn main() {
    let nqubits = 5;
    let ngates = 6;
    let use_eqclass = true;
    let evaluator = Evaluator::from_random(nqubits, &mut rand::rngs::StdRng::from_seed([0; 32]));
    let start1 = std::time::Instant::now();
    let (ecc1, _) = RawECCs::generate(&evaluator, vec![*H, *X, *TDG, *T, *CX], ngates);
    let duration1 = start1.elapsed();
    let start2 = std::time::Instant::now();
    let (ecc2, _) = RawECCs::generate_naive(&evaluator, vec![*H, *X, *TDG, *T, *CX], ngates);
    println!("Time {:?} {:?}", duration1, start2.elapsed());
    
    // for (key, ecc) in ecc1.iter() {
    //     for circ in &ecc.circuits {
    //         let mut instrs: Vec<_> = circ.circ.iter().cloned().collect();
    //         instrs.reverse();
    //         let (bv, _, _, _) = evaluator.evaluate(&instrs);
    //         assert!(key == &bv.hash_value());
    //     }
    // }
    // let instrs0 = vec![h(0), h(1), cx(0, 2), cx(2, 3), cx(1, 2), h(3), cx(1, 4)];
    // let instrs1 = vec![h(0), cx(0, 1), cx(1, 2), h(2), h(3), cx(3, 1), cx(3, 4)];
    // let (bv0, perms0, mut fv0) = evaluator.evaluate_multiple(&instrs0);
    // let (bv1, perms1, mut fv1) = evaluator.evaluate_multiple(&instrs1);
    // fv0.apply_permutation(fv0.get_permutation().inv());
    // fv1.apply_permutation(fv1.get_permutation().inv());

    // assert!(bv0 == bv1);
    // println!("{} {}", fv0, fv1);
    // println!("fv0: {} BV0: {}, perms0: {}", fv0.get_orderinfo(), bv0, perms0.iter().map(|(a, b)| format!("{a} {b}")).join_option(",", "", ""));
    // println!("fv0: {} BV0: {}, perms0: {}", fv1.get_orderinfo(), bv1, perms1.iter().map(|(a, b)| format!("{a} {b}")).join_option(",", "", ""));

    for (_, ecc) in ecc2.iter().progress() {
        for (instrs, _) in &ecc.simplify().0 {
            let (bv, f, p, _) = evaluator.evaluate(&instrs);
            if !ecc1.contains_key(&bv.hash_value()) {
                for i in 1..=instrs.len() {
                    let key = evaluator.evaluate(&instrs[..i]).0.hash_value();
                    println!("{} => {:?}",
                        instrs[..i].iter().join_option(" ", "", ""),
                        ecc1.get(&key)
                    );
                }
            }
            assert!(ecc1.contains_key(&bv.hash_value()));
        }
    }

    // let nqubits = 5;
    // let ngates = 6;
    // let use_eqclass = true;
    // let evaluator = Evaluator::from_random(nqubits, &mut rand::rng());
    // let (eccs, _) = if use_eqclass {
    //     RawECCs::generate(&evaluator, vec![*H, *X, *TDG, *T, *CX], ngates)  // *CY, *CZ, *Y, *Z, *SDG, *S
    // } else {
    //     RawECCs::generate_naive(&evaluator, vec![*H, *X, *TDG, *T, *CX], ngates) // *CY, *CZ, *Y, *Z, *SDG, *S
    // };

    // let eccs = eccs.simplify().filter_single();
    
    
    // postcard::to_io(&eccs, BufWriter::new(File::create("eccset.serde").unwrap())).expect("Failed to write ECCs to file");
    // let result = IdentityProver::build(eccs.as_slice());
    // println!("Number of assumed identities: {}", result.assumed_identities().len());
    // postcard::to_io(&eccs, BufWriter::new(File::create("prover6.serde").unwrap()))
    //     .expect("Failed to write ECCs to file");
    
    // for id in result.assumed_identities() {
    //     println!("{}", id);
    // }
    // serde_json::to_writer(
    //     BufWriter::new(File::create("rules17.json").unwrap()),
    //     &result.assumed_identities()).expect("Failed to write assumed identities to file");
    

    // let rules: Vec<IdentityCirc> = serde_json::from_reader(BufReader::new(File::open("rules200.json").unwrap())).expect("Failed to read rules from file");
    // let rules = rules.into_iter().filter_map(|id| result.prove_identity(id, 2, 50000)).collect_vec();
    // serde_json::to_writer(
    //     BufWriter::new(File::create("rules200_filtered.json").unwrap()),
    //     &rules,
    // ).expect("Failed to write proved identities to file");

    // println!("Number of ECCs: {}", eccs.len());
    // for problem in eccs.check() {
    //     eprintln!("Correctness Error:");
    //     for c in problem.circuits() {
    //         eprintln!("\t{}", c.iter().join_option(" ", "", ""))
    //     }
    // }

    // let _ = eccs.dump_quartz("eccset.json".into());
}