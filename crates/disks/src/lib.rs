// SPDX-FileCopyrightText: Copyright © 2025 Serpent OS Developers
//
// SPDX-License-Identifier: MPL-2.0

use std::{
    fs, io,
    path::{Path, PathBuf},
};

pub mod nvme;
pub mod scsi;
mod sysfs;

const SYSFS_DIR: &str = "/sys/class/block";
const DEVFS_DIR: &str = "/dev";

/// A block device on the system which can be either a physical disk or a partition.
#[derive(Debug)]
pub enum BlockDevice {
    /// A physical disk device
    Disk(Box<Disk>),
}

/// Represents the type of disk device.
#[derive(Debug)]
pub enum Disk {
    /// SCSI disk device (e.g. sda, sdb)
    Scsi(scsi::Disk),
    /// NVMe disk device (e.g. nvme0n1)
    Nvme(nvme::Disk),
}

/// A basic disk representation containing common attributes shared by all disk types.
/// This serves as the base structure that specific disk implementations build upon.
#[derive(Debug)]
pub struct BasicDisk {
    /// Device name (e.g. sda, nvme0n1)
    pub name: String,
    /// Total number of sectors on the disk
    pub sectors: u64,
    /// Path to the device in sysfs
    pub node: PathBuf,
    /// Path to the device in /dev
    pub device: PathBuf,
    /// Optional disk model name
    pub model: Option<String>,
    /// Optional disk vendor name
    pub vendor: Option<String>,
}

impl Disk {
    /// Returns the name of the disk device.
    ///
    /// # Examples
    ///
    /// ```
    /// // Returns strings like "sda" or "nvme0n1"
    /// let name = disk.name();
    /// ```
    pub fn name(&self) -> &str {
        match self {
            Disk::Scsi(disk) => disk.name(),
            Disk::Nvme(disk) => disk.name(),
        }
    }
}

/// Trait for initializing different types of disk devices from sysfs.
pub(crate) trait DiskInit: Sized {
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
        Some(Self {
            name: name.to_owned(),
            sectors: sysfs::sysfs_read(sysroot, &node, "size").unwrap_or(0),
            device: PathBuf::from(DEVFS_DIR).join(name),
            model: sysfs::sysfs_read(sysroot, &node, "device/model"),
            vendor: sysfs::sysfs_read(sysroot, &node, "device/vendor"),
            node,
        })
    }
}

impl BlockDevice {
    /// Discovers all block devices present in the system.
    ///
    /// # Returns
    ///
    /// A vector of discovered block devices or an IO error if the discovery fails.
    ///
    /// # Examples
    ///
    /// ```
    /// let devices = BlockDevice::discover()?;
    /// for device in devices {
    ///     println!("Found device: {:?}", device);
    /// }
    /// ```
    pub fn discover() -> io::Result<Vec<BlockDevice>> {
        Self::discover_in_sysroot("/")
    }

    /// Discovers block devices in a specified sysroot directory.
    ///
    /// # Arguments
    ///
    /// * `sysroot` - Path to the system root directory
    ///
    /// # Returns
    ///
    /// A vector of discovered block devices or an IO error if the discovery fails.
    pub fn discover_in_sysroot(sysroot: impl AsRef<str>) -> io::Result<Vec<BlockDevice>> {
        let sysroot = sysroot.as_ref();
        let sysfs_dir = PathBuf::from(sysroot).join(SYSFS_DIR);
        let mut devices = Vec::new();

        // Iterate over all block devices in sysfs and collect their filenames
        let entries = fs::read_dir(&sysfs_dir)?
            .filter_map(Result::ok)
            .filter_map(|e| Some(e.file_name().to_str()?.to_owned()));

        // For all the discovered block devices, try to create a Disk instance
        // At this point we completely ignore partitions. They come later.
        for entry in entries {
            let disk = if let Some(disk) = scsi::Disk::from_sysfs_path(&sysfs_dir, &entry) {
                Disk::Scsi(disk)
            } else if let Some(disk) = nvme::Disk::from_sysfs_path(&sysfs_dir, &entry) {
                Disk::Nvme(disk)
            } else {
                continue;
            };

            devices.push(BlockDevice::Disk(Box::new(disk)));
        }

        Ok(devices)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_discover() {
        let devices = BlockDevice::discover().unwrap();
        assert!(!devices.is_empty());
        eprintln!("devices: {devices:?}");
    }
}
