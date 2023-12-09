use clap::Parser;

use kwr103::{cli, command::*, EthConnection, Kwr103, UsbConnection};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
pub struct Kwr103Args {
    #[clap(rename_all = "lower")]
    pub connection: cli::Connection,

    #[command(flatten)]
    pub eth: cli::EthDetails,

    #[command(flatten)]
    pub usb: cli::UsbDetails,

    #[clap(subcommand)]
    pub command: cli::Command,
}

fn main() -> anyhow::Result<()> {
    let args = Kwr103Args::parse();

    let mut kwr103: Kwr103 = match args.connection {
        cli::Connection::Usb => {
            UsbConnection::new(&args.usb.device, args.usb.baud, args.usb.id)?.into()
        }
        cli::Connection::Eth => EthConnection::new((args.eth.ip, args.eth.port))?.into(),
    };

    match args.command {
        cli::Command::Voltage { u } => {
            kwr103.command(Voltage(u))?;
        }
        cli::Command::Current { i } => {
            kwr103.command(Current(i))?;
        }
        cli::Command::Output { switch } => {
            kwr103.command(Output(switch))?;
        }
        cli::Command::Status => {
            println!("{}", kwr103.query::<Status>()?)
        }
        cli::Command::Info => {
            println!("{}", kwr103.query::<DeviceInfo>()?)
        }
    }

    Ok(())
}
