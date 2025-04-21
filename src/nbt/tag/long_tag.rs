// src/nbt/tag/long_tag.rs
#![allow(dead_code)]

use crate::nbt::error::Result;
use crate::nbt::serializer::{NbtReader, NbtWriter}; // Removed NbtWrite
use crate::nbt::tag::tag::{IntegerishTag, Tag, TagType};
use std::any::Any;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct LongTag {
    pub value: i64,
}

impl LongTag {
    pub fn new(value: i64) -> Self {
        Self { value }
    }

    pub fn read(reader: &mut dyn NbtReader) -> Result<Self> {
        Ok(Self::new(reader.read_long()?))
    }
}

impl IntegerishTag<i64> for LongTag {
    fn min_value() -> i64 { crate::utils::limits::I64_MIN }
    fn max_value() -> i64 { crate::utils::limits::I64_MAX }
}

impl Tag for LongTag {
    fn get_type(&self) -> TagType {
        TagType::Long
    }

    fn write(&self, writer: &mut dyn NbtWriter) -> Result<()> {
        writer.write_long(self.value)
    }

    fn get_value(&self) -> Box<dyn Any + Send + Sync> {
        Box::new(self.value)
    }

    fn equals(&self, other: &dyn Tag) -> bool {
        other.as_any().downcast_ref::<LongTag>().map_or(false, |t| self.value == t.value)
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
        write!(f, "TAG_Long: {}", self.value)
    }
}