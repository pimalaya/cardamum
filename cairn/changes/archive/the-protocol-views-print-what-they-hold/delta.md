---
cairn: delta
change: the-protocol-views-print-what-they-hold
---

# Delta

Each requirement names its capability file in parentheses.

## ADDED Requirements

None.

## MODIFIED Requirements

### Requirement: A command prints what the server returned
(commands) A command SHALL render only the fields the server's response actually carries, and SHALL report success in words when the response carries nothing to show, rather than printing empty field slots. This matters most on the write path, where a JMAP `set` response echoes only the properties the server itself decided, and where a REST delete answers 204 with no body at all: text output SHALL then print nothing. `--json` still emits the raw native payload.

A protocol view SHALL print the fields an item holds, no fewer and no empty one: `carddav propfind` an address book's name, description, colour, CTag and sync token, `vdir list` a collection's name, description and colour beside its path, a property the item lacks printing no line; `msgraph contacts` a contact's name, then every email and every phone it holds, the mobile, business and home ones, each kind on one line.

#### Scenario: An address book with a description and no name
- GIVEN a CardDAV address book whose PROPFIND carries a description and a CTag and no display name nor colour
- WHEN `cardamum carddav propfind` runs
- THEN its record prints `Description:` and `CTag:` lines and no `Name:` nor `Color:` line

#### Scenario: A contact with a business phone alone
- GIVEN a Graph contact whose `businessPhones` holds one number and which has no mobile phone nor email
- WHEN `cardamum msgraph contacts get` prints it
- THEN it prints that number on a `Phone:` line and no `Email:` line

## REMOVED Requirements

None.
