use std::{fmt, net::IpAddr, path::PathBuf};

use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
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
