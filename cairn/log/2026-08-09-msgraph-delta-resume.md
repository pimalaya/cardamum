---
cairn: log
change: msgraph-delta-resume
landed: 2026-08-09
---

# `contact delta` can resume, and two smaller promises kept

The 2026-08-09 Microsoft Graph run found `msgraph contact delta` telling users, in its own help, to "feed the returned `@odata.deltaLink` back to resume", with no flag to feed it to. Every call opened a fresh round, so the incremental half of the incremental sync was unreachable from the command whose headline it is. The capability was there: passing the link to `msgraph request get <deltaLink>` returned what changed, which is how the run verified it.

io-msgraph turned out to have the missing piece on the mail side already: `MsgraphMessagesDelta::from_link` and `messages_delta_from_link` on the client, with a test. The contacts twin simply never got them, so this restores parity rather than inventing an API: `MsgraphContactsDelta::from_link` and `contacts_delta_from_link`, requesting the link verbatim since it already carries the folder, the `$select` and the token.

`msgraph contact delta` gained `--delta-link <URL>`, mutually exclusive with `--folder` and `--select` for that same reason. A truncated page now prints its `next-link` too, and says to pass it to the same flag, rather than naming a JSON member the CLI cannot consume.

Verified as a full cycle against the live account: open a round, create a contact, resume and see it `changed`, delete it, resume and see it `removed`, resume once more and see nothing.

Two smaller findings from the same run rode along, both of the "a command says something that is not so" kind.

**A raw `request delete` printed `null`.** A Graph 204 carries no body, which parses as JSON null, and the passthrough printed it. `RawJson` now prints nothing for a null payload in text mode, which covers the `request` command of every backend; `--json` still prints `null`, the honest JSON for an empty body.

**An empty `-k ""` reached the server.** It arrived as a PATCH on the folder collection, which Graph answers `405 The OData request is not supported`, explaining nothing. The shared `-k/--addressbook` resolver now rejects an empty id for every backend, since an empty path segment addresses a parent collection wherever it lands.

The capability [commands](../spec/commands.md) gained a requirement for each of the three.
