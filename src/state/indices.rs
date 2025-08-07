use std::array;

fn insert_bit(input: usize, at: usize, value: usize) -> usize {
    let l = input & ((1 << at) - 1);
    let h = input & !((1 << at) - 1);
    l | (h << 1) | (value << at)
}

fn qubit_matrix_indices<const M: usize, const N: usize>(
    nqubit: usize,
    qubits: [u8; N],
) -> impl Iterator<Item = [usize; M]> {
    let mut sorted: [(usize, usize); N] = array::from_fn(|i| (qubits[i] as usize, N - 1 - i as usize));
    sorted.sort_by_key(|x| x.0);
    (0usize..1 << (nqubit - N)).into_iter().map(move |i| {
        array::from_fn(|k| {
            sorted
                .iter()
                .fold(i, |i, (a, m)| insert_bit(i, *a, (k >> m) & 1))
        })
        .into()
    })
}

pub fn qubit_matrix_indices1(nqubit: usize, qubits: [u8; 1]) -> impl Iterator<Item = [usize; 2]> {
    qubit_matrix_indices::<2, 1>(nqubit, qubits)
}

pub fn qubit_matrix_indices2(nqubit: usize, qubits: [u8; 2]) -> impl Iterator<Item = [usize; 4]> {
    qubit_matrix_indices::<4, 2>(nqubit, qubits)
}

pub fn qubit_matrix_indices3(nqubit: usize, qubits: [u8; 3]) -> impl Iterator<Item = [usize; 8]> {
    qubit_matrix_indices::<8, 3>(nqubit, qubits)
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;

    use crate::state::indices::qubit_matrix_indices2;

    #[test]
    fn test1() {
        assert_eq!(
            format!("{:?}", qubit_matrix_indices2(4, [0, 1]).collect_vec()),
            "[[0, 2, 1, 3], [4, 6, 5, 7], [8, 10, 9, 11], [12, 14, 13, 15]]"
        );
        // assert_eq!(
        //     format!("{:?}", qubit_matrix_iter3(4, [1, 0, 2]).collect_vec()),
        //     "[0, 100, 1, 101, 10, 110, 11, 111] [1000, 1100, 1001, 1101, 1010, 1110, 1011, 1111] "
        // );
    }
}
