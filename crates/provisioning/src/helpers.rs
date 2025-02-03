// SPDX-FileCopyrightText: Copyright © 2025 Serpent OS Developers
//
// SPDX-License-Identifier: MPL-2.0

use kdl::{KdlEntry, KdlNode};

use crate::{Error, InvalidType, MissingProperty};

// Get a property from a node
pub(crate) fn get_kdl_property<'a>(node: &'a KdlNode, name: &'static str) -> Result<&'a KdlEntry, Error> {
    let entry = node.entry(name).ok_or_else(|| MissingProperty {
        at: node.span(),
        id: name,
        advice: Some(format!("add `{name}=...` to bind the property")),
    })?;

    Ok(entry)
}

// Get a string property from a value
pub(crate) fn kdl_value_to_string(entry: &kdl::KdlEntry) -> Result<String, Error> {
    let value = entry.value().as_string().ok_or(InvalidType { at: entry.span() })?;

    Ok(value.to_owned())
}

// Get a string property from a node
pub(crate) fn get_property_str(node: &KdlNode, name: &'static str) -> Result<String, Error> {
    let value = get_kdl_property(node, name).and_then(kdl_value_to_string)?;
    Ok(value.to_owned())
}
