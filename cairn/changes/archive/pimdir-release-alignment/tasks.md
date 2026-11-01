---
cairn: tasks
change: pimdir-release-alignment
---

# Tasks

- [x] Refuse a write into an address book no source syncs (src/pimdir/backend.rs), test failing first
- [x] No `etag` and no `--if-match` match for a card whose blob is missing (src/pimdir/backend.rs), tests failing first
- [x] Kind compared on the media type before `;` (src/pimdir/backend.rs), test failing first, the `text/calendar` exclusion covered too
- [x] `account check` opens and drops a producer (src/account/check.rs), test failing first
- [x] `pimdir.account` unset: derive the one account, refuse several naming them (src/pimdir/client.rs), test failing first; config doc, config.sample.toml and README
- [x] Queued wording for `card create`, `card update` and `card delete` on pimdir, `queued` in the create and update JSON (src/shared), test failing first
- [x] README.md:239 and src/pimdir/mod.rs say a staged create is not listed
- [x] Citations: "pimdir SPEC" to STORAGE in the spec, section numbers in src/pimdir/backend.rs
- [x] CHANGELOG under [Unreleased]
- [x] Fold into [backends](../../../spec/backends.md) and [commands](../../../spec/commands.md); log; land
