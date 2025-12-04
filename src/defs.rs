use std::hash::{DefaultHasher, Hash, Hasher};

use nalgebra::DMatrix;
use num_complex::{Complex, Complex64};

// pub type Qreal = f64;
// pub type Qcplx = Complex64;


pub const PERCISION_LEVEL : usize = 10;

pub const F64_PERCISION_EPSILON: f64 = 1f64 / ((1u64 << PERCISION_LEVEL) as f64);

pub fn f64_percision_repr(val: f64) -> u64 {
    if val.is_nan() || val.is_infinite() {
        val.to_bits()
    } else {
        let representative_val = (val / F64_PERCISION_EPSILON).round() * F64_PERCISION_EPSILON;
        representative_val.to_bits()
    }
}

pub fn f64_to_fixpoint(val: f64) -> i64 {
    assert!(val.is_finite(), "Value must be finite");
    (val * (1u64 << PERCISION_LEVEL) as f64) as i64
}

pub fn cmplx64_to_fixpoint(val: Complex64) -> Complex<i64> {
    Complex::new(
        f64_to_fixpoint(val.re),
        f64_to_fixpoint(val.im),
    )
}

pub fn cmplx64mat_to_fixpoint(val: &DMatrix<Complex64>) -> DMatrix<Complex<i64>> {
    val.map(cmplx64_to_fixpoint)
}
