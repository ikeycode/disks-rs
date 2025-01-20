// SPDX-FileCopyrightText: Copyright © 2025 Serpent OS Developers
//
// SPDX-License-Identifier: MPL-2.0

use std::{fs, path::PathBuf};

pub mod nvme;
pub mod scsi;

const SYSFS_DIR: &str = "/sys/class/block";

#[derive(Debug)]
pub struct Disk {
    /// Partial-name, ie "sda"
    pub name: String,

    // Number of sectors (* 512 sector size for data size)
    pub sectors: u64,
}

impl Disk {
    fn from_sysfs_block_name(name: impl AsRef<str>) -> Self {
        let name = name.as_ref().to_owned();
        let entry = PathBuf::from(SYSFS_DIR).join(&name);

        // Determine number of blocks
        let block_file = entry.join("size");
        let sectors = fs::read_to_string(block_file)
            .ok()
            .and_then(|s| s.trim().parse::<u64>().ok())
            .unwrap_or(0);

        Self { name, sectors }
    }

    /// Return usable size
    /// TODO: Grab the block size from the system. We know Linux is built on 512s though.
    pub fn size_in_bytes(&self) -> u64 {
        self.sectors * 512
    }
}
