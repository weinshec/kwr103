use thiserror::Error;

pub mod eth;

/// Common error type for any kind transactional errors (communication, decoding, etc.)
#[derive(Error, Debug)]
pub enum TransactionError {
    /// Connection to the power supply failed
    #[error("Connection error")]
    Connection(#[from] std::io::Error),

    /// The response from the power supply was invalid
    #[error("Failed to parse UPS response")]
    ResponseInvalid,

    /// The power supply sent less bytes as expected in response
    #[error("Incomplete UPS response")]
    ResponseIncomplete,

    /// Transmitting our request (command or query) failed
    #[error("Request failed")]
    RequestFailed,
}

/// Result type for any kind of transaction (command or query) with the power supply
pub type Result<T> = std::result::Result<T, TransactionError>;

pub mod commands {
    //! Command types to interact with any power supply

    /// Representing the state of a switchable feature or output
    #[derive(Debug, PartialEq)]
    pub enum Switch {
        /// Enable feature or output
        On,
        /// Disable feature or output
        Off,
    }

    /// Output voltage setting in units of volts
    #[derive(Debug, PartialEq)]
    pub struct Voltage {
        pub volts: f32,
    }

    /// Output current setting in units of ampere
    #[derive(Debug, PartialEq)]
    pub struct Current {
        pub ampere: f32,
    }

    /// Output power switch On/Off
    #[derive(Debug, PartialEq)]
    pub struct Output {
        pub switch: Switch,
    }
}
