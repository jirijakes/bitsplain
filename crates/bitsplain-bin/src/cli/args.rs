use std::collections::HashMap;
use std::path::PathBuf;
use std::str::FromStr;

use bitsplain_format::*;
use clap::{Parser, ValueEnum};

#[derive(Parser, Debug)]
#[command(about = "Decodes Bitcoin-related binary data")]
#[command(
    long_about = "Decodes Bitcoin-related binary data\n\nIf neither INPUT nor FILE is provided, read standard input."
)]
#[command(name = "bitsplain")]
#[command(bin_name = "bitsplain")]
#[command(version, author)]
pub struct Args {
    /// Value to parse (hex, base64, base58, …)
    pub input: Option<String>,

    /// Level of details to display
    #[arg(long, short)]
    pub details: Option<Detail>,

    #[arg(long, default_value = "false")]
    /// Show all known decoders
    pub list_decoders: bool,

    /// Do not show documentation strings
    #[arg(long)]
    pub show_doc: Option<bool>,

    /// Show value identifiers (can be used for selecting)
    #[arg(long)]
    pub show_ids: Option<bool>,

    /// Read data from file
    #[arg(short = 'i', display_order = 0)]
    pub file: Option<PathBuf>,

    /// Save result into file
    #[arg(short = 'o')]
    pub outfile: Option<PathBuf>,

    /// Output format
    #[arg(
        long,
        short = 'f',
        value_name = "FORMAT",
        display_order = 100,
        default_value = "pretty",
        value_enum
    )]
    pub format: Format,

    /// Set format parameter.
    #[arg(short = 'P', value_name = "KEY=VALUE")]
    pub params: Vec<Param>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum Format {
    Pretty,
    Json,
    Html,
    Png,
    Xml,
}

/// A simple key-value parameter that can be specified by command line
/// and is passed to a format processor.
///
/// The parameter can be obtained by parsing a string of form 'key=value'.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Param {
    /// Name of the parameter.
    pub key: String,
    /// Value of the parameter.
    pub value: String,
}

impl FromStr for Param {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.splitn(2, '=').collect::<Vec<_>>().as_slice() {
            [k, v] if !k.is_empty() => Ok(Param {
                key: k.to_string(),
                value: v.to_string(),
            }),
            [k] if !k.is_empty() => Ok(Param {
                key: k.to_string(),
                value: "true".to_string(),
            }),
            _ => Err("Invalid format param".to_string()),
        }
    }
}

impl<'a> FromIterator<&'a Param> for HashMap<String, String> {
    fn from_iter<T: IntoIterator<Item = &'a Param>>(iter: T) -> Self {
        iter.into_iter()
            .map(|p| (p.key.clone(), p.value.clone()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::Param;

    #[test]
    fn params() {
        assert_eq!(
            "key=value".parse::<Param>(),
            Ok(Param {
                key: "key".to_string(),
                value: "value".to_string()
            })
        );

        assert_eq!(
            "key".parse::<Param>(),
            Ok(Param {
                key: "key".to_string(),
                value: "true".to_string()
            })
        );

        assert_eq!(
            "key=".parse::<Param>(),
            Ok(Param {
                key: "key".to_string(),
                value: "".to_string()
            })
        );

        assert_eq!(
            "key=value=pair".parse::<Param>(),
            Ok(Param {
                key: "key".to_string(),
                value: "value=pair".to_string()
            })
        );

        assert!("".parse::<Param>().is_err());
        assert!("=value".parse::<Param>().is_err());
    }
}
