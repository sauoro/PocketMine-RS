// src/nbt/tag/mod.rs
#![allow(dead_code)]

mod byte_array_tag;
mod byte_tag;
mod compound_tag;
mod double_tag;
mod float_tag;
mod int_array_tag;
mod int_tag;
mod list_tag;
mod long_tag;
mod short_tag;
mod string_tag;
pub mod tag;

// Re-export concrete tag types and the base trait/enum
pub use byte_array_tag::ByteArrayTag;
pub use byte_tag::ByteTag;
pub use compound_tag::CompoundTag;
pub use double_tag::DoubleTag;
pub use float_tag::FloatTag;
pub use int_array_tag::IntArrayTag;
pub use int_tag::IntTag;
pub use list_tag::ListTag;
pub use long_tag::LongTag;
pub use short_tag::ShortTag;
pub use string_tag::StringTag;
pub use tag::{Tag, TagType};

use crate::nbt::error::{NbtError, Result};
use crate::nbt::serializer::NbtReader;
use crate::nbt::reader_tracker::ReaderTracker;
use std::boxed::Box;
use std::fmt; // Keep fmt for Display macro

// NbtTag enum definition remains removed


// Factory function equivalent to NBT::createTag (remains the same logic)
pub fn create_tag(tag_type: TagType, reader: &mut dyn NbtReader, tracker: &mut ReaderTracker) -> Result<Box<dyn Tag>> {
    match tag_type {
        TagType::Byte => ByteTag::read(reader).map(|t| Box::new(t) as Box<dyn Tag>),
        TagType::Short => ShortTag::read(reader).map(|t| Box::new(t) as Box<dyn Tag>),
        TagType::Int => IntTag::read(reader).map(|t| Box::new(t) as Box<dyn Tag>),
        TagType::Long => LongTag::read(reader).map(|t| Box::new(t) as Box<dyn Tag>),
        TagType::Float => FloatTag::read(reader).map(|t| Box::new(t) as Box<dyn Tag>),
        TagType::Double => DoubleTag::read(reader).map(|t| Box::new(t) as Box<dyn Tag>),
        TagType::ByteArray => ByteArrayTag::read(reader).map(|t| Box::new(t) as Box<dyn Tag>),
        TagType::String => StringTag::read(reader).map(|t| Box::new(t) as Box<dyn Tag>),
        TagType::List => {
            tracker.increase_depth()?;
            let result = ListTag::read(reader, tracker);
            tracker.decrease_depth();
            result.map(|t| Box::new(t) as Box<dyn Tag>)
        },
        TagType::Compound => {
            tracker.increase_depth()?;
            let result = CompoundTag::read(reader, tracker);
            tracker.decrease_depth();
            result.map(|t| Box::new(t) as Box<dyn Tag>)
        },
        TagType::IntArray => IntArrayTag::read(reader).map(|t| Box::new(t) as Box<dyn Tag>),
        TagType::End => Err(NbtError::new_data_error("Cannot create TagType::End")),
    }
}

// Define the Display macro here
#[macro_export]
macro_rules! impl_display_for_tag {
    ($($t:ty),+ $(,)?) => {
        $(
            impl std::fmt::Display for $t {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    use crate::nbt::tag::Tag; // Ensure Tag trait is in scope
                    self.fmt_pretty(f, 0)
                }
            }
        )+
    };
}

// Apply the macro to all concrete tag types
impl_display_for_tag!(
    ByteTag, ShortTag, IntTag, LongTag, FloatTag, DoubleTag,
    ByteArrayTag, StringTag, ListTag, CompoundTag, IntArrayTag,
);

// TryFrom/Into implementations removed from here, now defined in compound_tag.rs