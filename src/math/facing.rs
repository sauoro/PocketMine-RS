// src/math/facing.rs

#![allow(dead_code)]

use crate::math::axis::Axis;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Facing {
    Down = (Axis::Y as u8) << 1,
    Up = ((Axis::Y as u8) << 1) | Facing::FLAG_AXIS_POSITIVE,
    North = (Axis::Z as u8) << 1,
    South = ((Axis::Z as u8) << 1) | Facing::FLAG_AXIS_POSITIVE,
    West = (Axis::X as u8) << 1,
    East = ((Axis::X as u8) << 1) | Facing::FLAG_AXIS_POSITIVE,
}

impl Facing {
    pub const FLAG_AXIS_POSITIVE: u8 = 1;

    pub const ALL: [Facing; 6] = [
        Facing::Down,
        Facing::Up,
        Facing::North,
        Facing::South,
        Facing::West,
        Facing::East,
    ];

    pub const HORIZONTAL: [Facing; 4] = [
        Facing::North,
        Facing::South,
        Facing::West,
        Facing::East,
    ];

    pub const OFFSET: [[i8; 3]; 6] = [
        [0, -1, 0], // Down
        [0, 1, 0],  // Up
        [0, 0, -1], // North
        [0, 0, 1],  // South
        [-1, 0, 0], // West
        [1, 0, 0],  // East
    ];

    // Lazy static initialization for HashMaps might be better, but for simplicity:
    fn get_clockwise_map() -> HashMap<Axis, HashMap<Facing, Facing>> {
        let mut map = HashMap::new();

        let mut y_map = HashMap::new();
        y_map.insert(Facing::North, Facing::East);
        y_map.insert(Facing::East, Facing::South);
        y_map.insert(Facing::South, Facing::West);
        y_map.insert(Facing::West, Facing::North);
        map.insert(Axis::Y, y_map);

        let mut z_map = HashMap::new();
        z_map.insert(Facing::Up, Facing::East);
        z_map.insert(Facing::East, Facing::Down);
        z_map.insert(Facing::Down, Facing::West);
        z_map.insert(Facing::West, Facing::Up);
        map.insert(Axis::Z, z_map);

        let mut x_map = HashMap::new();
        x_map.insert(Facing::Up, Facing::North);
        x_map.insert(Facing::North, Facing::Down);
        x_map.insert(Facing::Down, Facing::South);
        x_map.insert(Facing::South, Facing::Up);
        map.insert(Axis::X, x_map);

        map
    }

    pub fn from_int(direction: u8) -> Option<Self> {
        match direction {
            0 => Some(Facing::Down),
            1 => Some(Facing::Up),
            2 => Some(Facing::North),
            3 => Some(Facing::South),
            4 => Some(Facing::West),
            5 => Some(Facing::East),
            _ => None,
        }
    }

    pub const fn axis(direction: Facing) -> Axis {
        // Safe because enum repr ensures values 0..5
        unsafe { std::mem::transmute::<u8, Axis>((direction as u8) >> 1) }
    }

    pub const fn is_positive(direction: Facing) -> bool {
        (direction as u8 & Facing::FLAG_AXIS_POSITIVE) == Facing::FLAG_AXIS_POSITIVE
    }

    pub const fn opposite(direction: Facing) -> Facing {
        // Safe because XORing with 1 toggles the last bit, mapping valid facings to their opposites
        unsafe { std::mem::transmute(direction as u8 ^ Facing::FLAG_AXIS_POSITIVE) }
    }

    pub fn rotate(direction: Facing, axis: Axis, clockwise: bool) -> Option<Facing> {
        let map = Facing::get_clockwise_map();
        if let Some(axis_map) = map.get(&axis) {
            if let Some(&rotated) = axis_map.get(&direction) {
                return Some(if clockwise {
                    rotated
                } else {
                    Facing::opposite(rotated)
                });
            }
        }
        None
    }

    pub fn rotate_y(direction: Facing, clockwise: bool) -> Option<Facing> {
        Facing::rotate(direction, Axis::Y, clockwise)
    }

    pub fn rotate_z(direction: Facing, clockwise: bool) -> Option<Facing> {
        Facing::rotate(direction, Axis::Z, clockwise)
    }

    pub fn rotate_x(direction: Facing, clockwise: bool) -> Option<Facing> {
        Facing::rotate(direction, Axis::X, clockwise)
    }

    pub fn validate(facing: u8) -> Result<(), String> {
        if Facing::from_int(facing).is_some() {
            Ok(())
        } else {
            Err(format!("Invalid direction {}", facing))
        }
    }

    pub fn to_string(facing: Facing) -> Option<&'static str> {
        match facing {
            Facing::Down => Some("down"),
            Facing::Up => Some("up"),
            Facing::North => Some("north"),
            Facing::South => Some("south"),
            Facing::West => Some("west"),
            Facing::East => Some("east"),
        }
    }

    pub fn get_offset(facing: Facing) -> [i8; 3] {
        Facing::OFFSET[facing as usize]
    }
}