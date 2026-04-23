use std::{
    fmt,
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
};

use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Scope {
    Public,
    Lan,
    Local,
}

impl fmt::Display for Scope {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Public => formatter.write_str("PUBLIC"),
            Self::Lan => formatter.write_str("LAN"),
            Self::Local => formatter.write_str("LOCAL"),
        }
    }
}

impl Scope {
    pub fn classify(addr: IpAddr) -> Self {
        match addr {
            IpAddr::V4(addr) => classify_v4(addr),
            IpAddr::V6(addr) => classify_v6(addr),
        }
    }
}

fn classify_v4(addr: Ipv4Addr) -> Scope {
    if addr.is_loopback() {
        return Scope::Local;
    }

    if addr.is_private() || addr.is_link_local() {
        return Scope::Lan;
    }

    Scope::Public
}

fn classify_v6(addr: Ipv6Addr) -> Scope {
    if addr.is_loopback() {
        return Scope::Local;
    }

    let first_segment = addr.segments()[0];
    let is_unique_local = (first_segment & 0xfe00) == 0xfc00;
    let is_unicast_link_local = (first_segment & 0xffc0) == 0xfe80;

    if is_unique_local || is_unicast_link_local {
        return Scope::Lan;
    }

    Scope::Public
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_loopback_addresses_as_local() {
        assert_eq!(
            Scope::classify(IpAddr::V4(Ipv4Addr::LOCALHOST)),
            Scope::Local
        );
        assert_eq!(
            Scope::classify(IpAddr::V6(Ipv6Addr::LOCALHOST)),
            Scope::Local
        );
    }

    #[test]
    fn classifies_private_and_link_local_addresses_as_lan() {
        assert_eq!(
            Scope::classify(IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1))),
            Scope::Lan
        );
        assert_eq!(
            Scope::classify(IpAddr::V4(Ipv4Addr::new(172, 16, 0, 1))),
            Scope::Lan
        );
        assert_eq!(
            Scope::classify(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1))),
            Scope::Lan
        );
        assert_eq!(
            Scope::classify(IpAddr::V6("fc00::1".parse().unwrap())),
            Scope::Lan
        );
        assert_eq!(
            Scope::classify(IpAddr::V6("fe80::1".parse().unwrap())),
            Scope::Lan
        );
    }

    #[test]
    fn classifies_wildcard_and_global_addresses_as_public() {
        assert_eq!(
            Scope::classify(IpAddr::V4(Ipv4Addr::UNSPECIFIED)),
            Scope::Public
        );
        assert_eq!(
            Scope::classify(IpAddr::V6(Ipv6Addr::UNSPECIFIED)),
            Scope::Public
        );
        assert_eq!(
            Scope::classify(IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8))),
            Scope::Public
        );
    }
}
