// SPDX-FileCopyrightText: Copyright © 2025 Serpent OS Developers
//
// SPDX-License-Identifier: MPL-2.0

//! NVME device enumeration and handling
//!
//! This module provides functionality to enumerate and handle NVME devices.

use std::{fs, io};

use regex::Regex;

use crate::{Disk, SYSFS_DIR};

pub fn enumerate() -> io::Result<Vec<Disk>> {
    // Filter for NVME block devices in format nvmeXnY where X and Y are digits
    // Exclude partitions (nvmeXnYpZ) and character devices
    let nvme_pattern = Regex::new(r"^nvme\d+n\d+$").unwrap();

    let items = fs::read_dir(SYSFS_DIR)?
        .filter_map(Result::ok)
        .filter_map(|e| Some(e.file_name().to_str()?.to_owned()))
        .filter(|name| nvme_pattern.is_match(name))
        .map(Disk::from_sysfs_block_name)
        .collect();
    Ok(items)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enumerate() {
        let devices = enumerate().expect("failed to collect nvme disks");
        eprintln!("nvme devices: {devices:?}");
        for device in devices.iter() {
            let mut size = device.size_in_bytes() as f64;
            size /= 1024.0 * 1024.0 * 1024.0;
            // Cheeky emulation of `fdisk -l` output
            eprintln!(
                "Disk /dev/{}: {:.2} GiB, {} bytes, {} sectors",
                device.name,
                size,
                device.size_in_bytes(),
                device.sectors
            );
        }
    }
}
