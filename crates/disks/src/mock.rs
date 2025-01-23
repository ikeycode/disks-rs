// SPDX-FileCopyrightText: Copyright © 2025 Serpent OS Developers
//
// SPDX-License-Identifier: MPL-2.0

//! Mock disk device for testing.
//!
//! This module provides a mock disk implementation that can be used for testing
//! disk-related functionality without requiring actual hardware devices.

use std::{ops::Deref, path::PathBuf};

use crate::{partition::Partition, BasicDisk};

/// Represents a mock disk device.
///
/// This struct wraps a BasicDisk to provide mock functionality for testing.
#[derive(Debug)]
pub struct MockDisk(pub BasicDisk);

impl Deref for MockDisk {
    type Target = BasicDisk;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl MockDisk {
    /// Creates a new mock disk with the specified size in bytes
    pub fn new(size_bytes: u64) -> Self {
        let sectors = size_bytes / 512;
        let disk = BasicDisk {
            name: "mock0".to_string(),
            sectors,
            device: PathBuf::from("/dev/mock0"),
            model: Some("Mock Device".to_string()),
            vendor: Some("Mock Vendor".to_string()),
            partitions: Vec::new(),
        };
        Self(disk)
    }

    /// Add a partition to the mock disk at the specified byte offsets
    pub fn add_partition(&mut self, start_bytes: u64, end_bytes: u64) {
        let partition_number = self.0.partitions().len() + 1;
        let start = start_bytes / 512;
        let end = end_bytes / 512;

        let partition = Partition {
            number: partition_number as u32,
            start,
            end,
            size: end - start,
            name: format!("mock0p{}", partition_number),
            node: PathBuf::from("/sys/class/block/mock0/mock0p1"),
            device: PathBuf::from(format!("/dev/mock0p{}", partition_number)),
        };

        self.0.partitions_mut().push(partition);
    }
}
