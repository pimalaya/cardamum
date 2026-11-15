---
cairn: delta
change: a-read-prints-its-listing-row
---

# Delta

Each requirement names its capability file in parentheses.

## ADDED Requirements

### Requirement: A card read prints its listing row and its vCard
(commands) `card read` SHALL print the row `card list` prints for the card, every key with the same value (`id`, `addressbookId`, `etag`, `fnValue`, `email`, `tel` and the common fields), `etag` `null` where the backend versions no card, plus what only a read carries, the raw vCard, as pimalaya-cli's output.md says a read prints ("Collections are named and present").

The vCard SHALL travel as `contents`, a string: the text when its bytes are valid UTF-8, `encoding` then `utf-8`, and their base64 (RFC 4648 standard alphabet, padded) when they are not, `encoding` then `base64`, so a script reads `encoding` and never takes base64 for a vCard (output.md, "Bytes are never raw under JSON"), as calendula prints an item's bytes. The bytes SHALL NOT be decoded lossily under `--json`, and a card that is not UTF-8 SHALL read rather than fail. The text form prints the card as it does, defused ("Plain output is printable"). `card build` SHALL carry its card the same way, `{contents, encoding}`, and in text SHALL print a card that is not UTF-8 as its bytes, which a pipe takes on unchanged.

#### Scenario: A card read
- GIVEN a vdir card `ada` whose `FN` is `Ada Lovelace`
- WHEN `cardamum --json card read -k personal ada` runs
- THEN it prints the keys its `card list` row prints with the same values, `"etag":null`, `"contents":"BEGIN:VCARD…"` and `"encoding":"utf-8"`

#### Scenario: A card that is not UTF-8
- GIVEN a card whose bytes hold a Latin-1 `é`
- WHEN `cardamum --json card read` reads it
- THEN it exits 0 and prints `"contents"` holding its bytes in base64 and `"encoding":"base64"`, its common fields `null`

## MODIFIED Requirements

### Requirement: A card projects its common fields
(commands) `card list` rows and the `card read` output SHALL carry the card's common fields, read through vcard-rs's decoded model rather than off raw lines: `uid`, `fullName`, `givenName` and `familyName` (the `N` components, space-joined), `emails` and `phones` (every `EMAIL` and `TEL` in document order, a `tel:` URI as its number), `organization` and `organizationUnits` (the `ORG` components), `title` and `note`. A card that does not parse, or whose bytes are not UTF-8, SHALL project nothing rather than fail the listing, never a lossy decode. The projection is read-only: the vCard stays the record.

## REMOVED Requirements
