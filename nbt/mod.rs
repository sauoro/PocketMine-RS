// src/nbt/mod.rs
#![allow(dead_code)]

pub mod error;
pub mod reader_tracker;
pub mod serializer;
pub mod tag;
pub mod tree_root;
pub mod big_endian_serializer;
pub mod little_endian_serializer;

// Re-export necessary types
pub use error::{NbtError, Result};
pub use tag::{CompoundTag, ListTag, Tag, TagType}; // NbtTag removed from re-export
pub use tree_root::TreeRoot;
pub use big_endian_serializer::BigEndianNbtSerializer;
pub use little_endian_serializer::LittleEndianNbtSerializer;