use std::collections::HashSet;
use std::net::IpAddr;
use std::net::Ipv4Addr;
use std::net::Ipv6Addr;

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

#[derive(Debug, Clone)]
pub struct AddResult {
    pub discovered: Option<DiscoveredAddress>,
    pub raw_ipv4: Option<Ipv4Addr>,
    pub raw_ipv6: Option<Ipv6Addr>,
}

pub struct Aggregator {
    seen: HashSet<DiscoveredAddress>,
    seen_ipv4_raw: HashSet<Ipv4Addr>,
    seen_ipv6_raw: HashSet<Ipv6Addr>,
}

impl Aggregator {
    pub fn new() -> Self {
        Self {
            seen: HashSet::new(),
            seen_ipv4_raw: HashSet::new(),
            seen_ipv6_raw: HashSet::new(),
        }
    }

    pub fn add(&mut self, addr: IpAddr) -> AddResult {
        if should_skip(&addr) {
            return AddResult {
                discovered: None,
                raw_ipv4: None,
                raw_ipv6: None,
            };
        }

        let raw_ipv4 = match addr {
            IpAddr::V4(v4) if self.seen_ipv4_raw.insert(v4) => Some(v4),
            _ => None,
        };

        let raw_ipv6 = match addr {
            IpAddr::V6(v6) if self.seen_ipv6_raw.insert(v6) => Some(v6),
            _ => None,
        };

        let discovered = ip_to_discovered(addr);
        let discovered = if self.seen.insert(discovered.clone()) {
            Some(discovered)
        } else {
            None
        };

        AddResult {
            discovered,
            raw_ipv4,
            raw_ipv6,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::net::IpAddr;

    use super::Aggregator;

    #[test]
    fn test_ipv6_same_64_emits_canonical_once_but_keeps_raw() {
        let mut aggregator = Aggregator::new();

        let first = aggregator.add("2607:6bc0::10".parse::<IpAddr>().unwrap());
        assert_eq!(
            first.discovered.as_ref().map(ToString::to_string),
            Some("2607:6bc0::/64".to_string())
        );
        assert_eq!(
            first.raw_ipv6.as_ref().map(ToString::to_string),
            Some("2607:6bc0::10".to_string())
        );

        let second = aggregator.add("2607:6bc0::11".parse::<IpAddr>().unwrap());
        assert_eq!(second.discovered, None);
        assert_eq!(
            second.raw_ipv6.as_ref().map(ToString::to_string),
            Some("2607:6bc0::11".to_string())
        );

        let duplicate = aggregator.add("2607:6bc0::11".parse::<IpAddr>().unwrap());
        assert_eq!(duplicate.discovered, None);
        assert_eq!(duplicate.raw_ipv4, None);
        assert_eq!(duplicate.raw_ipv6, None);
    }

    #[test]
    fn test_ipv4_same_24_emits_canonical_once_but_keeps_raw() {
        let mut aggregator = Aggregator::new();

        let first = aggregator.add("142.250.80.37".parse::<IpAddr>().unwrap());
        assert_eq!(
            first.discovered.as_ref().map(ToString::to_string),
            Some("142.250.80.0/24".to_string())
        );
        assert_eq!(
            first.raw_ipv4.as_ref().map(ToString::to_string),
            Some("142.250.80.37".to_string())
        );
        assert_eq!(first.raw_ipv6, None);

        let second = aggregator.add("142.250.80.99".parse::<IpAddr>().unwrap());
        assert_eq!(second.discovered, None);
        assert_eq!(
            second.raw_ipv4.as_ref().map(ToString::to_string),
            Some("142.250.80.99".to_string())
        );
        assert_eq!(second.raw_ipv6, None);

        let duplicate = aggregator.add("142.250.80.99".parse::<IpAddr>().unwrap());
        assert_eq!(duplicate.discovered, None);
        assert_eq!(duplicate.raw_ipv4, None);
        assert_eq!(duplicate.raw_ipv6, None);
    }
}
