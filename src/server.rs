extern crate serde_json;

use std::net::{SocketAddr, TcpListener, TcpStream};
use std::thread;
use std::sync::mpsc::{self, Sender, Receiver};
use std::io::{Read, Write, ErrorKind};

use message::{Message};
use util;

pub struct Server {
    address: String,
    message_size: usize,
}

struct ClientHandle {
    socket: TcpStream,
    address: SocketAddr,
}

impl Server {
    pub fn new(address: &str, message_size: usize) -> Server {
        Server {
            address: address.to_string(),
            message_size,
        }
    }

    pub fn listen(&mut self) {
        let server = TcpListener::bind(&self.address).unwrap();

        // Prevent blocking when reading from the socket
        server.set_nonblocking(true).unwrap();

        let (tx, rx): (Sender<String>, Receiver<String>) = mpsc::channel();

        let mut clients: Vec<ClientHandle> = Vec::new();

        loop {
            Server::handle_connections(&mut clients, &server, &tx, self.message_size);
            clients = Server::broadcast_message(clients, &rx, self.message_size);

            util::sleep();
        }
    }

    fn handle_connections(clients: &mut Vec<ClientHandle>, server: &TcpListener, tx: &Sender<String>, message_size: usize) {
        if let Ok((mut socket, address)) = server.accept() {
            println!("Client {} connected", address);

            clients.push(ClientHandle {
                address,
                socket: socket.try_clone().unwrap(),
            });

            let tx = tx.clone();

            thread::spawn(move || loop {
                // Non zero buffer size is needed to prevent continuous
                // 0 reading from the socket.
                let mut buffer = vec![0; message_size]; 
                match socket.read_exact(&mut buffer) {
                    Ok(_) => {
                        let buffer = util::trim_empty_buffer(buffer);
                        let message_string = String::from_utf8(buffer).unwrap();

                        println!("Client {} says: {}", &address, &message_string);

                        let message_struct = Message {
                            from: address,
                            content: message_string,
                        };

                        let message = serde_json::to_string(&message_struct).unwrap();

                        tx.send(message).unwrap();
                    },

                    // Caused by no message from the client
                    Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),

                    Err(_) => {
                        println!("Client {} disconnected", address);
                        break;
                    },
                }

                util::sleep();
            });
        }
    }

    fn broadcast_message(clients: Vec<ClientHandle>, rx: &Receiver<String>, message_size: usize) -> Vec<ClientHandle> {
        if let Ok(message) = rx.try_recv() {
            let message_struct: Message = serde_json::from_str(&message).unwrap();

            return clients
                .into_iter()
                .filter_map(|mut client_handle| {
                    if client_handle.address != message_struct.from {
                        let mut buffer = message.clone().into_bytes();
                        // Without matching buffer size the server
                        // will fail to `read_exact`
                        buffer.resize(message_size, 0);

                        // If sending message to a client failed,
                        // remove it because the client has disconnected
                        // TODO: Maybe the removal of client should be in the thread
                        // above
                        client_handle.socket
                            .write_all(&buffer)
                            .map(|_| client_handle)
                            .ok()
                    } else {
                        Some(client_handle)
                    }
                })
                .collect();
        };

        clients
    }
}
