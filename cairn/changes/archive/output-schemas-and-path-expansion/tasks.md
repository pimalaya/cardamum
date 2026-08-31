---
cairn: tasks
change: output-schemas-and-path-expansion
---

# Tasks

- [x] `pimdir.root` expands at deserialize; `PimdirClient::new` drops its call-site expansion
- [x] `tls.cert` expands at deserialize through a private `opt_shell_expanded_path`, with a TODO naming the shared helper
- [x] round-trip tests for both, mandatory and optional
- [x] `CarddavAuthConfig::None`, wired through `build_auth` and the wizard, with a test parsing all three schemes
- [x] rename every output type to `<Domain><Target><Verb>Output` and derive `JsonSchema` on each
- [x] document every public field of an output type, which is what the schema descriptions are built from
- [x] `carddav propfind` answers one untagged `CarddavPropfindOutput`, its two shapes being one command
- [x] turn the four confirmations carrying a server-assigned id, or the properties an update kept, into output types of their own
- [x] src/json_schema.rs registry plus the `json-schema` command, aliased `json-schemas`
- [x] a test asserting every registered key names a real command path
- [x] route `comfy_table` through `pimalaya_cli::table` and drop the direct dependency
- [x] config.sample.toml, README and CHANGELOG
- [x] gate the two pre-existing ungated items the feature sweep uncovered, so every backend tests and lints on its own
- [x] cargo fmt, check, test and clippy clean on the full feature matrix, and the live configuration still loads
