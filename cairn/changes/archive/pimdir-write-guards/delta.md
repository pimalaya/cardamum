---
cairn: change
id: pimdir-write-guards
status: landed
created: 2026-08-09
---

# Delta

## ADDED Requirements

### Requirement: A command resolves its addressbook before acting
A shared command SHALL resolve the addressbook it was given before reading or writing through it, and SHALL fail naming it when it does not exist. Listing an unknown addressbook SHALL NOT read as empty, and writing into one SHALL NOT create it.

### Requirement: A cache renames nothing it cannot push
The pimdir backend SHALL refuse every `addressbook update` field. Its collection row (id, display name, description, colour) is written by the sync from the server, and the backend stages item mutations only, so any local edit of it is a change no sync would carry.

## MODIFIED Requirements

### Requirement: pimdir addressbook limits are explicit
`create_addressbook` SHALL declare a local `text/vcard` collection and say nothing about a remote one, since pimdir stages item mutations only. `update_addressbook` SHALL always fail, per the requirement above: io-pimdir's `rename_collection` renames the identifier rather than the display name, and the row it would touch is the sync's to write. `delete_addressbook` SHALL always fail with a message naming the alternative (delete on the server and sync, or remove the store directory), because io-pimdir exposes no collection removal and io-replica has no collection-level mutation to stage one.

## REMOVED Requirements
