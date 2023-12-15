use crate::command as cmd;
use clap::{Args, Subcommand};

#[derive(Debug, Args)]
#[group(required = false, multiple = false)]
pub struct Connection {
    /// Specify device for serial connection [example: /dev/ttyACM0]
    #[arg(long)]
    pub device: Option<String>,

    /// Specify IP address for ethernet connection [example: 192.168.1.195]
    #[arg(long)]
    pub ip: Option<String>,
}

#[derive(Debug, Args, Clone)]
pub struct UsbDetails {
    /// Serial baud rate
    #[clap(long, default_value_t = 115200)]
    pub baud: u32,

    /// Optional RS485 device ID
    #[clap(long)]
    pub id: Option<u8>,
}

#[derive(Debug, Args, Clone)]
pub struct EthDetails {
    /// UDP port for ethernet connected devices
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
    /// Turn DHCP 'on' or 'off'
    Dhcp {
        #[clap(help = "on/off")]
        switch: cmd::Switch,
    },
}
