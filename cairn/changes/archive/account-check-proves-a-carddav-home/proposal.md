---
cairn: change
id: account-check-proves-a-carddav-home
status: landed
created: 2026-11-15
---

# account check proves a CardDAV home

## Why

`account check` on an account whose `[carddav]` block names `home` proved nothing about its credential: the `home` route only opens the connection (src/carddav/client.rs, `open_carddav_client`) and returns, so a refused token reported `ok: true` and exit 0. The live test `anonymous::carddav_account_check_refused` (tests/google.rs, a made-up token against Google's CardDAV home) expects exit 3 and got 0, keeping CI red at 601f8b9. Task C1 of the family's cli-alignment-2026-11-15 plan (.github/cairn/changes/cli-alignment-2026-11-15, audit section 7); calendula has the same branch (task K1).

## What

- `carddav::client::propfind_home` sends the home set one `PROPFIND` of depth 0 asking for `DAV:resourcetype`, over the client's own stream, its error io-webdav's `WebdavClientStdError` (`Send(HttpStatus)`), so the message (`WebDAV server returned HTTP <status>`) and the code (`refusal_code`: 401 none, 403 `permission-denied`) are the ones the `server` route's discovery fails with on the same answer.
- `account check` calls it after connecting when the block names `home`, and only then.
- The request is the check's alone, not the connect's. Every other command's first request on the `home` route is already authenticated (`addressbook list` a `PROPFIND`, `card read` a `GET`, a write a `PUT`), so a refused credential fails it; adding one to the connect would cost every command a round trip the `home` route exists to save. Only the check returned success without sending anything. The `server` route needs none either: its walk is authenticated.
- Spec: backends ("account check proves a CardDAV home" added).
