extern crate ws;
extern crate futures;

struct Server {}

impl Server {
    fn new() -> Server {
        let s = Server{};
        s
    }

    fn listen<T: Into<String>>(self, bind_addr: T) {
        let addr: String = bind_addr.into();
        ws::listen((addr + ":3333").as_str(), |out| {
            move |msg: ws::Message| {
                println!("{:?}", msg);
                let msg_str = msg.as_text().unwrap();
                let foo = "backtalk: ".to_string() + msg_str;
                out.send(ws::Message::text(foo))
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_works() {
        let s = Server::new();
        s.listen("127.0.0.1");
    }
}
