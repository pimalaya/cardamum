# Journal

- 2026-11-23 A1: `cargo update -p url` gives url 2.5.9, form_urlencoded 1.2.3; `cargo deny check` advisories, bans, licenses, sources ok.
- 2026-11-23 A2: the first build resolved the lock: io-pimdir's source (git 7f16b27) and libc alone; no rusqlite in a carddav build.
- 2026-11-23 B1: `--sort` variants `Title` with `#[value(alias = "name")]` (clap hides a value alias); tests `the_sort_named_name_is_title` in src/pimdir/search.rs and tests/search.rs.
- 2026-11-23 B5: gate G green, all-features 368 passed (366 + the two B1 tests), 0 failed, 19 ignored; vdir and carddav CI steps 154 passed; 11 clippy sets clean; fmt clean.
- 2026-11-23 A3: `nix build .#default.cargoDeps` vendors io-pimdir from 7f16b27; `nix build .#default --cores 4 --max-jobs 1` builds cardamum-0.3.0; package.nix unchanged.
