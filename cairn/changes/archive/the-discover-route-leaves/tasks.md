---
cairn: tasks
change: the-discover-route-leaves
---

# Tasks

- [x] src/carddav/client.rs: drop the `discover` route, the PACC and RFC 6764 lookups and the Google probe
- [x] src/config.rs: `carddav.discover` refused by name (`CarddavDiscoverRemoved`), naming `server`, `home` and MIGRATION.md
- [x] Cargo.toml: io-pim-discovery, its feature wiring and the `[patch.crates-io]` section out; Cargo.lock drops what only it pulled
- [x] tests/account.rs: `a_carddav_discover_key_is_refused_by_name` through the binary; tests/google.rs: the CardDAV account's home the subject's
- [x] config.sample.toml, README, CONTRIBUTING, the `carddav discover` help, CHANGELOG, MIGRATION.md
- [x] Fold the delta into spec/backends.md and spec/config.md, log
- [x] fmt, tests, the CI clippy sets, the two reduced test builds, `cargo deny check`, `cargo tree -i io-pim-discovery` empty
