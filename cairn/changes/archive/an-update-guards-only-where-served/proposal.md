---
cairn: change
id: an-update-guards-only-where-served
status: landed
created: 2026-11-15
---

# An update guards only where the backend takes a guard

> Line C21 of the family's CLI alignment round (/home/soywod/code/pimalaya/.github/cairn/changes/cli-alignment-2026-11-15), a defect found by live testing on the Microsoft test account on 2026-11-15.

## Why

On released cardamum 601f8b9, `cardamum -a microsoft card update -k <folder> <id> --full-name x`, with no `--if-match`, failed at once with "Microsoft Graph does not support If-Match guarded updates", exit 1, no write sent, while `cardamum msgraph contacts update` changed the same contact.

The cause is the shared command, not the CardDAV adapter. The card-composer change (2026-08-31, requirement "An update guards on the version it read") made `card update` with no vCard source send the `etag` of the card it read as the write's `if_match` (src/shared/card/update.rs). The Graph read reports the contact's `changeKey` as that `etag` and the JMAP read a hash of the card, while both adapters refuse any `if_match` (src/msgraph/backend.rs and src/jmap/backend.rs, `update_card`), their tables declaring `card.update.if-match` at `none`. So every field-flag or `-i` update failed on Graph and on JMAP. The carddav-update-guard-etag change (2026-08-09) reads the version inside the CardDAV adapter only, and reaches no other backend.

The other backends were clean: vdir reads no version (`etag: None`), carddav, gpeople and pimdir take the guard. `card delete` sends only the caller's `--if-match`, which the capability check refuses on the backends without one, and no `addressbook` command reads a guard.

## What

- `AddressbookClient::serves`, whether the backend declares a capability above `none`.
- `card update` keeps the read version as the guard only where the backend serves `card.update.if-match`: on jmap and msgraph the write is the plain one the adapter makes. An explicit `--if-match` there is still refused `capability-missing` before connecting, and the adapters keep their own refusal.
- Spec: commands ("An update guards on the version it read", modified).

## Left open

Graph and JMAP could honour a guard as calendula's Graph backend does: compare the `changeKey` (or the JMAP hash) against the base each `update_card` already reads, refusing `precondition-failed` on a mismatch, at no extra request, declared `partial` since a write landing between the read and the `PATCH` is not caught. Not done here: it changes what the tables declare.
