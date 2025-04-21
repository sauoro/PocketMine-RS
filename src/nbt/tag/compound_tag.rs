// src/nbt/tag/compound_tag.rs
#![allow(dead_code)]

use crate::nbt::error::{NbtError, Result};
use crate::nbt::serializer::{NbtReader, NbtWriter};
use crate::nbt::tag::tag::{Tag, TagType};
use crate::nbt::tag; // For create_tag factory
use crate::nbt::reader_tracker::ReaderTracker;
use crate::utils::limits;
use std::collections::HashMap;
use std::any::Any;
use std::fmt;
// Removed TryInto, TryFrom

// Import specific tag types for getters/setters and From impls
use super::{
    ByteTag, ShortTag, IntTag, LongTag, FloatTag, DoubleTag,
    ByteArrayTag, StringTag, ListTag, IntArrayTag
};

#[derive(Debug, Clone)]
pub struct CompoundTag {
    value: HashMap<String, Box<dyn Tag>>,
}

impl PartialEq for CompoundTag {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}
impl Eq for CompoundTag {}


impl CompoundTag {
    pub fn new() -> Self {
        Self { value: HashMap::new() }
    }

    pub(crate) fn read(reader: &mut dyn NbtReader, tracker: &mut ReaderTracker) -> Result<Self> {
        let mut compound = CompoundTag::new();
        loop {
            let type_id = reader.read_byte()?;
            let tag_type = TagType::from_id(type_id)
                .ok_or_else(|| NbtError::new_data_error(&format!("Invalid tag type ID in CompoundTag: {}", type_id)))?;

            if tag_type == TagType::End {
                break;
            }

            let name = reader.read_string()?;
            let tag = tag::create_tag(tag_type, reader, tracker)?;

            compound.value.insert(name, tag);
        }
        Ok(compound)
    }

    pub fn len(&self) -> usize {
        self.value.len()
    }

    pub fn is_empty(&self) -> bool {
        self.value.is_empty()
    }

    pub fn contains_key(&self, name: &str) -> bool {
        self.value.contains_key(name)
    }


    pub fn get_tag(&self, name: &str) -> Option<&dyn Tag> {
        self.value.get(name).map(|b| &**b)
    }

    pub fn get_tag_mut(&mut self, name: &str) -> Option<&mut dyn Tag> {
        self.value.get_mut(name).map(|b| &mut **b)
    }

    fn get_typed_tag<T: Tag + 'static>(&self, name: &str) -> Result<Option<&T>> {
        match self.get_tag(name) {
            None => Ok(None),
            Some(tag) => tag.as_any().downcast_ref::<T>()
                .ok_or_else(|| NbtError::new_unexpected_tag_type(&format!(
                    "Expected tag '{}' to be type {}, but found {}",
                    name, std::any::type_name::<T>(), tag.get_type().get_name()
                )))
                .map(Some)
        }
    }

    pub fn get_list_tag(&self, name: &str) -> Result<Option<&ListTag>> {
        self.get_typed_tag(name)
    }
    pub fn get_compound_tag(&self, name: &str) -> Result<Option<&CompoundTag>> {
        self.get_typed_tag(name)
    }

    // Simplified primitive getter using From impls defined below
    fn get_primitive_value<T, V>(&self, name: &str, default: Option<V>) -> Result<V>
    where
        T: Tag + 'static,
        V: Clone + 'static,
        for<'a> V: From<&'a T>, // Use From bound here
    {
        match self.get_typed_tag::<T>(name)? {
            Some(tag_ref) => Ok(V::from(tag_ref)), // Use From conversion
            None => default.ok_or_else(|| NbtError::new_no_such_tag(&format!("Tag \"{}\" does not exist", name))),
        }
    }

    // --- Primitive Getters (No change needed here, rely on From impls below) ---
    pub fn get_byte(&self, name: &str, default: Option<i8>) -> Result<i8> { self.get_primitive_value::<ByteTag, _>(name, default) }
    pub fn get_short(&self, name: &str, default: Option<i16>) -> Result<i16> { self.get_primitive_value::<ShortTag, _>(name, default) }
    pub fn get_int(&self, name: &str, default: Option<i32>) -> Result<i32> { self.get_primitive_value::<IntTag, _>(name, default) }
    pub fn get_long(&self, name: &str, default: Option<i64>) -> Result<i64> { self.get_primitive_value::<LongTag, _>(name, default) }
    pub fn get_float(&self, name: &str, default: Option<f32>) -> Result<f32> { self.get_primitive_value::<FloatTag, _>(name, default) }
    pub fn get_double(&self, name: &str, default: Option<f64>) -> Result<f64> { self.get_primitive_value::<DoubleTag, _>(name, default) }
    pub fn get_byte_array(&self, name: &str, default: Option<Vec<u8>>) -> Result<Vec<u8>> { self.get_primitive_value::<ByteArrayTag, _>(name, default) }
    pub fn get_string(&self, name: &str, default: Option<String>) -> Result<String> { self.get_primitive_value::<StringTag, _>(name, default) }
    pub fn get_int_array(&self, name: &str, default: Option<Vec<i32>>) -> Result<Vec<i32>> { self.get_primitive_value::<IntArrayTag, _>(name, default) }


    // --- Setters (Remain the same) ---
    pub fn set_tag(&mut self, name: String, tag: Box<dyn Tag>) -> Result<()> {
        if name.len() > limits::I16_MAX as usize {
            return Err(NbtError::new_invalid_tag_value(&format!(
                "Tag name must be at most {} bytes, but got {} bytes",
                limits::I16_MAX, name.len()
            )));
        }
        self.value.insert(name, tag);
        Ok(())
    }
    pub fn remove_tag(&mut self, name: &str) -> Option<Box<dyn Tag>> { self.value.remove(name) }
    pub fn remove_tags(&mut self, names: &[&str]) { for name in names { self.value.remove(*name); } }
    pub fn set_byte(&mut self, name: String, value: i8) -> Result<()> { self.set_tag(name, Box::new(ByteTag::new(value))) }
    pub fn set_short(&mut self, name: String, value: i16) -> Result<()> { self.set_tag(name, Box::new(ShortTag::new(value))) }
    pub fn set_int(&mut self, name: String, value: i32) -> Result<()> { self.set_tag(name, Box::new(IntTag::new(value))) }
    pub fn set_long(&mut self, name: String, value: i64) -> Result<()> { self.set_tag(name, Box::new(LongTag::new(value))) }
    pub fn set_float(&mut self, name: String, value: f32) -> Result<()> { self.set_tag(name, Box::new(FloatTag::new(value))) }
    pub fn set_double(&mut self, name: String, value: f64) -> Result<()> { self.set_tag(name, Box::new(DoubleTag::new(value))) }
    pub fn set_byte_array(&mut self, name: String, value: Vec<u8>) -> Result<()> { self.set_tag(name, Box::new(ByteArrayTag::new(value))) }
    pub fn set_string(&mut self, name: String, value: String) -> Result<()> { self.set_tag(name, Box::new(StringTag::new(value))) }
    pub fn set_int_array(&mut self, name: String, value: Vec<i32>) -> Result<()> { self.set_tag(name, Box::new(IntArrayTag::new(value))) }
    pub fn set_list(&mut self, name: String, value: ListTag) -> Result<()> { self.set_tag(name, Box::new(value)) }
    pub fn set_compound(&mut self, name: String, value: CompoundTag) -> Result<()> { self.set_tag(name, Box::new(value)) }


    // --- Iteration (Remain the same) ---
    pub fn iter(&self) -> impl Iterator<Item = (&String, &dyn Tag)> { self.value.iter().map(|(k, v)| (k, &**v)) }
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&String, &mut dyn Tag)> { self.value.iter_mut().map(|(k, v)| (k, &mut **v)) }

    // --- Merging (Remains the same) ---
    pub fn merge(&self, other: &CompoundTag) -> CompoundTag {
        let mut new_compound = self.clone();
        for (k, v) in &other.value {
            new_compound.value.insert(k.clone(), v.clone_tag());
        }
        new_compound
    }
}

impl Tag for CompoundTag {
    fn get_type(&self) -> TagType { TagType::Compound }
    fn write(&self, writer: &mut dyn NbtWriter) -> Result<()> {
        for (name, tag) in &self.value {
            writer.write_byte(tag.get_type() as u8)?;
            writer.write_string(name)?;
            tag.write(writer)?;
        }
        writer.write_byte(TagType::End as u8)?;
        Ok(())
    }
    fn get_value(&self) -> Box<dyn Any + Send + Sync> { Box::new(self.value.clone()) }
    fn equals(&self, other: &dyn Tag) -> bool { other.as_any().downcast_ref::<CompoundTag>().map_or(false, |t| self == t) }
    fn clone_tag(&self) -> Box<dyn Tag> { Box::new(self.clone()) }
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn fmt_pretty(&self, f: &mut fmt::Formatter<'_>, indentation: usize) -> fmt::Result {
        writeln!(f, "TAG_Compound: {} entries {{", self.value.len())?;
        let indent_str = " ".repeat((indentation + 1) * 2);
        let mut sorted_keys: Vec<_> = self.value.keys().collect();
        sorted_keys.sort();
        for key in sorted_keys {
            if let Some(tag) = self.value.get(key) {
                write!(f, "{}{:?}: ", indent_str, key)?;
                tag.fmt_pretty(f, indentation + 1)?;
                writeln!(f)?;
            }
        }
        write!(f, "{}}}", " ".repeat(indentation * 2))
    }
}

impl Default for CompoundTag { fn default() -> Self { Self::new() } }

// --- From Implementations (Moved to module scope below impl CompoundTag) ---
impl<'a> From<&'a ByteTag> for i8 { fn from(tag: &'a ByteTag) -> Self { tag.value } }
impl<'a> From<&'a ShortTag> for i16 { fn from(tag: &'a ShortTag) -> Self { tag.value } }
impl<'a> From<&'a IntTag> for i32 { fn from(tag: &'a IntTag) -> Self { tag.value } }
impl<'a> From<&'a LongTag> for i64 { fn from(tag: &'a LongTag) -> Self { tag.value } }
impl<'a> From<&'a FloatTag> for f32 { fn from(tag: &'a FloatTag) -> Self { tag.value } }
impl<'a> From<&'a DoubleTag> for f64 { fn from(tag: &'a DoubleTag) -> Self { tag.value } }
impl<'a> From<&'a ByteArrayTag> for Vec<u8> { fn from(tag: &'a ByteArrayTag) -> Self { tag.value.clone() } }
impl<'a> From<&'a StringTag> for String { fn from(tag: &'a StringTag) -> Self { tag.value.clone() } }
impl<'a> From<&'a IntArrayTag> for Vec<i32> { fn from(tag: &'a IntArrayTag) -> Self { tag.value.clone() } }