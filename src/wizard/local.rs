//! Local backend wizard.
//!
//! A typed path pointing at an existing folder configures a local
//! store. The backend kind is auto-detected from the directory's on-disk
//! markers: a `pimdir.db` index (with its `objects/` blob directory) means
//! pimdir, a subdirectory holding `.vcf` files means vdir. When detection
//! is inconclusive (an empty or ambiguous directory) and both backends are
//! compiled in, the user picks; otherwise the sole compiled backend is used.

#[cfg(feature = "vdir")]
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Result;

#[cfg(feature = "pimdir")]
use crate::config::PimdirConfig;
#[cfg(feature = "vdir")]
use crate::config::VdirConfig;

/// A configured local backend.
pub enum Local {
    #[cfg(feature = "vdir")]
    Vdir(VdirConfig),
    #[cfg(feature = "pimdir")]
    Pimdir(PimdirConfig),
}

/// Configures a local backend rooted at `root`, auto-detecting its kind
/// from the on-disk markers and only prompting when that is inconclusive.
pub fn configure(root: PathBuf) -> Result<Local> {
    if let Some(local) = detect(&root) {
        return Ok(local);
    }

    pick(root)
}

/// Detects the backend kind from `root`'s markers: the pimdir `pimdir.db`
/// index, or a vdir tree (an immediate subdirectory holding at least one
/// `.vcf` card). Returns `None` when no marker of a compiled-in backend is
/// present, leaving the choice to [`pick`].
#[cfg_attr(
    not(all(feature = "vdir", feature = "pimdir")),
    allow(unused_variables)
)]
fn detect(root: &Path) -> Option<Local> {
    #[cfg(feature = "pimdir")]
    if root.join("pimdir.db").is_file() {
        return Some(Local::Pimdir(PimdirConfig {
            root: root.to_path_buf(),
            source: None,
            account: None,
        }));
    }

    #[cfg(feature = "vdir")]
    if holds_a_collection(root) {
        return Some(Local::Vdir(VdirConfig {
            home_dir: root.to_string_lossy().into_owned(),
        }));
    }

    None
}

/// Whether `root` looks like a vdir home: at least one immediate
/// subdirectory holding a `.vcf` card. An empty home is inconclusive rather
/// than a match, since a fresh pimdir store looks the same from outside.
#[cfg(feature = "vdir")]
fn holds_a_collection(root: &Path) -> bool {
    let Ok(entries) = fs::read_dir(root) else {
        return false;
    };

    entries.flatten().any(|entry| {
        entry.path().is_dir()
            && fs::read_dir(entry.path()).is_ok_and(|mut cards| {
                cards.any(|card| {
                    card.is_ok_and(|card| card.path().extension().is_some_and(|ext| ext == "vcf"))
                })
            })
    })
}

#[cfg(all(feature = "vdir", feature = "pimdir"))]
fn pick(root: PathBuf) -> Result<Local> {
    use pimalaya_cli::prompt;

    const VDIR: &str = "vdir";
    const PIMDIR: &str = "pimdir";

    let kind = prompt::item("Local backend:", [VDIR, PIMDIR], None)?;

    Ok(match kind {
        VDIR => Local::Vdir(VdirConfig {
            home_dir: root.to_string_lossy().into_owned(),
        }),
        _ => Local::Pimdir(PimdirConfig {
            root,
            source: None,
            account: None,
        }),
    })
}

#[cfg(all(feature = "vdir", not(feature = "pimdir")))]
fn pick(root: PathBuf) -> Result<Local> {
    Ok(Local::Vdir(VdirConfig {
        home_dir: root.to_string_lossy().into_owned(),
    }))
}

#[cfg(all(feature = "pimdir", not(feature = "vdir")))]
fn pick(root: PathBuf) -> Result<Local> {
    Ok(Local::Pimdir(PimdirConfig {
        root,
        source: None,
        account: None,
    }))
}
