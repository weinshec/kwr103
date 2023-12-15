//! UDP communication for ethernet connected power supplies

use std::io;
use std::net::{Ipv4Addr, ToSocketAddrs, UdpSocket};
use std::time::Duration;

use crate::{Kwr103, ResponseError, TransactionError, Transport};

/// Communication channel for an ethernet connected power supply
pub struct EthConnection {
    socket: UdpSocket,
    read_timeout: Duration,
}

impl EthConnection {
    /// Create a new ethernet communication channel
    pub fn new<A: ToSocketAddrs>(socket_address: A) -> Result<Self, TransactionError> {
        let socket = UdpSocket::bind("0.0.0.0:18190")?;
        socket.connect(socket_address)?;
        Ok(Self {
            socket,
            read_timeout: Duration::from_millis(150),
        })
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
        receive_udp_with_timeout(&self.socket, self.read_timeout)
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

/// Connection details for an ethernet connected power supply
#[derive(Debug, Clone)]
pub struct ConnectionDetails {
    /// IP Address of the power supply
    pub ip: Ipv4Addr,

    /// UDP port the power supply listens on
    pub port: u16,
}

impl ConnectionDetails {
    /// Attempt to open a [`EthConnection`] using this connection details
    pub fn open(self) -> Result<EthConnection, TransactionError> {
        EthConnection::new((self.ip, self.port))
    }
}

const FIND_PATTERN: &[u8] = b"find_ka000";

fn broadcast_find_and_listen() -> Result<Vec<u8>, TransactionError> {
    let socket = UdpSocket::bind("0.0.0.0:18191")?;
    socket.set_broadcast(true)?;
    socket.send_to(FIND_PATTERN, "255.255.255.255:18191")?;

    let received = receive_udp_with_timeout(&socket, Duration::from_millis(50))?;

    if let Some(s) = received.strip_prefix("find_ka000".as_bytes()) {
        return Ok(s.to_vec());
    }
    Ok(received)
}

/// Discover serial connected devices
pub fn find_devices() -> Vec<ConnectionDetails> {
    if let Ok(received) = broadcast_find_and_listen() {
        let response = String::from_utf8_lossy(&received);
        let mut tokens = response.split_whitespace();
        return tokens
            .clone()
            .enumerate()
            .step_by(3)
            .filter_map(
                |(idx, ip_token)| match (ip_token.parse(), tokens.nth(idx + 2)?.parse()) {
                    (Ok(ip), Ok(port)) => Some(ConnectionDetails { ip, port }),
                    (_, _) => None,
                },
            )
            .collect();
    }
    vec![]
}

fn receive_udp_with_timeout(
    socket: &UdpSocket,
    timeout: Duration,
) -> Result<Vec<u8>, TransactionError> {
    let mut received = Vec::new();
    let mut buffer = [0; 4096];

    socket.set_read_timeout(Some(timeout))?;

    loop {
        match socket.recv_from(&mut buffer) {
            Ok((size, _)) => {
                received.extend_from_slice(&buffer[..size]);
            }
            Err(err) => match err.kind() {
                io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut => {
                    break;
                }
                _ => {
                    return Err(TransactionError::ResponseError(ResponseError::Incomplete));
                }
            },
        }
    }

    Ok(received)
}
