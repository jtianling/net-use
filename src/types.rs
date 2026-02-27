use std::fmt;
use std::net::{Ipv4Addr, Ipv6Addr};

use thiserror::Error;

#[derive(Debug, Clone)]
pub struct AppInfo {
    pub display_name: String,
    pub bundle_id: Option<String>,
    pub executable_name: String,
    pub app_path: Option<String>,
    pub pid: Option<i32>,
}

#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub pid: i32,
    pub name: String,
    pub command: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MonitorTarget {
    Pid(i32),
    Name(String),
    Bundle(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DiscoveredAddress {
    Ipv4Subnet(Ipv4Addr),
    Ipv6Subnet64(Ipv6Addr),
}

impl fmt::Display for DiscoveredAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DiscoveredAddress::Ipv4Subnet(addr) => {
                let octets = addr.octets();
                write!(f, "{}.{}.{}.0/24", octets[0], octets[1], octets[2])
            }
            DiscoveredAddress::Ipv6Subnet64(addr) => write!(f, "{addr}/64"),
        }
    }
}

impl DiscoveredAddress {
    pub fn from_ipv4(addr: Ipv4Addr) -> Self {
        let octets = addr.octets();
        DiscoveredAddress::Ipv4Subnet(Ipv4Addr::new(octets[0], octets[1], octets[2], 0))
    }

    pub fn from_ipv6(addr: Ipv6Addr) -> Self {
        let segments = addr.segments();
        DiscoveredAddress::Ipv6Subnet64(Ipv6Addr::new(
            segments[0],
            segments[1],
            segments[2],
            segments[3],
            0,
            0,
            0,
            0,
        ))
    }
}

#[derive(Debug, Clone)]
pub enum MonitorEvent {
    NewAddress(DiscoveredAddress),
    NewIpv4Raw(Ipv4Addr),
    NewIpv6Raw(Ipv6Addr),
    ProcessAdded(ProcessInfo),
    ProcessRemoved(i32),
    TargetLost,
    TargetFound,
}

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Process with PID {0} not found")]
    ProcessNotFound(i32),
    #[error("Bundle ID '{0}' not found in /Applications")]
    BundleNotFound(String),
    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

#[cfg(test)]
mod tests {
    use std::net::{Ipv4Addr, Ipv6Addr};

    use super::DiscoveredAddress;

    #[test]
    fn test_ipv4_display_uses_24_mask() {
        let addr = DiscoveredAddress::from_ipv4(Ipv4Addr::new(142, 250, 80, 37));
        assert_eq!(addr.to_string(), "142.250.80.0/24");
    }

    #[test]
    fn test_ipv6_display_uses_64_mask() {
        let addr = DiscoveredAddress::from_ipv6("2607:6bc0::10".parse::<Ipv6Addr>().unwrap());
        assert_eq!(addr.to_string(), "2607:6bc0::/64");
    }
}
