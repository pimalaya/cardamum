---
cairn: delta
change: vdir-home-dir-becomes-root
---

# Delta

Each requirement names its capability file in parentheses.

## ADDED Requirements

### Requirement: The vdir home is `vdir.root`
(config) An account's `vdir` block SHALL name the vdir home, the directory holding one address book per immediate subdirectory, in `root`, its `~` and environment variables expanded as it is read ("Path fields are shell-expanded"), as `pimdir.root` names a store. `vdir.home-dir`, the key v0.3 read, SHALL be refused by name in the family's sentence (pimalaya-config's `removed_key!`), alone or beside `root`: ``The `vdir.home-dir` key is removed, the vdir home being named `root` as a store root is: rename it `vdir.root`, the path unchanged, see MIGRATION.md``. A build without `vdir` loads the block unread.

#### Scenario: A v0.3 vdir home
- GIVEN an account holding `vdir.home-dir = "~/Contacts"`
- WHEN `cardamum --json addressbook list` runs
- THEN it exits 1, the sentence naming the `vdir.home-dir` key as removed, `vdir.root` and MIGRATION.md in `sources`

#### Scenario: The same home under `root`
- GIVEN an account holding `vdir.root = "~/Contacts"`
- WHEN the configuration is loaded
- THEN the vdir home is the Contacts directory of the user's home

## MODIFIED Requirements

### Requirement: Local storage backends
(backends) vdir and pimdir SHALL adapt io-vdir and io-pimdir. vdir stores each addressbook as an immediate subdirectory of `vdir.root` and each card as a `.vcf` file inside it, byte-faithfully. pimdir is an offline cache a sync engine populates through io-pimdir, which holds both the store and the engine, and follows the cache requirements below.

## REMOVED Requirements
