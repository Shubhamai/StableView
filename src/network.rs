/// Deals with sending the data (x,y,depth,yaw,pitch,roll) to opentrack (https://github.com/opentrack/opentrack) using UDP socket
use crate::structs::network::SocketNetwork;
use anyhow::{Context, Result};
use std::{io::Error, net::UdpSocket};

impl SocketNetwork {
    pub fn new(ip: String, port: String) -> Result<Self> {
        tracing::info!("Sending data to {} on port {}", ip, port);

        let address = format!("{}:{}", ip, port);

        let socket_network = UdpSocket::bind("0.0.0.0:0")
            .with_context(|| format!("Unable to bind socket : {:?}:{:?}", ip, port))?;

        Ok(Self {
            address,
            socket_network,
        })
    }

    // TODO : Cleaning and possibly removing unsafe code
    pub fn send(&mut self, data: [f32; 6]) -> Result<()> {
        let data: [f64; 6] = [
            data[0] as f64,
            data[1] as f64,
            data[2] as f64,
            data[3] as f64,
            data[4] as f64,
            data[5] as f64,
        ];

        // Convert an array to f64 to array of u8
        let out =
            unsafe { std::slice::from_raw_parts(data.as_ptr() as *const u8, data.len() * 10) };

        // Send data
        self.socket_network.send_to(out, &self.address)?;

        Ok(())
    }
}

#[test]
pub fn test_socket_network() -> Result<(), Error> {
    let mut socket_network = SocketNetwork::new("127.0.0.1".to_owned(), "4242".to_owned());
    socket_network.send([1., 2., 3., 4., 5., 6.])?;

    Ok(())
}
