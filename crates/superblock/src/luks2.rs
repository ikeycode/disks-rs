// SPDX-FileCopyrightText: Copyright © 2025 Serpent OS Developers
//
// SPDX-License-Identifier: MPL-2.0

//! # LUKS2 superblock support
//!
//! This module provides functionality for reading and parsing LUKS2 (Linux Unified Key Setup 2)
//! superblocks and their associated metadata.
//!
//! LUKS2 is a disk encryption format that uses the dm-crypt subsystem. It stores metadata
//! like encryption parameters, key slots and segment information in JSON format.
//!

mod config;
mod superblock;

pub use config::*;
pub use superblock::*;
