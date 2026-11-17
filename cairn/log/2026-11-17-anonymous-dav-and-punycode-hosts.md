---
cairn: log
change: anonymous-dav-and-punycode-hosts
date: 2026-11-17
---

# Anonymous DAV and Punycode hosts

Audit line COH-6, the owner's two rulings of 2026-11-17. Capabilities **config** and **packaging** moved.

- **src/config.rs**: `carddav.auth` defaults to `CarddavAuthConfig::None`, so a block with no `auth` loads and sends no `Authorization` header. `parse_url` maps url's `IdnaError` to "Host `<host>` is not ASCII: write it in its Punycode form (xn--...)"; `carddav.home` deserializes through it and JMAP's `parse_server` calls it.
- **src/carddav/client.rs**: `parse_carddav_server` refuses a non-ASCII host where it used to prefix the failed URL with `https://`, which read `https://bücher.example/dav/` as the host `https` (seen on the HEAD sources: `Url { host: Some(Domain("https")), path: "//b%C3%BCcher.example/dav/" }`). `unauthenticated` puts "CardDAV server answered 401: it asks for credentials, and the account configures no `auth`" above a 401 when the block sends no credential, the code left `null`.
- **src/shared/client.rs, src/session.rs, src/cli.rs, src/account/check.rs**: the sentence reaches the unified commands (`AddressbookClient::explain`, applied by `run_unified`), the `carddav` namespace and `account check`.
- **Cargo.toml**: url drops its `serde` feature, nothing deserializing a `Url` any more.
- **Evidence**: `a_carddav_block_without_auth_loads`, `a_non_ascii_dav_host_names_its_punycode_form`, the extended `a_non_ascii_host_is_refused` (src/config.rs) and `a_401_without_auth_names_the_missing_auth` (src/account/check.rs, a loopback server answering 401) fail on the HEAD sources ("missing field `auth`", the host-`https` URL, "invalid international domain name", an `error` of `WebDAV server returned HTTP 401` alone) and pass after.

CHANGELOG (Changed, Fixed). config.sample.toml: the `auth = "none"` line says it is the same as no `auth`. Spec: config ("A DAV server may ask for no credentials", modified, two scenarios), packaging ("Hosts are ASCII", modified, one scenario).
