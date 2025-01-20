// SPDX-FileCopyrightText: Copyright © 2025 Serpent OS Developers
//
// SPDX-License-Identifier: MPL-2.0

//! SCSI device enumeration and handling
//!
//! OK. Not quite true. Per modern conventions, all libata devices are also considered SCSI devices.
//! This means all `/dev/sd*` devices.

use std::{fs, io};

use crate::{Disk, SYSFS_DIR};

pub fn enumerate() -> io::Result<Vec<Disk>> {
    // Filtered list of SCSI devices whose paths begin with "sd" but not ending with a digit
    let items = fs::read_dir(SYSFS_DIR)?
        .filter_map(Result::ok)
        .filter_map(|e| Some(e.file_name().to_str()?.to_owned()))
        .filter(|e| e.starts_with("sd") && e[2..].chars().all(char::is_alphabetic))
        .map(Disk::from_sysfs_block_name)
        .collect();
    Ok(items)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enumerate() {
        let devices = enumerate().expect("Failed to enumerate SCSI devices");
        eprintln!("scsi devices: {devices:?}");
    }
}
