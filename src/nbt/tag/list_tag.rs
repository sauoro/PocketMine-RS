// src/nbt/tag/list_tag.rs
#![allow(dead_code)]

use crate::nbt::error::{NbtError, Result};
use crate::nbt::serializer::{NbtReader, NbtWriter};
use crate::nbt::tag::tag::{Tag, TagType};
use crate::nbt::reader_tracker::ReaderTracker;
use crate::nbt::tag;
use std::any::Any;
use std::fmt;
use std::convert::TryInto;

#[derive(Debug, Clone)]
pub struct ListTag {
    value: Vec<Box<dyn Tag>>,
    tag_type: TagType,
}

impl PartialEq for ListTag {
    fn eq(&self, other: &Self) -> bool {
        self.tag_type == other.tag_type && self.value == other.value
    }
}
impl Eq for ListTag {}


impl ListTag {
    pub fn new(tag_type: TagType) -> Self {
        Self { value: Vec::new(), tag_type }
    }

    pub(crate) fn read(reader: &mut dyn NbtReader, tracker: &mut ReaderTracker) -> Result<Self> {
        let tag_type_id = reader.read_byte()?;
        let size = reader.read_int()?;

        let tag_type = TagType::from_id(tag_type_id)
            .ok_or_else(|| NbtError::new_data_error(&format!("Invalid tag type ID in ListTag: {}", tag_type_id)))?;

        if size < 0 {
            return Err(NbtError::new_data_error(&format!("Invalid negative size for ListTag: {}", size)));
        }
        let usize_size: usize = size.try_into().map_err(|_| NbtError::new_data_error("ListTag size too large"))?;

        let mut list = ListTag::new(tag_type);

        if usize_size > 0 {
            if tag_type == TagType::End {
                return Err(NbtError::new_data_error("Unexpected non-empty list of TAG_End"));
            }
            list.value.reserve(usize_size);
            // Depth is managed by caller
            for _ in 0..usize_size {
                let element = tag::create_tag(tag_type, reader, tracker)?;
                if element.get_type() != tag_type {
                    return Err(NbtError::new_unexpected_tag_type(&format!(
                        "List tag type mismatch: expected {:?}, got {:?}",
                        tag_type, element.get_type()
                    )));
                }
                list.value.push(element);
            }
        } else if tag_type != TagType::End {
            list.tag_type = tag_type;
        }

        Ok(list)
    }

    pub fn get_tag_type(&self) -> TagType {
        self.tag_type
    }

    pub fn len(&self) -> usize {
        self.value.len()
    }

    pub fn is_empty(&self) -> bool {
        self.value.is_empty()
    }

    fn check_tag_type(&mut self, tag: &dyn Tag) -> Result<()> {
        let type_to_add = tag.get_type();
        if self.is_empty() && self.tag_type == TagType::End {
            self.tag_type = type_to_add;
            Ok(())
        } else if type_to_add == self.tag_type {
            Ok(())
        } else {
            Err(NbtError::new_unexpected_tag_type(&format!(
                "Invalid tag type {:?} assigned to ListTag, expected {:?}",
                type_to_add, self.tag_type
            )))
        }
    }

    pub fn push(&mut self, tag: Box<dyn Tag>) -> Result<()> {
        self.check_tag_type(&*tag)?;
        self.value.push(tag);
        Ok(())
    }

    pub fn pop(&mut self) -> Option<Box<dyn Tag>> {
        self.value.pop()
    }

    pub fn insert(&mut self, index: usize, tag: Box<dyn Tag>) -> Result<()> {
        self.check_tag_type(&*tag)?;
        if index > self.len() {
            Err(NbtError::new_invalid_operation("Index out of bounds for ListTag insert"))
        } else {
            self.value.insert(index, tag);
            Ok(())
        }
    }

    pub fn remove(&mut self, index: usize) -> Option<Box<dyn Tag>> {
        if index < self.len() {
            Some(self.value.remove(index))
        } else {
            None
        }
    }

    pub fn get(&self, index: usize) -> Option<&dyn Tag> {
        self.value.get(index).map(|b| &**b)
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut dyn Tag> {
        self.value.get_mut(index).map(|b| &mut **b)
    }

    pub fn set(&mut self, index: usize, tag: Box<dyn Tag>) -> Result<()> {
        self.check_tag_type(&*tag)?;
        if index >= self.len() {
            Err(NbtError::new_invalid_operation("Index out of bounds for ListTag set"))
        } else {
            self.value[index] = tag;
            Ok(())
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &dyn Tag> {
        self.value.iter().map(|b| &**b)
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut dyn Tag> {
        self.value.iter_mut().map(|b| &mut **b)
    }
}

impl Tag for ListTag {
    fn get_type(&self) -> TagType {
        TagType::List
    }

    fn write(&self, writer: &mut dyn NbtWriter) -> Result<()> {
        writer.write_byte(self.tag_type as u8)?;
        let len: i32 = self.value.len().try_into().map_err(|_| NbtError::new_data_error("ListTag size too large for i32"))?;
        writer.write_int(len)?;
        for tag in &self.value {
            tag.write(writer)?;
        }
        Ok(())
    }

    fn get_value(&self) -> Box<dyn Any + Send + Sync> {
        Box::new(self.value.clone())
    }

    fn equals(&self, other: &dyn Tag) -> bool {
        other.as_any().downcast_ref::<ListTag>().map_or(false, |t| self == t)
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

    fn fmt_pretty(&self, f: &mut fmt::Formatter<'_>, indentation: usize) -> fmt::Result {
        writeln!(f, "TAG_List({:?}): {} entries {{", self.tag_type, self.value.len())?;
        let indent_str = " ".repeat((indentation + 1) * 2);
        for tag in &self.value {
            write!(f, "{}", indent_str)?;
            tag.fmt_pretty(f, indentation + 1)?;
            writeln!(f)?;
        }
        write!(f, "{}}}", " ".repeat(indentation * 2))
    }
}