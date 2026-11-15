---
cairn: log
change: the-discover-route-leaves
date: 2026-11-15
---

# The discover route leaves

Cardamum no longer depends on io-pim-discovery (owner decision 2026-11-15), as himalaya, ortie and neverest do not: the `discover` route of the `[carddav]` block, its last user, leaves, and a configuration still holding it is refused by name. Capabilities **backends** and **config** moved.

- **src/carddav/client.rs**: `open_carddav_client` takes `home` or `server`, failing ``CardDAV config needs `server` or `home` `` with neither. `discover_server`, `discover_via_pacc`, `discover_via_rfc6764`, `DEFAULT_RESOLVER`, and the Google-only authenticated `.well-known` PROPFIND (`is_google`, `google_carddav_server`, `GOOGLE_API_HOST`, `GOOGLE_API_ORIGIN`) are gone; the unauthenticated bare-origin probe stays for `server`.
- **src/config.rs**: `CarddavConfig::discover` is a `CarddavDiscoverRemoved`, which refuses any value naming `carddav.server` (Fastmail's `https://carddav.fastmail.com/dav/`), `carddav.home` on Google and MIGRATION.md. A build without `carddav` loads the block unread, as before.
- **Cargo.toml**: io-pim-discovery, its `carddav` and TLS feature wiring and its `[patch.crates-io]` git pin leave, the section with it. The Cargo.lock diff drops io-pim-discovery, domain, domain-macros, octseq, psl, psl-types, resolv-conf, ipconfig, socket2 0.6.5, widestring, windows-registry, windows-result and windows-strings, and moves nothing else; `cargo tree -i io-pim-discovery` matches no package.
- **Google**: `server = "https://www.googleapis.com/.well-known/carddav"` is no replacement, since Google redirects only an authenticated PROPFIND there and io-webdav 0.9 surfaces a redirect during the principal walk as `UnexpectedRedirect`. The sample's Google CardDAV section names `carddav.home` (`…/principals/alice@gmail.com/lists`), the route README and the 2026-08-09 live run used; tests/google.rs points the credentialed suite's home at its subject.
- **Evidence**: `a_carddav_discover_key_is_refused_by_name` (tests/account.rs) runs the binary under `--json` over `carddav.discover = "fastmail.com"`: exit 1, the parse error naming the key, `carddav.server`, `carddav.home` and MIGRATION.md. On the old code the key loaded and `account capability` exited 0.

config.sample.toml, README, CONTRIBUTING, the `carddav discover` help, CHANGELOG (**Breaking:** removed the `discover` route; the dependency removal), MIGRATION.md. Spec: backends ("CardDAV home-set resolution routes", "CardDAV probes a bare origin before walking"), config ("CardDAV resolves no domain" added, "A proxy is set per account and per backend").
