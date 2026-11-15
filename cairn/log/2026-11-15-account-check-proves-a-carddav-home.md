---
cairn: log
change: account-check-proves-a-carddav-home
date: 2026-11-15
---

# account check proves a CardDAV home

`account check` on a `carddav.home` account sends the home one `PROPFIND` of depth 0, so a refused credential fails the row (exit 3) where it passed: the `home` route connected without a request. This is what kept CI red at 601f8b9 (`anonymous::carddav_account_check_refused`). Capability **backends** moved.

- **src/carddav/client.rs**: `propfind_home` sends `PROPFIND` depth 0 for `DAV:resourcetype` on the cached home set, pumping io-webdav's `WebdavPropfind` over the client's stream as `carddav report raw` pumps its report; the error stays a `WebdavClientStdError`, so the message (`WebDAV server returned HTTP 401`) and the code (`refusal_code`: 401 none, 403 `permission-denied`) are the `server` route's.
- **src/account/check.rs**: `connect_carddav` calls it after `open_carddav_client` when the block names `home`. The connect sends nothing more, for any command: each command's own first request is authenticated, and the `home` route exists to skip discovery round trips.
- **Evidence**: `a_refused_credential_fails_a_home_check_as_a_server_one` runs both routes against a scripted local server answering 401, then 403: same `code`, same `error`, `ok: false`. It fails with the request disabled (`{"ok":true,"code":null,"error":null}`, the CI symptom). The live test was not run.

CHANGELOG (Fixed). Spec: backends ("account check proves a CardDAV home" added). calendula's twin is K1 of the alignment plan, fixed apart.
