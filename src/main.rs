use std::env;

extern crate chatter;

use chatter::server::Server;
use chatter::client::Client;
use chatter::util::{self, Input};

const ADDRESS: &'static str = "127.0.0.1:6000";
const HELP: &'static str = "Usage:
    - As server, run `chatter server`
    - As client, run `chatter client <username>`";
const MESSAGE_SIZE: usize = 512;

fn main() {
    util::clear();

    let args: Vec<String> = env::args().collect();

    if args.len() == 2 && args[1].contains("server") {
        let mut server = Server::new(ADDRESS, MESSAGE_SIZE);
        server.listen();

        println!("Server is listening on {}", ADDRESS);
    } else if args.len() == 3 && args[1].contains("client")  {
        let name = &args[2];

        let mut client = Client::new(name, ADDRESS, MESSAGE_SIZE);

        println!("Attempting to connect as \"{}\" on {}", name, ADDRESS);

        client.connect();

        println!("Connected as \"{}\" on {}", name, ADDRESS);
        println!();

        loop {
            match util::get_user_input() {
                Input::Message(message) => client.send(&message),
                Input::Quit => break,
            }
        }
    } else {
        println!("{}", HELP);
    }
}
