---
cairn: tasks
change: the-wizard-leaves
---

# Tasks

- [x] Delete src/wizard/, `configure`, the offer, the `wizard` feature and the long help pitch
- [x] src/config.rs: drop `render`, its key orders, `is_default`, the `Serialize` derives and `skip_serializing_if`
- [x] Drop `cardamum-configure` and `test_account`; io-pim-discovery behind `carddav` with `pacc`, `rfc6764` and `stream`; shellexpand out
- [x] Bare `cardamum` prints the help; a missing configuration fails naming the path and the sample
- [x] README, config.sample.toml header, AGENTS.md, CHANGELOG (one Removed entry in place of the unreleased wizard entries)
- [x] Fold the delta: spec/wizard.md deleted, config and commands requirements reworded
- [x] Clippy `-D warnings` (all features, carddav, jmap and vdir alone), tests
