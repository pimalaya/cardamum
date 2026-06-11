//! Interactive editor for an existing account.
//!
//! Loads the named account from the merged config, walks the same
//! prompts the discover wizard does, and writes the updated config back
//! to `target`.

use std::path::Path;

use anyhow::{Result, anyhow};
#[cfg(feature = "carddav")]
use log::debug;
use log::info;
use pimalaya_cli::prompt;
#[cfg(feature = "carddav")]
use pimalaya_cli::{
    spinner::Spinner,
    wizard::carddav::{
        self as carddav_wizard, CarddavAuth, CarddavSecret, Encryption as CarddavEncryption,
        WizardCarddavConfig,
    },
};
#[cfg(feature = "carddav")]
use pimalaya_config::secret::Secret;
#[cfg(feature = "carddav")]
use pimconf::{
    rfc6186::types::SrvService,
    rfc6764::{client::DiscoveryRfc6764ClientStd, types::Rfc6764Report},
};

use crate::config::{AccountConfig, Config};

#[cfg(feature = "vdir")]
use crate::config::VdirConfig;
#[cfg(feature = "carddav")]
use url::Url;

#[cfg(feature = "carddav")]
use crate::carddav::client::parse_carddav_server;
#[cfg(feature = "carddav")]
use crate::config::{CarddavAuthConfig, CarddavConfig};

#[cfg(feature = "carddav")]
const DEFAULT_RESOLVER: &str = "tcp://1.1.1.1:53";

pub fn edit_account(target: &Path, mut config: Config, account_name: &str) -> Result<Config> {
    let existing = config.accounts.remove(account_name);

    let email = prompt::text::<&str>("Email address:", None)?;
    let (_local_part, _domain) = email
        .rsplit_once('@')
        .ok_or_else(|| anyhow!("Invalid email address `{email}`: missing `@`"))?;
    #[cfg(feature = "carddav")]
    let (local_part, domain) = (_local_part.to_owned(), _domain.to_owned());

    let is_first_account = config.accounts.is_empty() && existing.is_none();
    let default = existing
        .as_ref()
        .map(|a| a.default)
        .unwrap_or(is_first_account);

    #[cfg(feature = "vdir")]
    let existing_vdir = existing.as_ref().and_then(|a| a.vdir.clone());
    #[cfg(feature = "carddav")]
    let existing_carddav = existing.as_ref().and_then(|a| a.carddav.clone());

    #[cfg(feature = "vdir")]
    let want_vdir = prompt::bool("Configure a local vdir backend?", existing_vdir.is_some())?;
    #[cfg(not(feature = "vdir"))]
    let want_vdir = false;

    #[cfg(feature = "carddav")]
    let want_carddav = prompt::bool(
        "Configure a remote CardDAV (CardDAV) backend?",
        existing_carddav.is_some(),
    )?;
    #[cfg(not(feature = "carddav"))]
    let want_carddav = false;

    if !want_vdir && !want_carddav {
        return Err(anyhow!(
            "At least one backend (vdir or carddav) must be configured"
        ));
    }

    #[cfg(feature = "vdir")]
    let vdir = if want_vdir {
        let default_home = existing_vdir.as_ref().map(|v| v.home_dir.clone());
        let home_dir = prompt::text("Vdir home directory:", default_home.as_deref())?;
        Some(VdirConfig { home_dir })
    } else {
        None
    };

    #[cfg(feature = "carddav")]
    let carddav = if want_carddav {
        Some(prompt_carddav(
            account_name,
            &local_part,
            &domain,
            existing_carddav.as_ref(),
        )?)
    } else {
        None
    };

    let account = AccountConfig {
        default,
        table: existing
            .as_ref()
            .map(|a| a.table.clone())
            .unwrap_or_default(),
        addressbook: existing
            .as_ref()
            .map(|a| a.addressbook.clone())
            .unwrap_or_default(),
        card: existing
            .as_ref()
            .map(|a| a.card.clone())
            .unwrap_or_default(),
        #[cfg(feature = "vdir")]
        vdir,
        #[cfg(feature = "carddav")]
        carddav,
    };

    config.accounts.insert(account_name.to_owned(), account);
    config.write(target)?;
    info!("Configuration written to {}", target.display());

    Ok(config)
}

#[cfg(feature = "carddav")]
fn prompt_carddav(
    account_name: &str,
    local_part: &str,
    domain: &str,
    existing: Option<&CarddavConfig>,
) -> Result<CarddavConfig> {
    let prev_wizard = existing.map(carddav_to_wizard_defaults);
    let srv_defaults = srv_discover_carddav(domain);
    let defaults = prev_wizard.as_ref().or(srv_defaults.as_ref());

    let cfg = carddav_wizard::run(account_name, local_part, domain, defaults)?;
    wizard_to_carddav(cfg)
}

/// Runs the RFC 6764 SRV discovery chain for `_carddav` / `_carddavs`
/// against `domain` and folds the best record into a wizard default.
/// Prefers `_carddavs` (TLS) over `_carddav` (plain) when both are
/// published.
#[cfg(feature = "carddav")]
fn srv_discover_carddav(domain: &str) -> Option<WizardCarddavConfig> {
    let spinner = Spinner::start(format!("Probing SRV records for {domain}"));
    let resolver: Url = DEFAULT_RESOLVER
        .parse()
        .expect("DEFAULT_RESOLVER must be a valid URL");
    let mut client = DiscoveryRfc6764ClientStd::new(resolver);

    match client.discover(domain) {
        Ok(report) if !report_is_empty(&report) => {
            spinner.success(srv_summary(domain, &report));
            carddav_from_report(&report)
        }
        Ok(_) => {
            spinner.failure(format!("SRV: no records for {domain}"));
            None
        }
        Err(err) => {
            debug!("SRV discovery for {domain} failed: {err}");
            spinner.failure(format!("SRV: no records for {domain}"));
            None
        }
    }
}

#[cfg(feature = "carddav")]
fn report_is_empty(report: &Rfc6764Report) -> bool {
    report.caldav.is_none()
        && report.caldavs.is_none()
        && report.carddav.is_none()
        && report.carddavs.is_none()
}

#[cfg(feature = "carddav")]
fn srv_summary(domain: &str, report: &Rfc6764Report) -> String {
    let mut protos = Vec::with_capacity(2);
    if report.caldav.is_some() || report.caldavs.is_some() {
        protos.push("CalDAV");
    }
    if report.carddav.is_some() || report.carddavs.is_some() {
        protos.push("CardDAV");
    }
    format!("SRV: discovered {} for {domain}", protos.join(" + "))
}

#[cfg(feature = "carddav")]
fn carddav_from_report(report: &Rfc6764Report) -> Option<WizardCarddavConfig> {
    let (service, encryption) = if let Some(s) = report.carddavs.as_ref() {
        (s, CarddavEncryption::Tls)
    } else if let Some(s) = report.carddav.as_ref() {
        (s, CarddavEncryption::None)
    } else {
        return None;
    };

    Some(srv_service_to_wizard(service, encryption))
}

#[cfg(feature = "carddav")]
fn srv_service_to_wizard(
    service: &SrvService,
    encryption: CarddavEncryption,
) -> WizardCarddavConfig {
    WizardCarddavConfig {
        host: service.host.clone(),
        port: service.port,
        encryption,
        home_url: None,
        // NOTE: empty Basic placeholder; the wizard re-prompts for
        // strategy when the username field is empty.
        auth: CarddavAuth::Basic {
            username: String::new(),
            secret: CarddavSecret::Raw(String::new().into()),
        },
    }
}

/// Folds an existing [`CarddavConfig`] into a [`WizardCarddavConfig`]
/// so the wizard can populate prompt defaults from it.
#[cfg(feature = "carddav")]
fn carddav_to_wizard_defaults(existing: &CarddavConfig) -> WizardCarddavConfig {
    // The wizard model is host/port/encryption; the persisted config is
    // a bare `discover` domain (or an explicit `server` URL), so derive
    // the prompt defaults from whichever is set.
    let server = existing
        .server
        .as_deref()
        .and_then(|server| parse_carddav_server(server).ok());
    let host = server
        .as_ref()
        .and_then(|url| url.host_str().map(str::to_owned))
        .or_else(|| existing.discover.clone())
        .unwrap_or_default();
    let scheme = server.as_ref().map(Url::scheme);
    let encryption = if matches!(scheme, Some("http")) {
        CarddavEncryption::None
    } else {
        CarddavEncryption::Tls
    };
    let port = server
        .as_ref()
        .and_then(Url::port)
        .unwrap_or(default_port(encryption));

    let home_url = existing.home.as_ref().map(Url::to_string);

    let auth = match &existing.auth {
        CarddavAuthConfig::Basic { username, .. } => CarddavAuth::Basic {
            username: username.clone(),
            secret: CarddavSecret::Raw(String::new().into()),
        },
        CarddavAuthConfig::Bearer { .. } => CarddavAuth::Bearer {
            secret: CarddavSecret::Raw(String::new().into()),
        },
    };

    WizardCarddavConfig {
        host,
        port,
        encryption,
        home_url,
        auth,
    }
}

#[cfg(feature = "carddav")]
fn default_port(encryption: CarddavEncryption) -> u16 {
    match encryption {
        CarddavEncryption::Tls => 443,
        CarddavEncryption::None => 80,
    }
}

#[cfg(feature = "carddav")]
fn wizard_to_carddav(cfg: WizardCarddavConfig) -> Result<CarddavConfig> {
    // Persist the host as a bare `discover` domain; pimconf re-derives
    // scheme + port (and follows `.well-known`) at connection time.
    let home = match cfg.home_url {
        Some(raw) => {
            Some(Url::parse(&raw).map_err(|err| anyhow!("Invalid home URL `{raw}`: {err}"))?)
        }
        None => None,
    };

    Ok(CarddavConfig {
        discover: Some(cfg.host),
        server: None,
        home,
        tls: Default::default(),
        auth: carddav_auth_to_config(cfg.auth),
    })
}

#[cfg(feature = "carddav")]
fn carddav_auth_to_config(auth: CarddavAuth) -> CarddavAuthConfig {
    match auth {
        CarddavAuth::Basic { username, secret } => CarddavAuthConfig::Basic {
            username,
            password: carddav_secret_to_secret(secret),
        },
        CarddavAuth::Bearer { secret } => CarddavAuthConfig::Bearer {
            token: carddav_secret_to_secret(secret),
        },
    }
}

#[cfg(feature = "carddav")]
fn carddav_secret_to_secret(secret: CarddavSecret) -> Secret {
    match secret {
        CarddavSecret::Raw(s) => Secret::Raw(s),
        CarddavSecret::Command(cmd) => Secret::Command(pimalaya_config::command::shell(&cmd)),
    }
}
