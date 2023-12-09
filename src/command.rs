//! Command and query types to interact with the power supply

use std::str::FromStr;
use std::{fmt, net};

use crate::{Command, Query, ResponseError};

/// Representing the state of a switchable feature or output
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Switch {
    /// Disable feature or output
    Off = 0,
    /// Enable feature or output
    On = 1,
}

/// Output voltage setting in units of volts
#[derive(Debug, PartialEq)]
pub struct Voltage(pub f32);

impl Query for Voltage {
    fn serialize(device_id: Option<u8>) -> Vec<u8> {
        match device_id {
            Some(id) => format!("VSET{:02}?\n", id),
            None => String::from("VSET?\n"),
        }
        .into_bytes()
    }

    fn parse(bytes: &[u8]) -> std::result::Result<Self, ResponseError> {
        Ok(Self(parse_single_value(bytes)?))
    }
}

impl Command for Voltage {
    fn serialize(cmd: Self, device_id: Option<u8>) -> Vec<u8> {
        match device_id {
            Some(id) => format!("VSET{:02}:{:.3}\n", id, cmd.0),
            None => format!("VSET:{:.3}\n", cmd.0),
        }
        .into_bytes()
    }
}

/// Output current setting in units of ampere
#[derive(Debug, PartialEq)]
pub struct Current(pub f32);

impl Query for Current {
    fn serialize(device_id: Option<u8>) -> Vec<u8> {
        match device_id {
            Some(id) => format!("ISET{:02}?\n", id),
            None => String::from("ISET?\n"),
        }
        .into_bytes()
    }

    fn parse(bytes: &[u8]) -> std::result::Result<Self, ResponseError> {
        Ok(Self(parse_single_value(bytes)?))
    }
}

impl Command for Current {
    fn serialize(cmd: Self, device_id: Option<u8>) -> Vec<u8> {
        match device_id {
            Some(id) => format!("ISET{:02}:{:.3}\n", id, cmd.0),
            None => format!("ISET:{:.3}\n", cmd.0),
        }
        .into_bytes()
    }
}

/// Output power switch On/Off
#[derive(Debug, PartialEq)]
pub struct Output(pub Switch);

impl Query for Output {
    fn serialize(device_id: Option<u8>) -> Vec<u8> {
        match device_id {
            Some(id) => format!("OUT{:02}?\n", id),
            None => String::from("OUT?\n"),
        }
        .into_bytes()
    }

    fn parse(bytes: &[u8]) -> std::result::Result<Self, ResponseError> {
        Ok(Self(parse_single_value::<Switch>(bytes)?))
    }
}

impl Command for Output {
    fn serialize(cmd: Self, device_id: Option<u8>) -> Vec<u8> {
        match device_id {
            Some(id) => format!("OUT{:02}:{}\n", id, cmd.0 as u8),
            None => format!("OUT:{}\n", cmd.0 as u8),
        }
        .into_bytes()
    }
}

/// Actual output voltage and current state
#[derive(Debug, PartialEq)]
pub struct Status {
    /// Output power state On/Off
    pub power: Switch,
    /// Current output voltage in volts
    pub voltage: f32,
    /// Current output current in ampere
    pub current: f32,
}

impl Query for Status {
    fn serialize(device_id: Option<u8>) -> Vec<u8> {
        match device_id {
            Some(id) => format!("OUT{:02}?\nVOUT{:02}?\nIOUT{:02}?\n", id, id, id),
            None => String::from("OUT?\nVOUT?\nIOUT?\n"),
        }
        .into_bytes()
    }

    fn parse(bytes: &[u8]) -> std::result::Result<Self, ResponseError> {
        let response = String::from_utf8_lossy(bytes);
        let mut tokens = response.split_whitespace();

        Ok(Self {
            power: parse_next_token(&mut tokens)?,
            voltage: parse_next_token(&mut tokens)?,
            current: parse_next_token(&mut tokens)?,
        })
    }
}

/// System settings information
#[derive(Debug, PartialEq)]
pub struct DeviceInfo {
    /// Obtain IP address by DHCP
    pub dhcp: Switch,
    /// IPv4 address
    pub ip: net::Ipv4Addr,
    /// Subnet mask
    pub netmask: net::Ipv4Addr,
    /// Gateway
    pub gateway: net::Ipv4Addr,
    /// MAC address
    pub mac: String,
    /// UDP port,
    pub port: u16,
    /// Serial baud rate
    pub baud: u32,
}

impl Query for DeviceInfo {
    fn serialize(_device_id: Option<u8>) -> Vec<u8> {
        String::from(":SYST:DEVINFO?\n").into_bytes()
    }

    fn parse(bytes: &[u8]) -> std::result::Result<Self, ResponseError> {
        let response = String::from_utf8_lossy(bytes);
        let mut tokens = response.split_whitespace();

        Ok(Self {
            dhcp: strip_and_parse_next_token("DHCP:", &mut tokens)?,
            ip: strip_and_parse_next_token("IP:", &mut tokens)?,
            netmask: strip_and_parse_next_token("NETMASK:", &mut tokens)?,
            gateway: strip_and_parse_next_token("GateWay:", &mut tokens)?,
            mac: strip_and_parse_next_token("MAC:", &mut tokens)?,
            port: strip_and_parse_next_token("PORT:", &mut tokens)?,
            baud: strip_and_parse_next_token("BAUDRATE:", &mut tokens)?,
        })
    }
}

impl std::str::FromStr for Switch {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "0" => Ok(Switch::Off),
            "1" => Ok(Switch::On),
            "off" => Ok(Switch::Off),
            "on" => Ok(Switch::On),
            _ => Err("Invalid value for Switch (must be either 0/1 or on/off)"),
        }
    }
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Output: {:?}, Voltage[V]: {:5.3}, Current[A]: {:5.3}",
            self.power, self.voltage, self.current,
        )
    }
}

impl fmt::Display for DeviceInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "DHCP:       {:?}\n\
             IP Address: {}\n\
             Netmask:    {}\n\
             Gateway:    {}\n\
             MAC:        {}\n\
             UDP Port:   {}\n\
             Baudrate:   {}",
            self.dhcp, self.ip, self.netmask, self.gateway, self.mac, self.port, self.baud
        )
    }
}

fn parse_single_value<T: FromStr>(bytes: &[u8]) -> Result<T, ResponseError> {
    String::from_utf8_lossy(bytes)
        .strip_suffix('\n')
        .ok_or(ResponseError::Incomplete)?
        .parse()
        .map_err(|_| ResponseError::Invalid)
}

fn parse_next_token<'a, I, T>(iter: &mut I) -> Result<T, ResponseError>
where
    I: Iterator<Item = &'a str>,
    T: FromStr,
{
    iter.next()
        .ok_or(ResponseError::Incomplete)?
        .parse()
        .map_err(|_| ResponseError::Invalid)
}

fn strip_and_parse_next_token<'a, I, T>(prefix: &str, iter: &mut I) -> Result<T, ResponseError>
where
    I: Iterator<Item = &'a str>,
    T: FromStr,
{
    iter.next()
        .ok_or(ResponseError::Incomplete)?
        .strip_prefix(prefix)
        .ok_or(ResponseError::Invalid)?
        .parse()
        .map_err(|_| ResponseError::Invalid)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn switch_from_string() {
        assert_eq!(Ok(Switch::Off), "0".parse());
        assert_eq!(Ok(Switch::On), "1".parse());
        assert_eq!(Ok(Switch::Off), "off".parse());
        assert_eq!(Ok(Switch::On), "on".parse());
        assert!("2".parse::<Switch>().is_err());
        assert!("_".parse::<Switch>().is_err());
    }

    #[test]
    fn switch_into_u8() {
        assert_eq!(0u8, Switch::Off as u8);
        assert_eq!(1u8, Switch::On as u8);
    }

    #[test]
    fn query_voltage() {
        assert_eq!(
            <Voltage as Query>::serialize(Some(2)),
            "VSET02?\n".as_bytes()
        );
        assert_eq!(<Voltage as Query>::serialize(None), "VSET?\n".as_bytes());
        assert_eq!(
            <Voltage as Query>::parse("42.123\n".as_bytes()).unwrap(),
            Voltage(42.123)
        );
    }

    #[test]
    fn command_voltage() {
        assert_eq!(
            Command::serialize(Voltage(42.123), Some(2)),
            "VSET02:42.123\n".as_bytes()
        );
        assert_eq!(
            Command::serialize(Voltage(42.123), None),
            "VSET:42.123\n".as_bytes()
        );
    }

    #[test]
    fn query_current() {
        assert_eq!(
            <Current as Query>::serialize(Some(2)),
            "ISET02?\n".as_bytes()
        );
        assert_eq!(<Current as Query>::serialize(None), "ISET?\n".as_bytes());
        assert_eq!(
            <Current as Query>::parse("2.123\n".as_bytes()).unwrap(),
            Current(2.123)
        );
    }

    #[test]
    fn command_current() {
        assert_eq!(
            Command::serialize(Current(2.001), Some(2)),
            "ISET02:2.001\n".as_bytes()
        );
        assert_eq!(
            Command::serialize(Current(2.001), None),
            "ISET:2.001\n".as_bytes()
        );
    }

    #[test]
    fn query_power() {
        assert_eq!(<Output as Query>::serialize(Some(2)), "OUT02?\n".as_bytes());
        assert_eq!(<Output as Query>::serialize(None), "OUT?\n".as_bytes());
        assert_eq!(
            <Output as Query>::parse("1\n".as_bytes()).unwrap(),
            Output(Switch::On)
        );
        assert_eq!(
            <Output as Query>::parse("0\n".as_bytes()).unwrap(),
            Output(Switch::Off)
        );
    }

    #[test]
    fn command_power() {
        assert_eq!(
            Command::serialize(Output(Switch::On), Some(2)),
            "OUT02:1\n".as_bytes()
        );
        assert_eq!(
            Command::serialize(Output(Switch::Off), Some(2)),
            "OUT02:0\n".as_bytes()
        );
        assert_eq!(
            Command::serialize(Output(Switch::On), None),
            "OUT:1\n".as_bytes()
        );
        assert_eq!(
            Command::serialize(Output(Switch::Off), None),
            "OUT:0\n".as_bytes()
        );
    }

    #[test]
    fn query_output() {
        assert_eq!(
            <Status as Query>::serialize(Some(2)),
            "OUT02?\nVOUT02?\nIOUT02?\n".as_bytes()
        );
        assert_eq!(
            <Status as Query>::serialize(None),
            "OUT?\nVOUT?\nIOUT?\n".as_bytes()
        );
        assert_eq!(
            <Status as Query>::parse("1\n1.234\n5.678\n".as_bytes()).unwrap(),
            Status {
                power: Switch::On,
                voltage: 1.234,
                current: 5.678,
            }
        );
    }

    #[test]
    fn query_deviceinfo() {
        assert_eq!(
            <DeviceInfo as Query>::serialize(Some(2)),
            ":SYST:DEVINFO?\n".as_bytes()
        );
        assert_eq!(
            <DeviceInfo as Query>::serialize(None),
            ":SYST:DEVINFO?\n".as_bytes()
        );

        let response = "DHCP:0\nIP:192.168.1.198\nNETMASK:255.255.255.0\nGateWay:192.168.1.1\nMAC:88-06-00-00-ff-ff\nPORT:18190\nBAUDRATE:115200\n".as_bytes();
        assert_eq!(
            <DeviceInfo as Query>::parse(response).unwrap(),
            DeviceInfo {
                dhcp: Switch::Off,
                ip: net::Ipv4Addr::new(192, 168, 1, 198),
                netmask: net::Ipv4Addr::new(255, 255, 255, 0),
                gateway: net::Ipv4Addr::new(192, 168, 1, 1),
                mac: String::from("88-06-00-00-ff-ff"),
                port: 18190,
                baud: 115200,
            }
        );
    }
}
