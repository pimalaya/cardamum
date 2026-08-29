//! # JSON input
//!
//! The positional JSON body shared by the commands taking one: a
//! JSContact card, or a raw JMAP request object.

use std::{
    fs,
    io::{Read, stdin},
    path::PathBuf,
};

use anyhow::{Context, Result};
use clap::Parser;
use serde_json::{Map, Value};

/// Positional raw JSON source of a command taking a JSON body.
#[derive(Debug, Parser)]
pub struct JsonArg {
    /// A path to a JSON file, raw JSON, or `-` for stdin.
    #[arg(value_name = "JSON")]
    pub body: String,
}

impl JsonArg {
    /// Reads the source into a JSON object.
    ///
    /// `-` reads stdin, an existing file is read, otherwise the value is
    /// the JSON itself.
    pub fn read(self) -> Result<Map<String, Value>> {
        let raw = if self.body == "-" {
            let mut buf = String::new();
            stdin()
                .read_to_string(&mut buf)
                .context("Read JSON from stdin error")?;
            buf
        } else {
            let path = PathBuf::from(&self.body);
            if path.is_file() {
                fs::read_to_string(&path)
                    .with_context(|| format!("Read JSON from `{}` error", path.display()))?
            } else {
                self.body
            }
        };

        match serde_json::from_str(&raw).context("Parse JSON error")? {
            Value::Object(map) => Ok(map),
            _ => anyhow::bail!("The JSON body must be an object"),
        }
    }
}
