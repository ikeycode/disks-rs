// SPDX-FileCopyrightText: Copyright © 2025 Serpent OS Developers
//
// SPDX-License-Identifier: MPL-2.0

use std::{io, sync::Arc};

use miette::{Diagnostic, NamedSource, SourceSpan};
use thiserror::Error;

use crate::KdlType;

/// Error type for the provisioning crate
#[derive(Diagnostic, Debug, Error)]
pub enum Error {
    #[error(transparent)]
    IO(#[from] io::Error),

    #[diagnostic(transparent)]
    #[error(transparent)]
    Kdl(#[from] kdl::KdlError),

    #[error("unknown type")]
    UnknownType,

    #[diagnostic(transparent)]
    #[error(transparent)]
    InvalidType(#[from] InvalidType),

    #[diagnostic(transparent)]
    #[error(transparent)]
    UnsupportedNode(#[from] UnsupportedNode),

    #[diagnostic(transparent)]
    #[error(transparent)]
    MissingProperty(#[from] MissingProperty),

    #[diagnostic(transparent)]
    #[error(transparent)]
    ParseError(#[from] ParseError),
}

/// Merged error for parsing failures
/// Returns a list of diagnostics for the user
#[derive(Debug, Diagnostic, Error)]
#[error("failed to parse KDL")]
pub struct ParseError {
    pub src: NamedSource<Arc<String>>,
    #[related]
    pub diagnostics: Vec<Error>,
}

/// Error for invalid types
#[derive(Debug, Diagnostic, Error)]
#[error("property {id} should be {expected_type}, not {found_type}")]
#[diagnostic(severity(error))]
pub struct InvalidType {
    #[source_code]
    pub src: NamedSource<Arc<String>>,

    #[label("here")]
    pub at: SourceSpan,

    #[help]
    pub advice: Option<String>,

    pub id: &'static str,
    pub expected_type: KdlType,
    pub found_type: KdlType,
}

/// Error for missing mandatory properties
#[derive(Debug, Diagnostic, Error)]
#[error("{name} is missing mandatory property: {id}")]
#[diagnostic(severity(error))]
pub struct MissingProperty {
    #[source_code]
    pub src: NamedSource<Arc<String>>,

    #[label("here")]
    pub at: SourceSpan,

    // The name of the node
    pub name: String,

    // The name of the missing property
    pub id: &'static str,
}

/// Error for unsupported node types
#[derive(Debug, Diagnostic, Error)]
#[error("unsupported node: {id}")]
#[diagnostic(severity(warning))]
pub struct UnsupportedNode {
    #[source_code]
    pub src: NamedSource<Arc<String>>,

    #[label("here")]
    pub at: SourceSpan,

    // The name of the node
    pub id: String,

    #[help]
    pub advice: Option<String>,
}
