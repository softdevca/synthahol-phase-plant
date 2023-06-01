//! Phase Plant preset reading and writing.
//!
//! Functionality for reading and writing presets should be confined to this module where possible.
//!
//! # File Format
//!
//! The file format changed in some ways between Phase Plant 2.0.11 and 2.0.12.
//! In particular there is an additional 256 bytes in the init preset starting
//! at position 19055.

use std::mem::size_of;

use serde::{Deserialize, Serialize};

pub(crate) use generators::GeneratorBlock;
pub(crate) use modulators::ModulatorBlock;

pub use self::effects::*;
pub use self::read::*;
pub use self::write::*;

pub(crate) mod effects;
mod generators;
mod modulators;
mod read;
mod write;

#[derive(Debug, Serialize, Deserialize)]
struct MetadataJson {
    pub description: Option<String>,
    pub author: Option<String>,
}

#[derive(Debug)]
pub struct DataBlockHeader {
    /// Size of the block, not including the header and mode ID.
    data_length: usize,
    is_used: bool,
    mode_id: Option<u32>,
}

impl DataBlockHeader {
    pub const UNUSED_BLOCK_HEADER_SIZE: usize = size_of::<u32>() + size_of::<u8>();
    pub const USED_BLOCK_HEADER_SIZE: usize = Self::UNUSED_BLOCK_HEADER_SIZE + size_of::<u32>();

    /// * `data_length` - size of the block, not including the header and optional mode
    fn new(data_length: usize, is_used: bool, mode_id: Option<u32>) -> Self {
        Self {
            data_length,
            is_used,
            mode_id,
        }
    }

    fn new_unused() -> Self {
        Self {
            data_length: 0,
            is_used: false,
            mode_id: None,
        }
    }

    /// * `data_length` - size of the block, not including the header mode
    fn new_used(data_length: usize, mode_id: u32) -> Self {
        Self {
            data_length,
            is_used: true,
            mode_id: Some(mode_id),
        }
    }

    fn data_length_with_header(&self) -> usize {
        self.data_length
            + if self.is_used {
                Self::USED_BLOCK_HEADER_SIZE
            } else {
                Self::UNUSED_BLOCK_HEADER_SIZE
            }
    }

    pub(crate) fn is_used(&self) -> bool {
        self.is_used
    }

    /// Will be `None` if the block is not used.
    pub(crate) fn mode_id(&self) -> Option<u32> {
        self.mode_id
    }
}
