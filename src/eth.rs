//! UDP communication for ethernet connected power supplies

use std::net::{ToSocketAddrs, UdpSocket};
use std::time::Duration;

use crate::command as cmd;
use crate::{ResponseError, TransactionError, UpsResponse};

/// A command to be issued to the power supply.
///
/// Types implementing this trait represent commands that are intended to change settings or the
/// status of the power supply and do not trigger a response message.
///
/// ## Protocol
///
/// Most commands follow the simple syntax of
/// ```text
/// <CMD>:<VAL>\n
/// ```
/// so in order to set the output voltage to 12.0V the serialized payload should look like
/// `VSET:12.0\n`.
///
/// Additionally, commands may even be concatenated (separated by the newline character),
/// e.g. `VSET:42.0\nISET:2.3\n` both sets the output voltage to 42.0V as well as the output
/// current to 2.3A
pub trait UdpCommand: Sized {
    /// Serialize the command to the UDP frame payload
    fn serialize(cmd: Self) -> Vec<u8>;
}

impl UdpCommand for cmd::Voltage {
    fn serialize(cmd: Self) -> Vec<u8> {
        format!("VSET:{:.3}\n", cmd.0).into_bytes()
    }
}
impl UdpCommand for cmd::Current {
    fn serialize(cmd: Self) -> Vec<u8> {
        format!("ISET:{:.3}\n", cmd.0).into_bytes()
    }
}
impl UdpCommand for cmd::Power {
    fn serialize(cmd: Self) -> Vec<u8> {
        format!("OUT:{}\n", cmd.0 as u8).into_bytes()
    }
}

/// A query to be issued to the power supply.
///
/// Types implementing this trait represent settings or status values that can be queried from the
/// power supply, i.e. after a query has been issued, the power supply will answer with a
/// corresponding response.
///
/// ## Protocol
///
/// Communication is a simple Query-Response whereas the query is formatted with the following
/// syntax:
/// ```text
/// <QUERY>?\n
/// ```
/// For example a query for the output voltage setting serializes as `VSET?\n`, with the response
/// following as `42.0\n` (newline terminated value).
///
/// Additionally, queries may even be concatenated (separated by the newline character),
/// e.g. `VSET?\nISET?\n` queries both the output voltage and current. Note however, that even
/// though the query was issued in a single UDP frame, the power supply will respond with one frame
/// for each queried value, i.e. 2 frames for the given example.
pub trait UdpQuery: UpsResponse {
    /// Serialize the query to the UDP frame payload
    fn serialize() -> Vec<u8>;
}

impl UdpQuery for cmd::Voltage {
    fn serialize() -> Vec<u8> {
        String::from("VSET?\n").into_bytes()
    }
}
impl UdpQuery for cmd::Current {
    fn serialize() -> Vec<u8> {
        String::from("ISET?\n").into_bytes()
    }
}
impl UdpQuery for cmd::Power {
    fn serialize() -> Vec<u8> {
        String::from("OUT?\n").into_bytes()
    }
}
impl UdpQuery for cmd::Output {
    fn serialize() -> Vec<u8> {
        String::from("OUT?\nVOUT?\nIOUT?\n").into_bytes()
    }
}

/// An ethernet connected KWR103 power supply.
pub struct Kwr103Eth {
    socket: UdpSocket,
}

impl Kwr103Eth {
    /// Create a new power supply instance
    pub fn new<A: ToSocketAddrs>(socket_address: A) -> Result<Self, TransactionError> {
        let socket = UdpSocket::bind("0.0.0.0:18190")?;
        socket.set_read_timeout(Some(Duration::from_millis(150)))?;
        socket.connect(socket_address)?;
        Ok(Self { socket })
    }

    /// Issue a command to the power supply
    ///
    /// # Examples
    /// ```no_run
    /// use kwr103::command::*;
    /// use kwr103::eth::Kwr103Eth;
    ///
    /// let ups = Kwr103Eth::new("192.168.1.198:18190").unwrap();
    /// ups.command(Voltage(42.0)).unwrap();
    /// ```
    pub fn command<C: UdpCommand>(&self, v: C) -> Result<(), TransactionError> {
        let payload = C::serialize(v);
        if self.socket.send(&payload)? != payload.len() {
            return Err(TransactionError::RequestError);
        }
        Ok(())
    }

    /// Issue a query to the power supply
    ///
    /// # Examples
    /// ```no_run
    /// use kwr103::command::*;
    /// use kwr103::eth::Kwr103Eth;
    ///
    /// let ups = Kwr103Eth::new("192.168.1.198:18190").unwrap();
    /// let voltage = ups.query::<Voltage>().unwrap();
    /// println!("Voltage = {:.3}V", voltage.0);
    /// ```
    pub fn query<C: UdpQuery>(&self) -> Result<C, TransactionError> {
        let payload = C::serialize();
        if self.socket.send(&payload)? != payload.len() {
            return Err(TransactionError::RequestError);
        }

        let mut response: Vec<u8> = Vec::new();
        let mut is_done = false;
        while !is_done {
            let mut buf: Vec<u8> = vec![0; 512];
            match self.socket.recv(buf.as_mut_slice()) {
                Ok(count) => {
                    response.extend(buf.drain(..count));
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
        Ok(C::parse(&response)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn udp_command_voltage() {
        assert_eq!(
            UdpCommand::serialize(cmd::Voltage(42.123)),
            "VSET:42.123\n".as_bytes()
        );
    }

    #[test]
    fn udp_command_current() {
        assert_eq!(
            UdpCommand::serialize(cmd::Current(2.001)),
            "ISET:2.001\n".as_bytes()
        );
    }

    #[test]
    fn udp_command_power() {
        assert_eq!(
            UdpCommand::serialize(cmd::Power(cmd::Switch::On)),
            "OUT:1\n".as_bytes()
        );
        assert_eq!(
            UdpCommand::serialize(cmd::Power(cmd::Switch::Off)),
            "OUT:0\n".as_bytes()
        );
    }

    #[test]
    fn udp_query_voltage() {
        assert_eq!(
            <cmd::Voltage as UdpQuery>::serialize(),
            "VSET?\n".as_bytes()
        );
    }

    #[test]
    fn udp_query_current() {
        assert_eq!(
            <cmd::Current as UdpQuery>::serialize(),
            "ISET?\n".as_bytes()
        );
    }

    #[test]
    fn udp_query_power() {
        assert_eq!(<cmd::Power as UdpQuery>::serialize(), "OUT?\n".as_bytes());
    }

    #[test]
    fn udp_query_output() {
        assert_eq!(
            <cmd::Output as UdpQuery>::serialize(),
            "OUT?\nVOUT?\nIOUT?\n".as_bytes()
        );
    }
}
