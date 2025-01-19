// SPDX-FileCopyrightText: Copyright © 2025 Serpent OS Developers
//
// SPDX-License-Identifier: MPL-2.0

use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};
use std::collections::HashMap;

/// Top-level LUKS2 configuration structure representing a LUKS2 encrypted device
#[derive(Debug, Deserialize, Serialize)]
pub struct Luks2Config {
    /// Core configuration data
    pub config: Luks2ConfigData,
    /// Map of keyslot IDs to keyslot configurations
    pub keyslots: HashMap<u64, Luks2Keyslot>,
    /// Map of segment IDs to segment configurations
    pub segments: HashMap<u64, Luks2Segment>,
    // pub tokens: HashMap<u64, Value>,
    // pub digests: HashMap<u64, Value>,
}

/// Core LUKS2 configuration data
#[serde_as]
#[derive(Debug, Deserialize, Serialize)]
pub struct Luks2ConfigData {
    /// Size of the JSON metadata area in bytes
    #[serde_as(as = "DisplayFromStr")]
    pub json_size: u64,

    /// Size of the keyslots area in bytes
    #[serde_as(as = "DisplayFromStr")]
    pub keyslots_size: u64,

    /// Optional configuration flags
    #[serde(default)]
    pub flags: Vec<String>,

    /// Requirements for device activation
    #[serde(default)]
    pub requirements: Vec<String>,
}

/// Key derivation function configuration
#[serde_as]
#[derive(Debug, Deserialize, Serialize)]
pub struct Luks2Kdf {
    /// Type of KDF (e.g. pbkdf2, argon2i, argon2id)
    #[serde(rename = "type")]
    pub kdf_type: String,
    /// Random salt used in key derivation
    pub salt: String,

    /// Hash algorithm (pbkdf2 only)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hash: Option<String>,
    /// Number of iterations (pbkdf2 only)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iterations: Option<u64>,

    /// Time cost (argon2i/argon2id only)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time: Option<u64>,
    /// Memory usage in bytes (argon2i/argon2id only)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory: Option<u64>,
    /// Number of parallel threads (argon2i/argon2id only)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpus: Option<u64>,
}

/// Configuration for a single keyslot
#[derive(Debug, Deserialize, Serialize)]
pub struct Luks2Keyslot {
    /// Type of keyslot
    #[serde(rename = "type")]
    pub slot_type: String,

    /// Size of the keyslot key in bytes
    pub key_size: u64,

    /// Storage area configuration
    pub area: Luks2KeyslotArea,
    /// Key derivation parameters
    pub kdf: Luks2Kdf,
}

/// Configuration for keyslot storage area
#[serde_as]
#[derive(Debug, Deserialize, Serialize)]
pub struct Luks2KeyslotArea {
    /// Type of storage area
    #[serde(rename = "type")]
    pub area_type: String,

    /// Offset in bytes where this area begins
    #[serde_as(as = "DisplayFromStr")]
    pub offset: u64,

    /// Size of this area in bytes
    #[serde_as(as = "DisplayFromStr")]
    pub size: u64,

    /// Encryption algorithm used
    pub encryption: String,

    /// Size of encryption key in bytes
    pub key_size: u64,
}

/// Configuration for a disk segment
#[derive(Debug, Deserialize, Serialize)]
pub struct Luks2Segment {
    /// Type of segment
    #[serde(rename = "type")]
    pub segment_type: String,
    /// Offset where segment begins
    pub offset: String,
    /// Size of segment
    pub size: String,
    /// Initialization vector tweak
    pub iv_tweak: String,
    /// Encryption algorithm
    pub encryption: String,
    /// Sector size in bytes
    pub sector_size: u64,
}
