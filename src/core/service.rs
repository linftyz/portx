use crate::{
    core::{KillPlan, KillResult, ListenerRecord, PortDetails, Scope, warnings_for_listener},
    error::{PortxError, Result},
    platform::{
        process::{ProcessInfo, ProcessSnapshot},
        sockets,
    },
};

use sysinfo::{Pid, Signal, System};

#[derive(Debug, Default)]
pub struct PortService;

impl PortService {
    pub fn list(&self, scope: Option<Scope>) -> Result<Vec<ListenerRecord>> {
        let processes = ProcessSnapshot::capture();
        let mut records = collect_listener_records(scope, &processes)?;

        sort_listener_records(&mut records);
        Ok(records)
    }

    pub fn info(&self, port: u16, pid: Option<u32>) -> Result<Vec<PortDetails>> {
        let processes = ProcessSnapshot::capture();
        let mut records =
            filter_records_for_info(collect_listener_records(None, &processes)?, port, pid);

        sort_listener_records(&mut records);

        Ok(records
            .into_iter()
            .map(|record| {
                let process = record.pid.and_then(|pid| processes.get(pid));
                record_to_details(record, process)
            })
            .collect())
    }

    pub fn find(&self, process_name: &str, scope: Option<Scope>) -> Result<Vec<ListenerRecord>> {
        let processes = ProcessSnapshot::capture();
        let mut records = collect_listener_records(scope, &processes)?
            .into_iter()
            .filter(|record| matches_process_name(record, process_name))
            .collect::<Vec<_>>();

        sort_listener_records(&mut records);
        Ok(records)
    }

    pub fn kill(&self, _port: u16, _pid: Option<u32>, _force: bool, _yes: bool) -> Result<()> {
        let processes = ProcessSnapshot::capture();
        let records = collect_listener_records(None, &processes)?;
        let plan = resolve_kill_plan(records, _port, _pid)?;
        let signal = if _force { Signal::Kill } else { Signal::Term };

        let system = System::new_all();
        let process = system
            .process(Pid::from_u32(plan.pid))
            .ok_or(PortxError::KillFailed { pid: plan.pid })?;

        match process.kill_with(signal) {
            Some(true) => Ok(()),
            Some(false) => Err(PortxError::KillFailed { pid: plan.pid }),
            None => Err(PortxError::UnsupportedSignal),
        }
    }

    pub fn watch(&self, _port: u16, _pid: Option<u32>) -> Result<()> {
        Ok(())
    }
}

pub fn build_kill_plan(
    _service: &PortService,
    port: u16,
    pid: Option<u32>,
    force: bool,
) -> Result<KillPlan> {
    let processes = ProcessSnapshot::capture();
    let records = collect_listener_records(None, &processes)?;
    let mut plan = resolve_kill_plan(records, port, pid)?;
    plan.force = force;
    Ok(plan)
}

pub fn execute_kill(service: &PortService, plan: KillPlan) -> Result<KillResult> {
    service.kill(plan.port, Some(plan.pid), plan.force, true)?;
    Ok(KillResult {
        port: plan.port,
        pid: plan.pid,
        process_name: plan.process_name,
        force: plan.force,
    })
}

fn collect_listener_records(
    scope: Option<Scope>,
    processes: &ProcessSnapshot,
) -> Result<Vec<ListenerRecord>> {
    Ok(sockets::listening_sockets()?
        .into_iter()
        .flat_map(|socket| {
            let scope = Scope::classify(socket.bind_addr);

            if socket.pids.is_empty() {
                return vec![ListenerRecord {
                    port: socket.port,
                    protocol: socket.protocol,
                    bind_addr: socket.bind_addr,
                    scope,
                    pid: None,
                    process_name: None,
                    command: None,
                }];
            }

            socket
                .pids
                .into_iter()
                .map(|pid| {
                    let process = processes.get(pid);

                    ListenerRecord {
                        port: socket.port,
                        protocol: socket.protocol,
                        bind_addr: socket.bind_addr,
                        scope,
                        pid: Some(pid),
                        process_name: process.and_then(|process| process.name.clone()),
                        command: process.and_then(|process| process.command.clone()),
                    }
                })
                .collect()
        })
        .filter(|record| scope.is_none_or(|scope| record.scope == scope))
        .collect())
}

fn record_to_details(mut listener: ListenerRecord, process: Option<&ProcessInfo>) -> PortDetails {
    if let Some(process) = process {
        listener.process_name = listener.process_name.or_else(|| process.name.clone());
        listener.command = listener.command.or_else(|| process.command.clone());
    }

    PortDetails {
        warnings: warnings_for_listener(&listener),
        listener,
        cwd: process.and_then(|process| process.cwd.clone()),
        user: process.and_then(|process| process.user.clone()),
        cpu_percent: process.and_then(|process| process.cpu_percent),
        memory_bytes: process.and_then(|process| process.memory_bytes),
        thread_count: process.and_then(|process| process.thread_count),
        uptime_seconds: process.and_then(|process| process.uptime_seconds),
        connection_count: None,
    }
}

fn filter_records_for_info(
    records: Vec<ListenerRecord>,
    port: u16,
    pid: Option<u32>,
) -> Vec<ListenerRecord> {
    records
        .into_iter()
        .filter(|record| record.port == port)
        .filter(|record| pid.is_none_or(|pid| record.pid == Some(pid)))
        .collect()
}

fn matches_process_name(record: &ListenerRecord, needle: &str) -> bool {
    let needle = needle.trim();
    if needle.is_empty() {
        return false;
    }

    let needle = needle.to_lowercase();
    record
        .process_name
        .as_ref()
        .map(|name| name.to_lowercase().contains(&needle))
        .unwrap_or(false)
}

fn resolve_kill_plan(
    records: Vec<ListenerRecord>,
    port: u16,
    pid: Option<u32>,
) -> Result<KillPlan> {
    let matches = records
        .into_iter()
        .filter(|record| record.port == port)
        .collect::<Vec<_>>();

    if matches.is_empty() {
        return Err(PortxError::PortNotFound { port });
    }

    if let Some(pid) = pid {
        let record = matches
            .into_iter()
            .find(|record| record.pid == Some(pid))
            .ok_or(PortxError::PidNotOnPort { port, pid })?;

        return build_plan_from_record(record, port);
    }

    let unique_pids = matches
        .iter()
        .filter_map(|record| record.pid)
        .collect::<std::collections::BTreeSet<_>>();

    match unique_pids.len() {
        0 => Err(PortxError::NoPidForPort { port }),
        1 => {
            let pid = unique_pids.into_iter().next().unwrap();
            let record = matches
                .into_iter()
                .find(|record| record.pid == Some(pid))
                .expect("unique pid should exist in listener records");

            build_plan_from_record(record, port)
        }
        _ => Err(PortxError::MultiplePidsForPort { port }),
    }
}

fn build_plan_from_record(record: ListenerRecord, port: u16) -> Result<KillPlan> {
    let pid = record.pid.ok_or(PortxError::NoPidForPort { port })?;
    Ok(KillPlan {
        port,
        pid,
        process_name: record.process_name,
        command: record.command,
        force: false,
    })
}

fn sort_listener_records(records: &mut [ListenerRecord]) {
    records.sort_by(|left, right| {
        left.port
            .cmp(&right.port)
            .then_with(|| left.bind_addr.cmp(&right.bind_addr))
            .then_with(|| protocol_rank(left.protocol).cmp(&protocol_rank(right.protocol)))
            .then_with(|| left.pid.cmp(&right.pid))
    });
}

fn protocol_rank(protocol: crate::core::Protocol) -> u8 {
    match protocol {
        crate::core::Protocol::Tcp => 0,
        crate::core::Protocol::Udp => 1,
    }
}

#[cfg(test)]
mod tests {
    use std::{
        net::{IpAddr, Ipv4Addr},
        path::PathBuf,
    };

    use crate::{core::Protocol, platform::process::ProcessInfo};

    use super::*;

    #[test]
    fn sorts_listener_records_by_port_address_protocol_and_pid() {
        let mut records = vec![
            record(3000, Ipv4Addr::new(127, 0, 0, 1), Protocol::Udp, Some(2)),
            record(2000, Ipv4Addr::new(127, 0, 0, 1), Protocol::Tcp, Some(1)),
            record(3000, Ipv4Addr::new(0, 0, 0, 0), Protocol::Tcp, None),
            record(3000, Ipv4Addr::new(127, 0, 0, 1), Protocol::Tcp, Some(1)),
        ];

        sort_listener_records(&mut records);

        let ordered = records
            .iter()
            .map(|record| (record.port, record.bind_addr, record.protocol, record.pid))
            .collect::<Vec<_>>();

        assert_eq!(
            ordered,
            vec![
                (
                    2000,
                    IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
                    Protocol::Tcp,
                    Some(1)
                ),
                (
                    3000,
                    IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
                    Protocol::Tcp,
                    None
                ),
                (
                    3000,
                    IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
                    Protocol::Tcp,
                    Some(1)
                ),
                (
                    3000,
                    IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
                    Protocol::Udp,
                    Some(2)
                ),
            ]
        );
    }

    #[test]
    fn filters_info_records_by_port_and_optional_pid() {
        let records = vec![
            record(3000, Ipv4Addr::LOCALHOST, Protocol::Tcp, Some(10)),
            record(3000, Ipv4Addr::LOCALHOST, Protocol::Tcp, Some(20)),
            record(4000, Ipv4Addr::LOCALHOST, Protocol::Tcp, Some(10)),
        ];

        let all_for_port = filter_records_for_info(records.clone(), 3000, None);
        let pid_for_port = filter_records_for_info(records, 3000, Some(20));

        assert_eq!(all_for_port.len(), 2);
        assert_eq!(pid_for_port.len(), 1);
        assert_eq!(pid_for_port[0].pid, Some(20));
    }

    #[test]
    fn builds_details_from_listener_and_process_info() {
        let listener = record(3000, Ipv4Addr::UNSPECIFIED, Protocol::Tcp, Some(10));
        let process = ProcessInfo {
            pid: 10,
            name: Some("demo".to_string()),
            command: Some("demo --serve".to_string()),
            cwd: Some(PathBuf::from("/tmp/demo")),
            user: Some("alice".to_string()),
            cpu_percent: Some(2.5),
            memory_bytes: Some(1024),
            thread_count: Some(4),
            uptime_seconds: Some(30),
        };

        let details = record_to_details(listener, Some(&process));

        assert_eq!(details.listener.process_name.as_deref(), Some("demo"));
        assert_eq!(details.listener.command.as_deref(), Some("demo --serve"));
        assert_eq!(details.cwd, Some(PathBuf::from("/tmp/demo")));
        assert_eq!(details.user.as_deref(), Some("alice"));
        assert_eq!(details.cpu_percent, Some(2.5));
        assert_eq!(details.memory_bytes, Some(1024));
        assert_eq!(details.thread_count, Some(4));
        assert_eq!(details.uptime_seconds, Some(30));
        assert_eq!(
            details.warnings,
            vec![crate::core::PortWarning::PublicWildcardBind]
        );
    }

    #[test]
    fn matches_process_name_case_insensitively() {
        let mut record = record(3000, Ipv4Addr::LOCALHOST, Protocol::Tcp, Some(10));
        record.process_name = Some("Node".to_string());

        assert!(matches_process_name(&record, "node"));
        assert!(matches_process_name(&record, "od"));
        assert!(!matches_process_name(&record, "python"));
        assert!(!matches_process_name(&record, ""));
    }

    #[test]
    fn resolves_single_pid_kill_plan() {
        let plan = resolve_kill_plan(
            vec![record(3000, Ipv4Addr::LOCALHOST, Protocol::Tcp, Some(10))],
            3000,
            None,
        )
        .unwrap();

        assert_eq!(plan.pid, 10);
        assert_eq!(plan.port, 3000);
        assert!(!plan.force);
    }

    #[test]
    fn rejects_multi_pid_kill_without_explicit_pid() {
        let error = resolve_kill_plan(
            vec![
                record(3000, Ipv4Addr::LOCALHOST, Protocol::Tcp, Some(10)),
                record(3000, Ipv4Addr::LOCALHOST, Protocol::Tcp, Some(20)),
            ],
            3000,
            None,
        )
        .unwrap_err();

        assert!(matches!(
            error,
            PortxError::MultiplePidsForPort { port: 3000 }
        ));
    }

    #[test]
    fn resolves_explicit_pid_kill_plan() {
        let plan = resolve_kill_plan(
            vec![
                record(3000, Ipv4Addr::LOCALHOST, Protocol::Tcp, Some(10)),
                record(3000, Ipv4Addr::LOCALHOST, Protocol::Tcp, Some(20)),
            ],
            3000,
            Some(20),
        )
        .unwrap();

        assert_eq!(plan.pid, 20);
    }

    #[test]
    fn rejects_missing_port_for_kill_plan() {
        let error = resolve_kill_plan(Vec::new(), 3000, None).unwrap_err();

        assert!(matches!(error, PortxError::PortNotFound { port: 3000 }));
    }

    #[test]
    fn rejects_pid_not_on_port_for_kill_plan() {
        let error = resolve_kill_plan(
            vec![record(3000, Ipv4Addr::LOCALHOST, Protocol::Tcp, Some(10))],
            3000,
            Some(20),
        )
        .unwrap_err();

        assert!(matches!(
            error,
            PortxError::PidNotOnPort {
                port: 3000,
                pid: 20
            }
        ));
    }

    #[test]
    fn rejects_record_without_pid_for_kill_plan() {
        let error = resolve_kill_plan(
            vec![record(3000, Ipv4Addr::LOCALHOST, Protocol::Tcp, None)],
            3000,
            None,
        )
        .unwrap_err();

        assert!(matches!(error, PortxError::NoPidForPort { port: 3000 }));
    }

    fn record(
        port: u16,
        bind_addr: Ipv4Addr,
        protocol: Protocol,
        pid: Option<u32>,
    ) -> ListenerRecord {
        ListenerRecord {
            port,
            protocol,
            bind_addr: IpAddr::V4(bind_addr),
            scope: Scope::classify(IpAddr::V4(bind_addr)),
            pid,
            process_name: None,
            command: None,
        }
    }
}
