use std::{collections::HashSet, fs::File, io::{BufReader, BufWriter}};

use indicatif::ProgressIterator;
use itertools::Itertools;
use quclif::{circ::{Instr32, gates::*}, groups::permutation::Permut32, identity::{circuit::Circ, eccproof::IdentityProver, idcircuit::IdentityCirc}, instr_vec, search::{ECCs, double_perm_search::{CircTriple, Evaluator, RawECCs}}, utils::JoinOptionIter};
use rand::SeedableRng;

fn main() {
    let nqubits = 5;
    let ngates = 6;
    println!("Generating ECCs for {} qubits and {} gates. PRECISION_LEVEL={}", nqubits, ngates, quclif::Qreal::PERCISION_LEVEL);
    let evaluator1 = Evaluator::from_random(nqubits, &mut rand::rngs::StdRng::from_seed([1; 32]));
    let (ecc1, _) = RawECCs::generate(&evaluator1, vec![*H, *X, *TDG, *T, *CX], ngates);
    println!("{}", ecc1.len());
    // println!("{}", ecc1.switch_evaluator(&evaluator2).len());
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