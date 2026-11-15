---
cairn: change
id: the-discover-route-leaves
status: landed
created: 2026-11-15
---

# The discover route leaves

## Why

The owner decided on 2026-11-15 that cardamum no longer depends on io-pim-discovery, as himalaya, ortie and neverest do not (ortie dropped it with its wizard). The `discover` route of the `[carddav]` block was its last user: it resolved a bare domain on every run through PACC and RFC 6764 (SRV, TXT `path`, `.well-known`), paying DNS and HTTP round-trips a configuration naming its server never needs, through a DNS stack (domain, octseq, psl, resolv-conf, ipconfig) the rest of the binary does not use. Calendula drops `caldav.discover` in parallel.

## What

- Remove the `discover` route: `CarddavConfig` keeps `server` (the context root, a bare origin probed through `.well-known/carddav`, then the principal and home set walk) and `home`. Delete `discover_server`, the PACC and RFC 6764 lookups, the DNS resolver constant, and the Google-only authenticated `.well-known` probe (`is_google`, `google_carddav_server`), which only `discover = "gmail.com"` reached.
- A configuration still holding `carddav.discover` is refused by name, as the round's removed tables are: ``The `carddav.discover` key is removed, cardamum resolving no domain: name the server in `carddav.server` instead (Fastmail's `https://carddav.fastmail.com/dav/`), or on Google the address book home in `carddav.home` (…), see MIGRATION.md``. A build without `carddav` loads the block unread, as before.
- Cargo.toml: drop io-pim-discovery, its feature wiring (`carddav`, the TLS features) and its `[patch.crates-io]` git pin, the last entry, so the section leaves. The Cargo.lock diff drops io-pim-discovery and what only it pulled.
- Google's CardDAV is reached through `home` (`https://www.googleapis.com/carddav/v1/principals/<email>/lists`), the route README and the 2026-08-09 live run already used. `server = "https://www.googleapis.com/.well-known/carddav"` does not replace `discover = "gmail.com"`: Google redirects only an authenticated PROPFIND there, and io-webdav surfaces a redirect during the principal walk as an error rather than follow it.
- config.sample.toml, README, CONTRIBUTING, the `carddav discover` command help, the Google live suite (its home the subject's), CHANGELOG and MIGRATION.md follow.

## Scope / non-goals

The `carddav discover` command stays: it reports the endpoints the client walked, and never used io-pim-discovery. No new discovery replaces the route.
