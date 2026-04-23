//! Process collection boundary for platform-specific implementations.

use std::{
    collections::HashMap,
    ffi::{OsStr, OsString},
    path::{Path, PathBuf},
};

use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System, UpdateKind, Users};

#[derive(Debug, Clone, PartialEq)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: Option<String>,
    pub command: Option<String>,
    pub cwd: Option<PathBuf>,
    pub user: Option<String>,
    pub cpu_percent: Option<f32>,
    pub memory_bytes: Option<u64>,
    pub thread_count: Option<usize>,
    pub uptime_seconds: Option<u64>,
}

#[derive(Debug)]
pub struct ProcessSnapshot {
    processes: HashMap<u32, ProcessInfo>,
}

impl ProcessSnapshot {
    pub fn capture() -> Self {
        let mut system = System::new();
        system.refresh_processes_specifics(
            ProcessesToUpdate::All,
            true,
            ProcessRefreshKind::nothing()
                .with_cmd(UpdateKind::OnlyIfNotSet)
                .with_cpu()
                .with_cwd(UpdateKind::OnlyIfNotSet)
                .with_memory()
                .with_tasks()
                .with_user(UpdateKind::OnlyIfNotSet),
        );

        let users = Users::new_with_refreshed_list();
        let processes = system
            .processes()
            .iter()
            .map(|(pid, process)| {
                let pid = pid.as_u32();
                let user = process
                    .user_id()
                    .and_then(|user_id| users.get_user_by_id(user_id))
                    .map(|user| user.name().to_string());

                (
                    pid,
                    ProcessInfo {
                        pid,
                        name: os_to_string(process.name()),
                        command: command_to_string(process.cmd()),
                        cwd: process.cwd().map(Path::to_path_buf),
                        user,
                        cpu_percent: Some(process.cpu_usage()),
                        memory_bytes: Some(process.memory()),
                        thread_count: process.tasks().map(|tasks| tasks.len()),
                        uptime_seconds: Some(process.run_time()),
                    },
                )
            })
            .collect();

        Self { processes }
    }

    pub fn get(&self, pid: u32) -> Option<&ProcessInfo> {
        self.processes.get(&pid)
    }
}

fn os_to_string(value: &OsStr) -> Option<String> {
    let value = value.to_string_lossy();
    if value.is_empty() {
        None
    } else {
        Some(value.into_owned())
    }
}

fn command_to_string(parts: &[OsString]) -> Option<String> {
    if parts.is_empty() {
        return None;
    }

    let command = parts
        .iter()
        .map(|part| part.to_string_lossy())
        .collect::<Vec<_>>()
        .join(" ");

    if command.is_empty() {
        None
    } else {
        Some(command)
    }
}
