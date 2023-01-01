use std::net::{SocketAddr, UdpSocket};

pub struct SocketNetwork {
    pub address: SocketAddr,
    pub socket_network: UdpSocket,
}
