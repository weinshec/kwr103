use clap::Parser;

use kwr103::cli::Command;
use kwr103::command::*;
use kwr103::eth::Kwr103Eth;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
pub struct Kwr103 {
    #[clap(subcommand)]
    pub command: Command,

    /// Power supply IP Address
    #[clap(short, long, default_value_t = String::from("192.168.1.198"))]
    pub ip: String,

    /// Power supply UDP port
    #[clap(short, long, default_value_t = 18190)]
    pub port: u16,
}

fn main() -> anyhow::Result<()> {
    let args = Kwr103::parse();

    let kwr103 = Kwr103Eth::new((args.ip, args.port))?;

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
        Command::Info => {
            println!("{}", kwr103.query::<DeviceInfo>()?)
        }
    }

    Ok(())
}
