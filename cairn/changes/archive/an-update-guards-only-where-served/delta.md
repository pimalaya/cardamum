---
cairn: delta
change: an-update-guards-only-where-served
---

# Delta

Each requirement names its capability file in parentheses.

## ADDED Requirements

## MODIFIED Requirements

### Requirement: An update guards on the version it read
(commands) `card update` given no vCard source SHALL send the ETag of the card it read as `If-Match`, an explicit `--if-match` still winning, on a backend declaring `card.update.if-match` at a level above `none` (carddav, gpeople, pimdir). Having read the card to change it, the command knows the version it diverged from, and an edit that took a minute SHALL NOT silently overwrite a write that landed during it. A source given on the command line is a rewrite that was asked for, and carries no such guard of its own.

A backend declaring `card.update.if-match` at `none` (jmap, msgraph, vdir) SHALL get no guard from the read: its read may still report a version (the Graph `changeKey`, a hash of the JMAP card) that its write takes no guard on, and the update SHALL be the plain write the backend makes, last-write-wins, never a refusal of a guard the caller did not ask for. An explicit `--if-match` there SHALL still fail `capability-missing` ("An unsupported precondition is refused, not dropped").

#### Scenario: An update on Microsoft Graph without --if-match
- GIVEN a Microsoft Graph account holding the contact `AAMk` in its default Contacts folder, read with a `changeKey`
- WHEN `cardamum --json card update -k contacts AAMk --full-name "Ada Byron"` runs, with no `--if-match`
- THEN it sends one `PATCH` carrying no `If-Match`, prints `{"id":"AAMk","keptProperties":[]}` and exits 0; the same command with `--if-match k1` fails `capability-missing` before connecting

## REMOVED Requirements
