// SPDX-FileCopyrightText: Copyright © 2025 Serpent OS Developers
//
// SPDX-License-Identifier: MPL-2.0

use log::info;
use std::{fs, io, path::Path};

/// Creates a sparse file at the specified path with the given size.
///
/// # Arguments
/// * `path` - Path where the sparse file should be created
/// * `size` - Size in bytes for the sparse file
///
/// # Returns
/// `io::Result<()>` indicating success or failure
pub fn create<P>(path: P, size: u64) -> io::Result<()>
where
    P: AsRef<Path>,
{
    info!("🗂️  Creating sparse file at {:?}", path.as_ref());

    let file = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&path)?;

    info!("📝 Setting file size to {} bytes", size);
    file.set_len(size)?;

    info!("✅ Successfully created sparse file");
    Ok(())
}
