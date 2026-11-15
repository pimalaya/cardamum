---
cairn: log
change: vdir-home-dir-becomes-root
date: 2026-11-15
---

# The vdir home is `vdir.root`

`vdir.home-dir` becomes `vdir.root`, as every store root of the family is named, and the old key is refused by name. cardamum's line C10 of the CLI alignment round (D13), calendula's K10 landing the same change. Capabilities **config** and **backends** moved.

- **src/config.rs**: `VdirConfig::root`, shell-expanded at deserialize as `pimdir.root`; `home_dir` is a `VdirHomeDirRemoved` (`removed_key!`, pimalaya-config 0.5.1), failing the load in the family sentence ``The `vdir.home-dir` key is removed, the vdir home being named `root` as a store root is: rename it `vdir.root`, the path unchanged, see MIGRATION.md``, alone or beside `root`.
- **src/vdir/backend.rs, src/vdir/client.rs, src/account/check.rs, src/session.rs**: read `root`. **src/vdir/cli.rs**: the `vdir` help names `vdir.root`.
- **config.sample.toml, README.md, tests/**: `vdir.root`.
- **Evidence**: tests/account.rs `a_vdir_home_dir_key_is_refused_by_name_and_root_is_read` runs the binary: `addressbook list` over `vdir.home-dir` exits 1 with the sentence in `sources`, over `vdir.root` lists the home; on the code before, the first half exits 0 and the second fails as an unknown field. src/config.rs `a_vdir_home_dir_is_refused_by_name` and `a_vdir_root_expands_its_tilde_and_variables` (`~` and `$HOME`).

CHANGELOG (Changed, breaking), MIGRATION.md ("From v0.3 to v0.4"). Spec: config ("The vdir home is `vdir.root`", added), backends ("Local storage backends", modified).
