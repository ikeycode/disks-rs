// SPDX-FileCopyrightText: Copyright © 2025 Serpent OS Developers
//
// SPDX-License-Identifier: MPL-2.0

use log::{debug, info};
use std::{fs, path::Path};

use disks::BlockDevice;
use partitioning::{
    gpt::{disk::LogicalBlockSize, mbr::ProtectiveMBR, partition_types, GptConfig},
    loopback,
    planner::{format_size, Planner},
    sparsefile,
    strategy::{AllocationStrategy, PartitionRequest, SizeRequirement, Strategy},
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
    info!("Creating protective MBR for disk of size {} bytes", disk_size);
    let mut gpt_device = fs::File::options().write(true).open(&path)?;
    let lb_size = (disk_size / 512) as u32;
    let lb_size = lb_size.saturating_sub(1); // subtract 1 for the MBR
    let mbr = ProtectiveMBR::with_lb_size(lb_size);
    mbr.overwrite_lba0(&mut gpt_device)?;
    info!("Successfully created protective MBR at {:?}", path.as_ref());
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
    let path = path.as_ref();
    info!("Creating default GPT partition scheme on {:?}", path);

    // Configure and create GPT disk
    let gpt_config = GptConfig::new()
        .writable(true)
        .logical_block_size(LogicalBlockSize::Lb512);

    let mut gpt_disk = gpt_config.create(&path)?;
    gpt_disk.write_inplace()?;

    eprintln!("GPT: {:?}", gpt_disk);

    let first_usable = gpt_disk.header().first_usable * 512;
    let last_usable = gpt_disk.header().last_usable * 512;

    // Connect the planner.
    let disk = disks::loopback::Device::from_device_path(path).unwrap();
    let block = BlockDevice::loopback_device(disk);
    let mut planner = Planner::new(block)
        .with_start_offset(first_usable)
        .with_end_offset(last_usable);
    let mut strategy = Strategy::new(AllocationStrategy::InitializeWholeDisk);

    // efi
    strategy.add_request(PartitionRequest {
        size: SizeRequirement::Range {
            min: 256 * 1024 * 1024,
            max: 1 * 1024 * 1024 * 1024,
        },
    });
    // xbootldr
    strategy.add_request(PartitionRequest {
        size: SizeRequirement::Range {
            min: 2 * 1024 * 1024 * 1024,
            max: 4 * 1024 * 1024 * 1024,
        },
    });
    // swap
    strategy.add_request(PartitionRequest {
        size: SizeRequirement::Range {
            min: 1 * 1024 * 1024 * 1024,
            max: 4 * 1024 * 1024 * 1024,
        },
    });
    // root
    strategy.add_request(PartitionRequest {
        size: SizeRequirement::Range {
            min: 30 * 1024 * 1024 * 1024,
            max: 120 * 1024 * 1024 * 1024,
        },
    });
    // home
    strategy.add_request(PartitionRequest {
        size: SizeRequirement::AtLeast(50 * 1024 * 1024 * 1024),
    });
    info!("Applying strategy: {}", strategy.describe());
    strategy.apply(&mut planner)?;
    info!("Computed: {}", planner.describe_changes());

    // TODO: Track the types in the API and use them here
    for (n, partition) in planner.current_layout().iter().enumerate() {
        info!("Adding partition: {:?}", &partition);
        assert_ne!(0, partition.size());
        let size = partitioning::planner::format_size(partition.size());
        info!("Partition {} size: {}, at {}", n, size, format_size(partition.start));
        gpt_disk.add_partition_at(
            "",
            n as u32 + 1,
            partition.start / 512,
            partition.size() / 512,
            partition_types::LINUX_FS,
            0,
        )?;
    }

    let _ = gpt_disk.write()?;

    info!("Successfully created partition scheme");
    Ok(())
}

/// Demonstrates usage of disk APIs including:
/// - Creating sparse files
/// - Setting up loopback devices
/// - Partitioning with GPT
/// - Enumerating block devices
fn main() -> Result<(), Box<dyn std::error::Error>> {
    pretty_env_logger::formatted_timed_builder()
        .filter_level(log::LevelFilter::Debug)
        .init();
    info!("Starting disk partitioning demo");

    // Create 100GB sparse image file and attach to loopback device
    let image_size = 100 * 1024 * 1024 * 1024;
    info!("Creating {}GB sparse image file", image_size / (1024 * 1024 * 1024));
    sparsefile::create("hello.world", image_size)?;

    info!("Setting up loopback device");
    let device = loopback::LoopDevice::create()?;
    device.attach("hello.world")?;
    info!("Loop device created at: {}", &device.path);

    // Initialize disk with protective MBR and partition scheme
    create_protective_mbr(image_size, &device.path)?;
    create_default_partition_scheme(&device.path)?;

    // Notify kernel of partition table changes
    debug!("Syncing partition table changes");
    blkpg::sync_gpt_partitions(&device.path)?;

    // Get list of all loopback devices
    debug!("Discovering block devices");
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
    info!("Device information:");
    for loop_device in loop_devices {
        if let Some(file) = loop_device.file_path() {
            if let Some(disk) = loop_device.disk() {
                info!(
                    "Loopback device: {} (backing file: {})",
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
    info!("Cleaning up resources");
    device.detach()?;
    //fs::remove_file("hello.world")?;

    info!("Demo completed successfully");
    Ok(())
}
