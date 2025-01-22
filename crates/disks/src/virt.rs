// SPDX-FileCopyrightText: Copyright © 2025 Serpent OS Developers
//
// SPDX-License-Identifier: MPL-2.0

//! Virtual disk device enumeration and handling.
//!
//! In Linux systems, virtual disk devices are exposed through
//! the block subsystem. This module handles enumeration and management of these devices,
//! which appear as `/dev/vd*` block devices.

use std::{ops::Deref, path::Path};

use crate::{BasicDisk, DiskInit};

/// Represents a virtual disk device.
///
/// This struct wraps a BasicDisk to provide virtual disk-specific functionality.
#[derive(Debug)]
pub struct Disk(pub BasicDisk);

impl Deref for Disk {
    type Target = BasicDisk;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DiskInit for Disk {
    /// Creates a new Disk instance from a sysfs path if the device name matches virtual disk naming pattern.
    ///
    /// # Arguments
    ///
    /// * `sysroot` - The root path of the sysfs filesystem
    /// * `name` - The device name to check (e.g. "vda", "vdb")
    ///
    /// # Returns
    ///
    /// * `Some(Disk)` if the name matches virtual disk pattern (starts with "vd" followed by letters)
    /// * `None` if the name doesn't match or the device can't be initialized
    fn from_sysfs_path(sysroot: &Path, name: &str) -> Option<Self> {
        let matching = name.starts_with("vd") && name[2..].chars().all(char::is_alphabetic);
        if matching {
            Some(Self(BasicDisk::from_sysfs_path(sysroot, name)?))
        } else {
            None
        }
    }
}
