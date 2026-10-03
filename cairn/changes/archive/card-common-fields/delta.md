---
cairn: change
change: card-common-fields
---

# Delta

## ADDED Requirements

### Requirement: A card projects its common fields
`card list` rows and the `card read` output SHALL carry the card's common fields, read through vcard-rs's decoded model rather than off raw lines: `uid`, `fullName`, `givenName` and `familyName` (the `N` components, space-joined), `emails` and `phones` (every `EMAIL` and `TEL` in document order, a `tel:` URI as its number), `organization` and `organizationUnits` (the `ORG` components), `title` and `note`. A card that does not parse SHALL project nothing rather than fail the listing. The projection is read-only: the vCard stays the record.

## MODIFIED Requirements

None.

## REMOVED Requirements

None.
