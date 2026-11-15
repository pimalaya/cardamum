---
cairn: tasks
change: vdir-home-dir-becomes-root
---

# Tasks

- [x] src/config.rs: `VdirConfig::root` (shell-expanded), `home_dir` refused through `VdirHomeDirRemoved` (`removed_key!`); the call sites (src/vdir/backend.rs, src/vdir/client.rs, src/account/check.rs, src/session.rs) and the `vdir` help (src/vdir/cli.rs).
- [x] Tests: src/config.rs `a_vdir_root_expands_its_tilde_and_variables` and `a_vdir_home_dir_is_refused_by_name`; tests/account.rs `a_vdir_home_dir_key_is_refused_by_name_and_root_is_read` (the binary refuses `home-dir` with the sentence in `sources`, lists the home under `root`); every test configuration on `vdir.root`.
- [x] config.sample.toml and README.md on `vdir.root`.
- [x] CHANGELOG (Changed, breaking) and MIGRATION.md ("From v0.3 to v0.4", before and after).
- [x] Fold the delta into config.md and backends.md, write the log entry, set `landed`, archive.
