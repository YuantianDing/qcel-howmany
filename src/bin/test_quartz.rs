use quclif::{circ::{gates::{CX, H, T, TDG, X, Z}, Instr}, instr_vec, search::{double_perm_search::CircuitECCs, ECCs}, utils::JoinOptionIter};



fn main() {
    let eccs = CircuitECCs::generate(5, vec![*H, *X, *TDG, *T, *CX], 5, &mut rand::rng());
    
    let instrs = instr_vec![
        TDG 1;
        CX 0,1;
        T 1;
        CX 0,1;
        T 1
    ];
    
    for i in 0..=instrs.len() {
        eccs.find(&instrs[..i]).map(|a| println!("{i} {}", a.simplify()));
    }

    let eccs = eccs.simplify();
    eccs.dump_quartz("eccset.json".into()).unwrap();
}