use quclif::{circ::{gates::{CX, H, T, TDG, X, Z}, Instr}, instr_vec, search::{double_perm_search::{CircTriple, CircuitECCs, Evaluator}, ECCs}, utils::JoinOptionIter};



fn main() {
    let evaluator = Evaluator::from_random(5, &mut rand::rng());
    let eccs = CircuitECCs::generate_naive(&evaluator, vec![*H, *X, *TDG, *T, *CX], 6);
    
    // let instrs = instr_vec![
    //     H 0;
    //     CX 0,1;
    //     H 0;
    //     CX 1,0;
    //     H 0;
    // ];
    
    // for i in 0..instrs.len() {
    //     for j in (i+1)..=instrs.len() {
    //         let (backstate, front_perm, back_perm) = evaluator.evaluate(&instrs[i..j]);
    //         println!("{i}..{j} {front_perm} {} {back_perm}", instrs[i..j].iter().join_option(" ", "", ""));
    //         eccs.get(&backstate.hash_value()).map(|ecc| {
    //             for c in ecc.circuits.iter() {
    //                 println!("\t{}", c);
    //             }
    //             println!("\t{}", CircTriple {
    //                 circ: instrs[i..j].iter().cloned().collect(),
    //                 front_perm,
    //                 back_perm,
    //             });
    //         });
    //     }
    // }

    let eccs = eccs.simplify();
    let quartz_data = eccs.as_quartz_no_perm_variants();
    let mut file = std::fs::File::create("eccset.json").unwrap();
    serde_json::to_writer(&mut file, &quartz_data).unwrap();
    
    println!("Number of ECCs: {}", eccs.len());
    for problem in eccs.check() {
        eprintln!("Correctness Error:");
        for c in problem.circuits() {
            eprintln!("\t{}", c.iter().join_option(" ", "", ""))
        }
    }

    // let _ = eccs.dump_quartz("eccset.json".into());
}