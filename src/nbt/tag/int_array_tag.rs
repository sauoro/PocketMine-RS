// src/nbt/tag/int_array_tag.rs
#![allow(dead_code)]

use crate::nbt::error::Result;
use crate::nbt::serializer::{NbtReader, NbtWriter}; // Removed NbtWrite
use crate::nbt::tag::tag::{Tag, TagType};
use std::any::Any;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IntArrayTag {
    pub value: Vec<i32>,
}

impl IntArrayTag {
    pub fn new(value: Vec<i32>) -> Self {
        Self { value }
    }

    pub fn read(reader: &mut dyn NbtReader) -> Result<Self> {
        Ok(Self::new(reader.read_int_array()?))
    }
}

impl Tag for IntArrayTag {
    fn get_type(&self) -> TagType {
        TagType::IntArray
    }

    fn write(&self, writer: &mut dyn NbtWriter) -> Result<()> {
        writer.write_int_array(&self.value)
    }

    fn get_value(&self) -> Box<dyn Any + Send + Sync> {
        Box::new(self.value.clone())
    }

    fn equals(&self, other: &dyn Tag) -> bool {
        other.as_any().downcast_ref::<IntArrayTag>().map_or(false, |t| self.value == t.value)
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
        // Limit the number of elements shown for brevity
        const MAX_INTS_DISPLAY: usize = 32;
        let display_ints = self.value.iter().take(MAX_INTS_DISPLAY).map(|i| i.to_string()).collect::<Vec<_>>().join(", ");
        let ellipsis = if self.value.len() > MAX_INTS_DISPLAY { "..." } else { "" };

        write!(f, "TAG_IntArray: [{} {}] ({} elements)",
               display_ints,
               ellipsis,
               self.value.len()
        )
    }
}