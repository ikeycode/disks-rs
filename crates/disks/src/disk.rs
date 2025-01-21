// SPDX-FileCopyrightText: Copyright © 2025 Serpent OS Developers
//
// SPDX-License-Identifier: MPL-2.0

use core::fmt;
use std::{
    fs,
    ops::Deref,
    path::{Path, PathBuf},
};

use crate::{mmc, nvme, partition::Partition, scsi, sysfs, DEVFS_DIR};

/// Represents the type of disk device.
#[derive(Debug)]
pub enum Disk {
    /// SCSI disk device (e.g. sda, sdb)
    Scsi(scsi::Disk),
    /// MMC disk device (e.g. mmcblk0)
    Mmc(mmc::Disk),
    /// NVMe disk device (e.g. nvme0n1)
    Nvme(nvme::Disk),
}

impl Deref for Disk {
    type Target = BasicDisk;

    // Let scsi and nvme disks deref to BasicDisk
    fn deref(&self) -> &Self::Target {
        match self {
            Disk::Mmc(disk) => disk,
            Disk::Nvme(disk) => disk,
            Disk::Scsi(disk) => disk,
        }
    }
}

/// A basic disk representation containing common attributes shared by all disk types.
/// This serves as the base structure that specific disk implementations build upon.
#[derive(Debug)]
pub struct BasicDisk {
    /// Device name (e.g. sda, nvme0n1)
    name: String,
    /// Total number of sectors on the disk
    sectors: u64,
    /// Path to the device in /dev
    device: PathBuf,
    /// Optional disk model name
    model: Option<String>,
    /// Optional disk vendor name
    vendor: Option<String>,
    /// Partitions
    partitions: Vec<Partition>,
}

impl fmt::Display for Disk {
    // forward Display to BasicDisk
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.deref().fmt(f)
    }
}

impl fmt::Display for BasicDisk {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let bytes = self.size();
        let gib = bytes as f64 / 1_073_741_824.0;

        write!(f, "{} ({:.2} GiB)", self.name(), gib)?;

        if let Some(vendor) = self.vendor() {
            write!(f, " - {}", vendor)?;
        }

        if let Some(model) = self.model() {
            write!(f, " {}", model)?;
        }

        Ok(())
    }
}

impl BasicDisk {
    /// Returns the name of the disk device.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the partitions on the disk.
    pub fn partitions(&self) -> &[Partition] {
        &self.partitions
    }

    /// Returns the path to the disk device in dev.
    pub fn device_path(&self) -> &Path {
        &self.device
    }

    /// Returns the total number of sectors on the disk.
    pub fn sectors(&self) -> u64 {
        self.sectors
    }

    /// Returns the size of the disk in bytes.
    pub fn size(&self) -> u64 {
        self.sectors() * 512
    }

    /// Returns the model name of the disk.
    pub fn model(&self) -> Option<&str> {
        self.model.as_deref()
    }

    /// Returns the vendor name of the disk.
    pub fn vendor(&self) -> Option<&str> {
        self.vendor.as_deref()
    }
}

/// Trait for initializing different types of disk devices from sysfs.
pub trait DiskInit: Sized {
    /// Creates a new disk instance by reading information from the specified sysfs path.
    ///
    /// # Arguments
    ///
    /// * `root` - The root sysfs directory path
    /// * `name` - The name of the disk device
    ///
    /// # Returns
    ///
    /// `Some(Self)` if the disk was successfully initialized, `None` otherwise
    fn from_sysfs_path(root: &Path, name: &str) -> Option<Self>;
}

impl DiskInit for BasicDisk {
    fn from_sysfs_path(sysroot: &Path, name: &str) -> Option<Self> {
        let node = sysroot.join(name);

        // Read the partitions of the disk if any
        let mut partitions: Vec<_> = fs::read_dir(&node)
            .ok()?
            .filter_map(Result::ok)
            .filter_map(|e| {
                let name = e.file_name().to_string_lossy().to_string();
                Partition::from_sysfs_path(sysroot, &name)
            })
            .collect();
        partitions.sort_by_key(|p| p.number);

        Some(Self {
            name: name.to_owned(),
            sectors: sysfs::read(sysroot, &node, "size").unwrap_or(0),
            device: PathBuf::from(DEVFS_DIR).join(name),
            model: sysfs::read(sysroot, &node, "device/model"),
            vendor: sysfs::read(sysroot, &node, "device/vendor"),
            partitions,
        })
    }
}
