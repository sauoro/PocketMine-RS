// src/nbt/tag/byte_array_tag.rs
#![allow(dead_code)]

use crate::nbt::error::Result;
use crate::nbt::serializer::{NbtReader, NbtWriter}; // Removed NbtWrite
use crate::nbt::tag::tag::{Tag, TagType};
use std::any::Any;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ByteArrayTag {
    pub value: Vec<u8>,
}

impl ByteArrayTag {
    pub fn new(value: Vec<u8>) -> Self {
        Self { value }
    }

    pub fn read(reader: &mut dyn NbtReader) -> Result<Self> {
        Ok(Self::new(reader.read_byte_array()?))
    }
}

impl Tag for ByteArrayTag {
    fn get_type(&self) -> TagType {
        TagType::ByteArray
    }

    fn write(&self, writer: &mut dyn NbtWriter) -> Result<()> {
        writer.write_byte_array(&self.value)
    }

    fn get_value(&self) -> Box<dyn Any + Send + Sync> {
        Box::new(self.value.clone())
    }

    fn equals(&self, other: &dyn Tag) -> bool {
        other.as_any().downcast_ref::<ByteArrayTag>().map_or(false, |t| self.value == t.value)
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
        // Limit the number of bytes shown for brevity
        const MAX_BYTES_DISPLAY: usize = 32;
        let display_bytes = self.value.iter().take(MAX_BYTES_DISPLAY).map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join(", ");
        let ellipsis = if self.value.len() > MAX_BYTES_DISPLAY { "..." } else { "" };
        write!(f, "TAG_ByteArray: [{} {}] ({} bytes)",
               display_bytes,
               ellipsis,
               self.value.len()
        )
    }
}