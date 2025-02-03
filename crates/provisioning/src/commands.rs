// SPDX-FileCopyrightText: Copyright © 2025 Serpent OS Developers
//
// SPDX-License-Identifier: MPL-2.0

use std::sync::Arc;

use kdl::KdlNode;
use miette::NamedSource;

/// Command evaluation context
pub(crate) struct Context<'a> {
    /// The document being parsed
    pub(crate) document: &'a NamedSource<Arc<String>>,

    /// The node being parsed
    pub(crate) node: &'a KdlNode,
}

/// A command
#[derive(Debug)]
pub enum Command {
    // TODO: Add command variants
    Unimplemented,
}

fn dummy_command(_context: Context) -> Result<Command, crate::Error> {
    unimplemented!("Command support not implemented");
}

/// Command execution function
type CommandExec = for<'a> fn(Context<'a>) -> Result<Command, crate::Error>;

/// Map of command names to functions
static COMMANDS: phf::Map<&'static str, CommandExec> = phf::phf_map! {
    // "find-disk" => dummy_command,
    // "create-partition" => dummy_command,
    // "create-partition-table" => dummy_command,
};

/// Parse a command from a node if possible
pub(crate) fn parse_command(context: Context<'_>) -> Result<Command, crate::Error> {
    let name = context.node.name().value();
    let func = COMMANDS.get(name).ok_or_else(|| crate::UnsupportedNode {
        src: context.document.clone(),
        at: context.node.span(),
        id: name.to_string(),
        advice: None,
    })?;

    func(context)
}
