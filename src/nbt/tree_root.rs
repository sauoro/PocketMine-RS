// src/nbt/tree_root.rs
#![allow(dead_code)]

use crate::nbt::error::{NbtError, Result};
use crate::nbt::tag::{Tag, CompoundTag};
use crate::utils::limits;
use std::fmt;

#[derive(Debug, Clone)]
pub struct TreeRoot {
    name: String,
    root: Box<dyn Tag>,
}

impl TreeRoot {
    pub fn new(name: String, root: Box<dyn Tag>) -> Result<Self> {
        if name.len() > limits::I16_MAX as usize {
            return Err(NbtError::new_invalid_tag_value(&format!(
                "Root tag name must be at most {} bytes, but got {} bytes",
                limits::I16_MAX, name.len()
            )));
        }
        Ok(Self { name, root })
    }

    pub fn get_name(&self) -> &str { &self.name }
    pub fn get_tag(&self) -> &dyn Tag { &*self.root }
    pub fn get_tag_mut(&mut self) -> &mut dyn Tag { &mut *self.root }

    pub fn must_get_compound_tag(&self) -> Result<&CompoundTag> {
        self.root.as_any().downcast_ref::<CompoundTag>()
            .ok_or_else(|| NbtError::new_unexpected_tag_type("Root tag is not a CompoundTag"))
    }
    pub fn must_get_compound_tag_mut(&mut self) -> Result<&mut CompoundTag> {
        self.root.as_any_mut().downcast_mut::<CompoundTag>()
            .ok_or_else(|| NbtError::new_unexpected_tag_type("Root tag is not a CompoundTag"))
    }
}

// Manual implementation of PartialEq for TreeRoot
impl PartialEq for TreeRoot {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name &&
            self.root == other.root.clone() // This uses the PartialEq for Box<dyn Tag>
    }
}
impl Eq for TreeRoot {}


impl fmt::Display for TreeRoot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "ROOT({}) {{", self.root.get_type().get_name())?;
        let indent_str = "  ";
        if !self.name.is_empty() {
            write!(f, "{}{:?}: ", indent_str, self.name)?;
        } else {
            write!(f, "{}", indent_str)?;
        }
        self.root.fmt_pretty(f, 1)?;
        writeln!(f)?;
        write!(f, "}}")
    }
}