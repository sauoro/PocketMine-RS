// src/color/mod.rs

#![allow(dead_code)]

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Color {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

impl Color {
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub const fn new_opaque(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 0xff }
    }

    pub const fn a(&self) -> u8 {
        self.a
    }

    pub const fn r(&self) -> u8 {
        self.r
    }

    pub const fn g(&self) -> u8 {
        self.g
    }

    pub const fn b(&self) -> u8 {
        self.b
    }

    pub fn mix(colors: &[Color]) -> Option<Color> {
        let count = colors.len();
        if count == 0 {
            return None;
        }

        let mut total_r: usize = 0;
        let mut total_g: usize = 0;
        let mut total_b: usize = 0;
        let mut total_a: usize = 0;

        for color in colors {
            total_r += color.r as usize;
            total_g += color.g as usize;
            total_b += color.b as usize;
            total_a += color.a as usize;
        }

        Some(Color::new(
            (total_r / count) as u8,
            (total_g / count) as u8,
            (total_b / count) as u8,
            (total_a / count) as u8,
        ))
    }

    pub const fn from_rgb(code: u32) -> Color {
        Color::new_opaque(
            ((code >> 16) & 0xff) as u8,
            ((code >> 8) & 0xff) as u8,
            (code & 0xff) as u8,
        )
    }

    pub const fn from_argb(code: u32) -> Color {
        Color::new(
            ((code >> 16) & 0xff) as u8,
            ((code >> 8) & 0xff) as u8,
            (code & 0xff) as u8,
            ((code >> 24) & 0xff) as u8,
        )
    }

    pub const fn to_argb(&self) -> u32 {
        ((self.a as u32) << 24) | ((self.r as u32) << 16) | ((self.g as u32) << 8) | (self.b as u32)
    }

    pub const fn from_rgba(code: u32) -> Color {
        Color::new(
            ((code >> 24) & 0xff) as u8,
            ((code >> 16) & 0xff) as u8,
            ((code >> 8) & 0xff) as u8,
            (code & 0xff) as u8,
        )
    }

    pub const fn to_rgba(&self) -> u32 {
        ((self.r as u32) << 24) | ((self.g as u32) << 16) | ((self.b as u32) << 8) | (self.a as u32)
    }
}