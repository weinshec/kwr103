use kwr103::command::*;
use kwr103::eth::Kwr103Eth;

fn main() -> anyhow::Result<()> {
    {
        let kwr103 = Kwr103Eth::new("192.168.1.198:18190")?;
        kwr103.command(Voltage(42.0))?;
        println!("{:?}", kwr103.query::<Output>());
    }

    Ok(())
}
