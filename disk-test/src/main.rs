// SPDX-FileCopyrightText: Copyright © 2025 Serpent OS Developers
//
// SPDX-License-Identifier: MPL-2.0

use std::fs;

use disks::BlockDevice;
use partitioning::{loopback, sparsefile};

// Demo of various disk APIs, enumeration, loopback and sparse.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // create 35GB img, attach loopback
    sparsefile::create("hello.world", 35 * 1024 * 1024 * 1024)?;
    let device = loopback::LoopDevice::create()?;
    device.attach("hello.world")?;
    eprintln!("loop device: {}", &device.path);

    // discover all loop devices
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

    // print loop devices
    for loop_device in loop_devices {
        if let Some(file) = loop_device.file_path() {
            if let Some(disk) = loop_device.disk() {
                println!(
                    "Loopback device: {} (backing file: {})",
                    loop_device.name(),
                    file.display()
                );
                println!("└─Disk: {} ({})", disk.name(), disk.model().unwrap_or("Unknown"));
                for partition in disk.partitions() {
                    println!("  ├─{} {partition}", partition.name);
                }
            }
        }
    }

    // detach loopback, remove img
    device.detach()?;
    fs::remove_file("hello.world")?;

    Ok(())
}
