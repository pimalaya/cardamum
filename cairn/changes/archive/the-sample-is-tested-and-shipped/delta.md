---
cairn: delta
change: the-sample-is-tested-and-shipped
---

# Delta

## ADDED Requirements

### Requirement: One sample configuration, with a section per backend
(config) config.sample.toml SHALL hold one `[accounts.example]` table, the options every account shares active at their defaults, then one commented section per backend or provider (CardDAV, CardDAV without credentials, Google CardDAV, JMAP, Fastmail, Microsoft Graph, Google People, vdir, pimdir). A test SHALL parse every variant pimalaya-config's `sample::variants` derives from it through the loader `-c` uses, `example` resolving as the default account and declaring a backend.

### Requirement: The sample ships beside the binary
(packaging) Packages and release archives SHALL install config.sample.toml as share/doc/cardamum/config.sample.toml, where the welcome finds it, and install.sh SHALL copy it under its prefix.

## MODIFIED Requirements

None.

## REMOVED Requirements

None.
