---
cairn: log
change: lib-version-bump
landed: 2026-08-09
---

# Update to the latest Pimalaya libraries

Bumped io-pim-discovery 0.3 to 0.5, pimalaya-cli 0.1 to 0.2 (dependency and build-dependency), pimalaya-stream to 0.1.2, io-jmap to 0.2.1, io-msgraph to 0.2.2, vcard-rs 0.1 to 0.2, comfy-table 7 to 8 and base64 0.22 to 0.23, and moved io-webdav to its current HEAD. Dropped the pimalaya-config git patch now that 0.1.1 is released; io-webdav, io-pimdir and io-replica stay patched to git until their next release.

The comfy-table bump was forced in lockstep: io-pim-discovery 0.5 exists only to bump pimalaya-cli to 0.2, which exists only to bump comfy-table to v8. v8 drops the positional preset string for a typed `TableStyle` builder, which would have broken the `table.preset` config option. Ported Himalaya's shared/table.rs instead, which maps the v7 spelling onto the v8 builder character by character, so every existing configuration stays valid; its seven tests assert the mapping against the upstream v8 constants. All 18 `load_preset` call sites moved to `load_style(style_from_preset(..))`.

io-webdav's HEAD carries the Pimalaya naming canon (`WebdavSendError`, `WebdavMultistatus`, `WebdavReport`, `WebdavResponseEntry`, `WebdavSyncChange`, `WebdavSyncDelta`, `CarddavAddressbook`, `CarddavCardEntry`, `CarddavCardRef`) plus the card-addressing fixes the CardDAV test reports had flagged: a card's id is the server-returned last path segment used verbatim, with no extension added or stripped anywhere, and a create returns the `Location` id when the server names the resource itself. Cardamum's carddav module already worked that way, so only the imports moved.

vcard-rs 0.2 cost two lines: `tree::prop::VcardPropLens` is now `tree::prop::lens::VcardPropLens`, the flattened re-export having moved onto its real module path. None of the crate's renamed items were in use.

One workaround worth flagging upstream: pimalaya-cli gates its shared keyring and token picker behind `any(imap, smtp, jmap)`, which excludes `carddav` and `caldav`. The picker is protocol-agnostic and should be gated on `wizard`. Cardamum enables `jmap` unconditionally to reach it, with a NOTE on the dependency line; a carddav-only build would otherwise lose the wizard's credential prompt.

Capabilities moved: config (table preset mapping), backends (verbatim card ids).
