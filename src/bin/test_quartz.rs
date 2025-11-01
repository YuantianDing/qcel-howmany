use std::{fs::File, io::{BufReader, BufWriter}};

use itertools::Itertools;
use quclif::{circ::{gates::{CX, CY, CZ, H, S, SDG, T, TDG, X, Y, Z}, Instr32}, identity::{eccproof::IdentityProver, idcircuit::IdentityCirc}, instr_vec, search::{double_perm_search::{CircTriple, Evaluator, RawECCs}, ECCs}, utils::JoinOptionIter};

fn main() {
    let nqubits = 5;
    let ngates = 6;
    let use_eqclass = true;
    let evaluator = Evaluator::from_random(nqubits, &mut rand::rng());
    let (eccs, _) = if use_eqclass {
        RawECCs::generate(&evaluator, vec![*H, *X, *TDG, *T, *CX], ngates)  // *CY, *CZ, *Y, *Z, *SDG, *S
    } else {
        RawECCs::generate_naive(&evaluator, vec![*H, *X, *TDG, *T, *CX], ngates) // *CY, *CZ, *Y, *Z, *SDG, *S
    };

    let eccs = eccs.simplify().filter_single();
    
    
    postcard::to_io(&eccs, BufWriter::new(File::create("eccset.serde").unwrap())).expect("Failed to write ECCs to file");
    let result = IdentityProver::build(eccs.as_slice());
    println!("Number of assumed identities: {}", result.assumed_identities().len());
    postcard::to_io(&eccs, BufWriter::new(File::create("prover6.serde").unwrap()))
        .expect("Failed to write ECCs to file");
    
    for id in result.assumed_identities() {
        println!("{}", id);
    }
    serde_json::to_writer(
        BufWriter::new(File::create("rules17.json").unwrap()),
        &result.assumed_identities()).expect("Failed to write assumed identities to file");
    

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