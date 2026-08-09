---
cairn: log
change: vdir-write-guards
landed: 2026-08-09
---

# vdir: writing through to the filesystem without looking first

The 2026-08-09 vdir run found the specific surface clean and six defects on the shared one. All six are fixed, and the shape of them is the point: the shared vdir commands built a path and wrote through it, where the specific arm had been resolving the collection since July.

**An update that created.** `card update -k contacts absent-id …` wrote `absent-id.vcf` and reported success, because the shared update handed `store_item` whatever id it was given, while the specific `vdir item update` on the same id failed properly. This is CardDAV's F5 on a filesystem. The update now locates the item first, so an unknown id fails on that lookup.

**Three commands that never resolved the collection.** `card create -k <unknown>` created it, `card list -k <unknown>` printed an empty table and exited 0, and `addressbook delete -k <unknown>` leaked `No such file or directory (os error 2)`. The shared `addressbook_path` now checks the directory, exactly as the specific arm's `collection_path` does, so all three fail with "Addressbook `<id>` not found".

**A clear that cleared nothing.** `addressbook update -k <id> -d ""` reported success and left the `description` file on disk, io-vdir's collection update writing a file per present non-empty value with no removal path, exactly like io-webdav's PROPPATCH before it learned `DAV:remove` earlier today. The coroutine now writes the collection it is handed: present values written, absent or empty ones removed, which is what taking a whole `VdirCollection` rather than a patch already promised. Removal is idempotent in the driver, since "this file should not exist" is already true when it never did.

**A precondition accepted and dropped.** `card update --if-match` was silently ignored. vdir has no ETag, which is fine, but msgraph and jmap bail on the same flag, and accepting a guard nobody checks is the one outcome that reads as protection and is none. It now bails too, and [backends](../spec/backends.md) gained the requirement.

Verified against a throwaway instance: metadata clears and comes back, an unknown collection fails on every verb, an absent card id fails and creates no file, `--if-match` is refused, and both surfaces still round-trip. io-vdir's own suite covers the removal step, including a collection carrying no metadata at all.

io-vdir is consumed through a local-path patch until this ships; the collection update is a behaviour change for any other caller passing a partially-filled collection, calendula included when it next picks the crate up.
