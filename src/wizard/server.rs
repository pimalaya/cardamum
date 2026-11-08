//! # Server URL
//!
//! The one server the wizard takes by hand, when discovery found nothing or
//! the review's Change asked for a custom server: a URL whose scheme
//! decides the protocol, then the sign-in. Encrypted servers only, plain
//! http on a loopback host aside.

use anyhow::{Context, Result, bail};
use pimalaya_cli::prompt;
use url::{Host, Url};

use crate::wizard::search::{Discovered, DiscoveredKind, SignIn};

/// Asks `Server URL:` until it parses, then the sign-in, answering the
/// typed option, or `None` when the answer is empty.
pub fn prompt() -> Result<Option<Discovered>> {
    let kind = loop {
        let url = prompt::text("Server URL:", None)?;
        let url = url.trim();

        if url.is_empty() {
            return Ok(None);
        }

        match parse(url) {
            Ok(kind) => break kind,
            Err(err) => eprintln!("{err:#}"),
        }
    };

    let sign_in = prompt::item("Sign in with:", [SignIn::Password, SignIn::Token], None)?;

    Ok(Some(Discovered {
        kind,
        sign_in,
        username: None,
    }))
}

/// Reads a typed server URL: `https`, `carddav` and `carddavs` as CardDAV,
/// `jmap` and `jmaps` as JMAP, all over https.
///
/// A cleartext `http` URL is refused naming it unless its host is this
/// machine, as is a URL with no scheme or another one, no host, or a
/// protocol this build does not compile.
fn parse(input: &str) -> Result<DiscoveredKind> {
    let Some((scheme, rest)) = input.split_once("://") else {
        bail!("Name the scheme of the server, as `https://{input}`");
    };

    let url = |scheme: &str| {
        let url = Url::parse(&format!("{scheme}://{rest}"))
            .with_context(|| format!("Invalid server URL `{input}`"))?;

        if url.host_str().is_none_or(str::is_empty) {
            bail!("Server URL `{input}` names no host");
        }

        if scheme == "http" && !loopback(&url) {
            bail!("Only encrypted servers are accepted, refusing `{input}`");
        }

        Ok(url)
    };

    let kind = match scheme.to_ascii_lowercase().as_str() {
        "https" | "carddav" | "carddavs" => DiscoveredKind::Carddav(url("https")?),
        "http" => DiscoveredKind::Carddav(url("http")?),
        "jmap" | "jmaps" => DiscoveredKind::Jmap(url("https")?),
        other => bail!("Unsupported server scheme `{other}`, use https, carddavs or jmaps"),
    };

    if !kind.supported() {
        bail!("{} is not compiled in this build", kind.protocol());
    }

    Ok(kind)
}

/// Whether `url` names this machine: `localhost` or a loopback address.
fn loopback(url: &Url) -> bool {
    match url.host() {
        Some(Host::Domain(domain)) => domain.eq_ignore_ascii_case("localhost"),
        Some(Host::Ipv4(ip)) => ip.is_loopback(),
        Some(Host::Ipv6(ip)) => ip.is_loopback(),
        None => false,
    }
}

#[cfg(all(test, feature = "carddav", feature = "jmap"))]
mod tests {
    use crate::wizard::server::*;

    /// The protocol and the https URL a typed server URL reads as.
    fn read(input: &str) -> (&'static str, String) {
        let kind = parse(input).expect("parse the server URL");
        let url = kind.server().expect("a server").to_string();

        (kind.protocol(), url)
    }

    #[test]
    fn a_scheme_decides_the_protocol() {
        assert_eq!(
            read("https://dav.example.org/dav/"),
            ("CardDAV", String::from("https://dav.example.org/dav/"))
        );
        assert_eq!(
            read("carddav://dav.example.org"),
            ("CardDAV", String::from("https://dav.example.org/"))
        );
        assert_eq!(
            read("CardDAVs://dav.example.org:8443/alice/"),
            (
                "CardDAV",
                String::from("https://dav.example.org:8443/alice/")
            )
        );
        assert_eq!(
            read("jmap://api.example.org/jmap/session"),
            ("JMAP", String::from("https://api.example.org/jmap/session"))
        );
        assert_eq!(
            read("jmaps://api.example.org"),
            ("JMAP", String::from("https://api.example.org/"))
        );
    }

    #[test]
    fn a_cleartext_server_is_refused_naming_it() {
        let err = parse("http://dav.example.org").expect_err("refuse http");

        assert!(err.to_string().contains("`http://dav.example.org`"));
    }

    #[test]
    fn a_cleartext_server_on_this_machine_is_kept() {
        assert_eq!(
            read("http://localhost:5232/"),
            ("CardDAV", String::from("http://localhost:5232/"))
        );
        assert_eq!(
            read("http://127.0.0.1:5232/"),
            ("CardDAV", String::from("http://127.0.0.1:5232/"))
        );
        assert!(parse("http://localhost.example.org/").is_err());
    }

    #[test]
    fn a_server_url_needs_a_known_scheme_and_a_host() {
        assert!(parse("dav.example.org").is_err());
        assert!(parse("dav.example.org:8443").is_err());
        assert!(parse("imaps://imap.example.org").is_err());
        assert!(parse("https://").is_err());
        assert!(parse("jmaps://:8443").is_err());
    }
}
