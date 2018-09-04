extern crate serde_json;

use std::net::{SocketAddr, TcpStream};
use std::thread;
use std::sync::mpsc::{self, Sender, Receiver};
use std::io::{Error, ErrorKind, Read, Write};

use message::{Message};
use util;

pub struct Client {
    name: String,
    address: String,
    message_size: usize,
    sender: Option<Sender<String>>,
}

impl Client {
    pub fn new(name: &str, address: &str, message_size: usize) -> Client {
        Client {
            name: name.to_string(),
            address: address.to_string(),
            message_size,
            sender: None,
        }
    }

    pub fn connect(&mut self) {
        let mut socket = TcpStream::connect(&self.address).unwrap();

        // Prevent blocking when reading from the socket
        socket.set_nonblocking(true).unwrap();

        let (tx, rx): (Sender<String>, Receiver<String>) = mpsc::channel();
        let message_size = self.message_size;

        thread::spawn(move || loop {
            Client::handle_outbound(&mut socket, &rx, message_size);
            Client::handle_inbound(&mut socket, message_size);

            util::sleep();
        });

        self.sender = Some(tx);
    }

    pub fn send(&self, message: &str) {
        if let Some(ref tx) = self.sender {
            tx.send(message.to_string()).unwrap();
        }
    }

    fn handle_outbound(socket: &mut TcpStream, rx: &Receiver<String>, message_size: usize) {
        if let Ok(message) = rx.try_recv() {
            let mut buffer = message.into_bytes();

            // Without matching buffer size the server
            // will fail to `read_exact`
            buffer.resize(message_size, 0);
            socket.write_all(&buffer).unwrap();
        };
    }

    fn handle_inbound(socket: &mut TcpStream, message_size: usize) {
        // Non zero buffer size is needed to prevent continuous
        // 0 reading from the socket.
        let mut buffer = vec![0; message_size]; 
        
        if let Ok(_) = socket.read_exact(&mut buffer) {
            let buffer = util::trim_empty_buffer(buffer);
            let message = String::from_utf8(buffer).unwrap();
            let message_struct: Message = serde_json::from_str(&message).unwrap();

            println!("{}: {}", message_struct.from, message_struct.content);
        };
    }
}
