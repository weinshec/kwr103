# KWR103
Remote control Korad KWR103 type programmable DC power supplies via serial/USB or ethernet/UDP.

---
> [!IMPORTANT]
> Development of **this repository has been migrated** to [Codeberg](https://codeberg.org/weinshec/kwr103). Please update your remotes accordingly.
---

## Installation

Choose from the following options

- Download the pre-compiled binary matching your target platform from the
[Releases](https://github.com/weinshec/kwr103/releases) page

- Build the binary from source by cloning this repository and run `cargo build --release`

## Usage

For **library usage**, please refer to the documention at [docs.rs](https://docs.rs/kwr103)

For **command line usage**, please refer to the built-in help
```bash
❯ kwr103 help
Remote control Korad KWR103 programmable DC power supplies

Usage: kwr103 [OPTIONS] <COMMAND>

Commands:
  voltage  Set the output voltage
  current  Set the output current
  output   Turn power supply output 'on' or 'off'
  status   Show current output voltage and current
  info     Show system information
  dhcp     Turn DHCP 'on' or 'off'
  help     Print this message or the help of the given subcommand(s)

Options:
      --device <DEVICE>  Specify device for serial connection [example: /dev/ttyACM0]
      --ip <IP>          Specify IP address for ethernet connection [example: 192.168.1.195]
      --port <PORT>      UDP port for ethernet connected devices [default: 18190]
      --baud <BAUD>      Serial baud rate [default: 115200]
      --id <ID>          Optional RS485 device ID
  -h, --help             Print help
  -V, --version          Print version
```

Example usage:
```bash
❯ kwr103 status
Output: Off, Voltage[V]: 0.000, Current[A]: 0.000

❯ kwr103 output on
❯ kwr103 status
Output: On, Voltage[V]: 42.000, Current[A]: 0.131
```

## Acknowledgments

The development of this crate is heavily inspired by
[Nicoretti/ka3005p](https://github.com/Nicoretti/ka3005p)
