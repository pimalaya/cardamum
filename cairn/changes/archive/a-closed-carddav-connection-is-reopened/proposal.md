---
cairn: change
id: a-closed-carddav-connection-is-reopened
status: landed
created: 2026-11-15
---

# A closed CardDAV connection is reopened

> cardamum's part of line W10 of the family's CLI alignment round (/home/soywod/code/pimalaya/.github/cairn/changes/cli-alignment-2026-11-15), the twin of calendula's K27, on io-webdav 0.11.0 (W9).

## Why

io-webdav resends a request once on a fresh connection when the server closed the one the client holds before answering (a keep-alive timed out, `Connection: close`), for every request its client runs. cardamum sent two requests by hand over the client's stream instead: the `carddav report raw` REPORT and the `PROPFIND` `account check` sends a `carddav.home` (`propfind_home`). Both met a closed connection with `reached unexpected EOF`, where any other CardDAV command would have sent its request again.

io-webdav 0.11.0 makes `WebdavClientStd::run` public: it runs a coroutine the caller builds over the client's stream with the resend its own methods get.

## What

- io-webdav 0.11.0 (`cargo update -p io-webdav`, the lock diff that crate alone), the requirement raised to `0.11.0`. Its break, the `href` added to `CarddavAddressbook` and the card types, reaches the two literals cardamum builds (src/carddav/backend.rs `create_addressbook`, src/carddav/mkcol.rs), which take `..Default::default()`: a create ignores `href`.
- src/carddav/report/raw.rs and src/carddav/client.rs `propfind_home` run their coroutine through `WebdavClientStd::run`, the hand-run loops gone. Their outputs are unchanged, and `propfind_home` keeps its error, io-webdav's. The REPORT's error now comes wrapped in `WebdavClientStdError`, as every CardDAV request's: its message and causes print as before, and a refusal carries the code "A server's refusal carries its code" asks (`permission-denied`, `quota-exceeded`, `rate-limited`), where the bare `WebdavSendError` carried none.
- Spec: backends ("A closed CardDAV connection is reopened", added), commands ("A raw CardDAV report addresses and counts as io-webdav does", modified).
