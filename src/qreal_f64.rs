use std::hash::{Hash, Hasher};
use num_traits::Num;

use num_complex::Complex;

use crate::utils::parse_usize;

#[derive(Clone, Copy, derive_more::Debug, derive_more::Display, derive_more::From, derive_more::Into)]
#[debug("{_0:?}")]
#[display("{_0}")]
pub struct Qreal(f64);

impl PartialEq for Qreal {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        self.percision_repr() == other.percision_repr()
    }
}

impl Eq for Qreal {}
impl Hash for Qreal {
    #[inline(always)]
    fn hash<H: Hasher>(&self, state: &mut H) {
        let repr = self.percision_repr();
        repr.hash(state);
    }
}

impl PartialOrd for Qreal {
    #[inline(always)]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.percision_repr().cmp(&other.percision_repr()))
    }
}

impl Ord for Qreal {
    #[inline(always)]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.percision_repr().cmp(&other.percision_repr())
    }
}

impl std::ops::Add for Qreal {
    type Output = Self;
    #[inline(always)]
    fn add(self, other: Self) -> Self {
        Qreal(self.0 + other.0)
    }
}

impl std::ops::Sub for Qreal {
    type Output = Self;
    #[inline(always)]
    fn sub(self, other: Self) -> Self {
        Qreal(self.0 - other.0)
    }
}

impl std::ops::AddAssign for Qreal {
    #[inline(always)]
    fn add_assign(&mut self, other: Self) {
        self.0 += other.0;
    }
}

impl std::ops::SubAssign for Qreal {
    #[inline(always)]
    fn sub_assign(&mut self, other: Self) {
        self.0 -= other.0;
    }
}

impl std::ops::Neg for Qreal {
    type Output = Self;
    #[inline(always)]
    fn neg(self) -> Self {
        Qreal(-self.0)
    }
}

impl std::ops::Mul for Qreal {
    type Output = Self;
    #[inline(always)]
    fn mul(self, other: Self) -> Self {
        Qreal(self.0 * other.0)
    }
}

impl std::ops::MulAssign for Qreal {
    #[inline(always)]
    fn mul_assign(&mut self, other: Self) {
        self.0 *= other.0;
    }
}

impl std::ops::Div for Qreal {
    type Output = Self;

    #[inline(always)]
    fn div(self, other: Self) -> Self {
        Qreal(self.0 / other.0)
    }
}

impl std::ops::DivAssign for Qreal {
    #[inline(always)]
    fn div_assign(&mut self, other: Self) {
        self.0 /= other.0;
    }
}

impl std::ops::Rem for Qreal {
    type Output = Self;
    #[inline(always)]
    fn rem(self, other: Self) -> Self {
        Qreal(self.0 % other.0)
    }
}

impl std::ops::RemAssign for Qreal {
    #[inline(always)]
    fn rem_assign(&mut self, other: Self) {
        self.0 %= other.0;
    }
}

impl num_traits::Zero for Qreal {
    #[inline(always)]
    fn zero() -> Self {
        Qreal(0.0)
    }
    #[inline(always)]
    fn is_zero(&self) -> bool {
        self.0 == 0.0
    }
}

impl num_traits::One for Qreal {
    #[inline(always)]
    fn one() -> Self {
        Qreal(1.0)
    }
}

impl num_traits::Num for Qreal {
    type FromStrRadixErr = num_traits::ParseFloatError;

    fn from_str_radix(str: &str, radix: u32) -> Result<Self, Self::FromStrRadixErr> {
        let val = f64::from_str_radix(str, radix)?;
        Ok(Qreal(val))
    }
}

impl Qreal {
    pub const PERCISION_LEVEL : usize = parse_usize(if let Some(a) = option_env!("PERCISION_LEVEL") { a } else { "24" });
    const PERCISION_EPSILON: Qreal = Qreal(1f64 / ((1u64 << Self::PERCISION_LEVEL) as f64));
    pub const FRAC_PI_4 : Self = Self(std::f64::consts::FRAC_PI_4);
    pub const FRAC_1_SQRT_2 : Self = Self(std::f64::consts::FRAC_1_SQRT_2);

    pub const IM_UNIT : Qcplx = Complex::new(Qreal(0.0), Qreal(1.0));
    pub const EXP_I_FRAC_PI_4 : Qcplx = Complex::new(Self::FRAC_1_SQRT_2, Self::FRAC_1_SQRT_2);
    #[inline(always)]
    pub fn frac(nom: i64, denom: i64) -> Qreal {
        Qreal(nom as f64 / denom as f64)
    }
    #[inline(always)]
    pub fn percision_repr(self: Qreal) -> u64 {
        (self.0 * ((1u64 << Self::PERCISION_LEVEL) as f64)).round() as i64 as u64
    }
    #[inline(always)]
    pub fn near_zero(self: Qreal) -> bool {
        self.0.abs() < Self::PERCISION_EPSILON.0
    }
    #[inline(always)]
    pub fn sqrt(self) -> Qreal {
        Qreal(self.0.sqrt())
    }
    #[inline(always)]
    pub fn expipi(self) -> Qcplx {
        let angle = self.0 * std::f64::consts::PI;
        Qcplx::new(Qreal(angle.cos()), Qreal(angle.sin()))
    }
}
pub type Qcplx = Complex<Qreal>;

impl<'py> pyo3::IntoPyObject<'py> for Qreal {
    fn into_pyobject(self, py: pyo3::Python<'py>) -> Result<Self::Output, Self::Error> {
        self.0.into_pyobject(py)
    }
    
    type Target = <f64 as pyo3::IntoPyObject<'py>>::Target;
    
    type Output = <f64 as pyo3::IntoPyObject<'py>>::Output;
    
    type Error =  <f64 as pyo3::IntoPyObject<'py>>::Error;
}

impl<'py> pyo3::FromPyObject<'py> for Qreal {
    fn extract_bound(ob: &pyo3::Bound<'py, pyo3::PyAny>) -> Result<Self, pyo3::PyErr> {
        let val: f64 = pyo3::FromPyObject::extract_bound(ob)?;
        Ok(Qreal(val))
    }
}


