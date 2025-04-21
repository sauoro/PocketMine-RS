// src/nbt/tag/float_tag.rs
#![allow(dead_code)]

use crate::nbt::error::Result;
use crate::nbt::serializer::{NbtReader, NbtWriter}; // Removed NbtWrite
use crate::nbt::tag::tag::{Tag, TagType};
use crate::utils;
use std::any::Any;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FloatTag {
    pub value: f32,
}

impl FloatTag {
    pub fn new(value: f32) -> Self {
        Self { value }
    }

    pub fn read(reader: &mut dyn NbtReader) -> Result<Self> {
        Ok(Self::new(reader.read_float()?))
    }
}

impl Tag for FloatTag {
    fn get_type(&self) -> TagType {
        TagType::Float
    }

    fn write(&self, writer: &mut dyn NbtWriter) -> Result<()> {
        writer.write_float(self.value)
    }

    fn get_value(&self) -> Box<dyn Any + Send + Sync> {
        Box::new(self.value)
    }

    fn equals(&self, other: &dyn Tag) -> bool {
        other.as_any().downcast_ref::<FloatTag>().map_or(false, |t| {
            let self_bytes = utils::binary::write_float(self.value).unwrap_or_default();
            let other_bytes = utils::binary::write_float(t.value).unwrap_or_default();
            self_bytes == other_bytes
        })
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
        write!(f, "TAG_Float: {}", self.value)
    }
}