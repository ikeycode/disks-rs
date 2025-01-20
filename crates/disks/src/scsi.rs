// SPDX-FileCopyrightText: Copyright © 2025 Serpent OS Developers
//
// SPDX-License-Identifier: MPL-2.0

//! SCSI device enumeration and handling.
//!
//! In modern Linux systems, all libata devices are exposed as SCSI devices through
//! the SCSI subsystem. This module handles enumeration and management of these devices,
//! which appear as `/dev/sd*` block devices.

use std::path::Path;

use crate::{BasicDisk, DiskInit};

/// Represents a SCSI disk device.
///
/// This struct wraps a BasicDisk to provide SCSI-specific functionality.
#[derive(Debug)]
pub struct Disk {
    pub(crate) disk: BasicDisk,
}

impl DiskInit for Disk {
    /// Creates a new Disk instance from a sysfs path if the device name matches SCSI naming pattern.
    ///
    /// # Arguments
    ///
    /// * `sysroot` - The root path of the sysfs filesystem
    /// * `name` - The device name to check (e.g. "sda", "sdb")
    ///
    /// # Returns
    ///
    /// * `Some(Disk)` if the name matches SCSI pattern (starts with "sd" followed by letters)
    /// * `None` if the name doesn't match or the device can't be initialized
    fn from_sysfs_path(sysroot: &Path, name: &str) -> Option<Self> {
        let matching = name.starts_with("sd") && name[2..].chars().all(char::is_alphabetic);
        if matching {
            Some(Self {
                disk: BasicDisk::from_sysfs_path(sysroot, name)?,
            })
        } else {
            None
        }
    }
}
