//! KWR103 remote controllable power supply
//!
//! This crate provides remote control access to Korad KWR103 type power supplies implementing
//! both, serial/USB and ethernet/UDP based, communication channels.
//!
//! # Example
//! ```no_run
//! use kwr103::{command::*, Kwr103, TransactionError, UsbConnection};
//!
//! fn main() -> Result<(), TransactionError> {
//!     // Establish USB connection
//!     let mut kwr103: Kwr103 = UsbConnection::new("/dev/ttyACM0", 115200, 1)?.into();
//!
//!     // Adjust voltage and current settings
//!     kwr103.command(Voltage(42.0))?;
//!     kwr103.command(Current(1.2))?;
//!     
//!     // Switch output on
//!     kwr103.command(Output(Switch::On))?;
//!
//!     // Query status, prints e.g. "Output: On, Voltage[V]: 42.000, Current[A]: 0.131"
//!     println!("{}", kwr103.query::<Status>()?);
//!
//!     Ok(())
//! }
//! ```
//!
//! # Command Line Interface
//! To control the power supply from a terminal, you can use the `kwr103` command line tool. Use
//! `kwr103 --help` for the full specification.
//!
//! ```text
//! > kwr103 usb status
//! Output: Off, Voltage[V]: 0.000, Current[A]: 0.000
//!
//! > kwr103 usb output on
//! > kwr103 usb status
//! Output: On, Voltage[V]: 42.000, Current[A]: 0.131
//! ```

#![deny(warnings)]
#![warn(missing_docs)]
pub mod command;
pub mod error;
pub mod eth;
pub mod usb;

pub use error::{ResponseError, TransactionError};
pub use eth::EthConnection;
pub use usb::UsbConnection;

#[doc(hidden)]
pub mod cli;

/// A command to be issued to the power supply.
///
/// Types implementing this trait represent commands that are intended to change settings or the
/// state of the power supply and do not trigger a response message.
///
/// ## Protocol
///
/// Most commands follow the simple syntax of
/// ```text
/// <CMD>[ID]:<VAL>\n
/// ```
/// so in order to set the output voltage to 12.0V on the power supply with device id 1 the
/// serialized payload should look like
/// `VSET01:12.0\n`.
///
/// If `device_id` is `None`, the `[ID]` field is ommitted.
///
/// Additionally, commands may even be concatenated (separated by the newline character), e.g.
/// `VSET01:42.0\nISET01:2.3\n` both sets the output voltage to 42.0V as well as the output current
/// to 2.3A
pub trait Command: Sized {
    /// Serialize the command to bytes for sending on the serial interface
    fn serialize(cmd: Self, device_id: Option<u8>) -> Vec<u8>;
}

/// A query to be issued to the power supply.
///
/// Types implementing this trait represent settings or state values that can be queried from the
/// power supply, i.e. after a query has been issued, the power supply will answer with a
/// corresponding response.
///
/// ## Protocol
///
/// Communication follows a simple Query-Response schema where the query is formatted with the
/// following syntax:
/// ```text
/// <QUERY>[ID]?\n
/// ```
/// so for example a query for the output voltage setting on the power supply with id 1 serializes
/// as `VSET01?\n`, with the response following as `42.0\n` (newline terminated value).
///
/// If `device_id` is `None`, the `[ID]` field is ommitted.
///
/// Additionally, queries may even be concatenated (separated by the newline character),
/// e.g. `VSET01?\nISET01?\n` queries both the output voltage and current.
pub trait Query: Sized {
    /// Serialize to bytes for sending
    fn serialize(device_id: Option<u8>) -> Vec<u8>;

    /// Parse `bytes` response from the power supply
    fn parse(bytes: &[u8]) -> std::result::Result<Self, ResponseError>;
}

/// A type implementing `Transport` defines how to physically communicate with the power supply
pub trait Transport {
    /// Attempt to send `bytes` to the power supply
    fn send(&mut self, bytes: &[u8]) -> Result<(), TransactionError>;

    /// Receive bytes from the power supply
    fn receive(&mut self) -> Result<Vec<u8>, TransactionError>;
}

/// A KWR103 type power supply
///
/// This is the main access point to control a power supply.
///
/// # Note
/// Rather than instantiating directly, use one of the [`Transport`] types specifying the
/// connection details.
///
/// ```no_run
/// use kwr103::{Kwr103, UsbConnection};
///
/// let mut kwr103 = Kwr103::from(UsbConnection::new("/dev/ttyACM0", 115200, 1).unwrap());
/// ```
pub struct Kwr103 {
    transport: Box<dyn Transport>,
    device_id: Option<u8>,
}

impl Kwr103 {
    /// Issue a [`Command`] to the power supply.
    ///
    /// Commands do not trigger any response from the power supply, so there is no acknowledgement
    /// from the power supply that it actually received and accepted the command. Use a
    /// corresponding [`Kwr103::query`] to check if the command succeeded.
    ///
    /// # Example
    /// ```no_run
    /// use kwr103::{command::*, Kwr103, UsbConnection};
    ///
    /// let mut kwr103 = Kwr103::from(UsbConnection::new("/dev/ttyACM0", 115200, 1).unwrap());
    /// kwr103.command(Voltage(42.0)).unwrap();
    /// ```
    pub fn command<C: Command>(&mut self, cmd: C) -> Result<(), TransactionError> {
        let payload = C::serialize(cmd, self.device_id);
        self.transport.send(payload.as_slice())
    }

    /// Issue a [`Query`] to the power supply.
    ///
    /// Queries obtain status informations or settings from the power supply and thus involve a
    /// request-response type communication.
    ///
    /// See [`Query`] for details and implementors.
    ///
    /// # Example
    /// ```no_run
    /// use kwr103::{command::*, Kwr103, UsbConnection};
    ///
    /// let mut kwr103 = Kwr103::from(UsbConnection::new("/dev/ttyACM0", 115200, 1).unwrap());
    /// let voltage = kwr103.query::<Voltage>().unwrap();
    /// println!("Voltage = {:.3}V", voltage.0);
    /// ```
    pub fn query<Q: Query>(&mut self) -> Result<Q, TransactionError> {
        let payload = Q::serialize(self.device_id);
        self.transport.send(payload.as_slice())?;

        let response = self.transport.receive()?;
        Ok(Q::parse(&response)?)
    }
}
