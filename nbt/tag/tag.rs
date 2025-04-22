// src/nbt/tag/tag.rs
#![allow(dead_code)]

use crate::nbt::error::{NbtError, Result};
use crate::nbt::serializer::NbtWriter;
use std::fmt::{Debug};
use std::any::Any;

// NBT Tag Type constants (Remains unchanged)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum TagType {
    End = 0, Byte = 1, Short = 2, Int = 3, Long = 4, Float = 5, Double = 6,
    ByteArray = 7, String = 8, List = 9, Compound = 10, IntArray = 11,
}

impl TagType {
    // from_id and get_name remain unchanged
    pub fn from_id(id: u8) -> Option<Self> {
        match id {
            0 => Some(TagType::End), 1 => Some(TagType::Byte), 2 => Some(TagType::Short),
            3 => Some(TagType::Int), 4 => Some(TagType::Long), 5 => Some(TagType::Float),
            6 => Some(TagType::Double), 7 => Some(TagType::ByteArray), 8 => Some(TagType::String),
            9 => Some(TagType::List), 10 => Some(TagType::Compound), 11 => Some(TagType::IntArray),
            _ => None,
        }
    }
    pub fn get_name(&self) -> &'static str {
        match self {
            TagType::End => "End", TagType::Byte => "Byte", TagType::Short => "Short",
            TagType::Int => "Int", TagType::Long => "Long", TagType::Float => "Float",
            TagType::Double => "Double", TagType::ByteArray => "ByteArray", TagType::String => "String",
            TagType::List => "List", TagType::Compound => "Compound", TagType::IntArray => "IntArray",
        }
    }
}


// Base trait for all NBT tags
pub trait Tag: Any + Debug + Send + Sync {
    fn get_type(&self) -> TagType;
    fn write(&self, writer: &mut dyn NbtWriter) -> Result<()>;
    fn get_value(&self) -> Box<dyn Any + Send + Sync>;
    fn equals(&self, other: &dyn Tag) -> bool;
    fn clone_tag(&self) -> Box<dyn Tag>;
    fn fmt_pretty(&self, f: &mut std::fmt::Formatter<'_>, indentation: usize) -> std::fmt::Result;
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

// Implement Clone for Box<dyn Tag> (Remains unchanged)
impl Clone for Box<dyn Tag> {
    fn clone(&self) -> Self {
        self.clone_tag()
    }
}

// Allow comparing Box<dyn Tag> using the equals method (Remains unchanged)
impl PartialEq for dyn Tag {
    fn eq(&self, other: &Self) -> bool {
        if self.as_any().type_id() != other.as_any().type_id() {
            return false;
        }
        self.equals(other)
    }
}
impl Eq for dyn Tag {}

// Common trait for integer-like tags
pub(crate) trait IntegerishTag<T: Copy + Ord + std::fmt::Display> {
    fn min_value() -> T;
    fn max_value() -> T;
    fn check_bounds(value: T) -> Result<T> {
        if value >= Self::min_value() && value <= Self::max_value() { // Corrected Self. to Self::
            Ok(value)
        } else {
            Err(NbtError::new_invalid_tag_value(&format!(
                "Value {} is outside the allowed range {} - {}",
                value, Self::min_value(), Self::max_value() // Corrected Self. to Self::
            )))
        }
    }
}

// Macro definition moved to tag/mod.rs