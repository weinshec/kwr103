use crate::command as cmd;
use clap::{Args, Subcommand, ValueEnum};

/// Connection type
#[derive(Debug, Clone, ValueEnum)]
pub enum Connection {
    Usb,
    Eth,
}

#[derive(Debug, Args)]
pub struct UsbDetails {
    /// USB: serial device
    #[clap(long, default_value_t = String::from("/dev/ttyACM0"))]
    pub device: String,

    /// USB: serial baud rate
    #[clap(long, default_value_t = 115200)]
    pub baud: u32,

    /// USB: RS485 device ID
    #[clap(long, default_value_t = 1)]
    pub id: u8,
}

#[derive(Debug, Args)]
pub struct EthDetails {
    /// ETH: IP Address
    #[clap(long, default_value_t = String::from("192.168.1.198"))]
    pub ip: String,

    /// ETH: UDP port
    #[clap(long, default_value_t = 18190)]
    pub port: u16,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Set the output voltage
    Voltage {
        #[clap(help = "Volts")]
        u: f32,
    },
    /// Set the output current
    Current {
        #[clap(help = "Ampere")]
        i: f32,
    },
    /// Turn power supply output 'on' or 'off'
    Output {
        #[clap(help = "on/off")]
        switch: cmd::Switch,
    },
    /// Show current output voltage and current
    Status,
    /// Show system information
    Info,
}
