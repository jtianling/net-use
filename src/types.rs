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
    Ipv6Full(Ipv6Addr),
}

impl fmt::Display for DiscoveredAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DiscoveredAddress::Ipv4Subnet(addr) => {
                let octets = addr.octets();
                write!(f, "{}.{}.{}.0/24", octets[0], octets[1], octets[2])
            }
            DiscoveredAddress::Ipv6Full(addr) => write!(f, "{addr}"),
        }
    }
}

impl DiscoveredAddress {
    pub fn from_ipv4(addr: Ipv4Addr) -> Self {
        let octets = addr.octets();
        DiscoveredAddress::Ipv4Subnet(Ipv4Addr::new(octets[0], octets[1], octets[2], 0))
    }

    pub fn from_ipv6(addr: Ipv6Addr) -> Self {
        DiscoveredAddress::Ipv6Full(addr)
    }
}

#[derive(Debug, Clone)]
pub enum MonitorEvent {
    NewAddress(DiscoveredAddress),
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
