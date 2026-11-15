---
cairn: tasks
change: graph-and-jmap-guard-on-the-version-read
---

# Tasks

- [x] src/msgraph/backend.rs and src/jmap/backend.rs compare the version; src/error.rs `version_matches` and `card_changed`.
- [x] src/capability.rs: both rows `partial`, the details; the capability tests and tests/capability.rs follow.
- [x] Tests on a scripted server: a stale version refused with no write sent, the version read let through, a delete reading first only when guarded (each refused `--if-match` outright before).
- [x] Fold the delta, log, CHANGELOG.
