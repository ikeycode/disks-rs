// SPDX-FileCopyrightText: Copyright © 2025 Serpent OS Developers
//
// SPDX-License-Identifier: MPL-2.0

/// Provides functionality for managing block device partitions
pub mod loopback;
pub mod sparsefile;

use disks::{BasicDisk, DiskInit};
use log::{debug, error, info, warn};
use std::{
    fs::File,
    io,
    os::fd::{AsFd, AsRawFd},
    path::{Path, PathBuf},
};
use thiserror::Error;

pub use gpt;
use linux_raw_sys::ioctl::BLKPG;
use nix::libc;

/// Errors that can occur during partition operations
#[derive(Error, Debug)]
pub enum Error {
    /// IO operation error
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    /// GPT-specific error
    #[error("GPT error: {0}")]
    Gpt(#[from] gpt::GptError),
}

/// Represents a block device partition for IOCTL operations
#[repr(C)]
struct BlkpgPartition {
    start: i64,
    length: i64,
    pno: i32,
    devname: [u8; 64],
    volname: [u8; 64],
}

/// IOCTL structure for partition operations
#[repr(C)]
struct BlkpgIoctl {
    op: i32,
    flags: i32,
    datalen: i32,
    data: *mut BlkpgPartition,
}

const BLKPG_ADD_PARTITION: i32 = 1;
const BLKPG_DEL_PARTITION: i32 = 2;

/// Adds a new partition to the specified block device
///
/// # Arguments
/// * `fd` - File descriptor for the block device
/// * `partition_number` - Number to assign to the new partition
/// * `start` - Starting offset in bytes
/// * `length` - Length of partition in bytes
///
/// # Returns
/// `io::Result<()>` indicating success or failure
pub(crate) fn add_partition<F>(fd: F, partition_number: i32, start: i64, length: i64) -> io::Result<()>
where
    F: AsRawFd,
{
    info!(
        "➕ Adding partition {} (start: {}, length: {})",
        partition_number, start, length
    );
    let mut part = BlkpgPartition {
        start,
        length,
        pno: partition_number,
        devname: [0; 64],
        volname: [0; 64],
    };

    let mut ioctl = BlkpgIoctl {
        op: BLKPG_ADD_PARTITION,
        flags: 0,
        datalen: std::mem::size_of::<BlkpgPartition>() as i32,
        data: &mut part,
    };

    let res = unsafe { libc::ioctl(fd.as_raw_fd(), BLKPG as _, &mut ioctl) };
    if res < 0 {
        let err = io::Error::last_os_error();
        error!("❌ Failed to add partition: {}", err);
        return Err(err);
    }
    info!("✅ Successfully added partition {}", partition_number);
    Ok(())
}

/// Deletes a partition from the specified block device
///
/// # Arguments
/// * `fd` - File descriptor for the block device
/// * `partition_number` - Number of the partition to delete
///
/// # Returns
/// `io::Result<()>` indicating success or failure
pub(crate) fn delete_partition<F>(fd: F, partition_number: i32) -> io::Result<()>
where
    F: AsRawFd,
{
    warn!("🗑️ Attempting to delete partition {}", partition_number);
    let mut part = BlkpgPartition {
        start: 0,
        length: 0,
        pno: partition_number,
        devname: [0; 64],
        volname: [0; 64],
    };

    let mut ioctl = BlkpgIoctl {
        op: BLKPG_DEL_PARTITION,
        flags: 0,
        datalen: std::mem::size_of::<BlkpgPartition>() as i32,
        data: &mut part,
    };

    let res = unsafe { libc::ioctl(fd.as_raw_fd(), BLKPG as _, &mut ioctl) };
    if res < 0 {
        let err = io::Error::last_os_error();
        error!("❌ Failed to delete partition {}: {}", partition_number, err);
        return Err(err);
    }
    info!("✅ Successfully deleted partition {}", partition_number);
    Ok(())
}

/// Updates kernel partition representations to match the GPT table
///
/// # Arguments
/// * `path` - Path to the block device
///
/// # Returns
/// `Result<(), Error>` indicating success or partition operation failure
pub fn sync_gpt_partitions<P: AsRef<Path>>(path: P) -> Result<(), Error> {
    info!("🔄 Syncing GPT partitions for {:?}", path.as_ref());
    let file = File::open(&path)?;

    // Read GPT table
    debug!("📖 Reading GPT table...");
    let gpt = gpt::GptConfig::new().writable(false).open(&path)?;
    let partitions = gpt.partitions();
    let block_size = 512;
    info!(
        "📊 Found {} partitions with block size {}",
        partitions.len(),
        block_size
    );

    warn!("🗑️  Deleting existing partitions...");

    // Find the disk for enumeration purposes
    let base_name = path
        .as_ref()
        .file_name()
        .ok_or(Error::Io(io::Error::from(io::ErrorKind::InvalidInput)))?
        .to_string_lossy()
        .to_string();
    let disk = BasicDisk::from_sysfs_path(&PathBuf::from("/sys/class/block"), &base_name)
        .ok_or(Error::Io(io::Error::from(io::ErrorKind::InvalidInput)))?;

    for partition in disk.partitions() {
        let _ = delete_partition(file.as_raw_fd(), partition.number as i32);
    }

    // Add partitions from GPT
    info!("➕ Adding new partitions from GPT...");
    for (i, partition) in partitions.iter() {
        add_partition(
            file.as_fd(),
            *i as i32,
            partition.first_lba as i64 * block_size,
            (partition.last_lba - partition.first_lba + 1) as i64 * block_size,
        )?;
    }

    info!("✨ GPT partition sync completed successfully");
    Ok(())
}
