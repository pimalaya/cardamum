---
cairn: delta
change: a-card-previews-its-folded-lines
---

# Delta

Each requirement names its capability file in parentheses.

## ADDED Requirements

None.

## MODIFIED Requirements

### Requirement: A card projects its common fields
(commands) `card list` rows and the `card read` output SHALL carry the card's common fields, read through vcard-rs's decoded model rather than off raw lines: `uid`, `fullName`, `givenName` and `familyName` (the `N` components, space-joined), `emails` and `phones` (every `EMAIL` and `TEL` in document order, a `tel:` URI as its number), `organization` and `organizationUnits` (the `ORG` components), `title` and `note`. A card that does not parse, or whose bytes are not UTF-8, SHALL project nothing rather than fail the listing, never a lossy decode. The projection is read-only: the vCard stays the record.

The `fnValue`, `email` and `tel` keys, and the name, email and phone columns of the `card list` text, SHALL be the first `FN`, `EMAIL` and `TEL` values as written, escapes kept and a `tel:` URI whole, read off the card's lines once unfolded (RFC 6350 §3.2), a property matched by its whole name, case-insensitive, whatever its group and its parameters (§3.3), its value starting at the first colon outside a quoted parameter value.

#### Scenario: A derived name and a folded address
- GIVEN a card holding `FN;DERIVED=TRUE:Jane Doe` and an `EMAIL;TYPE=home:jane@examp` line folded before `le.org`
- WHEN `cardamum --json card list` lists it
- THEN its `fnValue` is `Jane Doe` and its `email` `jane@example.org`, the text naming her

## REMOVED Requirements

None.
