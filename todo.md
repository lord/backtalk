## For 0.1

- [ ] refine Request and Reply APIs, maybe allow for non-JSON responses?
  - [x] instead of response code, a enum?
  - [ ] more convenience functions
  - [x] refine creation of replies, new_streamed should probably be inaccessible maybe? probably.
  - [x] way to create requests that supports tests
  - [ ] decide between params_mut and simple get and set methods
- [ ] should somehow indicate method to `Channel.send`, and it should somehow indicate that to client
- [ ] move MemoryAdapter to its own crate, finish in a bit (http://williamdurand.fr/2014/02/14/please-do-not-patch-like-an-idiot/)
- [ ] server should maybe double check that the request is valid?
- [ ] reply body stream should not return Hyper::Incomplete, maybe find a better error message?
- [x] better error messages, maybe make an `Error` struct

## For 0.2

- [ ] figure out authentication story around `Channel`s
- [ ] add `route` to `Server` for a Req->Resp closure that doesn't bind to all the additional other URLs?
- [ ] namespacing, so `company/<comp_id>/messages` can have a different realtime channel based on the company id. or basically just have a plan for a client to be able to customize exactly what messages they'll be getting.
- [ ] also right now we're throwing away the item ids on LISTEN commands
- [ ] should adapters accept a serializable object instead?
- [ ] better and actually spec compliant Accept header matching, should throw error if can't return either eventsource or json
- [ ] better and faster routing than hash matching

## The Only Objects Are

- Traits
  - `Resource`
  - `Channel`
  - `Adapter`
- Structs
  - `Server`
  - `Req`
  - `Reply`

## Debts

- [ ] make `Sender` be able to resend missed messages with the LastId header or whatever it's called
- [ ] should eventually make both websocket and http server run on the same event loop?
- [ ] there are shared locks everywhere in the form of `Arc`s. should ask about passing references to a future, it's not like they need mutable access or anything. but getting rid of the arcs sped up the code 2x in a 2000-parallel-connection load test
- [ ] the other reason it is slow is because of all the `boxed` allocations. should ask if it's possible to remove those in irc or something. maybe do a test first to see if you don't assemble from the Vec dynamically if it will compile with only a single box? may be able to speed up all the code with macros. and eventually, maybe could do some sort of recursion to avoid boxes. maybe. eh. not quite sure.
- [ ] server.rs should be split up and refactored
- [ ] add proper routing to `Server`, with RouteRef or something like that instead of a string. RouteRef would also contain, like, url params or something maybe? hmm. would be nice if broadcasting events in a resource only broadcasted to other clients with the same params
