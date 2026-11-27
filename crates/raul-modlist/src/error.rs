// SPDX-License-Identifier: GPL-3.0-only
//! What can be wrong with a Mod list file.

use std::fmt;

/// Why a Mod list was refused. `Display` is a plain sentence a person can act on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// The text is not TOML, or a field has the wrong type, an unknown name or a missing value.
    Syntax(String),
    /// The `schema` line is missing or names a version this raul does not read.
    Schema(String),
    /// The file reads, but its contents break the rules of the format; one line per problem.
    Invalid(Vec<String>),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Syntax(why) => write!(f, "the Mod list could not be read: {why}"),
            Error::Schema(why) => write!(f, "{why}"),
            Error::Invalid(problems) => {
                write!(f, "the Mod list breaks {} rule(s):", problems.len())?;
                for p in problems {
                    write!(f, "\n  {p}")?;
                }
                Ok(())
            }
        }
    }
}

impl std::error::Error for Error {}
