---
cairn: delta
change: an-unknown-addressbook-is-named
---

# Delta

Each requirement names its capability file in parentheses.

## ADDED Requirements

### Requirement: A network backend names an unknown address book
(backends) A unified command naming an address book a CardDAV server, JMAP or Microsoft Graph does not hold SHALL fail coded `addressbook-not-found`, ``Addressbook `<id>` not found``, the server's answer in `sources` ("A command resolves its addressbook before acting" in commands), never with the server's bare refusal: on CardDAV a 404 or a 410 of the collection, on `card list`, `addressbook update` and `addressbook delete`; on Graph a 404 or a 400 `ErrorInvalidIdMalformed` of the folder, on `card list`, `card add`, `addressbook update` and `addressbook delete`; on JMAP a `card list` whose `ContactCard/query` fails or finds nothing SHALL ask `AddressBook/get` for the id, one request on that path alone, an id the account does not hold failing so with the query's refusal in `sources`, an empty listing of one it holds staying empty and a refusal over one it holds staying the refusal. A server answering an unknown collection otherwise (iCloud's 400 `invalid collection ID URI`) keeps its refusal.

#### Scenario: An unknown address book on JMAP
- GIVEN a JMAP account answering a `ContactCard/query` filtered on `nope` with `invalidArguments` and holding no such address book
- WHEN `cardamum --json card list -k nope` runs
- THEN it exits 1 coded `addressbook-not-found`, ``Addressbook `nope` not found``, the `invalidArguments` refusal in `sources`, after the query and one `AddressBook/get`

## MODIFIED Requirements

None.

## REMOVED Requirements

None.
