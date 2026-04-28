
use qcel_howmany::{circ::gates::*, identity::eccprove::IdentityProver, search::double_perm_search::{Evaluator, RawECCs}};
use rand::SeedableRng;

fn main() {
    let nqubits = 5;
    let ngates = 7;
    let use_eqclass = true;
    let evaluator = Evaluator::from_random(nqubits, &mut rand::rngs::StdRng::from_seed([2; 32]));
    let (eccs, _) = if use_eqclass {
        RawECCs::generate(&evaluator, vec![*H, *X, *TDG, *T, *CX], ngates)  // *CY, *CZ, *Y, *Z, *SDG, *S
    } else {
        RawECCs::generate_naive(&evaluator, vec![*H, *X, *TDG, *T, *CX], ngates) // *CY, *CZ, *Y, *Z, *SDG, *S
    };

    println!("{}", eccs.len());
 
    // eccs.simplify().dump_quartz("./cache/_eccset.json".into()).expect("");
}