// SPDX-FileCopyrightText: Copyright © 2025 Serpent OS Developers
// SPDX-FileCopyrightText: Copyright © 2025 AerynOS Developers
//
// SPDX-License-Identifier: MPL-2.0

use crate::{get_kdl_property, get_property_str, Constraints, Context, FromKdlProperty, PartitionRole};

/// Command to create a partition
#[derive(Debug)]
pub struct Command {
    /// The disk ID to create the partition on
    pub disk: String,

    /// The reference ID of the partition
    pub id: String,

    /// The role, if any, of the partition
    pub role: Option<PartitionRole>,

    pub constraints: Constraints,
}

/// Generate a command to create a partition
pub(crate) fn parse(context: Context<'_>) -> Result<super::Command, crate::Error> {
    let disk = get_property_str(context.node, "disk")?;
    let id = get_property_str(context.node, "id")?;
    let role = if let Ok(role) = get_kdl_property(context.node, "role") {
        Some(PartitionRole::from_kdl_property(role)?)
    } else {
        None
    };

    let constraints =
        if let Some(constraints) = context.node.iter_children().find(|n| n.name().value() == "constraints") {
            Constraints::from_kdl_node(constraints)?
        } else {
            return Err(crate::Error::MissingNode("constraints"));
        };

    // TODO: Load constraints etc
    Ok(super::Command::CreatePartition(Box::new(Command {
        disk,
        id,
        role,
        constraints,
    })))
}
