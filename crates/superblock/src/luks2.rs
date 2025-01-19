// SPDX-FileCopyrightText: Copyright © 2025 Serpent OS Developers
//
// SPDX-License-Identifier: MPL-2.0

//! # LUKS2 superblock support
//!
//! This module provides functionality for reading and parsing LUKS2 (Linux Unified Key Setup 2)
//! superblocks and their associated metadata.
//!
//! LUKS2 is a disk encryption format that uses the dm-crypt subsystem. It stores metadata
//! like encryption parameters, key slots and segment information in JSON format.
//!

mod config;
mod superblock;

pub use config::*;
pub use superblock::*;

#[cfg(test)]
mod tests {

    use std::fs;

    use crate::{luks2, Superblock};

    #[test_log::test]
    fn test_basic() {
        let mut fi = fs::File::open("tests/luks+ext4.img.zst").expect("cannot open luks2 img");
        let mut stream = zstd::stream::Decoder::new(&mut fi).expect("Unable to decode stream");
        let sb = luks2::from_reader(&mut stream).expect("Cannot parse superblock");
        assert_eq!(sb.uuid().unwrap(), "be373cae-2bd1-4ad5-953f-3463b2e53e59");
        assert_eq!(sb.version.get(), 2);
        // Try reading the JSON config
        let config = sb.read_config(&mut stream).expect("Cannot read LUKS2 config");
        // pretty print as json
        eprintln!("{}", serde_json::to_string_pretty(&config).unwrap());
        assert!(config.config.json_size > 0);
        assert!(config.config.keyslots_size > 0);

        let keyslot = config.keyslots.get(&0).unwrap();
        assert_eq!(keyslot.area.encryption, "aes-xts-plain64");
    }
}
