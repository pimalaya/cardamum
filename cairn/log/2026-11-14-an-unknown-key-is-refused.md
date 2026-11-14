---
cairn: log
change: an-unknown-key-is-refused
date: 2026-11-14
---

# An unknown key is refused

A configuration key Cardamum does not know fails the load at every level (owner decision, 2026-11-14), the root and the accounts included, which tolerated any key. Capability **config** moved.

- **src/config.rs**: `deny_unknown_fields` on `Config` and `AccountConfig`, every other table carrying it already; no table flattens another, so none needed restructuring. The tables v0.3 read and this release removed are uninhabited types whose deserializer fails naming the table and MIGRATION.md, as Calendula's are: `TableRemoved` (`table`, root and account), `AccountTableRemoved` (the top-level `account`), `AddressbookListRemoved` (`addressbook.list`), `CardListTableRemoved` (`card.list.table`), the first two ignored until now, the last two refused as unknown fields. The block of a backend the build leaves out, and `proxy` in a build with no network backend, is an `IgnoredAny` field under `cfg(not(feature))`, so a file serving a full build still loads in a reduced one, as it did.
- **Evidence**: `an_unknown_root_key_is_refused`, `an_unknown_account_key_is_refused` and `a_removed_table_is_refused_by_name` fail on the old code; `an_unknown_backend_or_nested_key_is_refused` (`vdir.home-dri`, `carddav.tls.certificate`, `card.list.page-sise`) passes on it, those tables refusing already; `a_backend_left_out_of_the_build_still_loads` runs in the vdir-only test build of CI; `the_sample_configuration_parses_with_each_variant` passes.

CHANGELOG (**Breaking:** the strict root and accounts, the removed tables by name), MIGRATION.md (the removed tables and the unknown keys). Spec: config ("An unknown key is refused" added, "One backend block per backend" and "No colour is configured" modified).
