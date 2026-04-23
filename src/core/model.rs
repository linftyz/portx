use std::{
    fmt,
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
    path::PathBuf,
};

use serde::Serialize;

use super::Scope;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Protocol {
    Tcp,
    Udp,
}

impl fmt::Display for Protocol {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Tcp => formatter.write_str("TCP"),
            Self::Udp => formatter.write_str("UDP"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ListenerRecord {
    pub port: u16,
    pub protocol: Protocol,
    pub bind_addr: IpAddr,
    pub scope: Scope,
    pub pid: Option<u32>,
    pub process_name: Option<String>,
    pub command: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PortDetails {
    #[serde(flatten)]
    pub listener: ListenerRecord,
    pub cwd: Option<PathBuf>,
    pub user: Option<String>,
    pub cpu_percent: Option<f32>,
    pub memory_bytes: Option<u64>,
    pub thread_count: Option<usize>,
    pub uptime_seconds: Option<u64>,
    pub connection_count: Option<usize>,
    pub warnings: Vec<PortWarning>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PortWarning {
    PublicWildcardBind,
    PublicGlobalBind,
}

impl fmt::Display for PortWarning {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PublicWildcardBind => formatter.write_str("public wildcard bind"),
            Self::PublicGlobalBind => formatter.write_str("public global bind"),
        }
    }
}

pub fn warnings_for_listener(listener: &ListenerRecord) -> Vec<PortWarning> {
    match listener.bind_addr {
        IpAddr::V4(Ipv4Addr::UNSPECIFIED) | IpAddr::V6(Ipv6Addr::UNSPECIFIED) => {
            vec![PortWarning::PublicWildcardBind]
        }
        _ if listener.scope == Scope::Public => vec![PortWarning::PublicGlobalBind],
        _ => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flags_unspecified_bind_as_public_wildcard() {
        let listener = ListenerRecord {
            port: 3000,
            protocol: Protocol::Tcp,
            bind_addr: IpAddr::V4(Ipv4Addr::UNSPECIFIED),
            scope: Scope::Public,
            pid: Some(1),
            process_name: None,
            command: None,
        };

        assert_eq!(
            warnings_for_listener(&listener),
            vec![PortWarning::PublicWildcardBind]
        );
    }

    #[test]
    fn flags_public_global_bind_without_wildcard() {
        let listener = ListenerRecord {
            port: 3000,
            protocol: Protocol::Tcp,
            bind_addr: "8.8.8.8".parse().unwrap(),
            scope: Scope::Public,
            pid: Some(1),
            process_name: None,
            command: None,
        };

        assert_eq!(
            warnings_for_listener(&listener),
            vec![PortWarning::PublicGlobalBind]
        );
    }

    #[test]
    fn does_not_flag_local_listener() {
        let listener = ListenerRecord {
            port: 3000,
            protocol: Protocol::Tcp,
            bind_addr: IpAddr::V4(Ipv4Addr::LOCALHOST),
            scope: Scope::Local,
            pid: Some(1),
            process_name: None,
            command: None,
        };

        assert!(warnings_for_listener(&listener).is_empty());
    }
}
