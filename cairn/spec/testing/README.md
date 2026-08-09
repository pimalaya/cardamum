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

Most bugs surfaced by these runs were fixed in-tree or upstream. The CardDAV runs drove the io-webdav resource-id fix (a card's id is now the server-returned last path segment, verbatim, with no extension added or stripped), confirmed on three servers. The JMAP run drove the move of the JSContact projection from calcard to vcard-rs, since calcard's non-standard `vCard` container was rejected by Fastmail.

The 2026-08-09 Fastmail re-run drove two more: `card update` on an absent id used to create the card, since an unguarded WebDAV `PUT` is create-or-replace (F5), and `addressbook update -d "" / -C ""` was a no-op because io-webdav's addressbook PROPPATCH had no `DAV:remove` path (F4). Both are fixed and re-verified live; see the [carddav-write-guards](../../log/2026-08-09-carddav-write-guards.md) log entry.

The same run also found `carddav mkcol` and `carddav proppatch` leaking the `CarddavAddressbook` type name into their success message, fixed the same day. The CardDAV surface on Fastmail is now clean apart from the cosmetic observations listed in its reports.

The 2026-08-09 JMAP re-run drove one more upstream fix and a batch of CLI ones. The shared `card create` and `card update` could not write a contact carrying `URL`, `PHOTO`, `LOGO`, `SOUND`, `KEY`, `CALURI`, `FBURL`, `CALADRURI` or `SOURCE` to Fastmail at all, because vcard-rs's JSContact projection spelled those objects with the pre-RFC `*Resource` type names that RFC 9553 §2.6 renamed (J5); vcard-rs now emits the RFC names and the round-trip is verified. Alongside it, `jmap contact-card copy` had never run (a clap short-option collision), the create and update commands printed empty fields, and every rejected write printed its error through Rust `Debug`: see the [jmap-cli-alignment](../../log/2026-08-09-jmap-cli-alignment.md) log entry.

The 2026-08-09 iCloud re-run then caught a regression from the same day's CardDAV work: the `If-Match: *` existence guard that Fastmail honours is rejected by iCloud with `412` even for a card that exists, so `card update` failed on every iCloud account. The guard now uses the card's own ETag, read first, and both providers were re-verified; see [carddav-update-guard-etag](../../log/2026-08-09-carddav-update-guard-etag.md).

The 2026-08-09 Microsoft Graph re-run found the shared surface clean, and every cross-check the other runs prompted comes out right there: an update on an absent id creates nothing, a delete of an absent id fails, and a property Graph cannot model survives through its stash. Its specific API turned up one defect, `contact delta` advertising a resume it had no flag for, so the incremental sync could never advance from the CLI; that and two smaller promises (a raw `delete` printing `null`, an empty `-k` reaching the server) are fixed and re-verified, see the [msgraph-delta-resume](../../log/2026-08-09-msgraph-delta-resume.md) log entry.

The 2026-08-09 Google People re-run completes the round: every account in the configuration has now been exercised on both surfaces. It found a member count printed without ever being asked for, an HTML error page used as an error message, and an update that kept what it said it dropped, all three fixed and re-verified; see the [people-honest-writes](../../log/2026-08-09-people-honest-writes.md) log entry. The last of those is a Google limitation rather than a defect, so the fix was to name what stayed instead of reporting a clean success: on Google People a property living in the provider-side stash can be changed but never removed, verified down to a raw PATCH with an explicit empty `clientData`. Microsoft Graph, whose stash works the same way, clears it correctly.

**Open:** Fastmail rejects the standard `vCardProps` member (RFC 9555 §2.15), so a vCard carrying a property JSContact cannot model still fails to write over JMAP, now with a message naming that property (J6). Whether cardamum should stash such properties, as the Graph and People backends do, is a product decision. On CardDAV, the PROPPATCH gap that run exposed (I5) is fixed too: io-webdav now returns the multistatus plus the properties the request carried, and the client fails when one comes back refused or unmentioned, which is how iCloud answers for a collection it does not have. See [jmap-fastmail.md](jmap-fastmail.md) and [jmap-specific.md](jmap-specific.md).

Not yet run: the pimdir backend, which needs a store populated by a Neverest sync (see [backends.md](../backends.md) for what a run has to cover: the availability-aware read path and the staged-write path both need a real synced store).
