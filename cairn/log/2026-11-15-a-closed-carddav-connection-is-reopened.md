---
cairn: log
change: a-closed-carddav-connection-is-reopened
date: 2026-11-15
---

# A closed CardDAV connection is reopened

cardamum moves to io-webdav 0.11.0 and runs the two CardDAV requests it sent by hand, the `carddav report raw` REPORT and the `PROPFIND` `account check` sends a `carddav.home`, through `WebdavClientStd::run`, so a server that closed the connection gets the request again on a fresh one, as every other CardDAV request does. cardamum's part of line W10 of the CLI alignment round, calendula's K27 twin. Capabilities **backends** and **commands** moved.

- **Cargo.toml, Cargo.lock**: io-webdav 0.11.0, the requirement `0.11.0`, the lock diff that crate alone.
- **src/carddav/backend.rs, src/carddav/mkcol.rs**: the `CarddavAddressbook` literals take `..Default::default()` for the new `href`, which a create ignores.
- **src/carddav/report/raw.rs**: the REPORT runs through `WebdavClientStd::run`, `run_report` gone. The output is unchanged; the error, now a `WebdavClientStdError`, prints the same message and causes, and a refusal is coded as every CardDAV refusal is (`permission-denied`, `quota-exceeded`, `rate-limited`), where it carried no code.
- **src/carddav/client.rs**: `propfind_home` runs its `PROPFIND` through `WebdavClientStd::run`; its error and code are unchanged.
- **src/script.rs**: `closed`, a server that closed the connection, and `reconnect`, reconnecting an io-webdav client to a given script once.
- **Evidence**: `a_raw_report_is_resent_after_the_server_closed_the_connection` and `the_home_propfind_is_resent_after_the_server_closed_the_connection` send their request on a closed connection, then the same bytes on the fresh one, which answers; `a_refused_raw_report_is_coded` reads a 403 as `permission-denied`. Run before the routing, over the hand-run loops, all three failed: `HTTP/1.1 send failed: reached unexpected EOF`, `Send(Send(Eof))`, and a `None` code.

CHANGELOG (Changed: the bump; Fixed: the resend), MIGRATION.md (The libraries: io-webdav 0.11). Spec: backends ("A closed CardDAV connection is reopened", added), commands ("A raw CardDAV report addresses and counts as io-webdav does", modified).
