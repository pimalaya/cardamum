# Contacts mapping policy

How the API backends with no native vCard — Microsoft Graph (`msgraph/project.rs`) and Google People (`google/project.rs`) — project their JSON contact resource to and from the vCard document of record the shared API speaks. CardDAV and vdir are excluded: they store vCard directly.

## Managed fields

Only fields with a **well-defined vCard slot** are *managed*: the projection reads them into the vCard on the way out and writes them back on the way in. A managed field is authoritative in both directions, so clearing the vCard property clears (nulls / empties) the provider field on the next update.

Everything else is left alone:

- **provider-only fields** with no vCard equivalent (Graph's `fileAs`, `officeLocation`, `assistantName`, `manager`; People's `fileAses`, `memberships`, `events`, `photos`, …) stay out of every update mask and survive updates untouched;
- **provider-scoped fields** that mean nothing outside the account (Google external ids, misc keywords, locations) are *minted* as read-only `X-GOOGLE-*` / `X-MSGRAPH-*` vendor properties on read and *consumed* (dropped) on write, the server value staying authoritative;
- **vCard properties with no provider slot** are preserved verbatim in a custom-data stash — see [custom-data.md](custom-data.md).

## Slot shape differs per provider

- **Microsoft Graph** has *fixed* slots (e.g. a bounded set of emails / phones / IM addresses) and rejects bodies that overflow them. The first properties win; the overflow lands in the stash remainder like any unmanaged line, so it survives on the server and restores on read.
- **Google People** fields are *true lists*, so every vCard property of a managed kind projects without truncation.

The projection modules are ported verbatim from cardamum-android so both products treat the same provider quirks identically.
