use serde::Serialize;

use crate::{
    cli::ScopeArg,
    core::{ListenerRecord, PortDetails},
    error::Result,
};

#[derive(Debug, Serialize)]
struct ListEnvelope<'a> {
    kind: EnvelopeKind,
    scope_filter: Option<ScopeArgValue>,
    count: usize,
    items: &'a [ListenerRecord],
}

#[derive(Debug, Serialize)]
struct InfoEnvelope<'a> {
    kind: EnvelopeKind,
    port: u16,
    pid_filter: Option<u32>,
    count: usize,
    items: Vec<InfoItem<'a>>,
}

#[derive(Debug, Serialize)]
struct FindEnvelope<'a> {
    kind: EnvelopeKind,
    query: &'a str,
    scope_filter: Option<ScopeArgValue>,
    count: usize,
    items: &'a [ListenerRecord],
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
enum EnvelopeKind {
    List,
    Info,
    Find,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "lowercase")]
enum ScopeArgValue {
    Public,
    Lan,
    Local,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
enum ValueStatus {
    Available,
    Unavailable,
    NotApplicable,
    NotImplemented,
}

#[derive(Debug, Serialize)]
struct InfoItem<'a> {
    #[serde(flatten)]
    details: &'a PortDetails,
    thread_count_status: ValueStatus,
    connection_count_status: ValueStatus,
}

impl From<ScopeArg> for ScopeArgValue {
    fn from(value: ScopeArg) -> Self {
        match value {
            ScopeArg::Public => Self::Public,
            ScopeArg::Lan => Self::Lan,
            ScopeArg::Local => Self::Local,
        }
    }
}

pub fn print_list(records: &[ListenerRecord], scope: Option<ScopeArg>) -> Result<()> {
    print(&ListEnvelope {
        kind: EnvelopeKind::List,
        scope_filter: scope.map(Into::into),
        count: records.len(),
        items: records,
    })
}

pub fn print_info(details: &[PortDetails], port: u16, pid: Option<u32>) -> Result<()> {
    print(&InfoEnvelope {
        kind: EnvelopeKind::Info,
        port,
        pid_filter: pid,
        count: details.len(),
        items: details.iter().map(Into::into).collect(),
    })
}

pub fn print_find(
    records: &[ListenerRecord],
    process_name: &str,
    scope: Option<ScopeArg>,
) -> Result<()> {
    print(&FindEnvelope {
        kind: EnvelopeKind::Find,
        query: process_name,
        scope_filter: scope.map(Into::into),
        count: records.len(),
        items: records,
    })
}

fn print<T>(value: &T) -> Result<()>
where
    T: Serialize + ?Sized,
{
    println!("{}", serde_json::to_string_pretty(value)?);
    Ok(())
}

impl<'a> From<&'a PortDetails> for InfoItem<'a> {
    fn from(details: &'a PortDetails) -> Self {
        Self {
            details,
            thread_count_status: if details.thread_count.is_some() {
                ValueStatus::Available
            } else {
                ValueStatus::Unavailable
            },
            connection_count_status: match (details.connection_count, details.listener.protocol) {
                (Some(_), _) => ValueStatus::Available,
                (None, crate::core::Protocol::Udp) => ValueStatus::NotApplicable,
                (None, crate::core::Protocol::Tcp) => ValueStatus::NotImplemented,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use std::net::{IpAddr, Ipv4Addr};

    use serde_json::json;

    use crate::core::{ListenerRecord, PortDetails, PortWarning, Protocol, Scope};

    use super::*;

    #[test]
    fn serializes_info_as_object_envelope_with_explicit_filter_names() {
        let detail = sample_detail(Protocol::Tcp);

        let value = serde_json::to_value(InfoEnvelope {
            kind: EnvelopeKind::Info,
            port: 3000,
            pid_filter: Some(42),
            count: 1,
            items: vec![InfoItem::from(&detail)],
        })
        .unwrap();

        assert_eq!(value["kind"], json!("info"));
        assert_eq!(value["port"], json!(3000));
        assert_eq!(value["pid_filter"], json!(42));
        assert_eq!(value["count"], json!(1));
        assert!(value["items"].is_array());
        assert_eq!(value["items"][0]["port"], json!(3000));
        assert_eq!(value["items"][0]["thread_count_status"], json!("available"));
        assert_eq!(
            value["items"][0]["connection_count_status"],
            json!("not_implemented")
        );
    }

    #[test]
    fn serializes_find_with_query_and_scope_filter_metadata() {
        let record = sample_record(Protocol::Tcp);

        let value = serde_json::to_value(FindEnvelope {
            kind: EnvelopeKind::Find,
            query: "node",
            scope_filter: Some(ScopeArgValue::Local),
            count: 1,
            items: std::slice::from_ref(&record),
        })
        .unwrap();

        assert_eq!(value["kind"], json!("find"));
        assert_eq!(value["query"], json!("node"));
        assert_eq!(value["scope_filter"], json!("local"));
        assert_eq!(value["count"], json!(1));
        assert_eq!(value["items"][0]["process_name"], json!("demo"));
    }

    #[test]
    fn marks_udp_connection_count_as_not_applicable() {
        let detail = sample_detail(Protocol::Udp);

        let value = serde_json::to_value(InfoItem::from(&detail)).unwrap();

        assert_eq!(value["connection_count"], json!(null));
        assert_eq!(value["connection_count_status"], json!("not_applicable"));
        assert_eq!(value["thread_count_status"], json!("available"));
    }

    fn sample_record(protocol: Protocol) -> ListenerRecord {
        ListenerRecord {
            port: 3000,
            protocol,
            bind_addr: IpAddr::V4(Ipv4Addr::LOCALHOST),
            scope: Scope::Local,
            pid: Some(42),
            process_name: Some("demo".to_string()),
            command: Some("demo --serve".to_string()),
        }
    }

    fn sample_detail(protocol: Protocol) -> PortDetails {
        PortDetails {
            listener: sample_record(protocol),
            cwd: None,
            user: Some("alice".to_string()),
            cpu_percent: Some(1.5),
            memory_bytes: Some(1024),
            thread_count: Some(2),
            uptime_seconds: Some(5),
            connection_count: None,
            warnings: vec![PortWarning::PublicWildcardBind],
        }
    }
}
