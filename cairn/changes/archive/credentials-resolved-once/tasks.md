---
cairn: tasks
change: credentials-resolved-once
---

- [x] Move to pimalaya-config 0.2.0 and build `Secret::Command` from a `CommandConfig` in the wizard
- [x] Thread a `SecretResolver` through the CardDAV, JMAP, Graph and People resolution paths
- [x] Build one resolver per account in `account check` and in the wizard's connection test
- [x] State the requirement in the config spec
- [x] Verify the feature matrix, every backend resolving through the new seam on its own
