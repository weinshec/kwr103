use crate::command as cmd;
use clap::Subcommand;

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
    Power {
        #[clap(help = "on/off")]
        switch: cmd::Switch,
    },
    /// Show current output voltage and current
    Output,
}
