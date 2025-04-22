// src/math/axis.rs

#![allow(dead_code)]

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Axis {
    Y = 0,
    Z = 1,
    X = 2,
}

impl Axis {
    pub fn from_int(axis: u8) -> Option<Self> {
        match axis {
            0 => Some(Axis::Y),
            1 => Some(Axis::Z),
            2 => Some(Axis::X),
            _ => None,
        }
    }

    pub fn to_string(axis: Axis) -> Option<&'static str> {
        match axis {
            Axis::Y => Some("y"),
            Axis::Z => Some("z"),
            Axis::X => Some("x"),
        }
    }
}
