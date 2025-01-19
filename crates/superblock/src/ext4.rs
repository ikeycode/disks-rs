// SPDX-FileCopyrightText: Copyright © 2025 Serpent OS Developers
//
// SPDX-License-Identifier: MPL-2.0

//! EXT4 superblock handling

use crate::{Error, Kind, Superblock};
use log;
use std::io::{self, Read};
use uuid::Uuid;
use zerocopy::*;

/// EXT4 Superblock definition (as seen in the kernel)
#[derive(Debug, FromBytes)]
#[repr(C)]
pub struct Ext4 {
    inodes_count: U32<LittleEndian>,
    block_counts_lo: U32<LittleEndian>,
    r_blocks_count_lo: U32<LittleEndian>,
    free_blocks_count_lo: U32<LittleEndian>,
    free_inodes_count: U32<LittleEndian>,
    first_data_block: U32<LittleEndian>,
    log_block_size: U32<LittleEndian>,
    log_cluster_size: U32<LittleEndian>,
    blocks_per_group: U32<LittleEndian>,
    clusters_per_group: U32<LittleEndian>,
    inodes_per_group: U32<LittleEndian>,
    m_time: U32<LittleEndian>,
    w_time: U32<LittleEndian>,
    mnt_count: U16<LittleEndian>,
    max_mnt_count: U16<LittleEndian>,
    magic: U16<LittleEndian>,
    state: U16<LittleEndian>,
    errors: U16<LittleEndian>,
    minor_rev_level: U16<LittleEndian>,
    lastcheck: U32<LittleEndian>,
    checkinterval: U32<LittleEndian>,
    creator_os: U32<LittleEndian>,
    rev_level: U32<LittleEndian>,
    def_resuid: U16<LittleEndian>,
    def_resgid: U16<LittleEndian>,
    first_ino: U32<LittleEndian>,
    inode_size: U16<LittleEndian>,
    block_group_nr: U16<LittleEndian>,
    feature_compat: U32<LittleEndian>,
    feature_incompat: U32<LittleEndian>,
    feature_ro_compat: U32<LittleEndian>,
    uuid: [u8; 16],
    volume_name: [u8; 16],
    last_mounted: [u8; 64],
    algorithm_usage_bitmap: U32<LittleEndian>,
    prealloc_blocks: u8,
    prealloc_dir_blocks: u8,
    reserved_gdt_blocks: U16<LittleEndian>,
    journal_uuid: [u8; 16],
    journal_inum: U32<LittleEndian>,
    journal_dev: U32<LittleEndian>,
    last_orphan: U32<LittleEndian>,
    hash_seed: [U32<LittleEndian>; 4],
    def_hash_version: u8,
    jnl_backup_type: u8,
    desc_size: U16<LittleEndian>,
    default_mount_opts: U32<LittleEndian>,
    first_meta_bg: U32<LittleEndian>,
    mkfs_time: U32<LittleEndian>,
    jnl_blocks: [U32<LittleEndian>; 17],
    blocks_count_hi: U32<LittleEndian>,
    free_blocks_count_hi: U32<LittleEndian>,
    min_extra_isize: U16<LittleEndian>,
    want_extra_isize: U16<LittleEndian>,
    flags: U32<LittleEndian>,
    raid_stride: U16<LittleEndian>,
    mmp_update_interval: U16<LittleEndian>,
    mmp_block: U64<LittleEndian>,
    raid_stripe_width: U32<LittleEndian>,
    log_groups_per_flex: u8,
    checksum_type: u8,
    reserved_pad: U16<LittleEndian>,
    kbytes_written: U64<LittleEndian>,
    snapshot_inum: U32<LittleEndian>,
    snapshot_id: U32<LittleEndian>,
    snapshot_r_blocks_count: U64<LittleEndian>,
    snapshot_list: U32<LittleEndian>,
    error_count: U32<LittleEndian>,
    first_error_time: U32<LittleEndian>,
    first_error_inod: U32<LittleEndian>,
    first_error_block: U64<LittleEndian>,
    first_error_func: [u8; 32],
    first_error_line: U32<LittleEndian>,
    last_error_time: U32<LittleEndian>,
    last_error_inod: U32<LittleEndian>,
    last_error_line: U32<LittleEndian>,
    last_error_block: U64<LittleEndian>,
    last_error_func: [u8; 32],
    mount_opts: [u8; 64],
    usr_quota_inum: U32<LittleEndian>,
    grp_quota_inum: U32<LittleEndian>,
    overhead_clusters: U32<LittleEndian>,
    reserved: [U32<LittleEndian>; 108],
    checksum: U32<LittleEndian>,
}

const MAGIC: U16<LittleEndian> = U16::new(0xEF53);
const START_POSITION: u64 = 1024;

/// Attempt to decode the Superblock from the given read stream
pub fn from_reader<R: Read>(reader: &mut R) -> Result<Ext4, Error> {
    // Drop unwanted bytes (Seek not possible with zstd streamed inputs)
    io::copy(&mut reader.by_ref().take(START_POSITION), &mut io::sink())?;

    let data = Ext4::read_from_io(reader).map_err(|_| Error::InvalidSuperblock)?;

    if data.magic != MAGIC {
        Err(Error::InvalidMagic)
    } else {
        log::trace!(
            "valid magic field: UUID={} [volume label: \"{}\"]",
            data.uuid()?,
            data.label().unwrap_or_else(|_| "[invalid utf8]".into())
        );
        Ok(data)
    }
}

impl super::Superblock for Ext4 {
    /// Return the encoded UUID for this superblock
    fn uuid(&self) -> Result<String, Error> {
        Ok(Uuid::from_bytes(self.uuid).hyphenated().to_string())
    }

    /// Return the volume label as valid utf8
    fn label(&self) -> Result<String, super::Error> {
        Ok(std::str::from_utf8(&self.volume_name)?.into())
    }

    fn kind(&self) -> Kind {
        Kind::Ext4
    }
}

#[cfg(test)]
mod tests {

    use std::fs;

    use crate::{ext4::from_reader, Superblock};

    #[test_log::test]
    fn test_basic() {
        let mut fi = fs::File::open("tests/ext4.img.zst").expect("cannot open ext4 img");
        let mut stream = zstd::stream::Decoder::new(&mut fi).expect("Unable to decode stream");
        let sb = from_reader(&mut stream).expect("Cannot parse superblock");
        let label = sb.label().expect("Cannot determine volume name");
        assert_eq!(label, "blsforme testing");
        assert_eq!(sb.uuid().unwrap(), "731af94c-9990-4eed-944d-5d230dbe8a0d");
    }
}
