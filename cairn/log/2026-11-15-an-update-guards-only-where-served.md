---
cairn: log
change: an-update-guards-only-where-served
date: 2026-11-15
---

# An update guards only where the backend takes a guard

`card update` with no vCard source and no `--if-match` works again on Microsoft Graph and JMAP, sending the plain write, where it failed at once with "Microsoft Graph does not support If-Match guarded updates" (or the JMAP twin), exit 1, nothing sent. Line C21 of the CLI alignment round, found by live testing on the Microsoft test account. Capability **commands** moved.

- **Cause**: the shared command sent the version of the card it read as the write's guard (card-composer, 2026-08-31), and the Graph read reports the `changeKey`, the JMAP read a hash of the card, as that version, while both adapters refuse any guard. The CardDAV preflight read of carddav-update-guard-etag stays inside its adapter and was not involved.
- **src/shared/client.rs**: `AddressbookClient::serves`, whether the backend declares a capability above `none`; `open_msgraph`, test-only, opens the client on a scripted Graph stream.
- **src/shared/card/update.rs**: the read version guards the write only where the backend serves `card.update.if-match` (carddav, gpeople, pimdir); the help says so. An explicit `--if-match` on jmap, msgraph or vdir is still refused `capability-missing` before connecting.
- **Checked clean**: vdir reads no version; `card delete` sends only the caller's `--if-match`; no `addressbook` command reads a guard.
- **Evidence**: `a_graph_update_without_if_match_sends_one_plain_patch` (src/shared/card/update.rs) scripts the Graph reads and the `PATCH`: one `PATCH` without `If-Match`, `{"id":"AAMk","keptProperties":[]}`. It failed on the old code with the live error.
- **Open**: Graph and JMAP could honour a guard as calendula's Graph backend does, comparing the `changeKey` or hash against the base `update_card` already reads (`partial`); not done here.

CHANGELOG (Fixed). Spec: commands ("An update guards on the version it read", modified).
