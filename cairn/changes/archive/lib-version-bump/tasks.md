---
cairn: tasks
change: lib-version-bump
---

# Tasks

- [x] Cargo.toml: io-pim-discovery 0.3 to 0.5, pimalaya-cli 0.1 to 0.2 (build-dependency too), comfy-table 7 to 8, base64 0.22 to 0.23, vcard-rs 0.1 to 0.2; dependencies re-alphabetized.
- [x] Drop the pimalaya-config git patch (0.1.1 released); keep io-webdav, add io-pimdir and io-replica.
- [x] Port shared/table.rs from Himalaya (v7 preset string to v8 `TableStyle`), with its seven mapping tests.
- [x] Swap all 18 `load_preset` call sites for `load_style(style_from_preset(..))` and thread the import.
- [x] Point the two preset defaults at `DEFAULT_PRESET` instead of `presets::UTF8_FULL_CONDENSED`.
- [x] io-webdav naming canon across src/carddav/: `CarddavAddressbook`, `CarddavCardEntry`, `CarddavCardRef`, `WebdavSendError`, `WebdavMultistatus`, `WebdavReport`, `WebdavResponseEntry`, `WebdavSyncChange`, `WebdavSyncDelta`.
- [x] vcard-rs lens path in msgraph/project.rs and people/project.rs.
- [x] Enable pimalaya-cli `jmap` unconditionally so the shared keyring picker is reachable from every backend (upstream gates it behind the protocol features).
- [x] `cargo build --all-features`, `cargo test --all-features`, `cargo clippy --all-features`, per-backend feature matrix, `cargo fmt`.
- [x] `cargo tree -d`: comfy-table resolves to one version.
