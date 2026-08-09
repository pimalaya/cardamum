---
cairn: tasks
change: carddav-write-guards
---

# Tasks

- [x] io-webdav: `proppatch_body(set, remove)` emits `<D:remove>` alongside `<D:set>`
- [x] io-webdav: `WebdavProppatch::new` takes the `remove` list; CalDAV calendar update passes an empty one
- [x] io-webdav: add `CarddavAddressbookPatch` plus `property_updates`, and switch `CarddavAddressbookUpdate::new` and `WebdavClient::update_addressbook` to it
- [x] io-webdav: unit tests for the removal body and for a patch that sets and removes at once
- [x] cardamum: pin the io-webdav patch to the local checkout so the fix can be exercised
- [x] cardamum: carddav `update_card` defaults to `If-Match: *`
- [x] cardamum: carddav `update_addressbook` forwards the diff as a patch, dropping the read-then-merge round-trip
- [x] cardamum: carddav `proppatch` command builds a patch (set-only)
- [x] cargo fmt, clippy and the full feature matrix build on both crates
- [x] re-test against Fastmail: F4 and F5 variants plus the surrounding shared and specific commands
- [x] update the testing reports, fold the delta into the spec, write the log entry, archive the change
