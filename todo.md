## For 0.2

- [x] return Reply or Request or Error directly by implementing IntoFuture
- [x] backtalk-util crate for functions like `send_from_reply` and other things that make it much easier to write resources.
- [x] `and_then` on requests and replies
- [ ] should pass less JSON values around, since have to unwrap maps. instead, pass the maps around. maybe make our own map type?
- [ ] figure out authentication story around `Channel`s
- [ ] add `route` to `Server` for a Req->Resp closure that doesn't bind to all the additional other URLs?
- [ ] namespacing, so `company/<comp_id>/messages` can have a different realtime channel based on the company id. or basically just have a plan for a client to be able to customize exactly what messages they'll be getting.
- [ ] also right now we're throwing away the item ids on LISTEN commands
- [ ] should adapters accept a serializable object instead?
- [ ] better and actually spec compliant Accept header matching, should throw error if can't return either eventsource or json
- [ ] is_eventsource in server should only work for GET? 403 otherwise? better spec compliance
- [ ] better and faster routing than hash matching
- [ ] server should maybe double check that the request is valid?
- [ ] multithreaded servers, refine performance — see https://blog.guillaume-gomez.fr/articles/2017-02-22+Rust+asynchronous+HTTP+server+with+tokio+and+hyper
- [ ] combine std_error throughout into one function
- [ ] make `Sender` be able to resend missed messages with the LastId header or whatever it's called
- [ ] add tests

## The Only Objects Are

- Traits
  - `Resource`
  - `Channel`
  - `Adapter`
- Structs
  - `Server`
  - `Req`
  - `Reply`
