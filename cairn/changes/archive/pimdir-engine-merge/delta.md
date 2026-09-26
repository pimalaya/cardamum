---
cairn: change
id: pimdir-engine-merge
status: landed
created: 2026-09-03
---

# Delta

## ADDED Requirements

### Requirement: A store from an earlier draft is refused
The pimdir backend SHALL open a store as the current format defines it and SHALL NOT migrate one an earlier draft wrote. io-pimdir refuses such a store naming the table it lacks, and the backend reports that as the open failure, the fix being to delete the store and let the sync recreate it, the draft offering no migration.

## MODIFIED Requirements

### Requirement: Local storage backends
vdir and pimdir SHALL adapt io-vdir and io-pimdir. vdir stores each addressbook as an immediate subdirectory of `vdir.home-dir` and each card as a `.vcf` file inside it, byte-faithfully. pimdir is an offline cache a sync engine populates through io-pimdir, which holds both the store and the engine, and follows the cache requirements below.

### Requirement: pimdir is a cache, not a server
The pimdir backend SHALL treat the store as a possibly-partial cache. `get_card` on a card whose body is not local (`level < Full`, no stored object) SHALL report a clear "body not fetched" state, the cue to sync, rather than a data-loss error. The card still lists: `list_cards` SHALL project the stored contact summary (pimdir STORAGE Annex A) into a minimal preview vCard (`UID`, `FN`, `EMAIL`), the name escaped back into a content line, so a contact list reads correctly before a full sync, while `get_card` refuses outright so a preview can never be mistaken for the document of record.

### Requirement: pimdir derivations match the sync engine
The link id, summary and sort key of a card a pimdir write stages SHALL be io-pimdir's own derivations (`io_pimdir::summary`, pimdir STORAGE Annex A), and the backend SHALL carry none of its own. A queued `add` or `update` names the body alone: the store's owner derives the summary and the sort key from it when it applies the action, so a card Cardamum stages summarizes exactly as the same card arriving through a sync. The link id `create_card` reports is the same derivation run on the same bytes: the bare `UID`, with nothing prepended, or `hash:` over the body when the card states none.

## REMOVED Requirements

None.
