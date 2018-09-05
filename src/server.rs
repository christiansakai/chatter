extern crate serde_json;

use std::net::{SocketAddr, TcpListener, TcpStream};
use std::thread;
use std::sync::mpsc::{self, Sender, Receiver};
use std::io::{Read, Write, ErrorKind};

use message::{Message};
use util;

pub struct Server {
    address: SocketAddr,
    message_size: usize,
    server: TcpListener,
    client_handles: Vec<ClientHandle>,
}

struct ClientHandle {
    socket: TcpStream,
    address: SocketAddr,
}

impl Server {
    pub fn new(address: &str, message_size: usize) -> Server {
        let address = address.parse().unwrap();
        let server = TcpListener::bind(address).unwrap();
        // Prevent blocking when reading from the socket
        server.set_nonblocking(true).unwrap();

        let client_handles = Vec::new();

        Server {
            address,
            message_size,
            server,
            client_handles,
        }
    }

    pub fn listen(&mut self) {
        let (tx, rx): (Sender<Message>, Receiver<Message>) = mpsc::channel();

        loop {
            self.handle_connection(tx.clone());
            self.broadcast_message(&rx);

            util::sleep();
        }
    }

    fn handle_connection(&mut self, tx: Sender<Message>) {
        if let Ok((mut socket, address)) = self.server.accept() {
            println!("Client {} connected", address);

            let client_socket = socket.try_clone().unwrap();

            self.client_handles.push(ClientHandle {
                address,
                socket: client_socket,
            });

            let message_size = self.message_size;

            thread::spawn(move || loop {
                // Non zero buffer size is needed to prevent continuous
                // 0 reading from the socket.
                let mut buffer = vec![0; message_size]; 

                match socket.read_exact(&mut buffer) {
                    Ok(_) => {
                        let buffer = util::trim_empty_buffer(buffer);
                        let msg_from_client = String::from_utf8(buffer).unwrap();

                        println!("Client {} says: {}", &address, &msg_from_client);

                        let message = Message {
                            from: address,
                            content: msg_from_client,
                        };

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

    fn broadcast_message(&mut self, rx: &Receiver<Message>) {
        if let Ok(message) = rx.try_recv() {
            let mut found_disconnected_client = false;
            let mut disconnected_client_index = 0;

            for (index, client_handle) in self.client_handles.iter_mut().enumerate() {
                if client_handle.address != message.from {
                    let message_serialized = serde_json::to_string(&message).unwrap();
                    let mut buffer = message_serialized.into_bytes();
                    // Without matching buffer size the server
                    // will fail to `read_exact`
                    buffer.resize(self.message_size, 0);

                    // If sending message to a client failed,
                    // remove it because the client has disconnected
                    if let Err(_) = client_handle.socket.write_all(&buffer) {
                        found_disconnected_client = true;
                        disconnected_client_index = index;
                    }
                }
            }

            if found_disconnected_client {
                self.client_handles.remove(disconnected_client_index);
            }
        }
    }
}
