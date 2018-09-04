use std::net::{SocketAddr};

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    pub address: SocketAddr,
    pub content: String,
}
