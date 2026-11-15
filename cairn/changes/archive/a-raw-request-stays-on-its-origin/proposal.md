---
cairn: change
id: a-raw-request-stays-on-its-origin
status: landed
created: 2026-11-15
---

# A raw request stays on its origin

## Why

`msgraph request` and `gpeople request` took a full URL as given and sent it with the account's bearer token, so a URL on any host received the token. The coordinator's ruling of 2026-11-15 night makes the provider's API origin the safe default, himalaya and calendula taking the same rule.

## What

- `msgraph request` and `gpeople request` refuse a URL whose origin (scheme, host, port) is not the API base's, `https://graph.microsoft.com` or `https://people.googleapis.com`, before sending; the check runs on the resolved URL, so an absolute URL a path would join over the base (`HTTPS://…`, `ftp://…`) is caught too. A path, a next link and Graph's `beta` stay allowed.
- The refusal carries no code, the family naming none for it yet.
- `jmap request` takes no URL and posts to the session's `apiUrl` alone: no change, the spec says so.
