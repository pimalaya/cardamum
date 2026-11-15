---
cairn: change
id: duplicates-pages-and-queue-rows
status: landed
created: 2026-11-15
---

# Duplicates, pages and queue rows follow the family

> Rulings D17, D18 and D19 with D21 of the family's CLI alignment round (/home/soywod/code/pimalaya/.github/cairn/changes/cli-alignment-2026-11-15, tasks.md), now pimalaya-cli's contract at 595504e: cairn/spec/error.md (the noun rule, scenario "A collection created twice") and cairn/spec/output.md ("Paging resumes", "Protocol commands are raw-faithful", "A queue row keeps one shape").

## Why

- **D17 (X3 M4).** `addressbook create` and `vdir create` naming an address book a vdir home holds succeed, io-vdir creating the directory over the existing one and rewriting its metadata; a CardDAV server refusing a second `MKCOL` reads `permission-denied` (Fastmail's 403) or uncoded (the 405 of RFC 4918 §9.3.1); Graph's `ErrorFolderExists` and People's 409 read uncoded. The family now asks `addressbook-duplicate` wherever the backend can tell.
- **D18 (X3 M1, schema review A6).** `gpeople connections list`, `other-contacts list`, `contact-groups list` print People's `nextPageToken`, `msgraph contacts list`, `contact-folders list`, `child-folders` and `contacts delta` Graph's `@odata.nextLink`, and none takes a flag to resume but the delta's `--delta-link`; `jmap contact-card query --limit` stops early with no cursor at all. The family asks every protocol listing that can stop early to print `nextPage` and take `--page-token`, the provider's cursor key lifted, the rest raw.
- **D19 with D21 (X3 M9, schema review A4).** `pimdir queue list` prints a pending row as `{row, queuedAt, producer, collection, kind, linkId, fullName}` and a parked one as `{row, queuedAt, producer, collection, kind, attempts, unconfirmed, error}`. The family's row is one: `{row, kind, collection, linkId, itemId, producer, queuedAt, attempts, unconfirmed, error}` plus the domain's label, `fullName` here, every key printed.

## What

- **D17.** src/error.rs gains `AddressbookDuplicate` and `addressbook_duplicate`. vdir (src/vdir/client.rs `check_collection_absent`) refuses a name the home holds before writing, in `addressbook create` and `vdir create`. CardDAV (src/carddav/backend.rs `create_error`) lists the home set after a `MKCOL` refused 403 or 405 and codes the refusal when the id is listed, one `PROPFIND` on that path alone; RFC 5689 §3.2 keeps RFC 4918 §9.3.1's 405, Fastmail answers 403 (testing/carddav-specific.md), a status iCloud gives every `MKCOL`, so the status alone cannot tell. Graph (`is_folder_taken`) codes `ErrorFolderExists`, People (`is_group_taken`) the 409 contactGroups.create documents for a name the user holds. JMAP cannot tell, RFC 9610 §2 letting two AddressBooks share a name; pimdir creates no address book. The protocol creates on a server keep the server's refusal. Breaking: a vdir create of an existing name failed nowhere before.
- **D18.** The listings print pimalaya-cli's `output::Paginated` around their raw payload, the cursor field dropped from `GpeoplePersonsOutput`, `GpeopleContactGroupsOutput`, `MsgraphContactsOutput`, `MsgraphContactFoldersOutput` and `MsgraphContactDeltaOutput`, and take `PageTokenArg`: People sends it as `pageToken`, Graph gets the link as is (`page_get`, `contacts_delta_from_link`, io-msgraph refusing a link off its origin), the flags the link carries conflicting with it. The delta's `@odata.deltaLink` stays raw and `--delta-link` stays. `jmap contact-card query` prints the next position, sent back as `position`. The People searches print no cursor, People giving none. Breaking for a script reading `nextPageToken` or `@odata.nextLink`.
- **D19.** src/pimdir/queue/list.rs `PimdirQueueRow`, one type for both lists; src/pimdir/backend.rs reads a parked row's payload (`action_from_payload`) for its `linkId` and `itemId`, and the full name from the body the row pins, else the card it addresses. `pimdir queue list` is new in this unreleased major, so nothing breaks against v0.3.
