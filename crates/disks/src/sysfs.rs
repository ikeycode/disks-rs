// SPDX-FileCopyrightText: Copyright © 2025 Serpent OS Developers
//
// SPDX-License-Identifier: MPL-2.0

//! Helper functions for interacting with Linux sysfs interfaces

use std::{fs, path::Path, str::FromStr};

/// Reads a value from a sysfs node and attempts to parse it to type T
///
/// # Arguments
///
/// * `sysroot` - Base path of the sysfs mount point
/// * `node` - Path to specific sysfs node relative to sysroot
/// * `key` - Name of the sysfs attribute to read
///
/// # Returns
///
/// * `Some(T)` if the value was successfully read and parsed
/// * `None` if the file could not be read or parsed
///
/// # Type Parameters
///
/// * `T` - Target type that implements FromStr for parsing the raw value
pub(crate) fn sysfs_read<T>(sysroot: &Path, node: &Path, key: &str) -> Option<T>
where
    T: FromStr,
{
    let path = sysroot.join(node).join(key);
    fs::read_to_string(&path).ok()?.trim().parse().ok()
}
