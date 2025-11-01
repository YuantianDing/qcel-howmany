use std::{collections::HashMap, f64::consts::PI, sync::LazyLock};

use nohash_hasher::BuildNoHashHasher;
use numpy::Complex64;

use crate::{circ::{gates, Gate16}, state::{indices::{qubit_matrix_indices1, qubit_matrix_indices2}, StateVec}};




fn perform_x_gate(state: &mut StateVec, qubits: &[u8]) {
    for indices in qubit_matrix_indices1(state.nqubits(), qubits.try_into().unwrap()) {
        let mut vec = state.access(indices);
        vec.swap(0, 1);
        state.update(indices, vec);
    }
}


fn perform_h_gate(state: &mut StateVec, qubits: &[u8]) {
    for indices in qubit_matrix_indices1(state.nqubits(), qubits.try_into().unwrap()) {
        let mut vec = state.access(indices);
        let (a, b) = (vec[0], vec[1]);
        vec[0] = (a + b) / 2.0f64.sqrt();
        vec[1] = (a - b) / 2.0f64.sqrt();
        state.update(indices, vec);
    }
}

fn perform_s_gate(state: &mut StateVec, qubits: &[u8]) {
    for indices in qubit_matrix_indices1(state.nqubits(), qubits.try_into().unwrap()) {
        let mut vec = state.access(indices);
        vec[1] *= Complex64::from_polar(1.0, PI / 2.0);
        state.update(indices, vec);
    }
}

fn perform_sdg_gate(state: &mut StateVec, qubits: &[u8]) {
    for indices in qubit_matrix_indices1(state.nqubits(), qubits.try_into().unwrap()) {
        let mut vec = state.access(indices);
        vec[1] *= Complex64::from_polar(1.0, -PI / 2.0);
        state.update(indices, vec);
    }
}

fn perform_t_gate(state: &mut StateVec, qubits: &[u8]) {
    for indices in qubit_matrix_indices1(state.nqubits(), qubits.try_into().unwrap()) {
        let mut vec = state.access(indices);
        vec[1] *= Complex64::from_polar(1.0, PI / 4.0);
        state.update(indices, vec);
    }
}

fn perform_tdg_gate(state: &mut StateVec, qubits: &[u8]) {
    for indices in qubit_matrix_indices1(state.nqubits(), qubits.try_into().unwrap()) {
        let mut vec = state.access(indices);
        vec[1] *= Complex64::from_polar(1.0, -PI / 4.0);
        state.update(indices, vec);
    }
}

fn perform_z_gate(state: &mut StateVec, qubits: &[u8]) {
    for indices in qubit_matrix_indices1(state.nqubits(), qubits.try_into().unwrap()) {
        let mut vec = state.access(indices);
        vec[1] *= -1.0;
        state.update(indices, vec);
    }
}

fn perform_swap_gate(state: &mut StateVec, qubits: &[u8]) {
    for indices in qubit_matrix_indices2(state.nqubits(), qubits.try_into().unwrap()) {
        let mut vec = state.access(indices);
        vec.swap(1, 2);
        state.update(indices, vec);
    }
}

fn perform_cx_gate(state: &mut StateVec, qubits: &[u8]) {
    for indices in qubit_matrix_indices2(state.nqubits(), qubits.try_into().unwrap()) {
        let mut vec = state.access(indices);
        vec.swap(2, 3);
        state.update(indices, vec);
    }
}

pub static GATE_FUNCS: LazyLock<HashMap<Gate16, fn(&mut StateVec, &[u8]), BuildNoHashHasher<u64>>> = LazyLock::new(|| {
    let mut m: HashMap<Gate16, fn(&mut StateVec, &[u8]), BuildNoHashHasher<u64>> = Default::default();
    m.insert(*gates::H, perform_h_gate);
    m.insert(*gates::X, perform_x_gate);
    m.insert(*gates::Z, perform_z_gate);
    m.insert(*gates::T, perform_t_gate);
    m.insert(*gates::TDG, perform_tdg_gate);
    m.insert(*gates::S, perform_s_gate);
    m.insert(*gates::SDG, perform_sdg_gate);
    m.insert(*gates::SWAP, perform_swap_gate);
    m.insert(*gates::CX, perform_cx_gate);
    m
});

#[cfg(test)]
mod test {
    use crate::state::{gates::{perform_t_gate, perform_x_gate}, StateVec};

    #[test]
    fn test() {
        let mut state = StateVec::from_random(&mut rand::rng(), 1);
        let mut state2 = state.clone();

        perform_x_gate(&mut state, &[0]);
        perform_t_gate(&mut state2, &[0]);
        perform_x_gate(&mut state2, &[0]);
        perform_t_gate(&mut state2, &[0]);
        println!("{state:?} {state2:?}");
        state.normalize_arg();
        state2.normalize_arg();
        assert_eq!(state, state2);
        assert_eq!(state.hash_value(), state2.hash_value());

        state.normalize();
        state2.normalize();
        assert_eq!(state, state2);
        assert_eq!(state.hash_value(), state2.hash_value());
    }
}