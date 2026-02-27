use std::collections::HashSet;
use std::net::IpAddr;

use crate::types::DiscoveredAddress;

pub fn should_skip(addr: &IpAddr) -> bool {
    match addr {
        IpAddr::V4(v4) => {
            v4.is_loopback()
                || v4.is_unspecified()
                || (v4.octets()[0] == 169 && v4.octets()[1] == 254)
        }
        IpAddr::V6(v6) => {
            v6.is_loopback() || v6.is_unspecified() || (v6.segments()[0] & 0xffc0) == 0xfe80
        }
    }
}

fn ip_to_discovered(addr: IpAddr) -> DiscoveredAddress {
    match addr {
        IpAddr::V4(v4) => DiscoveredAddress::from_ipv4(v4),
        IpAddr::V6(v6) => DiscoveredAddress::from_ipv6(v6),
    }
}

pub struct Aggregator {
    seen: HashSet<DiscoveredAddress>,
}

impl Aggregator {
    pub fn new() -> Self {
        Self {
            seen: HashSet::new(),
        }
    }

    pub fn add(&mut self, addr: IpAddr) -> Option<DiscoveredAddress> {
        if should_skip(&addr) {
            return None;
        }
        let discovered = ip_to_discovered(addr);
        if self.seen.insert(discovered.clone()) {
            Some(discovered)
        } else {
            None
        }
    }
}
