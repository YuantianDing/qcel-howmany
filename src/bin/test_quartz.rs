use quclif::{circ::{gates::{CX, H, T, TDG, X, Z}, Instr}, instr_vec, search::{double_perm_search::{CircTriple, CircuitECCs, Evaluator}, ECCs}, utils::JoinOptionIter};



fn main() {
    let evaluator = Evaluator::from_random(3, &mut rand::rng());
    let eccs = CircuitECCs::generate(&evaluator, vec![*H, *X, *TDG, *T, *CX], 5);
    
    let instrs = instr_vec![
        H 0;
        CX 0,1;
        H 0;
        CX 1,0;
        H 0;
    ];
    
    for i in 0..instrs.len() {
        for j in (i+1)..=instrs.len() {
            let (backstate, front_perm, back_perm) = evaluator.evaluate(&instrs[i..j]);
            println!("{i}..{j} {front_perm} {} {back_perm}", instrs[i..j].iter().join_option(" ", "", ""));
            eccs.get(&backstate.hash_value()).map(|ecc| {
                for c in ecc.circuits.iter() {
                    println!("\t{}", c);
                }
                println!("\t{}", CircTriple {
                    circ: instrs[i..j].iter().cloned().collect(),
                    front_perm,
                    back_perm,
                });
            });
        }
    }

    let eccs = eccs.simplify();
    eccs.dump_quartz("eccset.json".into()).unwrap();
    println!("Number of ECCs: {}", eccs.len());
    for problem in eccs.check() {
        eprintln!("Correctness Error:");
        for c in problem.circuits() {
            eprintln!("\t{}", c.iter().join_option(" ", "", ""))
        }
    }

    eccs.dump_quartz("eccset.json".into());
}