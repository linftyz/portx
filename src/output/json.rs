use serde::Serialize;

use crate::{
    cli::ScopeArg,
    core::{ListenerRecord, PortDetails},
    error::Result,
};

#[derive(Debug, Serialize)]
struct ListEnvelope<'a> {
    scope: Option<ScopeArgValue>,
    count: usize,
    items: &'a [ListenerRecord],
}

#[derive(Debug, Serialize)]
struct InfoEnvelope<'a> {
    port: u16,
    pid: Option<u32>,
    count: usize,
    items: &'a [PortDetails],
}

#[derive(Debug, Serialize)]
struct FindEnvelope<'a> {
    query: &'a str,
    scope: Option<ScopeArgValue>,
    count: usize,
    items: &'a [ListenerRecord],
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "lowercase")]
enum ScopeArgValue {
    Public,
    Lan,
    Local,
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
        scope: scope.map(Into::into),
        count: records.len(),
        items: records,
    })
}

pub fn print_info(details: &[PortDetails], port: u16, pid: Option<u32>) -> Result<()> {
    print(&InfoEnvelope {
        port,
        pid,
        count: details.len(),
        items: details,
    })
}

pub fn print_find(
    records: &[ListenerRecord],
    process_name: &str,
    scope: Option<ScopeArg>,
) -> Result<()> {
    print(&FindEnvelope {
        query: process_name,
        scope: scope.map(Into::into),
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

#[cfg(test)]
mod tests {
    use std::net::{IpAddr, Ipv4Addr};

    use serde_json::json;

    use crate::core::{ListenerRecord, PortDetails, Protocol, Scope};

    use super::*;

    #[test]
    fn serializes_info_as_object_envelope() {
        let detail = sample_detail();

        let value = serde_json::to_value(InfoEnvelope {
            port: 3000,
            pid: Some(42),
            count: 1,
            items: std::slice::from_ref(&detail),
        })
        .unwrap();

        assert_eq!(value["port"], json!(3000));
        assert_eq!(value["pid"], json!(42));
        assert_eq!(value["count"], json!(1));
        assert!(value["items"].is_array());
        assert_eq!(value["items"][0]["port"], json!(3000));
    }

    #[test]
    fn serializes_find_with_query_metadata() {
        let record = sample_record();

        let value = serde_json::to_value(FindEnvelope {
            query: "node",
            scope: Some(ScopeArgValue::Local),
            count: 1,
            items: std::slice::from_ref(&record),
        })
        .unwrap();

        assert_eq!(value["query"], json!("node"));
        assert_eq!(value["scope"], json!("local"));
        assert_eq!(value["count"], json!(1));
        assert_eq!(value["items"][0]["process_name"], json!("demo"));
    }

    fn sample_record() -> ListenerRecord {
        ListenerRecord {
            port: 3000,
            protocol: Protocol::Tcp,
            bind_addr: IpAddr::V4(Ipv4Addr::LOCALHOST),
            scope: Scope::Local,
            pid: Some(42),
            process_name: Some("demo".to_string()),
            command: Some("demo --serve".to_string()),
        }
    }

    fn sample_detail() -> PortDetails {
        PortDetails {
            listener: sample_record(),
            cwd: None,
            user: Some("alice".to_string()),
            cpu_percent: Some(1.5),
            memory_bytes: Some(1024),
            thread_count: Some(2),
            uptime_seconds: Some(5),
            connection_count: None,
            warnings: Vec::new(),
        }
    }
}
