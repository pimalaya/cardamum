---
cairn: change
id: msgraph-delta-resume
status: landed
created: 2026-08-09
---

# Delta

## ADDED Requirements

### Requirement: An incremental-sync command can resume
A protocol-specific command exposing its backend's incremental sync SHALL accept the resume token that sync hands back, in whatever shape the protocol uses (a Graph `@odata.deltaLink` URL, a People sync token, a JMAP or DAV state string). A command that documents a resume SHALL provide the flag for it.

### Requirement: An empty addressbook id is rejected
The shared `-k/--addressbook` resolver SHALL reject an empty id rather than pass it to a backend, where it silently addresses the collection instead of a member.

## MODIFIED Requirements

### Requirement: A command prints what the server returned
A command SHALL render only the fields the server's response actually carries, and SHALL report success in words when the response carries nothing to show, rather than printing empty field slots. This matters most on the write path, where a JMAP `set` response echoes only the properties the server itself decided, and where a REST delete answers 204 with no body at all: text output SHALL then print nothing. `--json` still emits the raw native payload.

## REMOVED Requirements
