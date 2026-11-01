---
cairn: change
change: pimdir-status-shows-parked-actions
---

# Delta

## ADDED Requirements

### Requirement: pimdir status reports the store and its parked actions
`pimdir status`, gated on the `pimdir` feature, SHALL report the account being read, every account the store groups collections under, one row per address book of that account (how many cards it holds, how many of them carry a local body, how many creations are queued for the next sync), and every action the store's owner parked in those address books (`PimdirReader::parked_actions`), by its queue row (ROW), its address book, its kind and the error that parked it. A parked action addresses no card, so its row is the only id shown, under its own name. Another account's address books and parked rows SHALL NOT show. Its JSON output is `{"account", "accounts", "addressbooks": [{"id", "name", "total", "hydrated", "queued"}], "parked": [{"row", "addressbookId", "action", "error"}]}`, `parked` always present. It mirrors calendula's `pimdir status`, `calendars` named `addressbooks`.

## MODIFIED Requirements

### Requirement: Specific commands mirror their protocol
A protocol-specific command set SHALL match the remote protocol's own structure and vocabulary rather than the shared API's, following Himalaya's direction. The shape follows the protocol: CardDAV (WebDAV) is a flat list of WebDAV method names, JMAP, Microsoft Graph and Google People are nested by resource with sub-verbs naming the protocol methods, and vdir is flat collection verbs plus an `item` subcommand. Verb names track the protocol, so JMAP uses `destroy` while Graph and People use `delete`.

pimdir is a store rather than a protocol: its `pimdir` family covers `status` alone, what a reader can tell of the store behind the account, and the operator commands (retrying or cancelling a parked action, the retained items) ship as the separate `pimdir` binary in io-pimdir.

### Requirement: A queued pimdir edit names the body it replaced
`update_card` on pimdir SHALL state on the queued `update` the body the edit was made against (`if_match`, pimdir STORAGE §15.3): the version `--if-match` named, or the one `card update` read when given no vCard source, and SHALL state none only when the caller read no body. The owner parks a row whose stated body the item no longer holds, so a server change a sync brings in between the staging and the drain is not overwritten without a merge. cardamum's write has already said it is queued, so the parked row shows in `pimdir status`, with the error that parked it.
