//! Serial communication for USB connected power supplies

use std::time::Duration;

use serialport;

use crate::{Kwr103, ResponseError, TransactionError, Transport};

/// Communication channel for a serial/USB connected power supply
pub struct UsbConnection {
    serial: Box<dyn serialport::SerialPort>,
    device_id: Option<u8>,
}

impl UsbConnection {
    /// Create a new USB communication channel
    pub fn new(
        port_name: &str,
        baud_rate: u32,
        device_id: Option<u8>,
    ) -> Result<Self, TransactionError> {
        if let Some(id) = device_id {
            if id == 0 || id > 99 {
                return Err(TransactionError::InvalidConfiguration(
                    "KWR103 RS485 device id must be in [1; 99]".to_string(),
                ));
            }
        }

        let serial = serialport::new(port_name, baud_rate)
            .timeout(Duration::from_millis(150))
            .parity(serialport::Parity::None)
            .stop_bits(serialport::StopBits::One)
            .open()?;

        Ok(Self { serial, device_id })
    }
}

impl Transport for UsbConnection {
    fn send(&mut self, bytes: &[u8]) -> Result<(), TransactionError> {
        if self.serial.write(bytes)? != bytes.len() {
            return Err(TransactionError::RequestError);
        }
        Ok(())
    }

    fn receive(&mut self) -> Result<Vec<u8>, TransactionError> {
        let mut received: Vec<u8> = Vec::new();
        let mut is_done = false;
        while !is_done {
            let mut buf: Vec<u8> = vec![0; 512];
            match self.serial.read(buf.as_mut_slice()) {
                Ok(count) => {
                    received.extend(buf.drain(..count));
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => {
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

impl From<UsbConnection> for Kwr103 {
    fn from(con: UsbConnection) -> Self {
        let device_id = con.device_id;
        Kwr103 {
            transport: Box::new(con),
            device_id,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creating_new_usb_connection_with_invalid_id() {
        let lo = UsbConnection::new("/dev/ttyACM0", 115200, Some(0));
        assert!(lo.is_err_and(|e| e.to_string().contains("RS485 device id")));

        let hi = UsbConnection::new("/dev/ttyACM0", 115200, Some(100));
        assert!(hi.is_err_and(|e| e.to_string().contains("RS485 device id")));
    }
}
