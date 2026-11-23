# Journal

- 2026-11-23 A1: `cargo update -p url` gives url 2.5.9, form_urlencoded 1.2.3; `cargo deny check` advisories, bans, licenses, sources ok.
- 2026-11-23 A2: the first build resolved the lock: io-pimdir's source (git 7f16b27) and libc alone; no rusqlite in a carddav build.
- 2026-11-23 B1: `--sort` variants `Title` with `#[value(alias = "name")]` (clap hides a value alias); tests `the_sort_named_name_is_title` in src/pimdir/search.rs and tests/search.rs.
- 2026-11-23 B5: gate G green, all-features 368 passed (366 + the two B1 tests), 0 failed, 19 ignored; vdir and carddav CI steps 154 passed; 11 clippy sets clean; fmt clean.
- 2026-11-23 A3: `nix build .#default.cargoDeps` vendors io-pimdir from 7f16b27; `nix build .#default --cores 4 --max-jobs 1` builds cardamum-0.3.0; package.nix unchanged.
- 2026-11-23 re-pin: io-pimdir drop 1 cc3021b (Cargo.toml patch rev, Cargo.lock source line alone); builds clean.
- 2026-11-23 C2: the B5 binary on a copy of c10k-d15: `store-version`, exit 1, "Pimdir store is storage-draft-15, written by an earlier sync engine, and cardamum reads storage-draft-17 alone: a store is never migrated, so sync it with the engine that wrote it until nothing is queued, then move its directory aside and sync a new store".
- 2026-11-23 E1-E5: tests added, all pass on cc3021b: tests/search.rs `a_reserved_form_is_refused_on_every_backend`, `a_label_is_a_store_term`; src/pimdir/index.rs `a_reserved_form_is_refused_by_search_and_count`, `a_label_finds_its_card` (a label set `VIP` found by `label:vip`, plus `property:app/tier=gold`), `the_new_date_forms_are_accepted`; src/pimdir/backend.rs `a_set_property_row_names_its_card`; the matcher test gains `Émile Zola` and a card with no `FN`, title order and relevance set both equal to the index. Not re-run on 0.19.1 (it does not compile there); inventory I7 holds the 0.19.1 answers.
- 2026-11-23 E6: `card search` and `pimdir search` doc comments name `folder:`, `label:`, `property:` and the reserved forms.
