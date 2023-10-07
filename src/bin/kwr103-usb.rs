use kwr103::command::*;
use kwr103::usb::Kwr103Usb;

fn main() -> anyhow::Result<()> {
    {
        let mut kwr103 = Kwr103Usb::new("/dev/ttyACM0", 115200, 1)?;
        kwr103.command(Voltage(42.0))?;
        println!("{:?}", kwr103.query::<Output>());
    }

    Ok(())
}
