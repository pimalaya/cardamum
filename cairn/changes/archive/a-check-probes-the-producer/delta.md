---
cairn: change
change: a-check-probes-the-producer
---

# Delta

## ADDED Requirements

## MODIFIED Requirements

### Requirement: account check opens every role the backend takes
`account check` on pimdir SHALL open the store as a reader and list its address books, then open a producer and probe it (`PimdirProducer::probe`), which takes and releases the staging lock without waiting and the database's write lock, writing nothing into the store. A store whose staging lock or write lock cannot be taken fails the check rather than the first write; a collector holding the staging lock fails it at once with `store-busy`, and the check leaves nothing in objects/.

## REMOVED Requirements
