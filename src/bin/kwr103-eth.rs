use kwr103::commands::*;
use kwr103::eth::Kwr103Eth;

fn main() -> anyhow::Result<()> {
    {
        let kwr103 = Kwr103Eth::new()?;
        kwr103.command(Voltage(42.0))?;
        println!("{:?}", kwr103.query::<Voltage>());
    }

    Ok(())
}
