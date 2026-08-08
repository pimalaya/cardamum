---
cairn: spec
capability: testing
status: current
---

# Testing reports

Real-world test reports for Cardamum CLI, one per backend and provider. Each was produced by exercising every command variant against a live account (or a throwaway local store), following the golden rule: **operate only inside fake addressbooks created for the run, never on existing data**.

Method: [provider-test-plan.md](provider-test-plan.md).

Every backend pairs a **shared** run (the cross-protocol `addressbook` and `card` commands) with a **specific** run (the raw `cardamum <proto> …` surface), where the backend has one.

## Network backends

| Backend | Provider | Shared | Specific |
| --- | --- | --- | --- |
| CardDAV | Fastmail | [carddav-fastmail.md](carddav-fastmail.md) | [carddav-specific.md](carddav-specific.md) |
| CardDAV | iCloud | [carddav-icloud.md](carddav-icloud.md) | *(combined)* |
| CardDAV | Google | [carddav-google.md](carddav-google.md) | *(combined)* |
| JMAP | Fastmail | [jmap-fastmail.md](jmap-fastmail.md) | [jmap-specific.md](jmap-specific.md) |
| Microsoft Graph | Microsoft | [msgraph-microsoft.md](msgraph-microsoft.md) | [msgraph-specific.md](msgraph-specific.md) |
| Google People | Google | [google-people.md](google-people.md) | [people-specific.md](people-specific.md) |

## Local backends

| Backend | Shared | Specific |
| --- | --- | --- |
| vdir | [vdir-local.md](vdir-local.md) | *(combined)* |
| pimdir | *(not yet run)* | *(none: pimdir ships no protocol-specific surface)* |

## Status

Every bug surfaced by these runs was fixed in-tree or upstream. The CardDAV runs drove the io-webdav resource-id fix (a card's id is now the server-returned last path segment, verbatim, with no extension added or stripped), confirmed on three servers. The JMAP run drove the move of the JSContact projection from calcard to vcard-rs, since calcard's non-standard `vCard` container was rejected by Fastmail.

Not yet run: the pimdir backend, which needs a store populated by a Neverest sync (see [backends.md](../backends.md) for what a run has to cover: the availability-aware read path and the staged-write path both need a real synced store).
