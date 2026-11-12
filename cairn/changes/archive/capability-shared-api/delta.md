---
cairn: delta
change: capability-shared-api
---

# Delta

## ADDED Requirements

### Requirement: Capability registry
(commands) Each backend SHALL declare, in a table compiled into every build, each capability of the unified API (`addressbook.list`, `addressbook.create`, `addressbook.rename`, `addressbook.delete`, `addressbook.description`, `addressbook.color`, `card.list`, `card.read`, `card.add`, `card.update`, `card.remove`, `card.update.if-match`, `card.remove.if-match`) at `full`, `partial`, `server` or `none`, every level but `full` with a detail saying what differs, which server refuses, or what serves it instead. A backend declares a capability only when it gives the result every other declarer gives.

### Requirement: An unserved verb is refused before connecting
(commands) A unified command SHALL check the capabilities it reaches, those its flags reach included (`--if-match`, an addressbook's name, description or colour), against the backend the account resolves to, before reading the store or connecting, and SHALL fail with `capability-missing` naming the backend, the capability and the detail.

### Requirement: An account lists what it serves
(commands) `account capability` SHALL print, for the backend the unified commands resolve to (`--backend` honoured), every capability with its level and detail, connecting nowhere: `{"account", "backend", "capabilities": [{"name", "level", "detail"}]}`.

### Requirement: The help names what each backend serves
(commands) The `--help` of each unified leaf SHALL end with a `Backends:` footer generated from the tables: per capability the leaf reaches, the backends grouped by level, then each detail with the backends it applies to.

## MODIFIED Requirements

### Requirement: A JSON error carries a stable code
(commands) The code table gains `capability-missing`: the backend the account resolves to does not serve a capability the command reaches; nothing was read or written (the rest of the requirement unchanged).

### Requirement: An unsupported precondition is refused, not dropped
(backends) A backend with no entity-tag concept SHALL refuse `--if-match` rather than accept and ignore it: its table declares `card.update.if-match` or `card.remove.if-match` at `none`, so the command fails with `capability-missing` before connecting, and the adapter keeps its own refusal. Silently dropping a guard the caller asked for is the one outcome that reads as protection and is none.

## REMOVED Requirements

### Requirement: Unified commands stay least-common-denominator
