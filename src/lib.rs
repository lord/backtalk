extern crate ws;

#[cfg(test)]
mod tests {
    use ws;
    #[test]
    fn it_works() {
        ws::listen("127.0.0.1:3012", |out| {
            move |msg: ws::Message| {
                println!("{:?}", msg);
                let msg_str = msg.as_text().unwrap();
                let foo = "backtalk: ".to_string() + msg_str;
                out.send(ws::Message::text(foo))
            }
        });
    }
}
