use nalgebra::{DMatrix, Matrix2, Matrix4, Vector2, Vector4};
use numpy::Complex64;
use crate::{Qreal, Qcplx};
use pyo3::Bound;
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pymethods};
use rand::{Rng, SeedableRng, rngs::StdRng};
use core::panic;
use std::{
    array, cell::RefCell, hash::{Hash, Hasher}
};

use crate::{circ::{Gate16, Instr32}, groups::permutation::Permut32, state::indices::qubit_matrix_indices2};
pub mod order_info;
#[gen_stub_pyclass]
#[pyo3::pyclass(eq, str)]
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StateVec {
    pub re: Box<[Qreal]>,
    pub im: Box<[Qreal]>,
}

// impl Hash for StateVec {
//     fn hash<H: Hasher>(&self, state: &mut H) {
//         for &val in self.re.iter() {
//             val.percision_repr().hash(state);
//         }
//         for &val in self.im.iter() {
//             val.percision_repr().hash(state);
//         }
//     }
// }
// impl PartialOrd for StateVec {
//     fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
//         Some(self.cmp(other))
//     }
// }
// impl Ord for StateVec {
//     fn cmp(&self, other: &Self) -> Ordering {
//         for (a, b) in self.re.iter().zip(other.re.iter()) {
//             let ord = a.percision_repr().cmp(&b.percision_repr());
//             if ord != Ordering::Equal {
//                 return ord;
//             }
//         }
//         for (a, b) in self.im.iter().zip(other.im.iter()) {
//             let ord = a.percision_repr().cmp(&b.percision_repr());
//             if ord != Ordering::Equal {
//                 return ord;
//             }
//         }
//         Ordering::Equal
//     }
// }

impl std::fmt::Display for StateVec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[")?;
        for (i, a) in self.re.iter().zip(self.im.iter()).enumerate() {
            if i > 0 { write!(f, ", ")?; }
            if a.1.near_zero() {
                write!(f, "{:.4}", a.0)?;
            } else if a.0.near_zero() {
                write!(f, "{:.4}j", a.1)?;
            } else {
                write!(f, "{:.4}{:+.4}j", a.0, a.1)?;
            }
        }
        write!(f, "]")
    }
}
impl std::fmt::Debug for StateVec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[")?;
        for (i, a) in self.re.iter().zip(self.im.iter()).enumerate() {
            if i > 0 { write!(f, ", ")?; }
            if a.1.near_zero() {
                write!(f, "{:?}", a.0)?;
            } else if a.0.near_zero() {
                write!(f, "{:?}j", a.1)?;
            } else {
                write!(f, "{:?}{:+?}j", a.0, a.1)?;
            }
        }
        write!(f, "]")
    }
}

// impl PartialEq for StateVec {
//     fn eq(&self, other: &Self) -> bool {
//         // StateVecs are considered equal if their real and imaginary components
//         // are element-wise equal according to the precision logic.
//         if self.re.len() != other.re.len() || self.im.len() != other.im.len() {
//             return false;
//         }

//         for i in 0..self.re.len() {
//             if self.re[i].percision_repr() != other.re[i].percision_repr() {
//                 return false;
//             }
//         }

//         for i in 0..self.im.len() {
//             if self.im[i].percision_repr() != other.im[i].percision_repr() {
//                 return false;
//             }
//         }

//         true
//     }
// }

// impl Eq for StateVec {}

impl StateVec {
    pub fn new(re: Vec<Qreal>, im: Vec<Qreal>) -> Self {
        assert_eq!(
            re.len(),
            im.len(),
            "Real and imaginary parts must have the same length"
        );
        assert!(re.len().is_power_of_two(), "Length must be a power of 2");
        Self {
            re: re.into_boxed_slice(),
            im: im.into_boxed_slice(),
        }
    }
    pub fn zeros(num_qubits: u32) -> Self {
        let size = 1_usize
            .checked_shl(num_qubits)
            .expect("Number of qubits too large, resulting in overflow for state vector size.");
        // assert!(size > 0, "State vector size must be greater than 0");
        Self {
            re: vec![0.0.into(); size].into_boxed_slice(),
            im: vec![0.0.into(); size].into_boxed_slice(),
        }
    }
    // pub fn get_permutation(&self) -> Permut32 {
    //     Permut32::from_order(self.nqubits() as u8, |a, b| self.compare_qubits(a, b))
    // }
    // pub fn get_permutation_with_eq(&self) -> (Permut32, u8) {
    //     let mut eq_mask = 0u8;
    //     (Permut32::from_order(self.nqubits() as u8, |a, b| {
    //         let res = self.compare_qubits(a, b);
    //         if res == Ordering::Equal {
    //             eq_mask |= 1 << std::cmp::max(a, b);
    //         }
    //         res
    //     }), eq_mask)
    // }
    pub fn apply_permutation(&mut self, permut: Permut32) {
        reserve_state_vec_cache(self.nqubits());
        STATE_VEC_CACHE.with(|cache| {
            let cache = &mut cache.borrow_mut()[self.nqubits()];

            for i in 0..self.re.len() {
                let permuted_index = permut.permut_bv(i as u8) as usize;
                cache.re[permuted_index] = self.re[i];
                cache.im[permuted_index] = self.im[i];
            }
            std::mem::swap(cache, self)
        })
    }
    pub fn qubit_equiv(&mut self, q1: u8, q2: u8) -> bool {
        for indices in qubit_matrix_indices2(self.nqubits(), [q1, q2]) {
            let vec = self.access(indices);
            if vec[1] != vec[2] { return false; }
        }
        true
    }
    pub fn get_permutation(&self) -> Permut32 {
        self.get_orderinfo().as_bits().0
    }
    pub fn get_orderinfo(&self) -> order_info::OrderInfo {
        let nqubits = self.nqubits() as u8;
        let mut oi = order_info::OrderInfo::new(nqubits as usize);
        
        oi.sort_eqclass_by_key(0, |&q| {
            let a = self.at(1 << q);
            let b = self.at(((1 << self.nqubits()) - 1) & !(1 << q));
            [a.re, a.im, b.re, b.im]
        });

        // FixPoint Computation
        // let mut ready_part = 0;
        // while let Some(idx) = oi.first_eqclass_after(ready_part) {
        //     let arr: Vec<_> = oi.get_eqclass(idx).iter().map(|&q| self.hash_value_of_qubit(q as u8, &oi)).collect();
        //     if !oi.sort_eqclass_by_array(idx, arr) {
        //         ready_part = idx + 1;
        //     } else {
        //         ready_part = 0;
        //     }
        // }

        oi
    }
    // fn hash_value_of_qubit(&self, q: u8, oi: &order_info::OrderInfo) -> u64 {
    //     let mut length = 2u32;
    //     let mut vec = Vec::with_capacity(oi.n_eqclasses());
    //     vec.extend(oi.eqclasses().map(|cls| {
    //         let result = cls.iter().map(|&p| if p != (q as usize) { 1u8 << p } else { 0u8 }).fold(0u8, |acc, x| acc | x);
    //         length *= result.count_ones() + 1;
    //         result
    //     }));
    //     let mut table = vec![0u64; length as usize];

    //     for i in 0..self.re.len() as u8 {
    //         let mut index = (i & (1 << q) != 0) as usize;
    //         for mask in &vec {
    //             index *= mask.count_ones() as usize + 1;
    //             index += (mask & i).count_ones() as usize;
    //         }
    //         table[index] = unsafe { table[index].unchecked_add(make_hash(self.at(i as usize))) };
    //     }
        
    //     let mut hasher = std::collections::hash_map::DefaultHasher::new();
    //     table.hash(&mut hasher);
    //     hasher.finish()
    // }
    // fn hash_value_of_qubit2(&self, q: u8, oi: &order_info::OrderInfo) -> u64 {
    //     let mut map = BTreeMap::<Vec<usize>, u64>::new();

    //     for i in 0..self.re.len() {
    //         let mut vec = Vec::with_capacity(oi.n_eqclasses() + 1);
    //         vec.push((i & (1 << q) != 0) as usize);
    //         vec.extend(oi.eqclasses().map(|cls|
    //             cls.iter().cloned()
    //                 .filter(|&p|
    //                     p != (q as usize) && i & (1 << p) != 0
    //                 ).count()));
    //         let h = map.entry(vec).or_insert(Default::default());
    //         *h = unsafe { h.unchecked_add(make_hash(self.at(i))) };
    //     }
        
    //     let mut hasher = std::collections::hash_map::DefaultHasher::new();
    //     map.hash(&mut hasher);
    //     hasher.finish()
    // }
    // pub fn compare_qubits(&self, a: u8, b: u8) -> Ordering {
    //     let value = self.at(1 << a) - self.at(1 << b);
    //     if !value.re.near_zero() || !value.im.near_zero() {
    //         return if value.re > 0.0 { Ordering::Greater } else { Ordering::Less };
    //     }
    //     let mask = (1 << self.nqubits()) - 1;
    //     let value = self.at(mask & !(1 << a)) - self.at(mask & !(1 << b));
    //     if !value.re.near_zero() || !value.im.near_zero() {
    //         return if value.re > 0.0 { Ordering::Greater } else { Ordering::Less };
    //     }

    //     let mut aset = SmallVec::<[(i64,i64); 8]>::new();
    //     let mut bset = SmallVec::<[(i64,i64); 8]>::new();
    //     for i in 0u8..(self.nqubits() as u8) {
    //         if i != a {
    //             let v = Qreal::complex_to_i64(self.at((1 << a) | (1 << i)));
    //             aset.push((v.re, v.im));
    //         }
    //         if i != b {
    //             let v = Qreal::complex_to_i64(self.at((1 << b) | (1 << i)));
    //             bset.push((v.re, v.im));
    //         }
    //     }
    //     aset.sort();
    //     bset.sort();

    //     let result = aset.cmp(&bset);
    //     if result.is_ne() {
    //         return result;
    //     }

    //     let mut aset = SmallVec::<[(i64,i64); 8]>::new();
    //     let mut bset = SmallVec::<[(i64,i64); 8]>::new();
    //     for i in 0u8..(self.nqubits() as u8) {
    //         if i != a {
    //             let v = Qreal::complex_to_i64(self.at(mask & !(1 << a) & !(1 << i)));
    //             aset.push((v.re, v.im));
    //         }
    //         if i != b {
    //             let v = Qreal::complex_to_i64(self.at(mask & !(1 << b) & !(1 << i)));
    //             bset.push((v.re, v.im));
    //         }
    //     }
    //     aset.sort();
    //     bset.sort();
    //     let result = aset.cmp(&bset);
    //     if result.is_ne() {
    //         return result;
    //     }

    //     return Ordering::Equal;
    // }
}

thread_local! {
    static STATE_VEC_CACHE: RefCell<Vec<StateVec>> = Vec::new().into();
}

pub fn reserve_state_vec_cache(size: usize) {
    STATE_VEC_CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        while cache.len() <= size {
            let len = cache.len();
            cache.push(StateVec::zeros(len as u32));
        }
    });
}



#[gen_stub_pymethods]
#[pyo3::pymethods]
impl StateVec {
    #[new]
    pub fn new_py(re: Vec<f64>, im: Vec<f64>) -> Self {
        Self::new(
            re.into_iter().map(|a| Qreal::from(a)).collect(), 
            im.into_iter().map(|a| Qreal::from(a)).collect()
        )
    }

    pub fn hash_value(&self) -> u64 {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }

    #[staticmethod]
    pub fn random(num_qubits: u32) -> Self {
        Self::from_random(&mut StdRng::from_os_rng(), num_qubits)
    }
    
    #[staticmethod]
    pub fn random_symmetric(num_qubits: u32) -> Self {
        Self::from_random_symmetric(&mut StdRng::from_os_rng(), num_qubits)
    }

    pub fn nqubits(&self) -> usize {
        self.re.len().trailing_zeros() as usize
    }
    pub fn len(&self) -> usize {
        self.re.len()
    }

    /// Normalizes the state vector in place.
    ///
    /// Normalization ensures two conditions:
    /// 1. Amplitude Normalization: The sum of the squares of the absolute values of
    ///    the complex amplitudes is 1 (i.e., `sum(|psi_i|^2) = 1`).
    /// 2. Phase Normalization: The first non-zero element of the state vector
    ///    is made real and positive.
    ///
    /// If the vector is effectively all zeros initially (all amplitudes are very close to zero),
    /// it is normalized to the |0...0> state (i.e., the first element becomes 1.0 + 0.0i,
    /// and all other elements become 0.0 + 0.0i).
    /// If the vector has zero length, this function does nothing.
    pub fn normalize(&mut self) {
        if self.len() == 0 {
            return;
        }

        let mut norm_sq = Qreal::from(0.0);
        for i in 0..self.len() {
            let norm = self.re[i] * self.re[i] + self.im[i] * self.im[i];
            norm_sq += norm;
        }
        let norm = norm_sq.sqrt();
        for i in 0..self.len() {
            self.re[i] /= norm;
            self.im[i] /= norm;
        }
        self.normalize_arg();
    }
    pub fn normalize_arg(&mut self) {
        let mut unit = Qcplx::new(self.re[0], self.im[0]);
        if unit.re.near_zero() && unit.im.near_zero() {
            unit = Qcplx::new(*self.re.last().unwrap(), *self.im.last().unwrap());
            if unit.re.near_zero() && unit.im.near_zero() {
                unit = (0..self.re.len()).map(|i| self.at(i)).sum();
                unit /= unit.norm_sqr().sqrt();
            }
        }
        unit = unit.inv() * unit.norm_sqr().sqrt();
        for i in 0..self.len() {
            let r = Qcplx::new(self.re[i], self.im[i]) * unit;
            self.re[i] = r.re;
            self.im[i] = r.im;
        }
        self.im[0] = 0.0.into();
    }

    pub fn __getitem__<'a>(&self, index: usize) -> Complex64 {
        let v = self.at(index);
        num_complex::c64(v.re, v.im)
    }

    pub fn __setitem__(&mut self, index: usize, value: Complex64) {
        self.set(index, num_complex::Complex::new(
            Qreal::from(value.re), 
            Qreal::from(value.im)));
    }

    pub fn __imul__(slf: &Bound<'_, Self>, other: Instr32) {
        let mut slf = slf.borrow_mut();
        slf.apply(&other.1, other.0);
    }
    pub fn clone(&self) -> Self {
        Self {
            re: self.re.clone(),
            im: self.im.clone(),
        }
    }
    pub fn check(&self) -> bool {
        let mut norm_sq = Qreal::from(0.0);
        for i in 0..self.len() {
            norm_sq += self.re[i] * self.re[i] + self.im[i] * self.im[i];
        }
        (norm_sq - 1.0.into()).near_zero()
    }
}
type Matrix8 = nalgebra::SMatrix<Qcplx, 8, 8>;
type Vector8 = nalgebra::SVector<Qcplx, 8>;

impl StateVec {
    pub fn at(&self, index: usize) -> Qcplx {
        Qcplx::new(self.re[index], self.im[index])
    }

    pub fn set(&mut self, index: usize, value: Qcplx) {
        self.re[index] = value.re;
        self.im[index] = value.im;
    }
    pub fn access<const N: usize>(&self, indices: [usize; N]) -> [Qcplx; N] {
        array::from_fn(|i| {
            let index = indices[i];
            Qcplx::new(self.re[index], self.im[index])
        })
    }
    pub fn update<const N: usize>(&mut self, indices: [usize; N], values: [Qcplx; N]) {
        for i in 0..N {
            let index = indices[i];
            let value = values[i];
            self.re[index] = value.re;
            self.im[index] = value.im;
        }
    }

    pub fn apply(&mut self, qubits: &[u8], gate: Gate16) {
        if let Some(f) = gates::GATE_FUNCS.get(&gate) {
            f(self, qubits);
        } else {
            // panic!("Unsupported gate: {:?}", gate);
            gate.matrix(|matrix| {
                self.multiply(qubits, &matrix);
            });
        }
    }
    pub fn multiply(&mut self, qubits: &[u8], matrix: &DMatrix<Qcplx>) {
        match qubits {
            [i] => {
                assert!(matrix.nrows() == 2 && matrix.ncols() == 2);
                let matrix: Matrix2<Qcplx> = matrix.fixed_view::<2,2>(0, 0).into();
                for a in indices::qubit_matrix_indices1(self.nqubits(), [*i]) {
                    let mut vec = Vector2::from(self.access(a));
                    vec = matrix * vec;
                    self.update(a, vec.data.0[0]);
                }
            }
            [i, j] => {
                assert!(matrix.nrows() == 4 && matrix.ncols() == 4);
                let matrix: Matrix4<Qcplx> = matrix.fixed_view::<4,4>(0, 0).into();
                for a in indices::qubit_matrix_indices2(self.nqubits(), [*i, *j]) {
                    let mut vec = Vector4::from(self.access(a));
                    vec = matrix * vec;
                    self.update(a, vec.data.0[0]);
                }
            }
            [i, j, k] => {
                assert!(matrix.nrows() == 8 && matrix.ncols() == 8);
                let matrix: Matrix8 = matrix.fixed_view::<8,8>(0, 0).into();
                for a in indices::qubit_matrix_indices3(self.nqubits(), [*i, *j, *k]) {
                    let mut vec = Vector8::from(self.access(a));
                    vec = matrix * vec;
                    self.update(a, vec.data.0[0]);
                }
            }
            _ => panic!("Unsupported number of qubits: {}", qubits.len()),
        }
    }

    /// Creates a new StateVec with random amplitudes for a given number of qubits.
    /// The state vector is normalized after generation.
    ///
    /// # Arguments
    /// * `num_qubits`: The number of qubits. The state vector will have 2^num_qubits elements.
    ///
    /// # Panics
    /// * Panics if `num_qubits` is too large, causing `2^num_qubits` to overflow `usize`.
    /// * Panics if memory allocation for the state vector fails (e.g., out of memory).
    pub fn from_random(rng: &mut StdRng, num_qubits: u32) -> Self {
        let size = 1_usize
            .checked_shl(num_qubits)
            .expect("Number of qubits too large, resulting in overflow for state vector size.");

        // Generate random Qreal values, typically in the range [0.0, 1.0).
        let re_vec: Vec<Qreal> = (0..size).map(|_| rng.random::<f64>().into()).collect();
        let im_vec: Vec<Qreal> = (0..size).map(|_| rng.random::<f64>().into()).collect();

        // Unsafe is required by AliasPtr::from_box_slice.
        // StateVec takes ownership of the boxed slices, ensuring the data remains valid
        // as long as the StateVec exists. AliasPtr will handle deallocation on drop.
        let mut state_vec = Self {
            re: re_vec.into_boxed_slice(),
            im: im_vec.into_boxed_slice(),
        };

        state_vec.normalize();
        state_vec
    }

    pub fn from_random_symmetric(rng: &mut StdRng, num_qubits: u32) -> Self {
        let size = 1_usize
            .checked_shl(num_qubits)
            .expect("Number of qubits too large, resulting in overflow for state vector size.");

        let re_values = (0..num_qubits + 1)
            .map(|_| Qreal::from(rng.random::<f64>()))
            .collect::<Vec<_>>();
        let im_values = (0..num_qubits + 1)
            .map(|_| Qreal::from(rng.random::<f64>()))
            .collect::<Vec<_>>();

        let re_vec: Vec<Qreal> = (0..size)
            .map(|i| re_values[i.count_ones() as usize])
            .collect::<Vec<_>>();
        let im_vec: Vec<Qreal> = (0..size)
            .map(|i| im_values[i.count_ones() as usize])
            .collect::<Vec<_>>();
        let mut state_vec = Self {
            re: re_vec.into_boxed_slice(),
            im: im_vec.into_boxed_slice(),
        };

        state_vec.normalize();
        state_vec
    }

    pub fn loose_eq(&self, other: &StateVec) -> bool {
        self.re.iter().zip(other.re.iter()).all(|(a, b)| {
            a.loose_eq(*b)
        }) &&
        self.im.iter().zip(other.im.iter()).all(|(a, b)| {
            a.loose_eq(*b)
        })
    }
}


pub mod indices;
mod gates;

// fn make_hash<T: Hash>(val: T) -> u64 {
//     let mut hasher = DefaultHasher::new();
//     val.hash(&mut hasher);
//     hasher.finish()
// }