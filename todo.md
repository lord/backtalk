## For initial launch

- [x] figure out authentication story around `Channel`s
- [x] should pass less JSON values around, since have to unwrap maps. instead, pass the maps around. maybe make our own map type? we already have params
- [x] also right now we're throwing away the item ids on LISTEN commands
- [ ] add tests
  - [x] adapter
  - [ ] channel
  - [ ] error
  - [ ] reply
  - [ ] request
  - [ ] resource
  - [ ] server (oh god)
  - [ ] util
  - [ ] memory
- [ ] smarter params object? so you can guarantee it'll have `path`, etc, set, and don't have to unwrap
- [ ] add documentation

## Non-api-breaking later changes
- [ ] should adapters accept a serializable object instead? Wait for default types in Rust maybe.
- [ ] make `Sender` be able to resend missed messages with the LastId header or whatever it's called
- [ ] server should maybe double check that the request is valid?
- [ ] better and faster routing than hash matching
- [ ] is_eventsource in server should only work for GET? 403 otherwise? better spec compliance
- [ ] combine std_error throughout into one function
- [ ] better and actually spec compliant Accept header matching, should throw error if can't return either eventsource or json
- [ ] add `route` to `Server` for a Req->Resp closure that doesn't bind to all the additional other URLs?
- [ ] namespacing, so you can mount at `company/<comp_id>/messages` and have `comp_id` become part of the params object, maybe under `params.path.comp_id`
- [ ] multithreaded servers, refine performance — see https://blog.guillaume-gomez.fr/articles/2017-02-22+Rust+asynchronous+HTTP+server+with+tokio+and+hyper

## The Only Objects Are

- Traits
  - `Resource`
  - `Channel`
  - `Adapter`
- Structs
  - `Server`
  - `Req`
  - `Reply`
