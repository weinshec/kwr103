//! Serial communication for USB connected power supplies

use std::time::Duration;

use serialport;

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
/// <CMD><ID>:<VAL>\n
/// ```
/// so in order to set the output voltage to 12.0V on the power supply with device id 1 the
/// serialized payload should look like
/// `VSET01:12.0\n`.
///
/// Additionally, commands may even be concatenated (separated by the newline character), e.g.
/// `VSET01:42.0\nISET01:2.3\n` both sets the output voltage to 42.0V as well as the output current
/// to 2.3A
pub trait UsbCommand: Sized {
    /// Serialize the command to bytes for sending on the serial interface
    fn serialize(cmd: Self, device_id: u8) -> Vec<u8>;
}

impl UsbCommand for cmd::Voltage {
    fn serialize(cmd: Self, device_id: u8) -> Vec<u8> {
        format!("VSET{:02}:{:.3}\n", device_id, cmd.0).into_bytes()
    }
}
impl UsbCommand for cmd::Current {
    fn serialize(cmd: Self, device_id: u8) -> Vec<u8> {
        format!("ISET{:02}:{:.3}\n", device_id, cmd.0).into_bytes()
    }
}
impl UsbCommand for cmd::Power {
    fn serialize(cmd: Self, device_id: u8) -> Vec<u8> {
        format!("OUT{:02}:{}\n", device_id, cmd.0 as u8).into_bytes()
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
/// <QUERY><ID>?\n
/// ```
/// For example a query for the output voltage setting on the power supply with id 1 serializes as
/// `VSET01?\n`, with the response following as `42.0\n` (newline terminated value).
///
/// Additionally, queries may even be concatenated (separated by the newline character),
/// e.g. `VSET01?\nISET01?\n` queries both the output voltage and current.
pub trait UsbQuery: UpsResponse {
    /// Serialize the command to bytes for sending on the serial interface
    fn serialize(device_id: u8) -> Vec<u8>;
}

impl UsbQuery for cmd::Voltage {
    fn serialize(device_id: u8) -> Vec<u8> {
        format!("VSET{:02}?\n", device_id).into_bytes()
    }
}
impl UsbQuery for cmd::Current {
    fn serialize(device_id: u8) -> Vec<u8> {
        format!("ISET{:02}?\n", device_id).into_bytes()
    }
}
impl UsbQuery for cmd::Power {
    fn serialize(device_id: u8) -> Vec<u8> {
        format!("OUT{:02}?\n", device_id).into_bytes()
    }
}
impl UsbQuery for cmd::Output {
    fn serialize(device_id: u8) -> Vec<u8> {
        format!(
            "OUT{:02}?\nVOUT{:02}?\nIOUT{:02}?\n",
            device_id, device_id, device_id
        )
        .into_bytes()
    }
}
impl UsbQuery for cmd::DeviceInfo {
    fn serialize(_device_id: u8) -> Vec<u8> {
        String::from(":SYST:DEVINFO?\n").into_bytes()
    }
}

/// A USB connected KWR103 power supply.
pub struct Kwr103Usb {
    serial: Box<dyn serialport::SerialPort>,
    device_id: u8,
}

impl Kwr103Usb {
    /// Create a new power supply instance
    pub fn new(port_name: &str, baud_rate: u32, device_id: u8) -> Result<Self, TransactionError> {
        if device_id > 99 {
            return Err(TransactionError::InvalidConfiguration(
                "Kwr103Usb device id must be in [0; 99]".to_string(),
            ));
        }

        let serial = serialport::new(port_name, baud_rate)
            .timeout(Duration::from_millis(150))
            .parity(serialport::Parity::None)
            .stop_bits(serialport::StopBits::One)
            .open()?;

        Ok(Self { serial, device_id })
    }

    /// Issue a command to the power supply
    ///
    /// # Examples
    /// ```no_run
    /// use kwr103::command::*;
    /// use kwr103::usb::Kwr103Usb;
    ///
    /// let mut ups = Kwr103Usb::new("/dev/ttyACM0", 115200, 1).unwrap();
    /// ups.command(Voltage(42.0)).unwrap();
    /// ```
    pub fn command<C: UsbCommand>(&mut self, v: C) -> Result<(), TransactionError> {
        let payload = C::serialize(v, self.device_id);
        if self.serial.write(&payload)? != payload.len() {
            return Err(TransactionError::RequestError);
        }
        Ok(())
    }

    /// Issue a query to the power supply
    ///
    /// # Examples
    /// ```no_run
    /// use kwr103::command::*;
    /// use kwr103::usb::Kwr103Usb;
    ///
    /// let mut ups = Kwr103Usb::new("/dev/ttyACM0", 115200, 1).unwrap();
    /// let voltage = ups.query::<Voltage>().unwrap();
    /// println!("Voltage = {:.3}V", voltage.0);
    /// ```
    pub fn query<C: UsbQuery>(&mut self) -> Result<C, TransactionError> {
        let payload = C::serialize(self.device_id);
        if self.serial.write(&payload)? != payload.len() {
            return Err(TransactionError::RequestError);
        }

        let mut response: Vec<u8> = Vec::new();
        let mut is_done = false;
        while !is_done {
            let mut buf: Vec<u8> = vec![0; 512];
            match self.serial.read(buf.as_mut_slice()) {
                Ok(count) => {
                    response.extend(buf.drain(..count));
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => {
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
    fn usb_command_voltage() {
        assert_eq!(
            UsbCommand::serialize(cmd::Voltage(42.123), 2),
            "VSET02:42.123\n".as_bytes()
        );
    }

    #[test]
    fn usb_command_current() {
        assert_eq!(
            UsbCommand::serialize(cmd::Current(2.001), 2),
            "ISET02:2.001\n".as_bytes()
        );
    }

    #[test]
    fn usb_command_power() {
        assert_eq!(
            UsbCommand::serialize(cmd::Power(cmd::Switch::On), 2),
            "OUT02:1\n".as_bytes()
        );
        assert_eq!(
            UsbCommand::serialize(cmd::Power(cmd::Switch::Off), 2),
            "OUT02:0\n".as_bytes()
        );
    }

    #[test]
    fn usb_query_voltage() {
        assert_eq!(
            <cmd::Voltage as UsbQuery>::serialize(2),
            "VSET02?\n".as_bytes()
        );
    }

    #[test]
    fn usb_query_current() {
        assert_eq!(
            <cmd::Current as UsbQuery>::serialize(2),
            "ISET02?\n".as_bytes()
        );
    }

    #[test]
    fn usb_query_power() {
        assert_eq!(
            <cmd::Power as UsbQuery>::serialize(2),
            "OUT02?\n".as_bytes()
        );
    }

    #[test]
    fn usb_query_output() {
        assert_eq!(
            <cmd::Output as UsbQuery>::serialize(2),
            "OUT02?\nVOUT02?\nIOUT02?\n".as_bytes()
        );
    }

    #[test]
    fn usb_query_deviceinfo() {
        assert_eq!(
            <cmd::DeviceInfo as UsbQuery>::serialize(2),
            ":SYST:DEVINFO?\n".as_bytes()
        );
    }

    #[test]
    fn creating_new_kwr103_instance_with_invalid_id() {
        let kwr103 = Kwr103Usb::new("/dev/ttyACM0", 115200, 100);
        assert!(kwr103.is_err());
    }
}
