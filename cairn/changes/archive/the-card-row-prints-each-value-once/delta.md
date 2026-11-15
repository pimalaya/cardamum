---
cairn: delta
change: the-card-row-prints-each-value-once
---

# Delta

Each requirement names its capability file in parentheses.

## ADDED Requirements

None.

## MODIFIED Requirements

### Requirement: A card projects its common fields
(commands) `card list` rows and the `card read` output SHALL carry the card's common fields, read through vcard-rs's decoded model rather than off raw lines: `uid`, `fullName`, `givenName` and `familyName` (the `N` components, space-joined), `emails` and `phones` (every `EMAIL` and `TEL` in document order, a `tel:` URI as its number), `organization` and `organizationUnits` (the `ORG` components), `title` and `note`. A card that does not parse, or whose bytes are not UTF-8, SHALL project nothing rather than fail the listing, never a lossy decode. The projection is read-only: the vCard stays the record.

A card row SHALL print each value once: no key beside the common fields repeats one of them, so a row carries no `fnValue`, `email` or `tel`. The email, phone and name columns of the `card list` and `pimdir search` text SHALL be the first of `emails`, the first of `phones` and `fullName`, the values the JSON prints: a property read whatever its group and its parameters (RFC 6350 §3.3), a folded line whole (§3.2), an escape decoded and a `tel:` URI as its number.

#### Scenario: A derived name and a folded address
- GIVEN a card holding `FN;DERIVED=TRUE:Jane Doe` and an `EMAIL;TYPE=home:jane@examp` line folded before `le.org`
- WHEN `cardamum --json card list` lists it
- THEN its `fullName` is `Jane Doe` and its `emails` `["jane@example.org"]`, no `fnValue` or `email` key printed, the text naming her

### Requirement: A card read prints its listing row and its vCard
(commands) `card read` SHALL print the row `card list` prints for the card, every key with the same value (`id`, `addressbookId`, `etag` and the common fields), `etag` `null` where the backend versions no card, plus what only a read carries, the raw vCard, as pimalaya-cli's output.md says a read prints ("Collections are named and present").

The rest of the requirement stands: the vCard as `contents` with its `encoding`.

### Requirement: A card page names its context and its next page
(commands) `card list` SHALL print as a pimalaya-cli `listing::Listing`: above the cards `Account: <name> (<backend>)`, `Addressbook: <id>`, `Page: <n>, cards <a> to <b>` (`card <a>` for one, `empty` for none, `all` when unpaged) followed by `, the last` when no card follows, and `Next page: --page <n>` while one does, the values aligned in one column; then a blank line and one line per card with no column header (`id  email  phone  name`, the first of `emails`, the first of `phones` and `fullName`, the free-text name last, `-` for an empty cell). Below the cards SHALL print only the command's errors ("A command's errors print at its end").

## REMOVED Requirements

None.
