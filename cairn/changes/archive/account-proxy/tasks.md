---
cairn: tasks
change: account-proxy
---

# Tasks

- [x] config.rs: `ProxyConfig` with `resolve`, account `proxy`, `<backend>.proxy`, inheritance on account take, `proxy` in the render order.
- [x] JMAP and Graph: pass the resolved proxy through the connect options.
- [x] CardDAV: `connect_webdav` opens the proxied stream, `.well-known` probes take the proxy.
- [x] People: `connect_people` opens the proxied stream.
- [x] Wizard literals carry `proxy: None`.
- [x] Test: account proxy inherited, backend proxy wins.
- [x] config.sample.toml, CHANGELOG.md.
- [x] clippy over all features and every backend alone, tests, `cargo fmt`.
