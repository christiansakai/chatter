use std::net::{SocketAddr};

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    pub from: SocketAddr,
    pub content: String,
}
