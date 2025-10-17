use nalgebra::DMatrix;
use numpy::Complex64;

use crate::circ::Gate;



lazy_static::lazy_static!(
    pub static ref H: Gate = {
        Gate::new(
            "h".to_string(),
            vec![],
            DMatrix::from_row_slice(2, 2, &[
                Complex64::new(1.0 / 2f64.sqrt(), 0.0), Complex64::new(1.0 / 2f64.sqrt(), 0.0),
                Complex64::new(1.0 / 2f64.sqrt(), 0.0), Complex64::new(-1.0 / 2f64.sqrt(), 0.0),
            ])
        )
    };

    pub static ref X: Gate = {
        Gate::new(
            "x".to_string(),
            vec![],
            DMatrix::from_row_slice(2, 2, &[
                Complex64::new(0.0, 0.0), Complex64::new(1.0, 0.0),
                Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0),
            ])
        )
    };

    pub static ref Z: Gate = {
        Gate::new(
            "z".to_string(),
            vec![],
            DMatrix::from_row_slice(2, 2, &[
                Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0), Complex64::new(-1.0, 0.0),
            ])
        )
    };

    pub static ref T: Gate = {
        Gate::new(
            "t".to_string(),
            vec![],
            DMatrix::from_row_slice(2, 2, &[
                Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0), Complex64::new(0.7071067811865476, 0.7071067811865475),
            ])
        )
    };

    pub static ref TDG: Gate = {
        Gate::new(
            "tdg".to_string(),
            vec![],
            DMatrix::from_row_slice(2, 2, &[
                Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0), Complex64::new(0.7071067811865476, -0.7071067811865475),
            ])
        )
    };

    pub static ref S: Gate = {
        Gate::new(
            "s".to_string(),
            vec![],
            DMatrix::from_row_slice(2, 2, &[
                Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0), Complex64::new(0.0, 1.0),
            ])
        )
    };
    pub static ref SDG: Gate = {
        Gate::new(
            "sdg".to_string(),
            vec![],
            DMatrix::from_row_slice(2, 2, &[
                Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0), Complex64::new(0.0, -1.0),
            ])
        )
    };

    pub static ref CX: Gate = {
        Gate::new(
            "cx".to_string(),
            vec![],
            DMatrix::from_row_slice(4, 4, &[
                Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0), Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(1.0, 0.0),
                Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0),
            ])
        )
    };

    pub static ref SWAP : Gate = {
        Gate::new(
            "swap".to_string(),
            vec![],
            DMatrix::from_row_slice(4, 4, &[
                Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0), Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(1.0, 0.0),
            ])
        )
    };
);