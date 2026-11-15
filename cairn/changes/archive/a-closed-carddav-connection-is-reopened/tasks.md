---
cairn: tasks
change: a-closed-carddav-connection-is-reopened
---

# Tasks

- [x] io-webdav 0.11.0 in Cargo.lock (that crate alone) and Cargo.toml (`0.11.0`); the `CarddavAddressbook` literals of src/carddav/backend.rs and src/carddav/mkcol.rs take `..Default::default()`.
- [x] src/carddav/report/raw.rs runs its REPORT through `WebdavClientStd::run`, `run_report` gone.
- [x] src/carddav/client.rs `propfind_home` runs its `PROPFIND` through `WebdavClientStd::run`.
- [x] src/script.rs: `closed`, a server that closed the connection (each request answered with the end of the stream), and `reconnect`, a way to reconnect opening a given script once.
- [x] Tests: `a_raw_report_is_resent_after_the_server_closed_the_connection` (src/carddav/report/raw.rs) and `the_home_propfind_is_resent_after_the_server_closed_the_connection` (src/carddav/client.rs) send their request on a closed connection, then the same bytes on the fresh one, which answers; `a_refused_raw_report_is_coded` reads a 403 as `permission-denied`. On the hand-run code all three failed (`reached unexpected EOF`, `Send(Send(Eof))`, no code).
- [x] CHANGELOG (Changed: the bump; Fixed: the resend), MIGRATION.md (The libraries: io-webdav 0.11).
- [x] Fold the delta into backends.md and commands.md, write the log entry, set `landed`, archive.
