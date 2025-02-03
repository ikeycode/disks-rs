// SPDX-FileCopyrightText: Copyright © 2025 Serpent OS Developers
//
// SPDX-License-Identifier: MPL-2.0

use itertools::Itertools;

use crate::Context;

#[derive(Debug)]
pub struct Command {
    pub name: String,
}

/// Generate a command to find a disk
pub(crate) fn parse(context: Context<'_>) -> Result<super::Command, crate::Error> {
    let arguments = context
        .node
        .entries()
        .iter()
        .filter(|e| e.is_empty() || e.name().is_none())
        .collect_vec();

    let name = match arguments.len() {
        0 => {
            return Err(crate::InvalidArguments {
                at: context.node.span(),
                advice: Some("find-disk <name> - provide a name for the storage device".into()),
            }
            .into())
        }
        1 => arguments[0].value().as_string().ok_or(crate::InvalidType {
            at: arguments[0].span(),
        })?,
        _ => {
            return Err(crate::InvalidArguments {
                at: context.node.span(),
                advice: Some("find-disk <name> - only one positional argument supported".into()),
            }
            .into())
        }
    };

    Ok(super::Command::FindDisk(Box::new(Command { name: name.to_owned() })))
}
