---
cairn: change
id: carddav-write-guards
status: landed
created: 2026-08-09
---

# CardDAV write guards: update means update, clearing means clearing

The 2026-08-09 Fastmail re-run ([carddav-fastmail.md](../../spec/testing/carddav-fastmail.md)) left two defects standing, both on the write path, both making a command claim it did something it did not do.

**F5: `card update` on an absent id fabricates a card.** `cardamum card update -k <book> <absent-id> '<vcard>'` prints "successfully updated", exits 0, and the card shows up in `card list`. Nothing was replaced. The cause is that io-webdav's `CarddavCardUpdate` issues a plain `PUT` with no precondition, and an unconditional WebDAV `PUT` is create-or-replace (RFC 4918 §9.7). The shared `card` API distinguishes `create` from `update` and its own help says "Replace the bytes of an existing vCard", so silently creating is wrong twice over: it contradicts the contract, and it turns a typo into data.

**F4: clearing addressbook metadata is a no-op.** `addressbook update -k <book> -d ""` and `-C ""` print "successfully updated" and change nothing, although the `--help` documents `""` as the way to clear. cardamum already models the intent correctly (`AddressbookDiff` carries `Option<Option<String>>`, so an empty string arrives as `Some(None)`, meaning "remove this property"), but the intent dies at the io-webdav boundary: `CarddavAddressbookUpdate` takes a plain `CarddavAddressbook`, whose fields are a flat `Option<String>` and therefore cannot tell "leave alone" from "remove", and `property_set` emits a `<D:set>` entry for present values only. RFC 4918 §9.2 spells the missing half: a `<D:propertyupdate>` carries `<D:set>` and `<D:remove>` instructions, and only `<D:remove>` deletes a property.

## What changes

In io-webdav, teach the PROPPATCH path to remove properties, and give the addressbook update coroutine a patch type that can express removal:

- `proppatch_body` and `WebdavProppatch::new` take a `remove` list alongside `set`, and emit `<D:remove><D:prop>…</D:prop></D:remove>` for it. `prop_set_body` stays set-only: it also backs `MKCOL` and `MKCALENDAR`, where removal is meaningless.
- A new `CarddavAddressbookPatch` (id plus `Option<Option<String>>` per property) replaces `&CarddavAddressbook` on `CarddavAddressbookUpdate::new` and on the client's `update_addressbook`. A `property_updates` helper splits it into the set and remove lists.

In cardamum:

- The carddav `update_card` sends `If-Match: *` when the caller passed no ETag. RFC 9110 §13.1.1 defines `*` as "the resource must exist", so an absent id now fails with 412 instead of creating. An explicit `--if-match <etag>` keeps its stricter meaning, and the raw `carddav put` is untouched: create-or-replace is exactly what the protocol verb means.
- The carddav `update_addressbook` forwards its `AddressbookDiff` straight to a `CarddavAddressbookPatch` instead of listing every addressbook to merge unchanged fields by hand. The read-then-merge round-trip disappears with it, and `Some(None)` finally reaches the wire as a removal.

## What does not change

The other backends' `card update` is out of scope here: each is an object API whose update call targets an existing object by id, and none was observed fabricating one. Their re-runs will confirm it per backend.
