use clap::Parser;

use kwr103::{cli, command::*, eth, usb, EthConnection, Kwr103, UsbConnection};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
pub struct Kwr103Args {
    #[command(flatten)]
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
        cli::Connection {
            device: Some(dev),
            ip: None,
        } => UsbConnection::new(&dev, args.usb.baud, args.usb.id)?.into(),

        cli::Connection {
            device: None,
            ip: Some(ip),
        } => EthConnection::new((ip, args.eth.port))?.into(),

        _ => {
            let mut serial_devices = usb::find_devices(args.usb.baud, args.usb.id);
            let mut ethernet_devices = eth::find_devices();
            match (serial_devices.len(), ethernet_devices.len()) {
                (0, 0) => {
                    eprintln!("No devices found");
                    std::process::exit(1);
                }
                (1, 0) => serial_devices.remove(0).open()?.into(),
                (0, 1) => ethernet_devices.remove(0).open()?.into(),
                (_, _) => {
                    eprintln!(
                        "Multiple device connections found.\n\
                              Specify explicitely using `--device=<DEVICE>` or `--ip=<IP>`"
                    );
                    std::process::exit(1);
                }
            }
        }
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
        cli::Command::Dhcp { switch } => {
            kwr103.command(Dhcp(switch))?;
        }
    }

    Ok(())
}
