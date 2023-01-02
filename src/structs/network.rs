use std::net::UdpSocket;

pub struct SocketNetwork {
    pub address: String,
    pub socket_network: UdpSocket,
}
