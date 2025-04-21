// src/nbt/reader_tracker.rs
#![allow(dead_code)]

use crate::nbt::error::{NbtError, Result};

#[derive(Debug, Clone)]
pub struct ReaderTracker {
    max_depth: usize,
    current_depth: usize,
}

impl ReaderTracker {
    pub fn new(max_depth: usize) -> Self {
        // Depth 0 means no limit
        Self { max_depth, current_depth: 0 }
    }

    // Internal function called by create_tag for compound/list
    pub(crate) fn increase_depth(&mut self) -> Result<()> {
        if self.max_depth > 0 {
            self.current_depth = self.current_depth.checked_add(1)
                .ok_or_else(|| NbtError::new_data_error("Depth overflow during increase"))?;
            if self.current_depth > self.max_depth {
                // Decrement depth before returning error to reflect state before failed increase
                self.current_depth -= 1;
                return Err(NbtError::new_depth_limit_exceeded(&format!(
                    "Nesting level {} exceeds max depth {}",
                    self.current_depth + 1, self.max_depth
                )));
            }
        }
        Ok(())
    }

    // Internal function called by create_tag after reading compound/list
    pub(crate) fn decrease_depth(&mut self) {
        if self.max_depth > 0 {
            self.current_depth = self.current_depth.checked_sub(1)
                .expect("Depth underflow, decrease called without increase");
        }
    }
}