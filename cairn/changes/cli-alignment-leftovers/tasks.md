---
cairn: tasks
change: cli-alignment-leftovers
---

# Tasks

- [ ] C3: decide the wording of a pimdir write confirmation, which only queues locally (src/shared/card/update.rs:158, src/shared/card/create.rs:132, src/shared/card/delete.rs:40), and apply it to every pimdir write
- [ ] C4: resolve `pimdir.account` as himalaya does (himalaya/src/pimdir/client.rs:225): derive it when the store holds one account, error naming the accounts when it holds several (src/pimdir/client.rs:39-50)
- [ ] B3: add an `alpn` key to the `carddav` block, defaulting to `http/1.1` (src/carddav/client.rs:303); update config.sample.toml
- [ ] G2: build one `SecretResolver` per account where the runtime assembles its backends (src/shared/client.rs:238-252)
- [ ] A2: once released, drop the git pins for io-pimdir @3b4bed8 and io-webdav @62fad89 (Cargo.toml:80-81)
- [ ] H1, H2: once pimalaya-cli ships them, drop src/wizard/configure.rs:91, :220, :246 and src/cli.rs:174
- [ ] CHANGELOG under [Unreleased]
- [ ] Fold into the spec; log; land; move the folder to cairn/changes/archive/
