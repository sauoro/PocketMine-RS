// src/nbt/tag/string_tag.rs
#![allow(dead_code)]

use crate::nbt::error::{NbtError, Result};
use crate::nbt::serializer::{NbtReader, NbtWriter}; // Removed NbtWrite
use crate::nbt::tag::tag::{Tag, TagType};
use crate::utils::limits;
use std::any::Any;
use std::fmt;
use std::convert::TryInto;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StringTag {
    pub value: String,
}

impl StringTag {
    pub fn new(value: String) -> Self {
        Self { value }
    }

    pub fn read(reader: &mut dyn NbtReader) -> Result<Self> {
        Ok(Self::new(reader.read_string()?))
    }

    pub(crate) fn check_write_length(value: &str) -> Result<i16> {
        let len = value.len();
        if len > limits::I16_MAX as usize {
            Err(NbtError::new_invalid_tag_value(&format!(
                "StringTag cannot hold more than {} bytes, got string of length {}",
                limits::I16_MAX, len
            )))
        } else {
            Ok(len.try_into()?)
        }
    }
}

impl Tag for StringTag {
    fn get_type(&self) -> TagType {
        TagType::String
    }

    fn write(&self, writer: &mut dyn NbtWriter) -> Result<()> {
        writer.write_string(&self.value)
    }

    fn get_value(&self) -> Box<dyn Any + Send + Sync> {
        Box::new(self.value.clone())
    }

    fn equals(&self, other: &dyn Tag) -> bool {
        other.as_any().downcast_ref::<StringTag>().map_or(false, |t| self.value == t.value)
    }

    fn clone_tag(&self) -> Box<dyn Tag> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn fmt_pretty(&self, f: &mut fmt::Formatter<'_>, _indentation: usize) -> fmt::Result {
        write!(f, "TAG_String: {:?}", self.value)
    }
}