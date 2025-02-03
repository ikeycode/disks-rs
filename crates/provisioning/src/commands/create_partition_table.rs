// SPDX-FileCopyrightText: Copyright © 2025 Serpent OS Developers
//
// SPDX-License-Identifier: MPL-2.0

use crate::Context;
use crate::{get_kdl_property, FromKdlProperty, PartitionTableType};

/// Command to create a partition table
#[derive(Debug)]
pub struct Command {
    /// The type of partition table to create
    pub table_type: PartitionTableType,
}

/// Generate a command to create a partition table
pub(crate) fn parse(context: Context<'_>) -> Result<super::Command, crate::Error> {
    let kind = get_kdl_property(context.node, "type")?;
    let table_type = PartitionTableType::from_kdl_property(kind)?;

    Ok(super::Command::CreatePartitionTable(Box::new(Command { table_type })))
}
