// SPDX-FileCopyrightText: Copyright © 2025 Serpent OS Developers
//
// SPDX-License-Identifier: MPL-2.0

//! Helper functions for interacting with Linux sysfs interfaces

use std::{fs, path::Path, str::FromStr};

/// Reads a value from a sysfs node and attempts to parse it to type T
///
/// # Arguments
///
/// * `node` - Fully qualified path to specific sysfs node
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
pub(crate) fn read<T>(node: &Path, key: &str) -> Option<T>
where
    T: FromStr,
{
    fs::read_to_string(node.join(key)).ok()?.trim().parse().ok()
}
