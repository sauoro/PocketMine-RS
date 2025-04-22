// src/nbt/tag/byte_tag.rs
#![allow(dead_code)]

use crate::nbt::error::Result;
use crate::nbt::serializer::{NbtReader, NbtWriter}; // Removed NbtWrite
use crate::nbt::tag::tag::{IntegerishTag, Tag, TagType};
use std::any::Any;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ByteTag {
    pub value: i8,
}

impl ByteTag {
    pub fn new(value: i8) -> Self {
        Self { value }
    }

    pub fn read(reader: &mut dyn NbtReader) -> Result<Self> {
        Ok(Self::new(reader.read_signed_byte()?))
    }
}

impl IntegerishTag<i8> for ByteTag {
    fn min_value() -> i8 { crate::utils::limits::I8_MIN }
    fn max_value() -> i8 { crate::utils::limits::I8_MAX }
}

impl Tag for ByteTag {
    fn get_type(&self) -> TagType {
        TagType::Byte
    }

    fn write(&self, writer: &mut dyn NbtWriter) -> Result<()> {
        writer.write_signed_byte(self.value)
    }

    fn get_value(&self) -> Box<dyn Any + Send + Sync> {
        Box::new(self.value)
    }

    fn equals(&self, other: &dyn Tag) -> bool {
        other.as_any().downcast_ref::<ByteTag>().map_or(false, |t| self.value == t.value)
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
        write!(f, "TAG_Byte: {}", self.value)
    }
}