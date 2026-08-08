---
cairn: change
change: pimdir-local-backend
---

# Delta

## ADDED Requirements

### Requirement: pimdir is a cache, not a server
The pimdir backend SHALL treat the store as a possibly-partial cache. `get_card` on a card whose body is not local SHALL report a clear "body not fetched" state rather than a data-loss error, while `list_cards` projects the stored `v: 1` summary into a minimal preview vCard so a contact list reads correctly before a full sync.

### Requirement: pimdir writes are staged mutations
A pimdir write SHALL stage an io-replica `ReplicaMutation` through the store's mutate seam, never raw SQL, attributed to the configured `pimdir.source`, failing loudly on a store not synced as that source.

### Requirement: pimdir derivations match the sync engine
The link id, the `v: 1` summary, the sort key and the content hash a pimdir write records SHALL be derived exactly as Neverest's `text/vcard` kind derives them, since both write into the same store.

### Requirement: pimdir writes auto-source
When `pimdir.source` is unset, the backend SHALL attribute its writes to the store's single synced source when there is exactly one, falling back to `local` otherwise.

### Requirement: pimdir store path is shell-expanded
The backend SHALL expand `~` and environment variables on `pimdir.root` before opening the store and its blob reader.

### Requirement: pimdir addressbook limits are explicit
`create_addressbook` declares a local collection only, `update_addressbook` honours a rename and rejects description or colour, and `delete_addressbook` always fails with a message naming the alternative.

### Requirement: Local backend auto-detected
A typed folder path SHALL configure a local backend, auto-detecting the store kind from on-disk markers: `pimdir.db` means pimdir, a subdirectory holding a `.vcf` card means vdir. The wizard prompts only when both are compiled in and detection is inconclusive.

## MODIFIED Requirements

### Requirement: Local storage backends
vdir and pimdir SHALL adapt io-vdir and io-pimdir, where the previous requirement named vdir alone.

### Requirement: Backend selection is local-first
The `auto` priority order now puts vdir then pimdir before the network backends, so an offline store wins over a round-trip.

## REMOVED Requirements

None.
