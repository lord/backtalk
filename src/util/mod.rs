use {Channel, Reply, Method};
pub fn send_from_reply<C: Channel>(reply: Reply, chan: &C) -> Reply {
    match reply.method() {
        Method::Delete | Method::Post | Method::Patch | Method::Action(_) => {
            match reply.data() {
                Some(data) => chan.send(&reply.method().as_string(), data),
                None => (),
            }
        },
        _ => (),
    }

    reply
}
