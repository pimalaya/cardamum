---
cairn: log
change: carddav-write-guards
landed: 2026-08-09
---

# CardDAV write guards: update means update, clearing means clearing

The 2026-08-09 Fastmail re-run left two write-path defects standing, both cases of a command reporting work it had not done. Both are now fixed and re-verified live against Fastmail, and the capability [backends](../spec/backends.md) gained a requirement for each.

**`card update` no longer fabricates cards (F5).** An unconditional WebDAV `PUT` is create-or-replace, so `card update -k <book> <absent-id>` used to print "successfully updated", exit 0, and leave a brand new card behind. The carddav adapter now sends `If-Match: *` when the caller passed no ETag, which RFC 9110 §13.1.1 reads as "the resource must exist". An unknown id fails with 412 and writes nothing; an explicit `--if-match <etag>` keeps its stricter meaning; and the raw `carddav put` is untouched, because create-or-replace is exactly what that protocol verb means.

**Clearing addressbook metadata now clears it (F4).** `addressbook update -d "" -C ""` was a no-op despite the help documenting `""` as the way to clear. cardamum had modelled the intent correctly all along (`AddressbookDiff` carries `Option<Option<String>>`), but it died at the io-webdav boundary, whose update coroutine took a flat `CarddavAddressbook` that cannot tell "leave alone" from "remove", and whose PROPPATCH body only ever emitted `DAV:set`. io-webdav gained the missing half: `proppatch_body` and `WebdavProppatch::new` take a removal list and emit `DAV:remove` (RFC 4918 §9.2), and a new `CarddavAddressbookPatch` carries the doubly-optional properties that `CarddavAddressbookUpdate` and the client's `update_addressbook` now take. The CalDAV calendar twin stays set-only for now and passes an empty removal list.

A read-then-merge round-trip disappeared with it: the carddav adapter used to list every addressbook just to refill the fields the caller had not touched, which a real patch makes pointless. The `carddav proppatch` command builds the same patch, set-only, so its behaviour is unchanged.

The io-webdav side landed upstream, so cardamum consumes it through the usual git patch again; the local-path pin used while developing the fix is gone.

Verified on Fastmail: clearing the description leaves the color alone and vice versa, a set and a removal ride in one PROPPATCH, values can be set again after being cleared, an absent card id is rejected with 412 and creates nothing, and an existing card still updates with no ETag. The surrounding shared and specific commands were swept for regressions. Reports: [carddav-fastmail.md](../spec/testing/carddav-fastmail.md) and [carddav-specific.md](../spec/testing/carddav-specific.md).
