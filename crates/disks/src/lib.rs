// SPDX-FileCopyrightText: Copyright © 2025 Serpent OS Developers
//
// SPDX-License-Identifier: MPL-2.0

mod disk;
use std::{fs, io, path::PathBuf};

pub use disk::*;
pub mod nvme;
pub mod partition;
pub mod scsi;
mod sysfs;

const SYSFS_DIR: &str = "/sys/class/block";
const DEVFS_DIR: &str = "/dev";

/// A block device on the system which can be either a physical disk or a partition.
#[derive(Debug)]
pub enum BlockDevice {
    /// A physical disk device
    Disk(Box<Disk>),
    Unknown,
}

impl BlockDevice {
    /// Discovers all block devices present in the system.
    ///
    /// # Returns
    ///
    /// A vector of discovered block devices or an IO error if the discovery fails.
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
        for device in &devices {
            if let BlockDevice::Disk(disk) = device {
                println!("{}: {disk}", disk.name());
                for partition in disk.partitions() {
                    println!("├─{} {partition}", partition.name);
                }
            }
        }
    }
}
