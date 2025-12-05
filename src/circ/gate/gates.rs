use std::{cell::LazyCell, sync::LazyLock};

use nalgebra::DMatrix;

use crate::{Qcplx, Qreal, circ::{Gate16, GateData, Instr32}};

pub fn initial_gates() -> Vec<GateData> {
    vec![
        GateData::new(
            "i".to_string(),
            vec![],
            DMatrix::from_row_slice(2, 2, &[
                Qcplx::new(1.0.into(), 0.0.into()), Qcplx::new(0.0.into(), 0.0.into()),
                Qcplx::new(0.0.into(), 0.0.into()), Qcplx::new(1.0.into(), 0.0.into()),
            ])
        ),
        GateData::new(
            "h".to_string(),
            vec![],
            DMatrix::from_row_slice(2, 2, &[
                Qcplx::new(Qreal::FRAC_1_SQRT_2, 0.0.into()), Qcplx::new(Qreal::FRAC_1_SQRT_2, 0.0.into()),
                Qcplx::new(Qreal::FRAC_1_SQRT_2, 0.0.into()), Qcplx::new(-Qreal::FRAC_1_SQRT_2, 0.0.into()),
            ])
        ),
        GateData::new(
            "x".to_string(),
            vec![],
            DMatrix::from_row_slice(2, 2, &[
                Qcplx::new(0.0.into(), 0.0.into()), Qcplx::new(1.0.into(), 0.0.into()),
                Qcplx::new(1.0.into(), 0.0.into()), Qcplx::new(0.0.into(), 0.0.into()),
            ])
        ),
        GateData::new(
            "z".to_string(),
            vec![],
            DMatrix::from_row_slice(2, 2, &[
                Qcplx::new(1.0.into(), 0.0.into()), Qcplx::new(0.0.into(), 0.0.into()),
                Qcplx::new(0.0.into(), 0.0.into()), Qcplx::new((-1.0).into(), 0.0.into()),
            ])
        ),
        GateData::new(
            "y".to_string(),
            vec![],
            DMatrix::from_row_slice(2, 2, &[
                Qcplx::new(0.0.into(), 0.0.into()), Qcplx::new(0.0.into(), (-1.0).into()),
                Qcplx::new(0.0.into(), 1.0.into()), Qcplx::new(0.0.into(), 0.0.into()),
            ])
        ),
        GateData::new(
            "s".to_string(),
            vec![],
            DMatrix::from_row_slice(2, 2, &[
                Qcplx::new(1.0.into(), 0.0.into()), Qcplx::new(0.0.into(), 0.0.into()),
                Qcplx::new(0.0.into(), 0.0.into()), Qcplx::new(0.0.into(), 1.0.into()),
            ])
        ),
        GateData::new(
            "sdg".to_string(),
            vec![],
            DMatrix::from_row_slice(2, 2, &[
                Qcplx::new(1.0.into(), 0.0.into()), Qcplx::new(0.0.into(), 0.0.into()),
                Qcplx::new(0.0.into(), 0.0.into()), Qcplx::new(0.0.into(), (-1.0).into()),
            ])
        ),
        GateData::new(
            "t".to_string(),
            vec![],
            DMatrix::from_row_slice(2, 2, &[
                Qcplx::new(1.0.into(), 0.0.into()), Qcplx::new(0.0.into(), 0.0.into()),
                Qcplx::new(0.0.into(), 0.0.into()), Qreal::frac(1, 4).expipi(),
            ])
        ),
        GateData::new(
            "tdg".to_string(),
            vec![],
            DMatrix::from_row_slice(2, 2, &[
                Qcplx::new(1.0.into(), 0.0.into()), Qcplx::new(0.0.into(), 0.0.into()),
                Qcplx::new(0.0.into(), 0.0.into()), Qreal::frac(1, 4).expipi().conj(),
            ])
        ),
        GateData::new(
            "cx".to_string(),
            vec![],
            DMatrix::from_row_slice(4, 4, &[
                Qcplx::new(1.0.into(), 0.0.into()), Qcplx::new(0.0.into(), 0.0.into()), Qcplx::new(0.0.into(), 0.0.into()), Qcplx::new(0.0.into(), 0.0.into()),
                Qcplx::new(0.0.into(), 0.0.into()), Qcplx::new(1.0.into(), 0.0.into()), Qcplx::new(0.0.into(), 0.0.into()), Qcplx::new(0.0.into(), 0.0.into()),
                Qcplx::new(0.0.into(), 0.0.into()), Qcplx::new(0.0.into(), 0.0.into()), Qcplx::new(0.0.into(), 0.0.into()), Qcplx::new(1.0.into(), 0.0.into()),
                Qcplx::new(0.0.into(), 0.0.into()), Qcplx::new(0.0.into(), 0.0.into()), Qcplx::new(1.0.into(), 0.0.into()), Qcplx::new(0.0.into(), 0.0.into()),
            ])
        ),
        GateData::new(
            "cz".to_string(),
            vec![],
            DMatrix::from_row_slice(4, 4, &[
                Qcplx::new(1.0.into(), 0.0.into()), Qcplx::new(0.0.into(), 0.0.into()), Qcplx::new(0.0.into(), 0.0.into()), Qcplx::new(0.0.into(), 0.0.into()),
                Qcplx::new(0.0.into(), 0.0.into()), Qcplx::new(1.0.into(), 0.0.into()), Qcplx::new(0.0.into(), 0.0.into()), Qcplx::new(0.0.into(), 0.0.into()),
                Qcplx::new(0.0.into(), 0.0.into()), Qcplx::new(0.0.into(), 0.0.into()), Qcplx::new(1.0.into(), 0.0.into()), Qcplx::new(0.0.into(), 0.0.into()),
                Qcplx::new(0.0.into(), 0.0.into()), Qcplx::new(0.0.into(), 0.0.into()), Qcplx::new(0.0.into(), 0.0.into()), Qcplx::new((-1.0).into(), 0.0.into()),
            ])
        ),
        GateData::new(
            "cy".to_string(),
            vec![],
            DMatrix::from_row_slice(4, 4, &[
                Qcplx::new(1.0.into(), 0.0.into()), Qcplx::new(0.0.into(), 0.0.into()), Qcplx::new(0.0.into(), 0.0.into()), Qcplx::new(0.0.into(), 0.0.into()),
                Qcplx::new(0.0.into(), 0.0.into()), Qcplx::new(1.0.into(), 0.0.into()), Qcplx::new(0.0.into(), 0.0.into()), Qcplx::new(0.0.into(), 0.0.into()),
                Qcplx::new(0.0.into(), 0.0.into()), Qcplx::new(0.0.into(), 0.0.into()), Qcplx::new(0.0.into(), 0.0.into()), Qcplx::new(0.0.into(), (-1.0).into()),
                Qcplx::new(0.0.into(), 0.0.into()), Qcplx::new(0.0.into(), 0.0.into()), Qcplx::new(0.0.into(), 1.0.into()), Qcplx::new(0.0.into(), 0.0.into()),
            ])
        ),
        GateData::new(
            "cs".to_string(),
            vec![],
            DMatrix::from_row_slice(4, 4, &[
                Qcplx::new(1.0.into(), 0.0.into()), Qcplx::new(0.0.into(), 0.0.into()), Qcplx::new(0.0.into(), 0.0.into()), Qcplx::new(0.0.into(), 0.0.into()),
                Qcplx::new(0.0.into(), 0.0.into()), Qcplx::new(1.0.into(), 0.0.into()), Qcplx::new(0.0.into(), 0.0.into()), Qcplx::new(0.0.into(), 0.0.into()),
                Qcplx::new(0.0.into(), 0.0.into()), Qcplx::new(0.0.into(), 0.0.into()), Qcplx::new(1.0.into(), 0.0.into()), Qcplx::new(0.0.into(), 0.0.into()),
                Qcplx::new(0.0.into(), 0.0.into()), Qcplx::new(0.0.into(), 0.0.into()), Qcplx::new(0.0.into(), 0.0.into()), Qcplx::new(0.0.into(), 1.0.into()),
            ])
        ),
        GateData::new(
            "csdg".to_string(),
            vec![],
            DMatrix::from_row_slice(4, 4, &[
                Qcplx::new(1.0.into(), 0.0.into()), Qcplx::new(0.0.into(), 0.0.into()), Qcplx::new(0.0.into(), 0.0.into()), Qcplx::new(0.0.into(), 0.0.into()),
                Qcplx::new(0.0.into(), 0.0.into()), Qcplx::new(1.0.into(), 0.0.into()), Qcplx::new(0.0.into(), 0.0.into()), Qcplx::new(0.0.into(), 0.0.into()),
                Qcplx::new(0.0.into(), 0.0.into()), Qcplx::new(0.0.into(), 0.0.into()), Qcplx::new(1.0.into(), 0.0.into()), Qcplx::new(0.0.into(), 0.0.into()),
                Qcplx::new(0.0.into(), 0.0.into()), Qcplx::new(0.0.into(), 0.0.into()), Qcplx::new(0.0.into(), 0.0.into()), Qcplx::new(0.0.into(), (-1.0).into()),
            ])
        ),
        GateData::new(
            "swap".to_string(),
            vec![],
            DMatrix::from_row_slice(4, 4, &[
                Qcplx::new(1.0.into(), 0.0.into()), Qcplx::new(0.0.into(), 0.0.into()), Qcplx::new(0.0.into(), 0.0.into()), Qcplx::new(0.0.into(), 0.0.into()),
                Qcplx::new(0.0.into(), 0.0.into()), Qcplx::new(0.0.into(), 0.0.into()), Qcplx::new(1.0.into(), 0.0.into()), Qcplx::new(0.0.into(), 0.0.into()),
                Qcplx::new(0.0.into(), 0.0.into()), Qcplx::new(1.0.into(), 0.0.into()), Qcplx::new(0.0.into(), 0.0.into()), Qcplx::new(0.0.into(), 0.0.into()),
                Qcplx::new(0.0.into(), 0.0.into()), Qcplx::new(0.0.into(), 0.0.into()), Qcplx::new(0.0.into(), 0.0.into()), Qcplx::new(1.0.into(), 0.0.into()),
            ])
        ),
        GateData::new(
            "t1/2".to_string(),
            vec![],
            DMatrix::from_row_slice(2, 2, &[
                Qcplx::new(1.0.into(), 0.0.into()), Qcplx::new(0.0.into(), 0.0.into()),
                Qcplx::new(0.0.into(), 0.0.into()), Qreal::frac(1, 8).expipi(),
            ])
        ),
        GateData::new(
            "tdg1/2".to_string(),
            vec![],
            DMatrix::from_row_slice(2, 2, &[
                Qcplx::new(1.0.into(), 0.0.into()), Qcplx::new(0.0.into(), 0.0.into()),
                Qcplx::new(0.0.into(), 0.0.into()), Qreal::frac(1, 8).expipi().conj(),
            ])
        ),
        GateData::new(
            "rz".to_string(),
            vec!["pi/3".into()],
            DMatrix::from_row_slice(2, 2, &[
                Qcplx::new(1.0.into(), 0.0.into()), Qcplx::new(0.0.into(), 0.0.into()),
                Qcplx::new(0.0.into(), 0.0.into()), Qreal::frac(1, 3).expipi(),
            ])
        ),
        GateData::new(
            "rz".to_string(),
            vec!["-pi/3".into()],
            DMatrix::from_row_slice(2, 2, &[
                Qcplx::new(1.0.into(), 0.0.into()), Qcplx::new(0.0.into(), 0.0.into()),
                Qcplx::new(0.0.into(), 0.0.into()), Qreal::frac(1, 4).expipi().conj(),
            ])
        ),
    ]
}


pub static I: LazyLock<Gate16> = LazyLock::new(|| Gate16::from_name("i").unwrap());
pub static H: LazyLock<Gate16> = LazyLock::new(|| Gate16::from_name("h").unwrap());
pub static X: LazyLock<Gate16> = LazyLock::new(|| Gate16::from_name("x").unwrap());
pub static Z: LazyLock<Gate16> = LazyLock::new(|| Gate16::from_name("z").unwrap());
pub static Y: LazyLock<Gate16> = LazyLock::new(|| Gate16::from_name("y").unwrap());
pub static T: LazyLock<Gate16> = LazyLock::new(|| Gate16::from_name("t").unwrap());
pub static T_HALF: LazyLock<Gate16> = LazyLock::new(|| Gate16::from_name("t1/2").unwrap());
pub static TDG: LazyLock<Gate16> = LazyLock::new(|| Gate16::from_name("tdg").unwrap());
pub static TDG_HALF: LazyLock<Gate16> = LazyLock::new(|| Gate16::from_name("tdg1/2").unwrap());
pub static S: LazyLock<Gate16> = LazyLock::new(|| Gate16::from_name("s").unwrap());
pub static SDG: LazyLock<Gate16> = LazyLock::new(|| Gate16::from_name("sdg").unwrap());
pub static CX: LazyLock<Gate16> = LazyLock::new(|| Gate16::from_name("cx").unwrap());
pub static CZ : LazyLock<Gate16> = LazyLock::new(|| Gate16::from_name("cz").unwrap());
pub static CS : LazyLock<Gate16> = LazyLock::new(|| Gate16::from_name("cs").unwrap());
pub static CSDG : LazyLock<Gate16> = LazyLock::new(|| Gate16::from_name("csdg").unwrap());
pub static CY: LazyLock<Gate16> = LazyLock::new(|| Gate16::from_name("cy").unwrap());
pub static SWAP : LazyLock<Gate16> = LazyLock::new(|| Gate16::from_name("swap").unwrap());

pub fn i() -> Instr32 { I.instr([]) }
pub fn h(q1: u8) -> Instr32 { H.instr([q1]) }
pub fn x(q1: u8) -> Instr32 { X.instr([q1]) }
pub fn z(q1: u8) -> Instr32 { Z.instr([q1]) }
pub fn y(q1: u8) -> Instr32 { Y.instr([q1]) }
pub fn t(q1: u8) -> Instr32 { T.instr([q1]) }
pub fn t_half(q1: u8) -> Instr32 { T_HALF.instr([q1]) }
pub fn tdg(q1: u8) -> Instr32 { TDG.instr([q1]) }
pub fn tdg_half(q1: u8) -> Instr32 { TDG_HALF.instr([q1]) }
pub fn s(q1: u8) -> Instr32 { S.instr([q1]) }
pub fn sdg(q1: u8) -> Instr32 { SDG.instr([q1]) }
pub fn cx(q1: u8, q2: u8) -> Instr32 { CX.instr([q1, q2]) }
pub fn cz(q1: u8, q2: u8) -> Instr32 { CZ.instr([q1, q2]) }
pub fn cs(q1: u8, q2: u8) -> Instr32 { CS.instr([q1, q2]) }
pub fn csdg(q1: u8, q2: u8) -> Instr32 { CSDG.instr([q1, q2]) }
pub fn cy(q1: u8, q2: u8) -> Instr32 { CY.instr([q1, q2]) }
pub fn swap(q1: u8, q2: u8) -> Instr32 { SWAP.instr([q1, q2]) }

