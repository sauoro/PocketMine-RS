// src/nbt/tag/double_tag.rs
#![allow(dead_code)]

use crate::nbt::error::Result;
use crate::nbt::serializer::{NbtReader, NbtWriter}; // Removed NbtWrite
use crate::nbt::tag::tag::{Tag, TagType};
use std::any::Any;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DoubleTag {
    pub value: f64,
}

impl DoubleTag {
    pub fn new(value: f64) -> Self {
        Self { value }
    }

    pub fn read(reader: &mut dyn NbtReader) -> Result<Self> {
        Ok(Self::new(reader.read_double()?))
    }
}

impl Tag for DoubleTag {
    fn get_type(&self) -> TagType {
        TagType::Double
    }

    fn write(&self, writer: &mut dyn NbtWriter) -> Result<()> {
        writer.write_double(self.value)
    }

    fn get_value(&self) -> Box<dyn Any + Send + Sync> {
        Box::new(self.value)
    }

    fn equals(&self, other: &dyn Tag) -> bool {
        other.as_any().downcast_ref::<DoubleTag>().map_or(false, |t| self.value == t.value)
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
        write!(f, "TAG_Double: {}", self.value)
    }
}