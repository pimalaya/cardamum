use std::{
    fs,
    io::{Read, stdin},
    path::PathBuf,
};

use anyhow::{Context, Result, bail};
use clap::Parser;

/// Positional vCard source shared by `card create` and `card update`.
#[derive(Debug, Parser)]
pub struct VcardArg {
    /// A path to a vCard file, raw vCard contents, or `-` for stdin.
    #[arg(value_name = "VCARD")]
    pub vcard: String,
}

impl VcardArg {
    /// Resolves the source into raw vCard bytes: `-` reads stdin, an
    /// existing file is read, otherwise the value is treated as literal
    /// vCard contents.
    pub fn read(self) -> Result<Vec<u8>> {
        if self.vcard == "-" {
            let mut buf = Vec::new();
            stdin()
                .read_to_end(&mut buf)
                .context("Read vCard from stdin error")?;
            return Ok(buf);
        }

        let path = PathBuf::from(&self.vcard);

        if path.is_file() {
            return fs::read(&path)
                .with_context(|| format!("Read vCard from `{}` error", path.display()));
        }

        if self.vcard.trim_start().starts_with("BEGIN:VCARD") {
            return Ok(self.vcard.into_bytes());
        }

        bail!(
            "Source `{}` is neither a readable file nor vCard contents",
            self.vcard
        )
    }
}
