use thiserror::Error;

pub mod eth;
pub mod usb;

/// Common error type for any kind transactional errors (communication, decoding, etc.)
#[derive(Error, Debug)]
pub enum TransactionError {
    /// Connection to the power supply failed
    #[error("Ethernet connection error")]
    EthConnection(#[from] std::io::Error),

    /// Connection to the power supply failed
    #[error("Serial connection error")]
    UsbConnection(#[from] serialport::Error),

    /// The response from the power supply was invalid
    #[error("Failed to parse UPS response")]
    ResponseInvalid,

    /// The power supply sent less bytes as expected in response
    #[error("Incomplete UPS response")]
    ResponseIncomplete,

    /// Transmitting our request (command or query) failed
    #[error("Request failed")]
    RequestFailed,

    /// Invalid device configuration or parameter
    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),
}

/// Result type for any kind of transaction (command or query) with the power supply
pub type Result<T> = std::result::Result<T, TransactionError>;

pub mod commands {
    //! Command types to interact with any power supply

    /// Representing the state of a switchable feature or output
    #[derive(Debug, PartialEq)]
    pub enum Switch {
        /// Disable feature or output
        Off = 0,
        /// Enable feature or output
        On = 1,
    }

    /// Output voltage setting in units of volts
    #[derive(Debug, PartialEq)]
    pub struct Voltage(pub f32);

    /// Output current setting in units of ampere
    #[derive(Debug, PartialEq)]
    pub struct Current(pub f32);

    /// Output power switch On/Off
    #[derive(Debug, PartialEq)]
    pub struct Power(pub Switch);

    /// Actual output voltage and current state
    #[derive(Debug, PartialEq)]
    pub struct Output {
        pub power: Switch,
        pub voltage: f32,
        pub current: f32,
    }

    impl std::str::FromStr for Switch {
        type Err = &'static str;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            match s
                .parse::<u8>()
                .map_err(|_| "Switch value is not a valid integer")?
            {
                0 => Ok(Switch::Off),
                1 => Ok(Switch::On),
                _ => Err("Invalid digit for Switch (must be 0/1)"),
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn switch_from_string() {
            assert_eq!(Ok(Switch::Off), "0".parse());
            assert_eq!(Ok(Switch::On), "1".parse());
            assert!("2".parse::<Switch>().is_err());
            assert!("_".parse::<Switch>().is_err());
        }

        #[test]
        fn switch_into_u8() {
            assert_eq!(0u8, Switch::Off as u8);
            assert_eq!(1u8, Switch::On as u8);
        }
    }
}
