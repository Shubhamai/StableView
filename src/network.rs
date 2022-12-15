use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};

pub struct SocketNetwork {
    address: SocketAddr,
    socket_network: UdpSocket,
    buf: [u8; 8 * 6],
}

impl SocketNetwork {
    pub fn new(port: u16) -> Self {
        tracing::info!("Using port {port}");

        let address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), port);
        let buf = [0; 8 * 6];
        let socket_network = UdpSocket::bind("0.0.0.0:0").expect("failed to bind UDP socket");

        Self {
            address,
            socket_network,
            buf,
        }
    }

    pub fn send(&mut self, data: [f64; 6]) {
        unsafe {
            let ptr = self.buf.as_mut_ptr().cast::<[f64; 6]>();
            *ptr = data;
        }
        self.socket_network
            .send_to(&self.buf, self.address)
            .expect("failed to send data");
    }
}

#[test]
pub fn test_socket_network() {
    let mut socket_network = SocketNetwork::new(4242);
    socket_network.send([1., 2., 3., 4., 5., 6.])
}
