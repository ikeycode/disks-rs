// SPDX-FileCopyrightText: Copyright © 2025 Serpent OS Developers
//
// SPDX-License-Identifier: MPL-2.0

use std::fmt;

use kdl::KdlValue;

use crate::Error;

/// The type of a KDL value
#[derive(Debug)]
pub enum KdlType {
    /// A boolean value
    Boolean,
    /// A string value
    String,
    /// A null value
    Null,
    /// An integer value
    Integer,
}

impl KdlType {
    // Determine the kdl value type
    pub fn for_value(value: &KdlValue) -> Result<Self, Error> {
        if value.is_bool() {
            Ok(Self::Boolean)
        } else if value.is_string() {
            Ok(Self::String)
        } else if value.is_null() {
            Ok(Self::Null)
        } else if value.is_integer() {
            Ok(Self::Integer)
        } else {
            Err(Error::UnknownType)
        }
    }
}

impl fmt::Display for KdlType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KdlType::Boolean => f.write_str("boolean"),
            KdlType::String => f.write_str("string"),
            KdlType::Null => f.write_str("null"),
            KdlType::Integer => f.write_str("int"),
        }
    }
}
