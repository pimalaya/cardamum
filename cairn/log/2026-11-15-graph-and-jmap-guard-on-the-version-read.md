---
cairn: log
change: graph-and-jmap-guard-on-the-version-read
date: 2026-11-15
---

# Graph and JMAP guard on the version read

`--if-match` on Graph and JMAP is compared with the version the backend reads before the write (the contact's `changeKey`, the ContactCard's hash), at no extra request on an update, which reads its base anyway, and one read first on a guarded delete; a stale one fails `precondition-failed` before the write. Both declare `card.update.if-match` and `card.remove.if-match` `partial`, as calendula declares its Graph guard. `card update` with no source now guards there too. Task C22 of cli-alignment-2026-11-15. Capabilities **commands** and **backends** moved. Evidence: scripted Graph and JMAP servers (src/msgraph/backend.rs, src/jmap/backend.rs) assert the refusal with no write sent and the write let through; the Graph update test of C21 still sends one plain `PATCH`.
