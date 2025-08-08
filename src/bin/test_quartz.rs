use quclif::{circ::gates::{CNOT, H, T, TDG, X, Z}, search::ECCs, utils::JoinOptionIter};



fn main() {
    let eccs = ECCs::generate(5, vec![*H, *X, *TDG, *T, *CNOT], 6);
    println!("{eccs}");
    for problem in eccs.check() {
        eprintln!("Correctness Error:");
        for c in problem.circuits() {
            eprintln!("\t{}", c.iter().join_option(" ", "", ""))
        }
    }
}