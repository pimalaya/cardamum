---
cairn: tasks
change: default-is-a-name
---

# Tasks

- [x] Pin pimalaya-config d5a33cba, the Cargo.lock diff its source line alone.
- [x] `account-missing`, `account-default-missing`, `account-default-several` in `ErrorCode`, read from `AccountError` in the chain walk.
- [x] Tests: a unit test coding each variant, and tests/account.rs running the binary under `--json` for each code and for `-a default` (each failing on the old code).
- [x] No doc, sample or test of the app said `-a default` meant the default account.
- [x] CHANGELOG breaking entry, MIGRATION.md.
- [x] Fold the delta into config.md and commands.md, write the log entry, set `landed`.
