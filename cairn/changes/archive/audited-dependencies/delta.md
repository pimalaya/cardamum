---
cairn: delta
change: audited-dependencies
---

# Delta

## ADDED Requirements

### Requirement: A People update keeps what the vCard cannot show
(projection) A People update replaces each field of its mask whole, and a managed field holds more than its vCard shows: a further organization or occupation, a relation of another type, a custom type, a sub-field without a vCard slot, the client data of other clients. `update_card` SHALL re-attach all of it from the person it just read, in the fields of the mask only, through io-gpeople's `GpeoplePerson::merge_unprojected`, between computing the mask and sending the update, and SHALL carry no merge of its own.

#### Scenario: A second organization survives an edit of the first
- GIVEN a person holding the organizations Acme (symbol `ACM`) and Initech, and a `clientData` entry of another client
- WHEN the card is updated with `ORG` changed to Globex and a new `X-PET` line
- THEN the update sends Globex with its symbol `ACM`, then Initech, and the other client's entry beside the stash

### Requirement: A listing is whole or fails
(backends) A listing SHALL never come back shorter than the backend holds without saying so. A CardDAV listing whose answer is not one whole `multistatus` document (an HTML page, a cut body, an empty one) SHALL fail with io-webdav's error. A vdir listing SHALL leave out a card removed while it lists and SHALL fail on a card it cannot read. A Graph listing SHALL follow its next page through io-msgraph's `page_get`, which refuses a link off the Graph origin, so the token never follows a link elsewhere.

## MODIFIED Requirements

### Requirement: Oversized lines stay local
(projection) Added: on Google People, a create or an update leaving such a line out SHALL log a warning naming the card and each property left out (`GpeoplePersonVcard::dropped`), the receipt of the unified command staying the one every backend prints. Scenario "A photo too long for the stash" added.

### Requirement: Clearing an optional field removes it
(backends) Added: on JMAP a cleared description leaves as `null` in the `AddressBook/set` update.

### Requirement: Card ids are backend-native and verbatim
(backends) Added: a CardDAV write SHALL report the id io-webdav answers, never the caller's string: `card create` and `carddav put` print the last segment of the `Location` the server names, else of the URL sent, percent-encoded as a listing shows it.

## REMOVED Requirements

None.
