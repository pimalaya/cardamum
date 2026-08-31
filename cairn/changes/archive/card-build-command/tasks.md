---
cairn: tasks
change: card-build-command
---

# Tasks

- [x] Hoist the composer pair into `CardComposerArgs` in shared/arg.rs and flatten it into `card create` and `card update`.
- [x] Write shared/card/build.rs: the source, the field flags, the check, and the vCard it prints.
- [x] Resolve the account per card subcommand, so `card build` runs with no configuration.
- [x] Register `cardamum-card-build` in the JSON Schema registry.
- [x] Document the command in CHANGELOG.md.
- [x] Check, clippy and test the full feature set. The per-backend matrix was run and does not build on this tree, for reasons that predate this change: io-pim-discovery 0.7.0 imports and client `connect` signatures.
- [x] Fold the delta into the spec and write the log entry.
