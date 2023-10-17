use clap::Parser;

use kwr103::cli::Command;
use kwr103::command::*;
use kwr103::usb::Kwr103Usb;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
pub struct Kwr103 {
    #[clap(subcommand)]
    pub command: Command,

    /// Power supply serial device
    #[clap(short, long, default_value_t = String::from("/dev/ttyACM0"))]
    pub device: String,

    /// Power supply serial baud rate
    #[clap(short, long, default_value_t = 115200)]
    pub baud: u32,

    /// Power supply RS485 device ID
    #[clap(short, long, default_value_t = 1)]
    pub id: u8,
}

fn main() -> anyhow::Result<()> {
    let args = Kwr103::parse();

    let mut kwr103 = Kwr103Usb::new(&args.device, args.baud, args.id)?;

    match args.command {
        Command::Voltage { u } => {
            kwr103.command(Voltage(u))?;
        }
        Command::Current { i } => {
            kwr103.command(Current(i))?;
        }
        Command::Power { switch } => {
            kwr103.command(Power(switch))?;
        }
        Command::Output => {
            println!("{}", kwr103.query::<Output>()?)
        }
    }

    Ok(())
}
