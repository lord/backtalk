## Tasks

- [x] add JSON parsing and serialization into Req and Reply objects
- [x] add `Resource` trait with various methods
- [x] add http routing to server
- [x] add `BeforeHook` and `AfterHook` traits
- [x] also need to add `.action` handler to resource
- [x] broadcasting events to event listeners with a `Channel` trait
- [x] switch to SSE, remove `ws-rs` dependency
- [ ] better errors
- [ ] implement `BeforeHook` and `AfterHook` traits for the appropriate closures
- [x] maybe `Guard` and `Filter` aren't such bad names?
- [ ] maybe `Filter` can apply to outgoing messages too? annoying to have to implement separate security systems for events and requests.
- [ ] some sort of `feathers-reactive` inspired library for data sync, which is pretty much the main reason to have this stuff. maybe also look at how feathers uses rethinkdb and does subscriptions with that, there may be something to learn.
- [ ] better name than `Sender`
- [ ] make `Sender` be able to resend missed messages with the LastId header or whatever it's called
- [ ] namespacing, so `company/<comp_id>/messages` can have a different realtime channel based on the company id. or basically just have a plan for a client to be able to customize exactly what messages they'll be getting.
- [ ] maybe rename channel to radio?

## Debts

- [ ] should eventually make both websocket and http server run on the same event loop?
- [ ] there are shared locks everywhere in the form of `Arc`s. should ask about passing references to a future, it's not like they need mutable access or anything. but getting rid of the arcs sped up the code 2x in a 2000-parallel-connection load test
- [ ] the other reason it is slow is because of all the `boxed` allocations. should ask if it's possible to remove those in irc or something. maybe do a test first to see if you don't assemble from the Vec dynamically if it will compile with only a single box? may be able to speed up all the code with macros. and eventually, maybe could do some sort of recursion to avoid boxes. maybe. eh. not quite sure.
- [ ] server.rs should be split up and refactored
- [ ] add proper routing to `Server`, with RouteRef or something like that instead of a string. RouteRef would also contain, like, url params or something maybe? hmm. would be nice if broadcasting events in a resource only broadcasted to other clients with the same params

// TODO: eventually should be https://docs.rs/futures/0.1/futures/future/trait.IntoFuture.html

// TODO could a client continue the connection even after the 404? make sure not

// don't support PUT? https://tools.ietf.org/html/rfc7396 and http://williamdurand.fr/2014/02/14/please-do-not-patch-like-an-idiot/

// TODO be able to return a future of anything that can be IntoReply instead of just Reply?

// TODO I think macros can help with reducing usage of BoxFuture which is slower?
//      it would be cool if we used futures in a zero-cost way
//      also, it would be nice if we didn't have to write ok(fut).boxed() everywhere
//      see Rocket for inspiration
