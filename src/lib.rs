//! KWR103 remote controllable power supply

#![deny(warnings)]
#![warn(missing_docs)]
pub mod command;
pub mod eth;
pub mod usb;

mod error;
pub use error::{ResponseError, TransactionError};

/// Marks types that may be parsed from a power supply response.
///
/// Types implementing this trait represent settings or status values that can be queried from the
/// power supply, i.e. the response to a [`eth::UdpQuery`] or [`usb::UsbQuery`].
pub trait UpsResponse: Sized {
    /// Parse the power supply response
    fn parse(bytes: &[u8]) -> std::result::Result<Self, ResponseError>;
}
