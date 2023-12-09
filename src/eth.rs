//! UDP communication for ethernet connected power supplies

use std::net::{ToSocketAddrs, UdpSocket};
use std::time::Duration;

use crate::{Kwr103, ResponseError, TransactionError, Transport};

/// Communication channel for an ethernet connected power supply
pub struct EthConnection {
    socket: UdpSocket,
}

impl EthConnection {
    /// Create a new ethernet communication channel
    pub fn new<A: ToSocketAddrs>(socket_address: A) -> Result<Self, TransactionError> {
        let socket = UdpSocket::bind("0.0.0.0:18190")?;
        socket.set_read_timeout(Some(Duration::from_millis(150)))?;
        socket.connect(socket_address)?;
        Ok(Self { socket })
    }
}

impl Transport for EthConnection {
    fn send(&mut self, bytes: &[u8]) -> Result<(), TransactionError> {
        if self.socket.send(bytes)? != bytes.len() {
            return Err(TransactionError::RequestError);
        }
        Ok(())
    }

    fn receive(&mut self) -> Result<Vec<u8>, TransactionError> {
        let mut received: Vec<u8> = Vec::new();
        let mut is_done = false;
        while !is_done {
            let mut buf: Vec<u8> = vec![0; 512];
            match self.socket.recv(buf.as_mut_slice()) {
                Ok(count) => {
                    received.extend(buf.drain(..count));
                }
                Err(ref e)
                    if e.kind() == std::io::ErrorKind::TimedOut
                        || e.kind() == std::io::ErrorKind::WouldBlock =>
                {
                    is_done = true;
                }
                Err(_) => {
                    return Err(TransactionError::ResponseError(ResponseError::Incomplete));
                }
            };
        }
        Ok(received)
    }
}

impl From<EthConnection> for Kwr103 {
    fn from(con: EthConnection) -> Self {
        Kwr103 {
            transport: Box::new(con),
            device_id: None,
        }
    }
}
