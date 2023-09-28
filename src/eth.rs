//! UDP communication for ethernet connected power supplies

use std::net::UdpSocket;
use std::str::FromStr;
use std::time::Duration;

use crate::commands as cmd;
use crate::{Result, TransactionError};

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
        format!("VSET:{:.3}\n", cmd.volts).into_bytes()
    }
}
impl UdpCommand for cmd::Current {
    fn serialize(cmd: Self) -> Vec<u8> {
        format!("ISET:{:.3}\n", cmd.ampere).into_bytes()
    }
}
impl UdpCommand for cmd::Output {
    fn serialize(cmd: Self) -> Vec<u8> {
        format!(
            "OUT:{}\n",
            match cmd.switch {
                cmd::Switch::On => 1,
                cmd::Switch::Off => 0,
            }
        )
        .into_bytes()
    }
}

/// A query to be issued to the power supply.
///
/// Types implementing this trait represent settings or status values that can be queried from the
/// power supply, i.e. after a query has been issued, the power will answer with a corresponding
/// response.
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
pub trait UdpQuery: Sized {
    /// Serialize the query to the UDP frame payload
    fn serialize() -> Vec<u8>;

    /// Deserialize the response from the UDP response payload
    fn deserialize(bytes: &[u8]) -> Result<Self>;
}

impl UdpQuery for cmd::Voltage {
    fn serialize() -> Vec<u8> {
        String::from("VSET?\n").into_bytes()
    }

    fn deserialize(bytes: &[u8]) -> Result<Self> {
        Ok(Self {
            volts: parse_single_value(bytes)?,
        })
    }
}
impl UdpQuery for cmd::Current {
    fn serialize() -> Vec<u8> {
        String::from("ISET?\n").into_bytes()
    }

    fn deserialize(bytes: &[u8]) -> Result<Self> {
        Ok(Self {
            ampere: parse_single_value(bytes)?,
        })
    }
}
impl UdpQuery for cmd::Output {
    fn serialize() -> Vec<u8> {
        String::from("OUT?\n").into_bytes()
    }

    fn deserialize(bytes: &[u8]) -> Result<Self> {
        Ok(Self {
            switch: match parse_single_value::<u8>(bytes)? {
                0 => cmd::Switch::Off,
                1 => cmd::Switch::On,
                _ => return Err(TransactionError::ResponseInvalid),
            },
        })
    }
}

fn parse_single_value<T: FromStr>(bytes: &[u8]) -> Result<T> {
    String::from_utf8_lossy(bytes)
        .strip_suffix('\n')
        .ok_or(TransactionError::ResponseIncomplete)?
        .parse()
        .map_err(|_| TransactionError::ResponseInvalid)
}

/// An ethernet connected KWR103 power supply.
pub struct Kwr103Eth {
    socket: UdpSocket,
}

impl Kwr103Eth {
    /// Create a new power supply instance
    pub fn new() -> Result<Self> {
        let socket = UdpSocket::bind("0.0.0.0:18190")?;
        socket.set_read_timeout(Some(Duration::from_millis(150)))?;
        socket.connect("192.168.1.198:18190")?;
        Ok(Self { socket })
    }

    /// Issue a command to the power supply
    ///
    /// # Examples
    /// ```no_run
    /// use kwr103::commands::*;
    /// use kwr103::eth::Kwr103Eth;
    ///
    /// let ups = Kwr103Eth::new().unwrap();
    /// ups.command(Voltage { volts: 42.0 }).unwrap();
    /// ```
    pub fn command<C: UdpCommand>(&self, v: C) -> Result<()> {
        let payload = C::serialize(v);
        if self.socket.send(&payload)? != payload.len() {
            return Err(TransactionError::RequestFailed);
        }
        Ok(())
    }

    /// Issue a query to the power supply
    ///
    /// # Examples
    /// ```no_run
    /// use kwr103::commands::*;
    /// use kwr103::eth::Kwr103Eth;
    ///
    /// let ups = Kwr103Eth::new().unwrap();
    /// let voltage = ups.query::<Voltage>().unwrap();
    /// println!("Voltage = {:.3}V", voltage.volts);
    /// ```
    pub fn query<C: UdpQuery>(&self) -> Result<C> {
        let mut buf = [0; 512];
        self.socket.send(&C::serialize())?;
        let n = self.socket.recv(&mut buf)?;
        C::deserialize(&buf[..n])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn upd_command_voltage() {
        assert_eq!(
            UdpCommand::serialize(cmd::Voltage { volts: 42.123 }),
            "VSET:42.123\n".as_bytes()
        );
    }

    #[test]
    fn upd_command_current() {
        assert_eq!(
            UdpCommand::serialize(cmd::Current { ampere: 2.001 }),
            "ISET:2.001\n".as_bytes()
        );
    }

    #[test]
    fn upd_command_output() {
        assert_eq!(
            UdpCommand::serialize(cmd::Output {
                switch: cmd::Switch::On
            }),
            "OUT:1\n".as_bytes()
        );
        assert_eq!(
            UdpCommand::serialize(cmd::Output {
                switch: cmd::Switch::Off
            }),
            "OUT:0\n".as_bytes()
        );
    }

    #[test]
    fn udp_query_voltage() {
        assert_eq!(
            <cmd::Voltage as UdpQuery>::deserialize("42.123\n".as_bytes()).unwrap(),
            cmd::Voltage { volts: 42.123 }
        );
    }

    #[test]
    fn udp_query_current() {
        assert_eq!(
            <cmd::Current as UdpQuery>::deserialize("2.123\n".as_bytes()).unwrap(),
            cmd::Current { ampere: 2.123 }
        );
    }

    #[test]
    fn udp_query_output() {
        assert_eq!(
            <cmd::Output as UdpQuery>::deserialize("1\n".as_bytes()).unwrap(),
            cmd::Output {
                switch: cmd::Switch::On
            }
        );
        assert_eq!(
            <cmd::Output as UdpQuery>::deserialize("0\n".as_bytes()).unwrap(),
            cmd::Output {
                switch: cmd::Switch::Off
            }
        );
    }
}
