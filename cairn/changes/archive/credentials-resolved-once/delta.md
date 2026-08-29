---
cairn: delta
change: credentials-resolved-once
---

## ADDED Requirements

### Requirement: A credential command is spawned once per account
The backends of one account SHALL resolve their credentials through a single pimalaya-config `SecretResolver`, which spawns each distinct command once and hands its value to every field naming it. `account check` and the wizard's connection test reach every configured backend, so an account naming one `pass` entry from its `carddav` and `jmap` tables would otherwise pay two key unlocks for one entry.

Distinctness SHALL be the command as the configuration wrote it: a shell line and the argv spelling that runs it are two commands, since reading one as the other means guessing what the configuration meant. A raw secret resolves to itself, having nothing to spawn.

A resolver SHALL live no longer than the account it is assembled for. It holds plaintext, so it belongs where an account is reached as a whole and is dropped with it, never held on a client nor shared between accounts.

#### Scenario: One entry, one unlock
- GIVEN an account whose `carddav` and `jmap` tables name the same `pass` command
- WHEN `cardamum account check` reaches both backends
- THEN the command is spawned once and both authenticate with its value

## MODIFIED Requirements

## REMOVED Requirements
