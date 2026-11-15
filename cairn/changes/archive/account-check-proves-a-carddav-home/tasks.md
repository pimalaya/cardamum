---
cairn: tasks
change: account-check-proves-a-carddav-home
---

# Tasks

- [x] `propfind_home` in src/carddav/client.rs: one `PROPFIND` of depth 0 on the home set, pumped over the client's stream, io-webdav's error kept.
- [x] `connect_carddav` in src/account/check.rs sends it when the block names `home`.
- [x] Offline test over a scripted server (`a_refused_credential_fails_a_home_check_as_a_server_one`): a 401 and a 403 fail the `home` row with the code and error of the `server` row; fails without the request.
- [x] CHANGELOG Fixed line.
- [x] Fold the delta into backends.md, write the log entry, set `landed`, archive.
