//! # Fastmail live suite
//!
//! The binary against Fastmail's JMAP and CardDAV servers, with a made-up
//! credential: the JMAP section of config.sample.toml pointed at Fastmail
//! with a bearer token, and its CardDAV section pointed at Fastmail with a
//! password, Fastmail's CardDAV server taking Basic only. Only a browser
//! consent issues a Fastmail token, so the suite is `anonymous` only and
//! touches no account.
//!
//! Run with `cargo test --test fastmail -- --ignored`.

#![cfg(all(feature = "jmap", feature = "carddav"))]

mod common;

mod anonymous {
    use std::path::{Path, PathBuf};

    use tempfile::TempDir;

    use crate::common::*;

    /// Fastmail's CardDAV host, found through its `/.well-known/carddav`.
    const CARDDAV_SERVER: &str = r#""carddav.fastmail.com""#;

    /// Fastmail's JMAP session URL.
    const JMAP_SERVER: &str = r#""https://api.fastmail.com/jmap/session""#;

    /// Writes the sample's JMAP account on Fastmail, its bearer token read
    /// through [`TOKEN_COMMAND`].
    fn write_jmap_account(dir: &Path) -> PathBuf {
        let overrides = [
            ("jmap.server", JMAP_SERVER),
            ("jmap.auth.bearer.token.command", TOKEN_COMMAND),
        ];
        write_config(dir, "fastmail", &sample_section("JMAP", &overrides))
    }

    /// Writes the sample's CardDAV account on Fastmail, the sample's user
    /// and its password read through [`TOKEN_COMMAND`].
    fn write_carddav_account(dir: &Path) -> PathBuf {
        let overrides = [
            ("carddav.server", CARDDAV_SERVER),
            ("carddav.auth.basic.password.command", TOKEN_COMMAND),
        ];
        write_config(dir, "fastmail", &sample_section("CardDAV", &overrides))
    }

    #[test]
    #[ignore = "requires network and --ignored"]
    fn jmap_account_check_refused() {
        let dir = TempDir::new().unwrap();
        let config = write_jmap_account(dir.path());

        let out = cardamum(&config, MADE_UP, &["--json", "account", "check"]);
        json_check_failure(&out, "jmap", REFUSED);
    }

    #[test]
    #[ignore = "requires network and --ignored"]
    fn jmap_addressbook_list_refused() {
        let dir = TempDir::new().unwrap();
        let config = write_jmap_account(dir.path());

        let args = ["--json", "addressbook", "list"];
        json_failure(&cardamum(&config, MADE_UP, &args), REFUSED);
    }

    #[test]
    #[ignore = "requires network and --ignored"]
    fn carddav_account_check_refused() {
        let dir = TempDir::new().unwrap();
        let config = write_carddav_account(dir.path());

        let out = cardamum(&config, MADE_UP, &["--json", "account", "check"]);
        json_check_failure(&out, "carddav", REFUSED);
    }

    #[test]
    #[ignore = "requires network and --ignored"]
    fn carddav_addressbook_list_refused() {
        let dir = TempDir::new().unwrap();
        let config = write_carddav_account(dir.path());

        let args = ["--json", "addressbook", "list"];
        json_failure(&cardamum(&config, MADE_UP, &args), REFUSED);
    }
}
