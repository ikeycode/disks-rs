// SPDX-FileCopyrightText: Copyright © 2025 Serpent OS Developers
//
// SPDX-License-Identifier: MPL-2.0

//! BTRFS superblock handling
//! TODO: Add full representation of the superblock to allow us to
//! discover volumes and the root label.

use crate::{Error, Kind, Superblock};
use log;
use std::
    io::{self, Read}
;
use uuid::Uuid;
use zerocopy::*;

/// BTRFS superblock definition (as seen in the kernel)
/// This is a PARTIAL representation that matches only the
/// first 72 bytes, verifies the magic, and permits extraction
/// of the UUID
#[derive(FromBytes, Debug)]
#[repr(C)]
pub struct Btrfs {
    csum: [u8; 32],
    fsid: [u8; 16],
    bytenr: U64<LittleEndian>,
    flags: U64<LittleEndian>,
    magic: U64<LittleEndian>,
    generation: U64<LittleEndian>,
    root: U64<LittleEndian>,
    chunk_root: U64<LittleEndian>,
    log_root: U64<LittleEndian>,
}

// Superblock starts at 65536 for btrfs.
const START_POSITION: u64 = 0x10000;

// "_BHRfS_M"
const MAGIC: U64<LittleEndian> = U64::new(0x4D5F53665248425F);

/// Attempt to decode the Superblock from the given read stream
pub fn from_reader<R: Read>(reader: &mut R) -> Result<Btrfs, Error> {
    // Drop unwanted bytes (Seek not possible with zstd streamed inputs)
    io::copy(&mut reader.by_ref().take(START_POSITION), &mut io::sink())?;

    let data = Btrfs::read_from_io(reader).map_err(|_| Error::InvalidSuperblock)?;

    if data.magic != MAGIC {
        Err(Error::InvalidMagic)
    } else {
        log::trace!("valid magic field: UUID={}", data.uuid()?);
        Ok(data)
    }
}

impl Superblock for Btrfs {
    /// Return the encoded UUID for this superblock
    fn uuid(&self) -> Result<String, Error> {
        Ok(Uuid::from_bytes(self.fsid).hyphenated().to_string())
    }

    fn kind(&self) -> Kind {
        super::Kind::Btrfs
    }

    /// We don't yet support labels here.
    fn label(&self) -> Result<String, Error> {
        Err(Error::UnsupportedFeature)
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use crate::{btrfs::from_reader, Superblock};

    #[test_log::test]
    fn test_basic() {
        let mut fi = fs::File::open("tests/btrfs.img.zst").expect("cannot open ext4 img");
        let mut stream = zstd::stream::Decoder::new(&mut fi).expect("Unable to decode stream");
        let sb = from_reader(&mut stream).expect("Cannot parse superblock");
        assert_eq!(sb.uuid().unwrap(), "829d6a03-96a5-4749-9ea2-dbb6e59368b2");
    }
}
