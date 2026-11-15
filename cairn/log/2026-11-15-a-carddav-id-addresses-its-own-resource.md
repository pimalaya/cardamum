---
cairn: log
change: a-carddav-id-addresses-its-own-resource
date: 2026-11-15
---

# A CardDAV id addresses its own resource

cardamum moves to io-webdav 0.9.1 and refuses, before the account's secret is resolved or anything connects, a CardDAV address book or card id that would address the home set, its parent or the server root: `addressbook delete ..` could send a DELETE to the home set's parent on 0.9.0, and on 0.9.1 failed with an uncoded I/O error once connected. cardamum's part of line W2 of the CLI alignment round. Capabilities **backends** and **commands** moved.

- **Cargo.toml, Cargo.lock**: io-webdav 0.9.1, the requirement `0.9.1`, the lock diff that crate alone.
- **src/carddav/client.rs**: `check_addressbook_id` and `check_card_id` run io-webdav's `check_collection_id` and `check_resource_id`, a refused id failing `addressbook-not-found` or `card-not-found` with cardamum's sentence and io-webdav's reason in `sources`; `check_addressbook_name` refuses a name to create uncoded (``Addressbook name `<name>` names no CardDAV collection``, the empty-name refusal moved from `CarddavBackend::create_addressbook`).
- **src/shared/client.rs**: `AddressbookClient::check_ids` and `check_name` run them on a CardDAV account before the backend opens, in `addressbook create`, `update` and `delete` and every card call.
- **src/carddav/cli.rs, src/carddav/report/cli.rs, src/cli.rs**: `CarddavCommand::check` and `CarddavReportCommand::check` run them once the account is resolved, before its client is built, on every `carddav` command naming an id.
- **src/carddav/report/raw.rs**: the id checked, the path built by `rfc4918::collection_path` (percent-encoded, a slash closing it) where it was joined by hand, `dataBytes` read from the property's whole text where `text` trimmed the final CRLF.
- **Code**: the not-found codes rather than `usage-invalid`, as vdir (C18), the shared `-k` resolver's empty id and carillon's W2 code the same ids; a new name uncoded, as vdir's `addressbook create` and `vdir create`.
- **Through io-webdav 0.9.1**: a card listed or multigot keeps its final CRLF (W3), a relative id is sent percent-encoded (W4).
- **Evidence**: tests/carddav_id.rs runs the binary over a CardDAV account at 127.0.0.1:9 whose password command creates a file; with the three checks neutralised, `an_addressbook_id_out_of_the_home_is_not_found`, `a_card_id_out_of_its_addressbook_is_not_found` and `a_new_addressbook_name_out_of_the_home_is_refused` fail (the file created, the error uncoded), `a_clean_id_goes_on_to_open_the_account` passes on both. In src/carddav/report/raw.rs, `a_raw_report_counts_the_whole_card_on_an_encoded_path` fails on `text` (52 bytes for 54) and `a_raw_report_refuses_an_id_out_of_its_home` with the check neutralised (a request sent). `a_carddav_listing_keeps_the_whole_card` (src/carddav/backend.rs) pins io-webdav's W3, whose own test fails on a trimmed read.

CHANGELOG (Changed, Fixed). Spec: backends ("A CardDAV id addresses its own resource", "A CardDAV card reads whole", added), commands ("A raw CardDAV report addresses and counts as io-webdav does", added; the `card-not-found` and `addressbook-not-found` rows of "A JSON error carries a stable code", modified).
