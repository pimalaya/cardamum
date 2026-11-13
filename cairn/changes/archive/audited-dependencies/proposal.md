---
cairn: change
id: audited-dependencies
status: landed
created: 2026-11-13
---

# Build on the audited releases of the io-* libraries

## Why

The family audit of 2026-11-13 ended in a breaking release of every io-* library cardamum drives: io-http 0.7, io-oauth 0.6, io-jmap 0.8, io-webdav 0.8, io-gpeople 0.7, io-msgraph 0.8, io-vdir 0.3 and io-pim-discovery 0.12. Several of the defects it recorded are closed in the library only once the application uses what the release offers: a People update erased what a vCard cannot show, a property too long for the People stash was left out without a word, a Graph next page link was followed wherever it pointed, and a cleared JMAP description could not be sent.

## What

- Cargo.toml requires the eight releases; pimalaya-cli keeps its git pin at `eb37962`, untouched here. Cargo.lock holds one io-http (0.7.0) and one io-oauth (0.6.0), and no io-sasl.
- Google People: `update_card` calls `GpeoplePerson::merge_unprojected` between the mask and the update and drops its own `clientData` merge; `create_card` and `update_card` log a warning naming the properties `GpeoplePersonVcard::dropped` reports. The warning is a log line and not a key of the receipt, a unified output carrying no field one backend alone fills.
- Microsoft Graph: the unified listings follow `@odata.nextLink` through `MsgraphClientStd::page_get`, which refuses a link off the Graph origin, where they built a request for whatever URL the page named.
- JMAP: `addressbook update --description ""` sends `null` (`JmapAddressBookUpdate::description` is `Option<Option<String>>`), where the clear was flattened into an empty update; a set error io-jmap does not model (`Other { type_name, description }`) is named as the server sent it, `unknown` staying the word for one the server did not name.
- CardDAV: `carddav put` prints the id the write answered (the `Location` segment, else the percent-encoded segment sent), the form a listing returns, sanitized in its text form since a server may now have written it.
- No change of cardamum's own for io-vdir 0.3, io-oauth 0.6 (a dev-dependency of the live suites) and io-pim-discovery 0.12.

## Scope / non-goals

- io-pim-discovery's `trust` is carried by `DiscoveryServiceConfig`, which cardamum never builds: it resolves a CardDAV URL through the PACC and RFC 6764 clients, whose signatures did not move, and `carddav discover` prints the endpoints of the connected client. Nothing shows a trust until cardamum goes through the compose client.
- `addressbook create` on CardDAV still returns the name it was given as the id; io-webdav's `encode_id` for a caller-minted segment is not adopted here, unverified against the live servers.
- The Graph projection (io-msgraph) reports no dropped line, so a property over 8 KiB is still left out silently on Graph.
- The raw `request` commands keep taking a full URL as it is.
