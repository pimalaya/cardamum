---
cairn: delta
change: default-is-what-the-server-states
---

# Delta

Each requirement names its capability file in parentheses.

## ADDED Requirements

### Requirement: An address book says whether it is the default
(commands) `addressbook list` SHALL print `default` on each address book, `true` on the one the backend states a card goes to when none is named: JMAP's `isDefault` (RFC 9610 §2), the Graph default Contacts folder (`contacts`), People's myContacts, the pimdir store's role (STORAGE §14); `false` on the others, and on every CardDAV and vdir address book, which state none. It SHALL NOT read `addressbook.default`, which stays the configuration's.

#### Scenario: The default JMAP address book
- GIVEN a JMAP account whose `AddressBook/get` answers `RBk` with `isDefault: true`
- WHEN `cardamum --json addressbook list` runs
- THEN `RBk` carries `"default":true`, as `jmap address-book get` shows `isDefault`

## MODIFIED Requirements

None.

## REMOVED Requirements

None.
