---
cairn: change
id: a-carddav-id-addresses-its-own-resource
status: landed
created: 2026-11-15
---

# A CardDAV id addresses its own resource

> cardamum's part of line W2 of the family's CLI alignment round (/home/soywod/code/pimalaya/.github/cairn/changes/cli-alignment-2026-11-15), after io-webdav 0.9.1 (W1, W3, W4).

## Why

io-webdav 0.9.0 joined a collection or card id as given, and the url crate resolves dot segments when it builds the request URL: `cardamum addressbook delete ..` could send a DELETE to the home set's parent, `/` to the server root and `""` to the home set (`<home>//`), `addressbook update ..` a PROPPATCH there. Found by reading (W1), never tried on a server. io-webdav 0.9.1 refuses such an id before any byte (`rfc4918::check_collection_id`, `check_resource_id`), failing `WebdavClientStdError::Io` of kind `InvalidInput`, which cardamum printed uncoded, after resolving the account's secret and connecting.

`carddav report raw` built its path by hand (src/carddav/report/raw.rs), out of io-webdav's check and encoding, and counted `dataBytes` through `WebdavResponseEntry::text`, which trims: a card whose final CRLF the server escapes (`&#13;`, iCloud and Fastmail) counted two bytes short.

io-webdav 0.9.1 also returns `address-data` whole through its typed listings (W3) and percent-encodes a relative id (W4).

## What

- io-webdav 0.9.1 (`cargo update -p io-webdav`, the lock diff that crate alone), the requirement raised to `0.9.1`.
- `carddav::client::check_addressbook_id` and `check_card_id`: an id io-webdav refuses fails `addressbook-not-found` or `card-not-found`, cardamum's sentence, io-webdav's reason in `sources`, as vdir codes an id that is not one directory name. `check_addressbook_name`: the name of an address book to create (its id on CardDAV) fails uncoded, as vdir refuses a new name.
- The unified client runs them on a CardDAV account before it opens the backend (`AddressbookClient::check_ids`, `check_name`), so nothing connects and no secret command runs; `carddav` commands run them after resolving the account and before building the client (`CarddavCommand::check`, `CarddavReportCommand::check`).
- `carddav report raw` refuses the id the same way and builds its path with `rfc4918::collection_path`; `dataBytes` reads the property's whole text.
- Code chosen: not `usage-invalid` but the not-found codes, as vdir (C18) and carillon's W2 code the same ids and as the shared `-k` resolver codes an empty id; a new name stays uncoded, as vdir's `addressbook create` and `vdir create`, a choice for the twins' comparison.
- Spec: backends ("A CardDAV id addresses its own resource", "A CardDAV card reads whole", added), commands ("A raw CardDAV report addresses and counts as io-webdav does", added; the `card-not-found` and `addressbook-not-found` rows of "A JSON error carries a stable code", modified).
