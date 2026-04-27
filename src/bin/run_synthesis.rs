
use qcel_howmany::{circ::gates::*, identity::eccprove::IdentityProver, search::double_perm_search::{Evaluator, RawECCs}};
use rand::SeedableRng;

fn main() {
    let nqubits = 5;
    let ngates = 2;
    println!("Generating ECCs for {} qubits and {} gates. PERCISION_LEVEL={}", nqubits, ngates, qcel_howmany::Qreal::PERCISION_LEVEL);
    let evaluator1 = Evaluator::from_random(nqubits, &mut rand::rngs::StdRng::from_seed([1; 32]));
    let (ecc1, _) = RawECCs::generate(&evaluator1, vec![*H, *X, *TDG, *T, *CX, *CY, *CZ, *Y, *Z], ngates);
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

    let Some(ecc) = ecc1.find_equivalents(&evaluator1, &[cz(0, 1), cx(0, 2)]) else {
        println!("No equivalent circuits found.");
        return;
    };

    println!("ECC for cz(0, 1) cx(0, 2): {}", ecc);


    // let eccs = ecc1.simplify().filter_single();
    // for (id, initial, c) in eccs.to_identity_circuits() {
    //     println!("{} = {} + {}", id, initial, c);
    // }

    // postcard::to_io(&eccs, BufWriter::new(File::create("eccset.serde").unwrap())).expect("Failed to write ECCs to file");
    // let result =  IdentityProver::build(eccs.as_slice());
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
    //         eprintln!("\t{}", c.iter().fjoin(" "))
    //     }
    // }

    // let _ = eccs.dump_quartz("eccset.json".into());
}