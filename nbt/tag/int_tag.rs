// src/nbt/tag/int_tag.rs
#![allow(dead_code)]

use crate::nbt::error::Result;
use crate::nbt::serializer::{NbtReader, NbtWriter}; // Removed NbtWrite
use crate::nbt::tag::tag::{IntegerishTag, Tag, TagType};
use std::any::Any;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct IntTag {
    pub value: i32,
}

impl IntTag {
    pub fn new(value: i32) -> Self {
        Self { value }
    }

    pub fn read(reader: &mut dyn NbtReader) -> Result<Self> {
        Ok(Self::new(reader.read_int()?))
    }
}

impl IntegerishTag<i32> for IntTag {
    fn min_value() -> i32 { crate::utils::limits::I32_MIN }
    fn max_value() -> i32 { crate::utils::limits::I32_MAX }
}

impl Tag for IntTag {
    fn get_type(&self) -> TagType {
        TagType::Int
    }

    fn write(&self, writer: &mut dyn NbtWriter) -> Result<()> {
        writer.write_int(self.value)
    }

    fn get_value(&self) -> Box<dyn Any + Send + Sync> {
        Box::new(self.value)
    }

    fn equals(&self, other: &dyn Tag) -> bool {
        other.as_any().downcast_ref::<IntTag>().map_or(false, |t| self.value == t.value)
    }

    fn clone_tag(&self) -> Box<dyn Tag> {
        Box::new(*self)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn fmt_pretty(&self, f: &mut fmt::Formatter<'_>, _indentation: usize) -> fmt::Result {
        write!(f, "TAG_Int: {}", self.value)
    }
}