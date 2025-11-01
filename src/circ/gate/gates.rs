use std::{cell::LazyCell, sync::LazyLock};

use nalgebra::DMatrix;
use numpy::Complex64;

use crate::circ::{Gate16, GateData};

pub fn initial_gates() -> Vec<GateData> {
    vec![
        GateData::new(
            "i".to_string(),
            vec![],
            DMatrix::from_row_slice(2, 2, &[
                Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0), Complex64::new(1.0, 0.0),
            ])
        ),
        GateData::new(
            "h".to_string(),
            vec![],
            DMatrix::from_row_slice(2, 2, &[
                Complex64::new(1.0 / 2f64.sqrt(), 0.0), Complex64::new(1.0 / 2f64.sqrt(), 0.0),
                Complex64::new(1.0 / 2f64.sqrt(), 0.0), Complex64::new(-1.0 / 2f64.sqrt(), 0.0),
            ])
        ),
        GateData::new(
            "x".to_string(),
            vec![],
            DMatrix::from_row_slice(2, 2, &[
                Complex64::new(0.0, 0.0), Complex64::new(1.0, 0.0),
                Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0),
            ])
        ),
        GateData::new(
            "z".to_string(),
            vec![],
            DMatrix::from_row_slice(2, 2, &[
                Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0), Complex64::new(-1.0, 0.0),
            ])
        ),
        GateData::new(
            "y".to_string(),
            vec![],
            DMatrix::from_row_slice(2, 2, &[
                Complex64::new(0.0, 0.0), Complex64::new(0.0, -1.0),
                Complex64::new(0.0, 1.0), Complex64::new(0.0, 0.0),
            ])
        ),
        GateData::new(
            "s".to_string(),
            vec![],
            DMatrix::from_row_slice(2, 2, &[
                Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0), Complex64::new(0.0, 1.0),
            ])
        ),
        GateData::new(
            "sdg".to_string(),
            vec![],
            DMatrix::from_row_slice(2, 2, &[
                Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0), Complex64::new(0.0, -1.0),
            ])
        ),
        GateData::new(
            "t".to_string(),
            vec![],
            DMatrix::from_row_slice(2, 2, &[
                Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0), Complex64::new(0.7071067811865476, 0.7071067811865475),
            ])
        ),
        GateData::new(
            "tdg".to_string(),
            vec![],
            DMatrix::from_row_slice(2, 2, &[
                Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0), Complex64::new(0.7071067811865476, -0.7071067811865475),
            ])
        ),
        GateData::new(
            "cx".to_string(),
            vec![],
            DMatrix::from_row_slice(4, 4, &[
                Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0), Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(1.0, 0.0),
                Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0),
            ])
        ),
        GateData::new(
            "cz".to_string(),
            vec![],
            DMatrix::from_row_slice(4, 4, &[
                Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0), Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(-1.0, 0.0),
            ])
        ),
        GateData::new(
            "cy".to_string(),
            vec![],
            DMatrix::from_row_slice(4, 4, &[
                Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0), Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(0.0, -1.0),
                Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(0.0, 1.0), Complex64::new(0.0, 0.0),
            ])
        ),
        GateData::new(
            "cs".to_string(),
            vec![],
            DMatrix::from_row_slice(4, 4, &[
                Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0), Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(0.0, 1.0),
            ])
        ),
        GateData::new(
            "csdg".to_string(),
            vec![],
            DMatrix::from_row_slice(4, 4, &[
                Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0), Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(0.0, -1.0),
            ])
        ),
        GateData::new(
            "swap".to_string(),
            vec![],
            DMatrix::from_row_slice(4, 4, &[
                Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0), Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(1.0, 0.0),
            ])
        )
    ]
}


pub static I: LazyLock<Gate16> = LazyLock::new(|| Gate16::from_name("i").unwrap());
pub static H: LazyLock<Gate16> = LazyLock::new(|| Gate16::from_name("h").unwrap());
pub static X: LazyLock<Gate16> = LazyLock::new(|| Gate16::from_name("x").unwrap());
pub static Z: LazyLock<Gate16> = LazyLock::new(|| Gate16::from_name("z").unwrap());
pub static Y: LazyLock<Gate16> = LazyLock::new(|| Gate16::from_name("y").unwrap());
pub static T: LazyLock<Gate16> = LazyLock::new(|| Gate16::from_name("t").unwrap());
pub static TDG: LazyLock<Gate16> = LazyLock::new(|| Gate16::from_name("tdg").unwrap());
pub static S: LazyLock<Gate16> = LazyLock::new(|| Gate16::from_name("s").unwrap());
pub static SDG: LazyLock<Gate16> = LazyLock::new(|| Gate16::from_name("sdg").unwrap());
pub static CX: LazyLock<Gate16> = LazyLock::new(|| Gate16::from_name("cx").unwrap());
pub static CZ : LazyLock<Gate16> = LazyLock::new(|| Gate16::from_name("cz").unwrap());
pub static CS : LazyLock<Gate16> = LazyLock::new(|| Gate16::from_name("cs").unwrap());
pub static CSDG : LazyLock<Gate16> = LazyLock::new(|| Gate16::from_name("csdg").unwrap());
pub static CY: LazyLock<Gate16> = LazyLock::new(|| Gate16::from_name("cy").unwrap());
pub static SWAP : LazyLock<Gate16> = LazyLock::new(|| Gate16::from_name("swap").unwrap());