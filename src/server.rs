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

            let tx = tx.clone();
            let client = socket.try_clone().unwrap();
            let client = ClientHandle {
                address,
                socket: client,
            };
            clients.push(client);

            thread::spawn(move || loop {
                // Non zero buffer size is needed to prevent continuous
                // 0 reading from the socket.
                let mut buffer = vec![0; message_size]; 
                match socket.read_exact(&mut buffer) {
                    Ok(_) => {
                        let buffer = Server::trim_empty_buffer(buffer);
                        let message = String::from_utf8(buffer).unwrap();

                        let msg = Message {
                            address: address,
                            content: message.clone(),
                        };

                        let msgg = serde_json::to_string(&msg).unwrap();
                        println!("Client {} says: {}", address, message);

                        tx.send(msgg).unwrap();
                    },
                    Err(ref err) if err.kind() == ErrorKind::WouldBlock => {
                        // Caused by no message from the client
                        ()
                    },
                    Err(_) => {
                        println!("Client {} disconnected", address);
                        break;
                    },
                }

                util::sleep();
            });
        }
    }

    fn trim_empty_buffer(buffer: Vec<u8>) -> Vec<u8> {
        buffer
            .into_iter()
            .take_while(|&x| x != 0)
            .collect()
    }

    fn broadcast_message(clients: Vec<ClientHandle>, rx: &Receiver<String>, message_size: usize) -> Vec<ClientHandle> {
        if let Ok(message) = rx.try_recv() {
            let msg: Message = serde_json::from_str(&message).unwrap();

            return clients
                .into_iter()
                // .filter(|ref client_handle| client_handle.address != msg.address)
                .filter_map(|mut client_handle| {
                    let mut buffer = message.clone().into_bytes();
                    // Without matching buffer size the server
                    // will fail to `read_exact`
                    buffer.resize(message_size, 0);

                    // If sending message to a client failed,
                    // remove it because the client has disconnected
                    // TODO: Maybe the removal of client should be in the thread
                    // above
                    if client_handle.address != msg.address {
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
