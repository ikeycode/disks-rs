// SPDX-FileCopyrightText: Copyright © 2025 Serpent OS Developers
//
// SPDX-License-Identifier: MPL-2.0

//! MMC device enumeration and handling
//!
//! This module provides functionality to enumerate and handle MMC (MultiMediaCard)
//! storage devices by parsing sysfs paths and device names.

use crate::{BasicDisk, DiskInit};
use regex::Regex;
use std::{ops::Deref, path::Path, sync::OnceLock};

/// Regex pattern to match valid MMC device names (e.g. mmcblk0)
static MMC_PATTERN: OnceLock<Regex> = OnceLock::new();

/// Represents an MMC disk device
#[derive(Debug)]
pub struct Disk(pub BasicDisk);

impl Deref for Disk {
    type Target = BasicDisk;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DiskInit for Disk {
    /// Creates a new MMC disk from a sysfs path and device name
    ///
    /// # Arguments
    /// * `sysroot` - The sysfs root path
    /// * `name` - The device name to check
    ///
    /// # Returns
    /// * `Some(Disk)` if the device name matches MMC pattern
    /// * `None` if name doesn't match or basic disk creation fails
    fn from_sysfs_path(sysroot: &Path, name: &str) -> Option<Self> {
        let regex =
            MMC_PATTERN.get_or_init(|| Regex::new(r"^mmcblk\d+$").expect("Failed to initialise known-working regex"));
        if regex.is_match(name) {
            Some(Self(BasicDisk::from_sysfs_path(sysroot, name)?))
        } else {
            None
        }
    }
}
