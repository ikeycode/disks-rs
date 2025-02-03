// SPDX-FileCopyrightText: Copyright © 2025 Serpent OS Developers
//
// SPDX-License-Identifier: MPL-2.0

use std::sync::Arc;

use kdl::KdlNode;
use miette::NamedSource;

use crate::{Error, InvalidType, KdlType, MissingProperty};

// Get a string property from a node
pub(crate) fn get_property_str(
    ns: &NamedSource<Arc<String>>,
    node: &KdlNode,
    name: &'static str,
) -> Result<String, Error> {
    let entry = node.entry(name).ok_or_else(|| MissingProperty {
        name: node.name().to_string(),
        src: ns.clone(),
        at: node.span(),
        id: name,
    })?;
    let value = entry.value();
    let kind = KdlType::for_value(value)?;
    let value = entry.value().as_string().ok_or(InvalidType {
        src: ns.clone(),
        at: entry.span(),
        id: name,
        expected_type: KdlType::String,
        found_type: kind,
        advice: Some("try using a quoted string".to_string()),
    })?;

    Ok(value.to_owned())
}
