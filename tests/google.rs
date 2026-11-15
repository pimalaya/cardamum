//! # Google live suite
//!
//! The binary against the People API and Google's CardDAV server.
//! `anonymous` runs a made-up token, touching no account. `credentialed`
//! reads the test Workspace's contacts with a token the suite mints for a
//! service account acting as `CARDAMUM_GOOGLE_SERVICE_ACCOUNT_SUBJECT`
//! (RFC 7523 section 2.1), its key read from
//! `CARDAMUM_GOOGLE_SERVICE_ACCOUNT_KEY` or the file
//! `CARDAMUM_GOOGLE_SERVICE_ACCOUNT_KEY_FILE` names. It only reads.
//!
//! Both accounts are sections of config.sample.toml: Google People, and
//! Google CardDAV, its address book home the subject's in `credentialed`.
//!
//! Run with `cargo test --test google -- --ignored`. A failure never shows
//! a token: the binary reads it from its environment only.

#![cfg(all(feature = "gpeople", feature = "carddav"))]

mod common;

use std::path::{Path, PathBuf};

use crate::common::*;

/// Writes the sample's Google People account, its token read through
/// [`TOKEN_COMMAND`].
fn write_people_account(dir: &Path) -> PathBuf {
    let token = [("gpeople.auth.token.command", TOKEN_COMMAND)];
    write_config(dir, "google", &sample_section("Google People", &token))
}

/// Writes the sample's Google CardDAV account, its bearer token read
/// through [`TOKEN_COMMAND`] and, given a `user`, its address book home
/// that user's rather than the sample's.
fn write_carddav_account(dir: &Path, user: Option<&str>) -> PathBuf {
    let home = user
        .map(|user| format!("\"https://www.googleapis.com/carddav/v1/principals/{user}/lists\""));
    let mut overrides = vec![("carddav.auth.bearer.token.command", TOKEN_COMMAND)];
    if let Some(home) = &home {
        overrides.push(("carddav.home", home.as_str()));
    }
    write_config(dir, "google", &sample_section("Google CardDAV", &overrides))
}

mod anonymous {
    use tempfile::TempDir;

    use crate::{common::*, write_carddav_account, write_people_account};

    #[test]
    #[ignore = "requires network and --ignored"]
    fn people_account_check_refused() {
        let dir = TempDir::new().unwrap();
        let config = write_people_account(dir.path());

        let out = cardamum(&config, MADE_UP, &["--json", "account", "check"]);
        json_check_failure(&out, "gpeople", REFUSED);
    }

    #[test]
    #[ignore = "requires network and --ignored"]
    fn people_addressbook_list_refused() {
        let dir = TempDir::new().unwrap();
        let config = write_people_account(dir.path());

        let args = ["--json", "addressbook", "list"];
        json_failure(&cardamum(&config, MADE_UP, &args), REFUSED);
    }

    #[test]
    #[ignore = "requires network and --ignored"]
    fn carddav_account_check_refused() {
        let dir = TempDir::new().unwrap();
        let config = write_carddav_account(dir.path(), None);

        let out = cardamum(&config, MADE_UP, &["--json", "account", "check"]);
        json_check_failure(&out, "carddav", REFUSED);
    }

    #[test]
    #[ignore = "requires network and --ignored"]
    fn carddav_addressbook_list_refused() {
        let dir = TempDir::new().unwrap();
        let config = write_carddav_account(dir.path(), None);

        let args = ["--json", "addressbook", "list"];
        json_failure(&cardamum(&config, MADE_UP, &args), REFUSED);
    }
}

mod credentialed {
    use std::{collections::BTreeSet, sync::OnceLock, time::Duration};

    use io_oauth::{
        client::Oauth20ClientStd,
        rfc7523::{
            assertion::{Oauth20JwtBearerClaims, Oauth20JwtBearerKey},
            auth_grant::Oauth20JwtBearerGrantRequestParams,
        },
    };
    use pimalaya_stream::tls::Tls;
    use secrecy::ExposeSecret;
    use serde_json::Value;
    use tempfile::TempDir;

    use crate::{common::*, write_carddav_account, write_people_account};

    /// The user acted as when `CARDAMUM_GOOGLE_SERVICE_ACCOUNT_SUBJECT` is
    /// unset.
    const SUBJECT: &str = "google@pimalaya.org";

    /// The scopes of the People API and of CardDAV. The delegation grants
    /// both, and one scope it does not grant fails the whole request.
    const SCOPES: [&str; 2] = [
        "https://www.googleapis.com/auth/contacts",
        "https://www.googleapis.com/auth/carddav",
    ];

    /// The variable holding the service account key.
    const KEY_VAR: &str = "CARDAMUM_GOOGLE_SERVICE_ACCOUNT_KEY";

    /// The People addressbook every contact belongs to.
    const PEOPLE_BOOK: &str = "myContacts";

    /// The one addressbook Google's CardDAV server exposes.
    const CARDDAV_BOOK: &str = "default";

    /// The user the service account acts as.
    fn subject() -> String {
        env_var("CARDAMUM_GOOGLE_SERVICE_ACCOUNT_SUBJECT").unwrap_or_else(|| SUBJECT.to_owned())
    }

    /// A token of the service account acting as [`subject`], minted once
    /// per run. A failure never shows the key, nor what it signed.
    fn token() -> &'static str {
        static TOKEN: OnceLock<String> = OnceLock::new();

        TOKEN.get_or_init(|| {
            let Some(key) = env_content(KEY_VAR) else {
                panic!("set {KEY_VAR} or {KEY_VAR}_FILE to run this test");
            };
            let key: Value = serde_json::from_str(&key)
                .unwrap_or_else(|_| panic!("{KEY_VAR} is not a service account key"));
            let (Some(email), Some(pem), Some(endpoint)) = (
                key["client_email"].as_str(),
                key["private_key"].as_str(),
                key["token_uri"].as_str(),
            ) else {
                panic!("{KEY_VAR} is not a service account key");
            };
            let signer = Oauth20JwtBearerKey::from_pkcs8_pem(pem)
                .unwrap_or_else(|_| panic!("{KEY_VAR} holds no PKCS#8 key"));

            let tls = Tls::default();
            let client = Oauth20ClientStd::connect(endpoint.parse().unwrap(), &tls, email);
            let mut client = client.unwrap_or_else(|err| panic!("cannot reach {endpoint}: {err}"));

            let subject = subject();
            let claims = Oauth20JwtBearerClaims {
                iss: email.into(),
                sub: Some(subject.as_str().into()),
                scope: SCOPES.map(Into::into).into(),
                ..Default::default()
            };
            let validity = Duration::from_secs(600);
            let assertion = client
                .sign_jwt_bearer_assertion(&signer, claims, None, validity)
                .unwrap_or_else(|_| panic!("cannot sign the assertion"));

            let params = Oauth20JwtBearerGrantRequestParams {
                assertion,
                scope: BTreeSet::new(),
            };

            match client.request_jwt_bearer_grant(params) {
                Ok(Ok(success)) => success.access_token.expose_secret().to_owned(),
                Ok(Err(err)) => panic!(
                    "token refused: {:?}: {:?}",
                    err.error, err.error_description
                ),
                Err(err) => panic!("token request failed: {err}"),
            }
        })
    }

    #[test]
    #[ignore = "requires Google credentials and --ignored"]
    fn people_account_check() {
        let dir = TempDir::new().unwrap();
        let config = write_people_account(dir.path());

        let out = cardamum(&config, token(), &["--json", "account", "check"]);
        assert_check(&json(&out, 0), "gpeople");
    }

    #[test]
    #[ignore = "requires Google credentials and --ignored"]
    fn people_addressbook_list() {
        let dir = TempDir::new().unwrap();
        let config = write_people_account(dir.path());

        let out = cardamum(&config, token(), &["--json", "addressbook", "list"]);
        assert_addressbooks(&json(&out, 0), PEOPLE_BOOK);
    }

    #[test]
    #[ignore = "requires Google credentials and --ignored"]
    fn people_card_list() {
        let dir = TempDir::new().unwrap();
        let config = write_people_account(dir.path());

        let args = ["--json", "card", "list", "-s", "5", "-k", PEOPLE_BOOK];
        let out = cardamum(&config, token(), &args);
        assert_cards(&json(&out, 0), PEOPLE_BOOK, 5);
    }

    #[test]
    #[ignore = "requires Google credentials and --ignored"]
    fn carddav_account_check() {
        let dir = TempDir::new().unwrap();
        let config = write_carddav_account(dir.path(), Some(&subject()));

        let out = cardamum(&config, token(), &["--json", "account", "check"]);
        assert_check(&json(&out, 0), "carddav");
    }

    #[test]
    #[ignore = "requires Google credentials and --ignored"]
    fn carddav_addressbook_list() {
        let dir = TempDir::new().unwrap();
        let config = write_carddav_account(dir.path(), Some(&subject()));

        let out = cardamum(&config, token(), &["--json", "addressbook", "list"]);
        assert_addressbooks(&json(&out, 0), CARDDAV_BOOK);
    }

    #[test]
    #[ignore = "requires Google credentials and --ignored"]
    fn carddav_card_list() {
        let dir = TempDir::new().unwrap();
        let config = write_carddav_account(dir.path(), Some(&subject()));

        let args = ["--json", "card", "list", "-s", "5", "-k", CARDDAV_BOOK];
        let out = cardamum(&config, token(), &args);
        assert_cards(&json(&out, 0), CARDDAV_BOOK, 5);
    }
}
