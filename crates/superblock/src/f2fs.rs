// SPDX-FileCopyrightText: Copyright © 2025 Serpent OS Developers
//
// SPDX-License-Identifier: MPL-2.0

//! F2FS superblock handling

use crate::{Error, Kind, Superblock};
use std::io::{self, Read};
use uuid::Uuid;
use zerocopy::*;

// Constants to allow us to move away from unsafe{} APIs
// in future, i.e. read_array(MAX_EXTENSION) ...

const MAX_VOLUME_LEN: usize = 512;
const MAX_EXTENSION: usize = 64;
const EXTENSION_LEN: usize = 8;
const VERSION_LEN: usize = 256;
const MAX_DEVICES: usize = 8;
const MAX_QUOTAS: usize = 3;
const MAX_STOP_REASON: usize = 32;
const MAX_ERRORS: usize = 16;

#[derive(Debug, FromBytes, Unaligned)]
#[repr(C, packed)]
pub struct F2FS {
    magic: U32<LittleEndian>,
    major_ver: U16<LittleEndian>,
    minor_ver: U16<LittleEndian>,
    log_sectorsize: U32<LittleEndian>,
    log_sectors_per_block: U32<LittleEndian>,
    log_blocksize: U32<LittleEndian>,
    log_blocks_per_seg: U32<LittleEndian>,
    segs_per_sec: U32<LittleEndian>,
    secs_per_zone: U32<LittleEndian>,
    checksum_offset: U32<LittleEndian>,
    block_count: U64<LittleEndian>,
    section_count: U32<LittleEndian>,
    segment_count: U32<LittleEndian>,
    segment_count_ckpt: U32<LittleEndian>,
    segment_count_sit: U32<LittleEndian>,
    segment_count_nat: U32<LittleEndian>,
    segment_count_ssa: U32<LittleEndian>,
    segment_count_main: U32<LittleEndian>,
    segment0_blkaddr: U32<LittleEndian>,
    cp_blkaddr: U32<LittleEndian>,
    sit_blkaddr: U32<LittleEndian>,
    nat_blkaddr: U32<LittleEndian>,
    ssa_blkaddr: U32<LittleEndian>,
    main_blkaddr: U32<LittleEndian>,
    root_ino: U32<LittleEndian>,
    node_ino: U32<LittleEndian>,
    meta_ino: U32<LittleEndian>,
    uuid: [u8; 16],
    volume_name: [U16<LittleEndian>; MAX_VOLUME_LEN],
    extension_count: U32<LittleEndian>,
    extension_list: [[u8; EXTENSION_LEN]; MAX_EXTENSION],
    cp_payload: U32<LittleEndian>,
    version: [u8; VERSION_LEN],
    init_version: [u8; VERSION_LEN],
    feature: U32<LittleEndian>,
    encryption_level: u8,
    encryption_pw_salt: [u8; 16],
    devs: [Device; MAX_DEVICES],
    qf_ino: [U32<LittleEndian>; MAX_QUOTAS],
    hot_ext_count: u8,
    s_encoding: U16<LittleEndian>,
    s_encoding_flags: U16<LittleEndian>,
    s_stop_reason: [u8; MAX_STOP_REASON],
    s_errors: [u8; MAX_ERRORS],
    reserved: [u8; 258],
    crc: U32<LittleEndian>,
}

/// struct f2fs_device
#[derive(Debug, Clone, Copy, FromBytes)]
#[repr(C, packed)]
pub struct Device {
    path: [u8; 64],
    total_segments: U32<LittleEndian>,
}

const MAGIC: U32<LittleEndian> = U32::new(0xF2F52010);
const START_POSITION: u64 = 1024;

/// Attempt to decode the Superblock from the given read stream
pub fn from_reader<R: Read>(reader: &mut R) -> Result<F2FS, Error> {
    // Drop unwanted bytes (Seek not possible with zstd streamed inputs)
    io::copy(&mut reader.by_ref().take(START_POSITION), &mut io::sink())?;

    // Safe zero-copy deserialization
    let data = F2FS::read_from_io(reader).map_err(|_| Error::InvalidSuperblock)?;

    if data.magic != MAGIC {
        return Err(Error::InvalidMagic);
    }

    log::trace!(
        "valid magic field: UUID={} [volume label: \"{}\"]",
        data.uuid()?,
        data.label().unwrap_or_else(|_| "[invalid utf8]".into())
    );
    Ok(data)
}

impl Superblock for F2FS {
    /// Return the encoded UUID for this superblock
    fn uuid(&self) -> Result<String, Error> {
        Ok(Uuid::from_bytes(self.uuid).hyphenated().to_string())
    }

    /// Return the volume label as valid utf16 String
    fn label(&self) -> Result<String, Error> {
        // Convert the array of U16<LittleEndian> to u16
        let vol: Vec<u16> = self.volume_name.iter().map(|x| x.get()).collect();
        let prelim_label = String::from_utf16(&vol)?;
        // Need valid grapheme step and skip (u16)\0 nul termination in fixed block size
        Ok(prelim_label.trim_end_matches('\0').to_owned())
    }

    fn kind(&self) -> Kind {
        Kind::F2FS
    }
}

#[cfg(test)]
mod tests {

    use crate::{f2fs::from_reader, Superblock};
    use std::fs;

    #[test_log::test]
    fn test_basic() {
        let mut fi = fs::File::open("tests/f2fs.img.zst").expect("cannot open f2fs img");
        let mut stream = zstd::stream::Decoder::new(&mut fi).expect("Unable to decode stream");
        let sb = from_reader(&mut stream).expect("Cannot parse superblock");
        let label = sb.label().expect("Cannot determine volume name");
        assert_eq!(label, "blsforme testing");
        assert_eq!(sb.uuid().unwrap(), "d2c85810-4e75-4274-bc7d-a78267af7443");
    }
}
