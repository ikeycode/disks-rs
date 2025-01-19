// SPDX-FileCopyrightText: Copyright © 2025 Serpent OS Developers
//
// SPDX-License-Identifier: MPL-2.0

use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize)]
pub struct Luks2Config {
    pub config: Luks2ConfigData,
    pub keyslots: HashMap<u64, Luks2Keyslot>,
    pub segments: HashMap<u64, Luks2Segment>,
    // pub tokens: HashMap<u64, Value>,
    // pub digests: HashMap<u64, Value>,
}

#[serde_as]
#[derive(Debug, Deserialize, Serialize)]
pub struct Luks2ConfigData {
    #[serde_as(as = "DisplayFromStr")]
    pub json_size: u64,

    #[serde_as(as = "DisplayFromStr")]
    pub keyslots_size: u64,

    #[serde(default)]
    pub flags: Vec<String>,

    #[serde(default)]
    pub requirements: Vec<String>,
}

#[serde_as]
#[derive(Debug, Deserialize, Serialize)]
pub struct Luks2Kdf {
    #[serde(rename = "type")]
    pub kdf_type: String,
    pub salt: String,

    // only for pbkdf2
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iterations: Option<u64>,

    // only for argon2i and argon2id
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpus: Option<u64>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Luks2Keyslot {
    #[serde(rename = "type")]
    pub slot_type: String,

    pub key_size: u64,

    pub area: Luks2KeyslotArea,
    pub kdf: Luks2Kdf,
}

#[serde_as]
#[derive(Debug, Deserialize, Serialize)]
pub struct Luks2KeyslotArea {
    #[serde(rename = "type")]
    pub area_type: String,

    #[serde_as(as = "DisplayFromStr")]
    pub offset: u64,

    #[serde_as(as = "DisplayFromStr")]
    pub size: u64,

    pub encryption: String,

    pub key_size: u64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Luks2Segment {
    #[serde(rename = "type")]
    pub segment_type: String,
    pub offset: String,
    pub size: String,
    pub iv_tweak: String,
    pub encryption: String,
    pub sector_size: u64,
}
