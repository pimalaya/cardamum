---
cairn: tasks
change: a-backend-left-out-is-named
---

# Tasks

- [x] `backend::NAMES`, `is_built`, `not_built`; `AccountConfig::backends` over every block, built or not.
- [x] `resolve_backend` walks the declared blocks and fails `backend-not-built` under `auto`; `AddressbookClient::new` and `account capability` propagate it.
- [x] `--backend` naming a left-out backend names its feature.
- [x] `account check` rows and `account list` `notBuilt`.
- [x] Tests in tests/account.rs on a reduced build (`--features rustls-ring,vdir`, which CI tests, and `rustls-ring,carddav` for the order), each failing on the old code.
- [x] CHANGELOG, MIGRATION.md.
- [x] Fold the delta into backends.md, config.md and commands.md, write the log entry, set `landed`.
