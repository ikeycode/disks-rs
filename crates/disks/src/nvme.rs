// SPDX-FileCopyrightText: Copyright © 2025 Serpent OS Developers
//
// SPDX-License-Identifier: MPL-2.0

//! NVME device enumeration and handling
//!
//! This module provides functionality to enumerate and handle NVMe (Non-Volatile Memory Express)
//! storage devices by parsing sysfs paths and device names.

use crate::{BasicDisk, DiskInit};
use regex::Regex;
use std::{path::Path, sync::OnceLock};

/// Regex pattern to match valid NVMe device names (e.g. nvme0n1)
static NVME_PATTERN: OnceLock<Regex> = OnceLock::new();

/// Represents an NVMe disk device
#[derive(Debug)]
pub struct Disk {
    /// The underlying basic disk implementation
    pub(crate) disk: BasicDisk,
}

impl DiskInit for Disk {
    /// Creates a new NVMe disk from a sysfs path and device name
    ///
    /// # Arguments
    /// * `sysroot` - The sysfs root path
    /// * `name` - The device name to check
    ///
    /// # Returns
    /// * `Some(Disk)` if the device name matches NVMe pattern
    /// * `None` if name doesn't match or basic disk creation fails
    fn from_sysfs_path(sysroot: &Path, name: &str) -> Option<Self> {
        let regex = NVME_PATTERN
            .get_or_init(|| Regex::new(r"^nvme\d+n\d+$").expect("Failed to initialise known-working regex"));
        if regex.is_match(name) {
            Some(Self {
                disk: BasicDisk::from_sysfs_path(sysroot, name)?,
            })
        } else {
            None
        }
    }
}
