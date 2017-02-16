extern crate ws;
extern crate futures;

use futures::{BoxFuture, Future};

// defmodule HelloPhoenix.RoomChannel do
//   use Phoenix.Channel

//   def join("room:lobby", _message, socket) do
//     {:ok, socket}
//   end
//   def join("room:" <> _private_room_id, _params, _socket) do
//     {:error, %{reason: "unauthorized"}}
//   end

//   def handle_in("new_msg", %{"body" => body}, socket) do
//     broadcast! socket, "new_msg", %{body: body}
//     {:noreply, socket}
//   end

//   def handle_out("new_msg", payload, socket) do
//     push socket, "new_msg", payload
//     {:noreply, socket}
//   end
// end


// TODO: eventually should be https://docs.rs/futures/0.1/futures/future/trait.IntoFuture.html

pub struct Server {
  route_table: Option<Box<Fn(String) -> BoxFuture<String, String>>>
}

impl Server {
  pub fn new() -> Server {
    let s = Server{route_table: None};
    s
  }

  pub fn route<T>(&mut self, r: T)
    where T: Fn(String) -> BoxFuture<String, String> + 'static {
    self.route_table = Some(Box::new(r));
  }

  pub fn listen<T: Into<String>>(self, bind_addr: T) {
    let addr: String = bind_addr.into();
    // ws::listen((addr + ":3333").as_str(), |out| {
    //   move |msg: ws::Message| {
    //     out.send(msg);
    //     Ok(())
    //   }
    // });

    ws::listen((addr + ":3333").as_str(), |out| {
      let server = &self;
      move |msg: ws::Message| {
        println!("req: {:?}", msg);
        match server.route_table {
          None => {
            println!("resp failed");
            out.send(ws::Message::text("failed!"));
          },
          Some(ref route) => {
            let msg_str = msg.as_text().unwrap().to_string();
            route(msg_str).then(|resp| {
              println!("resp: {:?}", &resp);
              match resp {
                Ok(s) => out.send(ws::Message::text(s)),
                Err(s) => out.send(ws::Message::text(s)),
              }
            }).wait();
          }
        };
        Ok(())
      }
    });
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use futures::future::ok;
  #[test]
  fn it_works() {
    let mut s = Server::new();
    s.route(|msg| {
      ok(format!("backtalk echo: {}", msg)).boxed()
    });
    s.listen("127.0.0.1");
  }
}
