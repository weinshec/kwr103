# KWR103
Remote control Korad KWR103 type programmable DC power supplies via serial/USB or ethernet/UDP.

## Installation

TBD

## Usage

For **library usage**, please refer to the [InsertLink](https://docs.rs)

For **command line usage**, please refer to the built-in help
```bash
❯ kwr103 help
Remote control Korad KWR103 programmable DC power supplies

Usage: kwr103 [OPTIONS] <CONNECTION> <COMMAND>

Commands:
  voltage  Set the output voltage
  current  Set the output current
  output   Turn power supply output 'on' or 'off'
  status   Show current output voltage and current
  info     Show system information
  help     Print this message or the help of the given subcommand(s)

Arguments:
  <CONNECTION>  [possible values: usb, eth]

Options:
      --ip <IP>          ETH: IP Address [default: 192.168.1.198]
      --port <PORT>      ETH: UDP port [default: 18190]
      --device <DEVICE>  USB: serial device [default: /dev/ttyACM0]
      --baud <BAUD>      USB: serial baud rate [default: 115200]
      --id <ID>          USB: RS485 device ID [default: 1]
  -h, --help             Print help
  -V, --version          Print version
```

Example usage:
```bash
❯ kwr103 usb status
Output: Off, Voltage[V]: 0.000, Current[A]: 0.000

❯ kwr103 usb output on
❯ kwr103 usb status
Output: On, Voltage[V]: 42.000, Current[A]: 0.131
```

## Acknowledgments

The development of this crate is heavily inspired by
[Nicoretti/ka3005p](https://github.com/Nicoretti/ka3005p)
