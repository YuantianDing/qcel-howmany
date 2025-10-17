use nalgebra::{ArrayStorage, ComplexField, DMatrix, DVector, Matrix2, Matrix4, Vector2, Vector4, VectorN};
use num_complex::Complex64;
use pyo3::Bound;
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pymethods};
use rand::{rngs::ThreadRng, Rng};
use smallvec::SmallVec;
use std::{
    array, cell::{RefCell, UnsafeCell}, cmp::Ordering, collections::BTreeSet, hash::{Hash, Hasher}
};

use crate::{circ::{Gate, Instr}, defs::{cmplx64_to_fixpoint, f64_percision_repr, F64_PERCISION_EPSILON}, groups::permutation::Permut32, state::indices::qubit_matrix_indices2};

#[gen_stub_pyclass]
#[pyo3::pyclass(eq, str)]
#[derive(Clone)]
pub struct StateVec {
    pub re: Box<[f64]>,
    pub im: Box<[f64]>,
}

impl Hash for StateVec {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for &val in self.re.iter() {
            f64_percision_repr(val).hash(state);
        }
        for &val in self.im.iter() {
            f64_percision_repr(val).hash(state);
        }
    }
}

impl std::fmt::Display for StateVec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[")?;
        for (i, a) in self.re.iter().zip(self.im.iter()).enumerate() {
            if i > 0 { write!(f, ", ")?; }
            if a.1.abs() < F64_PERCISION_EPSILON {
                write!(f, "{:.4}", a.0)?;
            } else if a.0.abs() < F64_PERCISION_EPSILON {
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
            if a.1.abs() < F64_PERCISION_EPSILON {
                write!(f, "{:?}", a.0)?;
            } else if a.0.abs() < F64_PERCISION_EPSILON {
                write!(f, "{:?}j", a.1)?;
            } else {
                write!(f, "{:?}{:+?}j", a.0, a.1)?;
            }
        }
        write!(f, "]")
    }
}

impl PartialEq for StateVec {
    fn eq(&self, other: &Self) -> bool {
        // StateVecs are considered equal if their real and imaginary components
        // are element-wise equal according to the precision logic.
        if self.re.len() != other.re.len() || self.im.len() != other.im.len() {
            return false;
        }

        for i in 0..self.re.len() {
            if f64_percision_repr(self.re[i]) != f64_percision_repr(other.re[i]) {
                return false;
            }
        }

        for i in 0..self.im.len() {
            if f64_percision_repr(self.im[i]) != f64_percision_repr(other.im[i]) {
                return false;
            }
        }

        true
    }
}

impl Eq for StateVec {}

impl StateVec {
    pub fn new(re: Vec<f64>, im: Vec<f64>) -> Self {
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
        assert!(size > 0, "State vector size must be greater than 0");
        Self {
            re: vec![0.0; size].into_boxed_slice(),
            im: vec![0.0; size].into_boxed_slice(),
        }
    }
    pub fn get_permutation(&self) -> Permut32 {
        Permut32::from_order(self.nqubits() as u8, |a, b| self.compare_qubits(a, b))
    }
    pub fn get_permutation_with_eq(&self) -> (Permut32, u8) {
        let mut eq_mask = 0u8;
        (Permut32::from_order(self.nqubits() as u8, |a, b| {
            let res = self.compare_qubits(a, b);
            if res == Ordering::Equal {
                eq_mask |= 1 << std::cmp::max(a, b);
            }
            res
        }), eq_mask)
    }
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
            if cmplx64_to_fixpoint(vec[1]) != cmplx64_to_fixpoint(vec[2]) { return false; }
        }
        true
    }
    pub fn compare_qubits(&self, a: u8, b: u8) -> Ordering {
        let value = self.at(1 << a) - self.at(1 << b);
        if value.modulus() > F64_PERCISION_EPSILON {
            return if value.re > 0.0 { Ordering::Greater } else { Ordering::Less };
        }
        let mask = (1 << self.nqubits()) - 1;
        let value = self.at(mask & !(1 << a)) - self.at(mask & !(1 << b));
        if value.modulus() > F64_PERCISION_EPSILON {
            return if value.re > 0.0 { Ordering::Greater } else { Ordering::Less };
        }

        let mut aset = SmallVec::<[(i64,i64); 8]>::new();
        let mut bset = SmallVec::<[(i64,i64); 8]>::new();
        for i in 0u8..(self.nqubits() as u8) {
            if i != a {
                let v = cmplx64_to_fixpoint(self.at((1 << a) | (1 << i)));
                aset.push((v.re, v.im));
            }
            if i != b {
                let v = cmplx64_to_fixpoint(self.at((1 << b) | (1 << i)));
                bset.push((v.re, v.im));
            }
        }
        aset.sort();
        bset.sort();

        let result = aset.cmp(&bset);
        if result.is_ne() {
            return result;
        }

        let mut aset = SmallVec::<[(i64,i64); 8]>::new();
        let mut bset = SmallVec::<[(i64,i64); 8]>::new();
        for i in 0u8..(self.nqubits() as u8) {
            if i != a {
                let v = cmplx64_to_fixpoint(self.at(mask & !(1 << a) & !(1 << i)));
                aset.push((v.re, v.im));
            }
            if i != b {
                let v = cmplx64_to_fixpoint(self.at(mask & !(1 << b) & !(1 << i)));
                bset.push((v.re, v.im));
            }
        }
        aset.sort();
        bset.sort();
        let result = aset.cmp(&bset);
        if result.is_ne() {
            return result;
        }

        return Ordering::Equal;
    }
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
        Self::new(re, im)
    }

    pub fn hash_value(&self) -> u64 {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }

    #[staticmethod]
    pub fn random(num_qubits: u32) -> Self {
        Self::from_random_symmetric(&mut rand::rng(), num_qubits)
    }
    
    #[staticmethod]
    pub fn random_symmetric(num_qubits: u32) -> Self {
        Self::from_random_symmetric(&mut rand::rng(), num_qubits)
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

        let mut norm_sq = 0.0;
        let mut first_non_zero_index = None;
        for i in 0..self.len() {
            let norm = self.re[i] * self.re[i] + self.im[i] * self.im[i];
            norm_sq += norm;
            if norm > F64_PERCISION_EPSILON {
                if first_non_zero_index.is_none() {
                    first_non_zero_index = Some(i);
                }
            }
        }

        if let Some(first_index) = first_non_zero_index {
            let mut unit = num_complex::c64(self.re[first_index], self.im[first_index]);
            unit = unit.norm() / (unit * norm_sq.sqrt());
            assert!(unit.is_finite() && unit.modulus() >= F64_PERCISION_EPSILON, "Normalization unit is effectively zero.");
            for i in 0..self.len() {
                let r = num_complex::c64(self.re[i], self.im[i]) * unit;
                self.re[i] = r.re;
                self.im[i] = r.im;
            }
        } else {
            panic!("State vector is effectively all zeros. Cannot normalize.");
        }
    }
    pub fn normalize_arg(&mut self) {
        let mut unit = num_complex::c64(self.re[0], self.im[0]);
        if unit.modulus() < F64_PERCISION_EPSILON {
            unit = num_complex::c64(*self.re.last().unwrap(), *self.im.last().unwrap());
            if unit.modulus() < F64_PERCISION_EPSILON {
                self.normalize(); return;
            }
        }
        unit = unit.norm() / unit;
        for i in 0..self.len() {
            let r = num_complex::c64(self.re[i], self.im[i]) * unit;
            self.re[i] = r.re;
            self.im[i] = r.im;
        }
        self.im[0] = 0.0;
    }

    pub fn __getitem__<'a>(&self, index: usize) -> Complex64 {
        self.at(index)
    }

    pub fn __setitem__(&mut self, index: usize, value: Complex64) {
        self.set(index, value);
    }

    pub fn __imul__(slf: &Bound<'_, Self>, other: Instr) {
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
        let mut norm_sq = 0.0;
        for i in 0..self.len() {
            if !self.re[i].is_finite() || !self.im[i].is_finite() {
                return false;
            }
            norm_sq += self.re[i] * self.re[i] + self.im[i] * self.im[i];
        }
        (norm_sq - 1.0).abs() < F64_PERCISION_EPSILON
    }
}
type Matrix8 = nalgebra::SMatrix<Complex64, 8, 8>;
type Vector8 = nalgebra::SVector<Complex64, 8>;

impl StateVec {
    pub fn at(&self, index: usize) -> Complex64 {
        num_complex::c64(self.re[index], self.im[index])
    }

    pub fn set(&mut self, index: usize, value: Complex64) {
        self.re[index] = value.re;
        self.im[index] = value.im;
    }
    pub fn access<const N: usize>(&self, indices: [usize; N]) -> [Complex64; N] {
        array::from_fn(|i| {
            let index = indices[i];
            num_complex::c64(self.re[index], self.im[index])
        })
    }
    pub fn update<const N: usize>(&mut self, indices: [usize; N], values: [Complex64; N]) {
        for i in 0..N {
            let index = indices[i];
            let value = values[i];
            self.re[index] = value.re;
            self.im[index] = value.im;
        }
    }

    pub fn apply(&mut self, qubits: &[u8], gate: Gate) {
        if let Some(f) = gates::GATE_FUNCS.get(&gate) {
            f(self, qubits);
        } else {
            gate.matrix(|matrix| {
                self.multiply(qubits, &matrix);
            });
        }
    }
    pub fn multiply(&mut self, qubits: &[u8], matrix: &DMatrix<Complex64>) {
        match qubits {
            [i] => {
                assert!(matrix.nrows() == 2 && matrix.ncols() == 2);
                let matrix: Matrix2<Complex64> = matrix.fixed_view::<2,2>(0, 0).into();
                for a in indices::qubit_matrix_indices1(self.nqubits(), [*i]) {
                    let mut vec = Vector2::from(self.access(a));
                    vec = matrix * vec;
                    self.update(a, vec.data.0[0]);
                }
            }
            [i, j] => {
                assert!(matrix.nrows() == 4 && matrix.ncols() == 4);
                let matrix: Matrix4<Complex64> = matrix.fixed_view::<4,4>(0, 0).into();
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
    pub fn from_random(rng: &mut ThreadRng, num_qubits: u32) -> Self {
        let size = 1_usize
            .checked_shl(num_qubits)
            .expect("Number of qubits too large, resulting in overflow for state vector size.");

        // Generate random f64 values, typically in the range [0.0, 1.0).
        let re_vec: Vec<f64> = (0..size).map(|_| rng.random::<f64>()).collect();
        let im_vec: Vec<f64> = (0..size).map(|_| rng.random::<f64>()).collect();

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

    pub fn from_random_symmetric(rng: &mut ThreadRng, num_qubits: u32) -> Self {
        let size = 1_usize
            .checked_shl(num_qubits)
            .expect("Number of qubits too large, resulting in overflow for state vector size.");

        let re_values = (0..num_qubits + 1)
            .map(|_| rng.random::<f64>())
            .collect::<Vec<_>>();
        let im_values = (0..num_qubits + 1)
            .map(|_| rng.random::<f64>())
            .collect::<Vec<_>>();

        let re_vec: Vec<f64> = (0..size)
            .map(|i| re_values[i.count_ones() as usize])
            .collect::<Vec<_>>();
        let im_vec: Vec<f64> = (0..size)
            .map(|i| im_values[i.count_ones() as usize])
            .collect::<Vec<_>>();
        let mut state_vec = Self {
            re: re_vec.into_boxed_slice(),
            im: im_vec.into_boxed_slice(),
        };

        state_vec.normalize();
        state_vec
    }
    pub fn approx_eq(&self, other: &Self, epsilon: f64) -> bool {
        if self.len() != other.len() {
            return false;
        }
        
        (0..self.len()).map(|i| {
            (self.re[i] - other.re[i]).abs() + (self.im[i] - other.im[i]).abs() 
        }).sum::<f64>() < epsilon
    }
}


pub mod indices;
mod gates;
