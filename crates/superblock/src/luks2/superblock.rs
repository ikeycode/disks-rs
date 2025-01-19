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

use std::{io::Read, ops::Sub};

use crate::{Error, Kind, Superblock};
use log;
use zerocopy::*;

use super::Luks2Config;

const MAGIC_LEN: usize = 6;
const LABEL_LEN: usize = 48;
const CHECKSUM_ALG_LEN: usize = 32;
const SALT_LEN: usize = 64;
const UUID_LEN: usize = 40;
const CHECKSUM_LEN: usize = 64;

/// Per the `cryptsetup` docs for dm-crypt backed LUKS2, header is at first byte.
#[derive(FromBytes, Unaligned, Debug)]
#[repr(C, packed)]
pub struct Luks2 {
    pub magic: [u8; MAGIC_LEN],
    pub version: U16<BigEndian>,
    pub hdr_size: U64<BigEndian>,
    pub seqid: U64<BigEndian>,
    pub label: [u8; LABEL_LEN],
    pub checksum_alg: [u8; CHECKSUM_ALG_LEN],
    pub salt: [u8; SALT_LEN],
    pub uuid: [u8; UUID_LEN],
    pub subsystem: [u8; LABEL_LEN],
    pub hdr_offset: U64<BigEndian>,
    pub padding: [u8; 184],
    pub csum: [u8; CHECKSUM_LEN],
    pub padding4096: [u8; 7 * 512],
}

struct Magic;

// LUKS2 and SKUL2 are the two valid magic fields. Guess BigEndian came later?
impl Magic {
    const LUKS2: [u8; MAGIC_LEN] = [b'L', b'U', b'K', b'S', 0xba, 0xbe];
    const SKUL2: [u8; MAGIC_LEN] = [b'S', b'K', b'U', b'L', 0xba, 0xbe];
}

/// Attempt to decode the Superblock from the given read stream
pub fn from_reader<R: Read>(reader: &mut R) -> Result<Luks2, Error> {
    let data = Luks2::read_from_io(reader).map_err(|_| Error::InvalidSuperblock)?;

    match data.magic {
        Magic::LUKS2 | Magic::SKUL2 => {
            log::trace!(
                "valid magic field: UUID={} [volume label: \"{}\"]",
                data.uuid()?,
                data.label().unwrap_or_else(|_| "[invalid utf8]".into())
            );
            Ok(data)
        }
        _ => Err(Error::InvalidMagic),
    }
}

impl Superblock for Luks2 {
    fn kind(&self) -> Kind {
        Kind::LUKS2
    }

    /// NOTE: LUKS2 stores string UUID rather than 128-bit sequence..
    fn uuid(&self) -> Result<String, crate::Error> {
        Ok(std::str::from_utf8(&self.uuid)?.trim_end_matches('\0').to_owned())
    }

    /// NOTE: Label is often empty, set in config instead...
    fn label(&self) -> Result<String, crate::Error> {
        Ok(std::str::from_utf8(&self.label)?.trim_end_matches('\0').to_owned())
    }
}

impl Luks2 {
    /// Read and parse the JSON configuration areas from the LUKS2 header
    pub fn read_config<R: Read>(&self, reader: &mut R) -> Result<Luks2Config, Error> {
        let mut json_data = vec![0u8; self.hdr_size.get().sub(4096) as usize];
        reader.read_exact(&mut json_data)?;

        // clip the json_data at the first nul byte
        let raw_input = std::str::from_utf8(&json_data)?.trim_end_matches('\0');
        match serde_json::from_str(raw_input) {
            Ok(config) => Ok(config),
            Err(e) => {
                eprintln!("Error: {:?}", e);
                Err(Error::InvalidSuperblock)
            }
        }
    }
}
