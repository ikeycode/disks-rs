// SPDX-FileCopyrightText: Copyright © 2025 Serpent OS Developers
//
// SPDX-License-Identifier: MPL-2.0

use std::{fs, path::Path, sync::Arc};

use itertools::{Either, Itertools};
use kdl::{KdlDocument, KdlNode};
use miette::{Diagnostic, NamedSource, Severity};

mod errors;
pub use errors::*;

mod helpers;
use helpers::*;

mod types;
use types::*;

mod commands;
use commands::*;

/// A strategy definition
#[derive(Debug)]
pub struct StrategyDefinition {
    /// The name of the strategy
    pub name: String,

    /// A brief summary of the strategy
    pub summary: String,

    /// The strategy that this strategy inherits from
    pub inherits: Option<String>,

    /// The commands to execute
    pub commands: Vec<Command>,
}

/// A parser for provisioning strategies
pub struct Parser {
    pub strategies: Vec<StrategyDefinition>,
}

impl Parser {
    /// Create a new parser from a file path
    pub fn new_for_path<P>(file: P) -> Result<Self, Error>
    where
        P: AsRef<Path>,
    {
        let file = file.as_ref();
        let name = file.to_string_lossy();
        let txt = fs::read_to_string(file)?;
        Self::new(name.to_string(), txt)
    }

    /// Create a new parser from a string
    pub fn new(name: String, contents: String) -> Result<Self, Error> {
        let source = Arc::new(contents.to_string());
        let ns = NamedSource::new(name, source).with_language("KDL");
        let d = KdlDocument::parse_v2(ns.inner())?;

        let mut strategies = vec![];

        for node in d.nodes() {
            match node.name().value() {
                "strategy" => {
                    strategies.push(Self::parse_strategy(&ns, node)?);
                }
                what => {
                    return Err(UnsupportedNode {
                        src: ns.clone(),
                        at: node.span(),
                        id: what.to_string(),
                        advice: Some("only 'strategy' nodes are supported".to_owned()),
                    })?;
                }
            }
        }

        Ok(Self { strategies })
    }

    // Parse a strategy node
    fn parse_strategy(ns: &NamedSource<Arc<String>>, node: &KdlNode) -> Result<StrategyDefinition, Error> {
        let name = get_property_str(ns, node, "name")?;
        let summary = get_property_str(ns, node, "summary")?;
        let inherits = if node.entry("inherits").is_some() {
            Some(get_property_str(ns, node, "inherits")?)
        } else {
            None
        };

        // Collect all failures in this strategy
        let (commands, errors): (Vec<_>, Vec<_>) =
            node.iter_children()
                .partition_map(|node| match parse_command(Context { document: ns, node }) {
                    Ok(cmd) => Either::Left(cmd),
                    Err(e) => Either::Right(e),
                });

        let fatal_errors = errors.iter().filter(|e| match e.severity().unwrap_or(Severity::Error) {
            Severity::Error => true,
            _ => true,
        });

        // If we have any fatal errors, bail out
        // TODO: Add an error sink to allow bubbling up of warnings/diagnostics
        // for tooling integration
        if fatal_errors.clone().next().is_some() {
            return Err(ParseError {
                src: ns.clone(),
                diagnostics: errors,
            })?;
        }

        let strategy = StrategyDefinition {
            name,
            summary,
            inherits,
            commands,
        };

        Ok(strategy)
    }
}

#[cfg(test)]
mod tests {
    use crate::Parser;

    #[test]
    #[should_panic]
    fn test_basic() {
        let _p = Parser::new_for_path("tests/use_whole_disk.kdl").unwrap();
    }
}
