//! # Card composer
//!
//! The command a card is edited through, and the temporary file it is
//! handed.
//!
//! Cardamum writes the vCard to a file, spawns the configured command on
//! its path with every stream inherited, and reads the file back once the
//! command exits. Nothing is captured, which is the whole point: a
//! composer spawning an editor would otherwise hand it a pipe instead of
//! the terminal, and the editor hangs or writes where nothing reads.

use core::fmt;

use std::{
    env::temp_dir,
    fs,
    path::{Path, PathBuf},
    process::{ExitStatus, Stdio},
};

use anyhow::{Context, Error, Result, anyhow, bail};
use log::{debug, info};
use pimalaya_cli::{printer::Printer, prompt};
use pimalaya_config::command::{self, CommandConfig};

use crate::shared::{card::vcard::check, uuid::uuid_v4};

/// The command a card is edited through.
pub struct CardComposer {
    /// The configured command, spawned on the path of the vCard.
    pub command: CommandConfig,
}

impl CardComposer {
    /// Edits `seed` in the composer, then asks what to do with the result.
    ///
    /// The editor and the menu loop until the card is written or the edit
    /// is abandoned, so reviewing, re-editing and giving up are all one
    /// decision taken in one place. `None` means abandoned, whether the
    /// command emptied the file or the reader chose to abort.
    ///
    /// Nothing is written here: the draft comes back for the caller to
    /// write, which is what keeps the network out of the editor's way.
    pub fn edit(&self, printer: &mut impl Printer, seed: &[u8]) -> Result<Option<CardDraft>> {
        // NOTE: the composer inherits this process' stdout, where it would
        // interleave with the JSON payload, and a consumer reading that
        // output has no terminal to edit in either.
        if printer.is_json() {
            bail!("Cannot run a composer under --json; pass the vCard explicitly instead");
        }

        let path = temp_dir().join(format!("cardamum-{}.vcf", uuid_v4()?));
        info!("seeding the composer with {} bytes in {path:?}", seed.len());
        fs::write(&path, seed).with_context(|| format!("Cannot write vCard {path:?}"))?;

        let mut draft = CardDraft {
            path,
            contents: Vec::new(),
        };

        loop {
            let status = self.spawn(&draft.path)?;

            if !status.success() {
                // NOTE: a composer exiting non-zero is a person saying no,
                // so the file goes with it rather than piling up in the
                // temporary directory.
                draft.remove();
                bail!("Composer exited with {status}, leaving the card untouched");
            }

            let contents = fs::read(&draft.path)
                .with_context(|| format!("Cannot read vCard {:?}", draft.path))?;

            if contents.is_empty() {
                debug!("composer emptied the file");
                draft.remove();
                return Ok(None);
            }

            draft.contents = contents;

            // NOTE: checked before the menu rather than in it, so `Save`
            // is offered only for a card that would actually be accepted,
            // and a broken one leads straight back to the editor.
            let violations = check(&draft.contents);

            if !violations.is_empty() {
                println!("What the composer wrote is not a valid vCard:\n");

                for violation in &violations {
                    println!("  {violation}");
                }

                println!();

                match prompt::bool("Re-edit the card?", true) {
                    Ok(true) => continue,
                    Ok(false) => return Err(draft.keep(anyhow!(violations.join(", ")))),
                    Err(err) => return Err(draft.keep(err.into())),
                }
            }

            match self.review(&draft)? {
                CardChoice::Save => return Ok(Some(draft)),
                CardChoice::Edit => continue,
                // NOTE: review loops on a preview, so it only ever hands
                // back one of the three decisions.
                CardChoice::Preview | CardChoice::Abort => {
                    // NOTE: a card nobody touched is nothing to lose, so it
                    // goes. One that was worked on is kept and named:
                    // aborting is a decision, but so is ten minutes of
                    // typing, and only one of the two is expensive to get
                    // wrong.
                    if draft.contents == seed {
                        draft.remove();
                    } else {
                        println!("Card left at {:?}", draft.path);
                    }

                    return Ok(None);
                }
            }
        }
    }

    /// Asks what to do with the card the composer wrote.
    ///
    /// Previewing prints it and asks again, so the menu is the one place
    /// the decision is taken. The card has already been checked, so
    /// saving is always one of the answers.
    fn review(&self, draft: &CardDraft) -> Result<CardChoice> {
        loop {
            let choices = vec![
                CardChoice::Save,
                CardChoice::Preview,
                CardChoice::Edit,
                CardChoice::Abort,
            ];

            match prompt::item("Pick an action:", choices, None) {
                Ok(CardChoice::Preview) => {
                    println!("{}", String::from_utf8_lossy(&draft.contents))
                }
                Ok(choice) => return Ok(choice),
                Err(err) => return Err(draft.keep(err.into())),
            }
        }
    }

    /// Spawns the composer on `path`, inheriting every stream.
    fn spawn(&self, path: &Path) -> Result<ExitStatus> {
        let mut command = match &self.command {
            // NOTE: a shell invoked as `sh -c <line> <path>` binds the path
            // to `$0` rather than passing it on, so a shell composer takes
            // it interpolated into the line instead.
            CommandConfig::Shell(line) => {
                command::shell(&format!("{line} {}", quote(&path.to_string_lossy())))
            }
            CommandConfig::Argv { .. } => {
                let mut command = self.command.to_command();
                command.arg(path);
                command
            }
        };

        info!("spawning the composer on {path:?}");

        // NOTE: inherited is the default, and stating it is the point of
        // this module: capturing any of the three is what breaks an editor.
        command
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit());

        command.status().context("Cannot spawn composer")
    }
}

/// What to do with the card a composer wrote.
///
/// Saving is spelled the same on a create and on an update: at this point
/// the card is written or it is not, and which verb asked is already on
/// the command line.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CardChoice {
    /// Hand it back to the command, which writes it.
    Save,
    /// Print it, then ask again.
    Preview,
    /// Open the composer on it once more.
    Edit,
    /// Abandon it, the file staying where it is.
    Abort,
}

impl fmt::Display for CardChoice {
    /// The menu entry, which is what the prompt lists.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            Self::Save => "Save",
            Self::Preview => "Preview",
            Self::Edit => "Edit again",
            Self::Abort => "Abort",
        };

        write!(f, "{label}")
    }
}

/// A composed vCard, still in the file the composer wrote it to.
///
/// The file outlives the composer on purpose: a write that fails after a
/// minute of editing must not take the edit with it.
pub struct CardDraft {
    /// The temporary file the composer edited.
    path: PathBuf,
    /// What it left there.
    pub contents: Vec<u8>,
}

impl CardDraft {
    /// Settles the draft on the outcome of the write it fed.
    ///
    /// A write that landed drops the file, and one that failed keeps it
    /// and names it, so nothing typed is ever lost to a later failure.
    pub fn finish<T>(self, result: Result<T>) -> Result<T> {
        match result {
            Ok(value) => {
                self.remove();
                Ok(value)
            }
            Err(err) => Err(self.keep(err)),
        }
    }

    /// Keeps the file and names it in the error.
    ///
    /// The path is the recovery: the card is still there to be passed
    /// back in once whatever failed is dealt with.
    fn keep(&self, err: Error) -> Error {
        err.context(format!("Cannot edit vCard {:?}", self.path))
    }

    /// Drops the file, a failure to do so being nothing to report.
    fn remove(&self) {
        if let Err(err) = fs::remove_file(&self.path) {
            debug!("cannot remove {:?}: {err}", self.path);
        }
    }
}

/// Single-quotes a path for a shell command line.
fn quote(path: &str) -> String {
    format!("'{}'", path.replace('\'', r"'\''"))
}

#[cfg(test)]
mod tests {
    use super::quote;

    #[test]
    fn a_quoted_path_survives_a_quote_of_its_own() {
        assert_eq!(quote("/tmp/plain.vcf"), "'/tmp/plain.vcf'");
        assert_eq!(quote("/tmp/a b.vcf"), "'/tmp/a b.vcf'");
        assert_eq!(quote("/tmp/it's.vcf"), r"'/tmp/it'\''s.vcf'");
    }
}
