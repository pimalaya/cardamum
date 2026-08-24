---
cairn: change
id: msgraph-delta-resume
status: landed
created: 2026-08-09
---

# `contact delta` promises a resume it cannot do

The 2026-08-09 Microsoft Graph run ([msgraph-specific.md](../../../spec/testing/msgraph-specific.md)) found `msgraph contact delta` advertising, in its own help, "Feed the returned `@odata.deltaLink` back to resume". Its only flags are `--folder` and `--select`, so there is nowhere to put the link: every call starts a fresh round, and the incremental half of the incremental sync is unreachable from the command whose headline it is.

The capability exists underneath: passing the link to `msgraph request get <deltaLink>` returns exactly what changed since, which is how the resume was verified during the run. What is missing is the flag, and, under it, an io-msgraph constructor: `MsgraphContactsDelta::new` only ever builds the opening URL of a round.

Two smaller findings from the same run ride along, both of the "a command says something that is not so" kind this round has been clearing out.

**`request delete` prints `null`.** A Graph 204 carries no body, and the raw passthrough prints the parsed body regardless, so a successful delete reads as `null`.

**An empty `-k ""` is not guarded.** It reaches Graph as a PATCH on the folder collection, which answers `405 The OData request is not supported`. Nobody passes an empty id deliberately, but the resulting message explains nothing, and the check belongs client-side for every backend, not just this one.

## What changes

- io-msgraph gains `MsgraphContactsDelta::from_link` and the client method `contacts_delta_link`, which GET a `@odata.deltaLink` or `@odata.nextLink` verbatim: the link already carries the folder, the `$select` and the token.
- `msgraph contact delta` gains `--delta-link <URL>`, mutually exclusive with `--folder` and `--select` since the link encodes both. The People backend already spells the same idea `--sync-token`, so the flag is named after what Graph actually hands back.
- The raw `request` prints nothing in text mode when the response carries no body; `--json` still prints `null`, which is the honest JSON for it.
- The shared `-k/--addressbook` resolver rejects an empty id, so every backend gets the same clear message instead of whatever the server makes of an empty path segment.
