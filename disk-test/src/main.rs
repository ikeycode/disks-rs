// SPDX-FileCopyrightText: Copyright © 2025 Serpent OS Developers
//
// SPDX-License-Identifier: MPL-2.0

use log::{debug, info};
use std::{fs, path::Path};

use disks::BlockDevice;
use partitioning::{
    gpt::{disk::LogicalBlockSize, mbr::ProtectiveMBR, partition_types, GptConfig},
    loopback, sparsefile,
};

use partitioning::blkpg;

/// Creates a protective MBR on the specified disk
///
/// # Arguments
///
/// * `disk_size` - Size of the disk in bytes
/// * `path` - Path to the disk device
///
/// # Returns
///
/// Result indicating success or error
fn create_protective_mbr<P>(disk_size: u64, path: P) -> Result<(), Box<dyn std::error::Error>>
where
    P: AsRef<Path>,
{
    debug!("🛡️  Creating protective MBR for disk of size {} bytes", disk_size);
    let mut gpt_device = fs::File::options().write(true).open(&path)?;
    let lb_size = (disk_size / 512) as u32;
    let lb_size = lb_size.saturating_sub(1); // subtract 1 for the MBR
    let mbr = ProtectiveMBR::with_lb_size(lb_size);
    mbr.overwrite_lba0(&mut gpt_device)?;
    info!("✅ Successfully created protective MBR at {:?}", path.as_ref());
    Ok(())
}

/// Creates a default GPT partition scheme on the disk with:
/// - 256MB EFI System Partition
/// - 2GB Boot Partition
/// - 4GB Swap Partition
/// - Remaining space as Linux filesystem
///
/// # Arguments
///
/// * `path` - Path to the disk device
///
/// # Returns
///
/// Result indicating success or error
fn create_default_partition_scheme<P>(path: P) -> Result<(), Box<dyn std::error::Error>>
where
    P: AsRef<Path>,
{
    info!("💽 Creating default GPT partition scheme on {:?}", path.as_ref());

    // Configure and create GPT disk
    let gpt_config = GptConfig::new()
        .writable(true)
        .logical_block_size(LogicalBlockSize::Lb512);

    let mut gpt_disk = gpt_config.create(&path)?;

    debug!("📝 Creating EFI System Partition (256MB)");
    gpt_disk.add_partition("", 256 * 1024 * 1024, partition_types::EFI, 0, None)?;

    debug!("📝 Creating Boot Partition (2GB)");
    gpt_disk.add_partition("", 2 * 1024 * 1024 * 1024, partition_types::FREEDESK_BOOT, 0, None)?;

    debug!("📝 Creating Swap Partition (4GB)");
    gpt_disk.add_partition("", 4 * 1024 * 1024 * 1024, partition_types::LINUX_SWAP, 0, None)?;

    // Use remaining space for root partition
    let sectors = gpt_disk.find_free_sectors();
    debug!("📊 Available sectors: {sectors:?}");
    let (_, length) = sectors.iter().find(|(_, l)| *l > 0).unwrap();
    debug!("📝 Creating Root Partition ({}MB)", (length * 512) / (1024 * 1024));
    gpt_disk.add_partition("", *length * 512, partition_types::LINUX_FS, 0, None)?;
    let _ = gpt_disk.write()?;

    info!("✅ Successfully created partition scheme");
    Ok(())
}

/// Demonstrates usage of disk APIs including:
/// - Creating sparse files
/// - Setting up loopback devices
/// - Partitioning with GPT
/// - Enumerating block devices
fn main() -> Result<(), Box<dyn std::error::Error>> {
    pretty_env_logger::formatted_builder()
        .filter_level(log::LevelFilter::Trace)
        .init();
    info!("🚀 Starting disk partitioning demo");

    // Create 35GB sparse image file and attach to loopback device
    let image_size = 35 * 1024 * 1024 * 1024;
    info!("📁 Creating {}GB sparse image file", image_size / (1024 * 1024 * 1024));
    sparsefile::create("hello.world", image_size)?;

    debug!("🔄 Setting up loopback device");
    let device = loopback::LoopDevice::create()?;
    device.attach("hello.world")?;
    info!("💫 Loop device created at: {}", &device.path);

    // Initialize disk with protective MBR and partition scheme
    create_protective_mbr(image_size, "hello.world")?;
    create_default_partition_scheme("hello.world")?;

    // Notify kernel of partition table changes
    debug!("🔄 Syncing partition table changes");
    blkpg::sync_gpt_partitions(&device.path)?;

    // Get list of all loopback devices
    info!("🔍 Discovering block devices");
    let loop_devices = BlockDevice::discover()?
        .into_iter()
        .filter_map(|device| {
            if let BlockDevice::Loopback(loop_device) = device {
                Some(loop_device)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    // Display information about discovered devices
    info!("📋 Device information:");
    for loop_device in loop_devices {
        if let Some(file) = loop_device.file_path() {
            if let Some(disk) = loop_device.disk() {
                info!(
                    "💾 Loopback device: {} (backing file: {})",
                    loop_device.name(),
                    file.display()
                );
                info!("  └─Disk: {} ({})", disk.name(), disk.model().unwrap_or("Unknown"));
                for partition in disk.partitions() {
                    info!("    ├─{} {partition}", partition.name);
                }
            }
        }
    }

    // Clean up resources
    debug!("🧹 Cleaning up resources");
    device.detach()?;
    //fs::remove_file("hello.world")?;

    info!("✨ Demo completed successfully");
    Ok(())
}
