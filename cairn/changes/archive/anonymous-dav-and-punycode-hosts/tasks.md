---
cairn: tasks
change: anonymous-dav-and-punycode-hosts
---

# Tasks

- [x] src/config.rs: `carddav.auth` defaults to `none`; `parse_url` refuses a non-ASCII host naming its Punycode form, `carddav.home` deserialized through it, JMAP's `parse_server` calling it.
- [x] src/carddav/client.rs: `parse_carddav_server` refuses rather than prefixes a non-ASCII host; `unauthenticated` adds the missing-`auth` sentence to a 401.
- [x] src/shared/client.rs `AddressbookClient::explain` over src/session.rs `run_unified`, src/cli.rs `carddav` namespace, src/account/check.rs `connect_carddav`.
- [x] Cargo.toml: url without `serde`.
- [x] Tests `a_carddav_block_without_auth_loads`, `a_non_ascii_dav_host_names_its_punycode_form` (src/config.rs), `a_401_without_auth_names_the_missing_auth` (src/account/check.rs), each seen failing on the HEAD sources.
- [x] config.sample.toml, spec (config, packaging), CHANGELOG (Changed, Fixed).
