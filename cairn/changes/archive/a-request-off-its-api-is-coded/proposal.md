---
cairn: change
id: a-request-off-its-api-is-coded
status: landed
created: 2026-11-19
---

# A request off its API is coded

## Why

Owner ruling of 2026-11-19 on X18 of the CLI alignment plan: the origin refusal of `msgraph request` and `gpeople request` carried no code. pimalaya-cli 0.7.0 ships `CommonCode::RequestOrigin` (`request-origin`) and `error::request_origin(url, api)` with the family's one message (pimalaya-cli cairn/log/2026-11-19-a-request-off-its-api-is-coded.md).

## What

`resolve_url` in src/msgraph/request.rs and src/gpeople/request.rs returns `request_origin(url origin, base origin)` in place of its `bail!`; the message is unchanged. `jmap request` posts to the session's `apiUrl` alone and has nothing to refuse.
