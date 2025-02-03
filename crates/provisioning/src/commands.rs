// SPDX-FileCopyrightText: Copyright © 2025 Serpent OS Developers
//
// SPDX-License-Identifier: MPL-2.0

use crate::Context;

mod create_partition;
mod create_partition_table;
mod find_disk;

/// A command
#[derive(Debug)]
pub enum Command {
    CreatePartition,
    CreatePartitionTable(Box<create_partition_table::Command>),
    FindDisk,
}

/// Command execution function
type CommandExec = for<'a> fn(Context<'a>) -> Result<Command, crate::Error>;

/// Map of command names to functions
static COMMANDS: phf::Map<&'static str, CommandExec> = phf::phf_map! {
    //"find-disk" => find_disk::parse,
    //"create-partition" => create_partition::parse,
    "create-partition-table" => create_partition_table::parse,
};

/// Parse a command from a node if possible
pub(crate) fn parse_command(context: Context<'_>) -> Result<Command, crate::Error> {
    let name = context.node.name().value();
    let func = COMMANDS.get(name).ok_or_else(|| crate::UnsupportedNode {
        at: context.node.span(),
        name: name.into(),
    })?;

    func(context)
}
