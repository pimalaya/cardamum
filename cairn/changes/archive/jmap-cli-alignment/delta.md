---
cairn: change
id: jmap-cli-alignment
status: landed
created: 2026-08-09
---

# Delta

## ADDED Requirements

### Requirement: Specific commands take their ids positionally
A protocol-specific command SHALL take the ids it operates on as positional arguments, repeatable where the protocol accepts several, and reserve flags for the rest. A short option SHALL NOT be given to an argument whose letter the global options already use.

### Requirement: A command prints what the server returned
A command SHALL render only the fields the server's response actually carries, and SHALL report success in words when the response carries nothing to show, rather than printing empty field slots. `--json` still emits the raw native payload.

### Requirement: A rejected write is legible
A rejected protocol write SHALL surface as prose: the server's own error type and description, plus the properties it named, never a Rust `Debug` rendering of the error value. Where the rejection has a known cause the user can act on, the message SHALL name it.

## MODIFIED Requirements

## REMOVED Requirements
