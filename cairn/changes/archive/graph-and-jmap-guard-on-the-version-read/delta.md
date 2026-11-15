---
cairn: delta
change: graph-and-jmap-guard-on-the-version-read
---

# Delta

Each requirement names its capability file in parentheses.

## ADDED Requirements

## MODIFIED Requirements

### Requirement: An update guards on the version it read
(commands) `card update` given no vCard source SHALL guard the write with the version of the card it read, an explicit `--if-match` still winning, on a backend declaring `card.update.if-match` at a level above `none` (carddav, gpeople, pimdir, jmap, msgraph). Having read the card to change it, the command knows the version it diverged from, and an edit that took a minute SHALL NOT silently overwrite a write that landed during it. A source given on the command line is a rewrite that was asked for, and carries no such guard of its own.

CardDAV and People send the version with the write (`If-Match`, the People `etag`) and pimdir checks it before queueing. Microsoft Graph and JMAP take no guard on a contact write, so they SHALL compare it themselves, as calendula guards a Graph event, at no request of their own on an update: Graph with the `changeKey` of the contact the update reads as its delta base, JMAP with the hash of the ContactCard the update reads as its patch base (the version `card list` and `card read` report). A version that differs, surrounding double quotes ignored, SHALL fail `precondition-failed` before the write, ``Card `<id>` changed on <server> since it was read``; a write landing between that read and the write is not caught, which their `partial` level says. vdir, declaring `none`, reports no version on a read and SHALL refuse an explicit `--if-match` with `capability-missing` ("An unsupported precondition is refused, not dropped").

#### Scenario: An update on Microsoft Graph without --if-match
- GIVEN a Microsoft Graph account holding the contact `AAMk` in its default Contacts folder, read with the `changeKey` `k1`
- WHEN `cardamum --json card update -k contacts AAMk --full-name "Ada Byron"` runs, with no `--if-match`
- THEN it sends one `PATCH` carrying no `If-Match`, prints `{"id":"AAMk","keptProperties":[],"queued":false}` and exits 0; the same command with `--if-match k0` fails `precondition-failed` with no `PATCH` sent

### Requirement: A delete can be gated on a version
(commands) `card delete` SHALL take `--if-match <ETAG>`. CardDAV SHALL send it as `If-Match`, pimdir SHALL check it as an update does, and Graph and JMAP SHALL read the card first, only when `--if-match` is given, and compare its `changeKey` or its hash as an update does, failing `precondition-failed` before the delete. The backends that cannot gate a delete (vdir, gpeople) SHALL refuse it rather than drop it.

### Requirement: An unsupported precondition is refused, not dropped
(backends) A backend with no entity-tag concept SHALL refuse `--if-match` rather than accept and ignore it: its table declares `card.update.if-match` or `card.remove.if-match` at `none`, so the command fails with `capability-missing` before connecting, and the adapter keeps its own refusal. Silently dropping a guard the caller asked for is the one outcome that reads as protection and is none. A backend whose read reports a version its write takes no guard on (the Graph `changeKey`, the hash of a JMAP ContactCard) SHALL compare it with the card it reads just before the write and declare the guard `partial`, the detail saying a write landing in between is not caught ("A delete can be gated on a version", commands).

### Requirement: A JSON error carries a stable code
(commands) One row changes, the rest unchanged:

| Code | Raised when |
|---|---|
| `precondition-failed` | a write's `--if-match` names a version the card no longer has: pimdir's check, a CardDAV server answering 412, or the Graph `changeKey` or JMAP hash read before the write |

## REMOVED Requirements
