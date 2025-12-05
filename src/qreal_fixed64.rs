use std::{hash::{DefaultHasher, Hash, Hasher}, i64};

use fixed::types::extra::U63;
use fixed::traits::Fixed;
use nalgebra::DMatrix;
use num_complex::{Complex, Complex32};
use num_traits::Num;

pub type Qreal = fixed::types::I2F62;
pub const FRAC_PI_4 : Qreal = Qreal::FRAC_PI_4;
pub const FRAC_1_SQRT_2 : Qreal = Qreal::FRAC_1_SQRT_2;
pub type Qcplx = Complex<Qreal>;
pub const IM_UNIT : Qcplx = Complex::new(Qreal::ZERO, Qreal::ONE);
pub const EXP_I_FRAC_PI_4 : Qcplx = Complex::new(FRAC_1_SQRT_2, FRAC_1_SQRT_2);
pub const EXP_I_FRAC_PI_3 : Qcplx = Complex::new(
    Qreal::from_bits(Qreal::ONE.to_bits() >> 1),
    Qreal::from_bits(Qreal::SQRT_3.to_bits() >> 1)
);


pub const PERCISION_LEVEL : usize = 10;

pub const PERCISION_EPSILON: Qreal = Qreal::from_bits(i64::MAX >> PERCISION_LEVEL);

pub fn qreal_percision_repr(val: Qreal) -> u64 {
    let representative_val = (val / PERCISION_EPSILON).round() * PERCISION_EPSILON;
    representative_val.to_bits() as u64
}

pub fn qreal_to_fixpoint(val: Qreal) -> i64 {
    val.to_bits()
}

pub fn qcplx_to_fixpoint(val: Qcplx) -> Complex<i64> {
    Complex::new(
        qreal_to_fixpoint(val.re),
        qreal_to_fixpoint(val.im),
    )
}

pub fn qcplxmat_to_fixpoint(val: &DMatrix<Qcplx>) -> DMatrix<Complex<i64>> {
    val.map(qcplx_to_fixpoint)
}

// A function that requires a type to implement MyTrait
fn assert_implements_my_trait<T: Num>() {}

fn test_my_struct_implements_my_trait() {
    // This line will only compile if MyStruct implements MyTrait
    assert_implements_my_trait::<Qreal>();
}