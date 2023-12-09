//! Error types for the KWR103 power supply crate

use thiserror::Error;

/// Common error type for any kind transactional errors (communication, decoding, etc.)
#[derive(Error, Debug)]
pub enum TransactionError {
    /// Connection to the power supply failed
    #[error("Ethernet connection error")]
    EthConnection(#[from] std::io::Error),

    /// Connection to the power supply failed
    #[error("Serial connection error")]
    UsbConnection(#[from] serialport::Error),

    /// Error while handling a power supply response
    #[error("Response Error")]
    ResponseError(#[from] ResponseError),

    /// Transmitting our request (command or query) failed
    #[error("Request Error")]
    RequestError,

    /// Invalid device configuration or parameter
    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),
}

/// Errors that may occur while handling a power supply reponse
#[derive(Error, Debug)]
pub enum ResponseError {
    /// The response from the power supply was invalid
    #[error("Failed to parse power supply response")]
    Invalid,

    /// The power supply sent less bytes as expected in response
    #[error("No or incomplete response from power supply")]
    Incomplete,
}
