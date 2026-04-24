use std::{
    collections::BTreeMap,
    io::{self, IsTerminal, Write},
    net::IpAddr,
    time::SystemTime,
};

use chrono::{DateTime, Local};

use crate::{
    cli::ScopeArg,
    core::{KillPlan, KillResult, ListenerRecord, PortDetails, PortWarning, warnings_for_listener},
    error::{PortxError, Result},
};

use super::style;

const PORT_WIDTH: usize = 6;
const PROTOCOL_WIDTH: usize = 6;
const SCOPE_WIDTH: usize = 7;
const PID_WIDTH: usize = 8;
const ADDRESS_WIDTH: usize = 30;
const PROCESS_WIDTH: usize = 20;
const RISK_WIDTH: usize = 24;
const DETAIL_LABEL_WIDTH: usize = 12;

pub fn print_list(records: &[ListenerRecord], scope: Option<ScopeArg>) {
    let rows = group_listener_rows(records);

    if rows.is_empty() {
        println!("No listening ports found.");
        return;
    }

    println!(
        "{}",
        style::accent(&list_summary(rows.len(), records.len(), scope))
    );
    println!("{}", style::muted(&scope_breakdown(&rows)));
    println!();
    print_list_table(&rows);
}

pub fn print_find(records: &[ListenerRecord], process_name: &str, scope: Option<ScopeArg>) {
    let rows = group_listener_rows(records);

    println!(
        "{} {}",
        style::muted("Query:"),
        style::highlight(process_name)
    );
    println!(
        "{}",
        style::accent(&list_summary(rows.len(), records.len(), scope))
    );
    println!("{}", style::muted(&scope_breakdown(&rows)));
    println!();

    if rows.is_empty() {
        println!("No listeners matched the query.");
        return;
    }

    print_list_table(&rows);
}

pub fn print_details(details: &[PortDetails]) {
    if details.is_empty() {
        println!("No details found for this port.");
        return;
    }

    let port = details[0].listener.port;
    println!(
        "{} {}",
        style::accent("Port"),
        style::highlight(&port.to_string())
    );
    println!("{}", style::muted(&detail_count_label(details.len())));
    println!();

    for (index, detail) in details.iter().enumerate() {
        if index > 0 {
            println!();
            println!("{}", style::muted(&"=".repeat(80)));
            println!();
        }

        println!(
            "{}",
            style::accent(&listener_heading(index, details.len(), detail))
        );
        println!();
        print_section("Network");
        print_detail_line("Bind", detail.listener.bind_addr.to_string());
        print_detail_line(
            "Scope",
            style::scope_value(&detail.listener.scope.to_string()),
        );
        print_detail_line("Risk", style::warning_value(&detail.warnings));
        print_detail_line(
            "Connections",
            detail
                .connection_count
                .map_or_else(|| "N/A".to_string(), |connections| connections.to_string()),
        );
        println!();

        print_section("Process");
        print_detail_line(
            "Process",
            process_summary(
                detail.listener.process_name.as_deref().unwrap_or("N/A"),
                detail.listener.pid,
            ),
        );
        print_detail_line(
            "Command",
            detail
                .listener
                .command
                .as_deref()
                .unwrap_or("N/A")
                .to_string(),
        );
        print_detail_line(
            "CWD",
            detail
                .cwd
                .as_ref()
                .map_or_else(|| "N/A".to_string(), |cwd| cwd.display().to_string()),
        );
        print_detail_line("User", detail.user.as_deref().unwrap_or("N/A").to_string());
        println!();

        print_section("Resources");
        print_detail_line(
            "CPU",
            detail
                .cpu_percent
                .map_or_else(|| "N/A".to_string(), |cpu| format!("{cpu:.1}%")),
        );
        print_detail_line(
            "Memory",
            detail
                .memory_bytes
                .map_or_else(|| "N/A".to_string(), format_bytes),
        );
        print_detail_line(
            "Threads",
            detail
                .thread_count
                .map_or_else(|| "N/A".to_string(), |threads| threads.to_string()),
        );
        print_detail_line(
            "Uptime",
            detail
                .uptime_seconds
                .map_or_else(|| "N/A".to_string(), format_duration),
        );
    }
}

pub fn confirm_kill(plan: &KillPlan, skip_confirmation: bool) -> Result<()> {
    if skip_confirmation {
        print_kill_target(plan, false);
        return Ok(());
    }

    if !is_tty() {
        return Err(PortxError::ConfirmationRequired);
    }

    println!("{}", style::accent("Kill confirmation"));
    println!("{}", style::muted(&"=".repeat(80)));
    print_kill_target(plan, true);
    println!("{}", style::muted(&"=".repeat(80)));
    print!("{} ", style::warning("Continue? [y/N]:"));
    io::stdout()
        .flush()
        .map_err(|_| PortxError::ConfirmationRequired)?;

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .map_err(|_| PortxError::ConfirmationRequired)?;

    let answer = input.trim().to_ascii_lowercase();
    if answer == "y" || answer == "yes" {
        Ok(())
    } else {
        Err(PortxError::OperationCancelled)
    }
}

pub fn print_kill_result(result: &KillResult) {
    let mode = if result.force { "SIGKILL" } else { "SIGTERM" };
    println!("{}", style::success("Kill request sent"));
    println!("{}", style::muted(&"-".repeat(80)));
    print_detail_line("Port", style::highlight(&result.port.to_string()));
    print_detail_line("PID", style::highlight(&result.pid.to_string()));
    print_detail_line("Signal", style::warning(mode));
    print_detail_line(
        "Process",
        result.process_name.as_deref().unwrap_or("N/A").to_string(),
    );
    println!("{}", style::muted(&"-".repeat(80)));
    println!(
        "{}",
        style::muted("The process may take a moment to exit after receiving the signal.")
    );
}

pub fn print_error(error: &PortxError) {
    match error {
        PortxError::Cli(cli_error) => {
            eprint!("{cli_error}");
        }
        _ => {
            eprintln!("{}", style::danger("Error"));
            eprintln!("{}", style::muted(&"-".repeat(80)));
            eprintln!("{}", error_summary(error));

            if let Some(hint) = error_hint(error) {
                eprintln!();
                eprintln!("{}", style::muted("Try this:"));
                eprintln!("{hint}");
            }
        }
    }
}

pub fn print_watch_placeholder(port: u16, pid: Option<u32>) {
    match pid {
        Some(pid) => println!("Watch placeholder: monitoring port {port} for PID {pid}."),
        None => println!("Watch placeholder: monitoring port {port}."),
    }
}

pub fn print_watch_snapshot(port: u16, pid: Option<u32>, details: &[PortDetails]) -> Result<()> {
    clear_screen();
    println!("{}", style::accent("portx watch"));
    println!("{}", style::muted(&"=".repeat(80)));
    println!(
        "{}  {}  {}",
        watch_field("Port", port.to_string()),
        watch_field(
            "PID filter",
            pid.map_or_else(|| "-".to_string(), |pid| pid.to_string())
        ),
        watch_field("Listeners", details.len().to_string())
    );
    println!(
        "{}  {}",
        watch_field("Updated", local_timestamp_string()),
        watch_field(
            "Status",
            if details.is_empty() {
                "NOT LISTENING".to_string()
            } else {
                "LISTENING".to_string()
            }
        )
    );
    println!("{}", style::muted(&"=".repeat(80)));
    println!("{}", style::muted("Press Ctrl-C to stop."));
    println!();

    if details.is_empty() {
        println!(
            "{} {} {}",
            style::warning("Port"),
            style::highlight(&port.to_string()),
            style::warning("is not currently listening.")
        );
        return Ok(());
    }

    println!("{}", style::accent(&watch_summary(details.len())));
    println!();

    for (index, detail) in details.iter().enumerate() {
        if index > 0 {
            println!("{}", style::muted(&"-".repeat(80)));
        }

        print_watch_listener(index, detail);
    }

    Ok(())
}

fn print_list_table(rows: &[GroupedListenerRow]) {
    println!("{}", render_list_header());
    println!("{}", render_list_separator());

    for row in rows {
        println!("{}", render_list_row(row));
    }
}

fn render_list_header() -> String {
    style::accent(&render_table_row(&[
        ("PORT", PORT_WIDTH, Alignment::Right),
        ("PROTO", PROTOCOL_WIDTH, Alignment::Left),
        ("SCOPE", SCOPE_WIDTH, Alignment::Left),
        ("PID", PID_WIDTH, Alignment::Right),
        ("ADDRESS", ADDRESS_WIDTH, Alignment::Left),
        ("PROCESS", PROCESS_WIDTH, Alignment::Left),
        ("RISK", RISK_WIDTH, Alignment::Left),
    ]))
}

fn render_list_separator() -> String {
    style::muted(&render_table_row(&[
        (&"-".repeat(PORT_WIDTH), PORT_WIDTH, Alignment::Left),
        (&"-".repeat(PROTOCOL_WIDTH), PROTOCOL_WIDTH, Alignment::Left),
        (&"-".repeat(SCOPE_WIDTH), SCOPE_WIDTH, Alignment::Left),
        (&"-".repeat(PID_WIDTH), PID_WIDTH, Alignment::Left),
        (&"-".repeat(ADDRESS_WIDTH), ADDRESS_WIDTH, Alignment::Left),
        (&"-".repeat(PROCESS_WIDTH), PROCESS_WIDTH, Alignment::Left),
        (&"-".repeat(RISK_WIDTH), RISK_WIDTH, Alignment::Left),
    ]))
}

fn render_list_row(row: &GroupedListenerRow) -> String {
    let port = row.port.to_string();
    let protocol = row.protocol.to_string();
    let scope = row.scope.to_string();
    let pid = row
        .pid
        .map_or_else(|| "N/A".to_string(), |pid| pid.to_string());
    let address = row.address_summary();
    let process = row.process_name.as_deref().unwrap_or("N/A").to_string();
    let risk = format_warnings(&row.warnings);

    [
        format_cell(&port, PORT_WIDTH, Alignment::Right),
        format_cell(&protocol, PROTOCOL_WIDTH, Alignment::Left),
        style::table_scope_cell(&scope, SCOPE_WIDTH),
        format_cell(&pid, PID_WIDTH, Alignment::Right),
        format_cell(&address, ADDRESS_WIDTH, Alignment::Left),
        format_cell(&process, PROCESS_WIDTH, Alignment::Left),
        style::table_warning_cell(&risk, RISK_WIDTH),
    ]
    .join(" | ")
}

fn render_table_row(columns: &[(&str, usize, Alignment)]) -> String {
    columns
        .iter()
        .map(|(value, width, alignment)| format_cell(value, *width, *alignment))
        .collect::<Vec<_>>()
        .join(" | ")
}

fn format_cell(value: &str, width: usize, alignment: Alignment) -> String {
    let truncated = truncate(value, width);
    let padding = width.saturating_sub(truncated.chars().count());

    match alignment {
        Alignment::Left => format!("{truncated}{:padding$}", "", padding = padding),
        Alignment::Right => format!("{:padding$}{truncated}", "", padding = padding),
    }
}

#[derive(Clone, Copy)]
enum Alignment {
    Left,
    Right,
}

fn list_summary(displayed_count: usize, raw_count: usize, scope: Option<ScopeArg>) -> String {
    let noun = if displayed_count == 1 {
        "listener"
    } else {
        "listeners"
    };

    let summary = if displayed_count == raw_count {
        format!("Showing {displayed_count} {noun}")
    } else {
        let socket_noun = if raw_count == 1 { "socket" } else { "sockets" };
        format!("Showing {displayed_count} listener groups (from {raw_count} {socket_noun})")
    };

    match scope {
        Some(scope) => format!("{summary} (scope: {})", scope_label(scope)),
        None => summary,
    }
}

fn detail_count_label(count: usize) -> String {
    if count == 1 {
        "1 listener".to_string()
    } else {
        format!("{count} listeners")
    }
}

fn listener_heading(index: usize, total: usize, detail: &PortDetails) -> String {
    let prefix = if total > 1 {
        format!("[Listener {}] ", index + 1)
    } else {
        String::new()
    };

    format!(
        "{}{} / {} / {}",
        prefix, detail.listener.bind_addr, detail.listener.protocol, detail.listener.scope
    )
}

fn process_summary(process_name: &str, pid: Option<u32>) -> String {
    match pid {
        Some(pid) => format!("{process_name} (PID {pid})"),
        None => process_name.to_string(),
    }
}

fn watch_summary(count: usize) -> String {
    if count == 1 {
        "Monitoring 1 listener".to_string()
    } else {
        format!("Monitoring {count} listeners")
    }
}

fn print_section(title: &str) {
    println!("{}", style::accent(title));
    println!("{}", style::muted(&"-".repeat(title.len())));
}

fn print_detail_line(label: &str, value: String) {
    println!(
        "{} : {}",
        style::muted(&format!("{label:>width$}", width = DETAIL_LABEL_WIDTH)),
        value
    );
}

fn print_kill_target(plan: &KillPlan, include_hint: bool) {
    let mode = if plan.force { "SIGKILL" } else { "SIGTERM" };

    print_detail_line("Port", style::highlight(&plan.port.to_string()));
    print_detail_line("PID", style::highlight(&plan.pid.to_string()));
    print_detail_line("Signal", style::warning(mode));
    print_detail_line(
        "Process",
        plan.process_name.as_deref().unwrap_or("N/A").to_string(),
    );
    print_detail_line(
        "Command",
        plan.command.as_deref().unwrap_or("N/A").to_string(),
    );

    if include_hint {
        let hint = if plan.force {
            "Force mode will terminate the process immediately."
        } else {
            "Graceful mode asks the process to exit cleanly first."
        };
        println!();
        println!("{}", style::muted(hint));
    }
}

fn error_summary(error: &PortxError) -> String {
    match error {
        PortxError::NoPidForPort { port } => {
            format!(
                "Port {} is listening, but no killable PID was found.",
                style::highlight(&port.to_string())
            )
        }
        PortxError::PortNotFound { port } => {
            format!(
                "Port {} is not currently listening.",
                style::highlight(&port.to_string())
            )
        }
        PortxError::MultiplePidsForPort { port } => format!(
            "Port {} has multiple owning processes.",
            style::highlight(&port.to_string())
        ),
        PortxError::PidNotOnPort { port, pid } => format!(
            "PID {} is not listening on port {}.",
            style::highlight(&pid.to_string()),
            style::highlight(&port.to_string())
        ),
        PortxError::ConfirmationRequired => {
            "Kill requires confirmation in a non-interactive terminal.".to_string()
        }
        PortxError::OperationCancelled => "Kill was cancelled.".to_string(),
        PortxError::UnsupportedSignal => {
            "This platform does not support the requested kill signal.".to_string()
        }
        PortxError::TuiRequiresTerminal => {
            "The TUI can only run in an interactive terminal.".to_string()
        }
        PortxError::KillFailed { pid } => format!(
            "Failed to send the signal to PID {}.",
            style::highlight(&pid.to_string())
        ),
        PortxError::Socket(_) => {
            "Failed to collect socket information from the system.".to_string()
        }
        PortxError::Io(error) => format!("I/O error: {error}"),
        PortxError::Json(error) => format!("Failed to render JSON output: {error}"),
        PortxError::NotImplemented => "This feature is not implemented yet.".to_string(),
        PortxError::Cli(error) => error.to_string(),
    }
}

fn error_hint(error: &PortxError) -> Option<String> {
    match error {
        PortxError::NoPidForPort { port } => Some(format!(
            "Run {} to inspect the listener details.",
            style::highlight(&format!("portx info {port}"))
        )),
        PortxError::PortNotFound { port: _ } => Some(format!(
            "Run {} to list active listeners and confirm the port number.",
            style::highlight("portx list")
        )),
        PortxError::MultiplePidsForPort { port } => Some(format!(
            "Run {} and then retry with {}.",
            style::highlight(&format!("portx info {port}")),
            style::highlight(&format!("portx kill {port} --pid <pid>"))
        )),
        PortxError::PidNotOnPort { port, .. } => Some(format!(
            "Run {} to see which PIDs currently own the port.",
            style::highlight(&format!("portx info {port}"))
        )),
        PortxError::ConfirmationRequired => Some(format!(
            "Pass {} only when you intentionally want to skip the prompt.",
            style::highlight("--yes")
        )),
        PortxError::OperationCancelled => Some(format!(
            "Re-run {} if you want to try again.",
            style::highlight("portx kill <port>")
        )),
        PortxError::UnsupportedSignal => None,
        PortxError::TuiRequiresTerminal => Some(format!(
            "Run {} in a local terminal session instead.",
            style::highlight("portx tui")
        )),
        PortxError::KillFailed { pid } => Some(format!(
            "Check permissions or try {} if the process ignores SIGTERM.",
            style::highlight(&format!("portx kill <port> --pid {pid} --force"))
        )),
        PortxError::Socket(_) => Some(
            "Try running the command with elevated privileges if the system restricts socket inspection."
                .to_string(),
        ),
        PortxError::Io(_) => None,
        PortxError::Json(_) => None,
        PortxError::NotImplemented => None,
        PortxError::Cli(_) => None,
    }
}

fn watch_field(label: &str, value: String) -> String {
    format!("{} {}", style::muted(&format!("{label}:")), value)
}

fn print_watch_listener(index: usize, detail: &PortDetails) {
    println!(
        "{} {} / {} / {}",
        style::accent(&format!("[Listener {}]", index + 1)),
        detail.listener.bind_addr,
        detail.listener.protocol,
        style::scope_value(&detail.listener.scope.to_string())
    );
    println!(
        "{}",
        watch_line(&[
            (
                "Process",
                process_summary(
                    detail.listener.process_name.as_deref().unwrap_or("N/A"),
                    detail.listener.pid,
                ),
            ),
            ("User", detail.user.as_deref().unwrap_or("N/A").to_string()),
        ])
    );
    println!(
        "{}",
        watch_line(&[
            ("Risk", style::warning_value(&detail.warnings)),
            (
                "Command",
                detail
                    .listener
                    .command
                    .as_deref()
                    .map_or_else(|| "N/A".to_string(), |command| truncate(command, 52)),
            ),
        ])
    );
    println!(
        "{}",
        watch_line(&[
            (
                "CPU",
                detail
                    .cpu_percent
                    .map_or_else(|| "N/A".to_string(), |cpu| format!("{cpu:.1}%")),
            ),
            (
                "Memory",
                detail
                    .memory_bytes
                    .map_or_else(|| "N/A".to_string(), format_bytes),
            ),
            (
                "Threads",
                detail
                    .thread_count
                    .map_or_else(|| "N/A".to_string(), |threads| threads.to_string()),
            ),
            (
                "Connections",
                detail
                    .connection_count
                    .map_or_else(|| "N/A".to_string(), |connections| connections.to_string()),
            ),
            (
                "Uptime",
                detail
                    .uptime_seconds
                    .map_or_else(|| "N/A".to_string(), format_duration),
            ),
        ])
    );
}

fn watch_line(fields: &[(&str, String)]) -> String {
    fields
        .iter()
        .map(|(label, value)| format!("{} {}", style::muted(&format!("{label}:")), value))
        .collect::<Vec<_>>()
        .join("  |  ")
}

fn format_warnings(warnings: &[PortWarning]) -> String {
    style::warning_text(warnings)
}

fn group_listener_rows(records: &[ListenerRecord]) -> Vec<GroupedListenerRow> {
    let mut groups = BTreeMap::<GroupKey, GroupedListenerRow>::new();

    for record in records {
        let warnings = warnings_for_listener(record);
        let key = GroupKey {
            port: record.port,
            protocol: record.protocol,
            scope_rank: scope_rank(record.scope),
            pid: record.pid,
            process_name: record.process_name.clone(),
            warning_key: warning_sort_key(&warnings),
        };

        groups
            .entry(key)
            .and_modify(|row| row.addresses.push(record.bind_addr))
            .or_insert_with(|| GroupedListenerRow {
                port: record.port,
                protocol: record.protocol,
                scope: record.scope,
                pid: record.pid,
                process_name: record.process_name.clone(),
                addresses: vec![record.bind_addr],
                warnings,
            });
    }

    let mut rows = groups.into_values().collect::<Vec<_>>();
    rows.sort_by(|left, right| {
        left.port
            .cmp(&right.port)
            .then_with(|| protocol_rank(left.protocol).cmp(&protocol_rank(right.protocol)))
            .then_with(|| left.pid.cmp(&right.pid))
            .then_with(|| scope_rank(left.scope).cmp(&scope_rank(right.scope)))
            .then_with(|| left.addresses.cmp(&right.addresses))
    });

    rows
}

fn scope_breakdown(rows: &[GroupedListenerRow]) -> String {
    let (mut public, mut lan, mut local) = (0usize, 0usize, 0usize);

    for row in rows {
        match row.scope {
            crate::core::Scope::Public => public += 1,
            crate::core::Scope::Lan => lan += 1,
            crate::core::Scope::Local => local += 1,
        }
    }

    format!("Public: {public}  Lan: {lan}  Local: {local}")
}

fn protocol_rank(protocol: crate::core::Protocol) -> u8 {
    match protocol {
        crate::core::Protocol::Tcp => 0,
        crate::core::Protocol::Udp => 1,
    }
}

fn scope_rank(scope: crate::core::Scope) -> u8 {
    match scope {
        crate::core::Scope::Public => 0,
        crate::core::Scope::Lan => 1,
        crate::core::Scope::Local => 2,
    }
}

fn warning_sort_key(warnings: &[PortWarning]) -> String {
    warnings
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("|")
}

fn scope_label(scope: ScopeArg) -> &'static str {
    match scope {
        ScopeArg::Public => "public",
        ScopeArg::Lan => "lan",
        ScopeArg::Local => "local",
    }
}

fn is_tty() -> bool {
    std::io::stdin().is_terminal() && std::io::stdout().is_terminal()
}

fn clear_screen() {
    print!("\x1B[2J\x1B[H");
}

fn local_timestamp_string() -> String {
    let now = SystemTime::now();
    let datetime: DateTime<Local> = now.into();
    datetime.format("%Y-%m-%d %H:%M:%S").to_string()
}

fn truncate(value: &str, width: usize) -> String {
    let count = value.chars().count();
    if count <= width {
        return value.to_string();
    }

    if width == 0 {
        return String::new();
    }

    if width <= 3 {
        return ".".repeat(width);
    }

    let mut truncated = value.chars().take(width - 3).collect::<String>();
    truncated.push_str("...");
    truncated
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct GroupKey {
    port: u16,
    protocol: crate::core::Protocol,
    scope_rank: u8,
    pid: Option<u32>,
    process_name: Option<String>,
    warning_key: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct GroupedListenerRow {
    port: u16,
    protocol: crate::core::Protocol,
    scope: crate::core::Scope,
    pid: Option<u32>,
    process_name: Option<String>,
    addresses: Vec<IpAddr>,
    warnings: Vec<PortWarning>,
}

impl GroupedListenerRow {
    fn address_summary(&self) -> String {
        self.addresses
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(", ")
    }
}

fn format_bytes(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KiB", "MiB", "GiB", "TiB"];

    let mut value = bytes as f64;
    let mut unit = 0usize;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }

    if unit == 0 {
        format!("{bytes} {}", UNITS[unit])
    } else {
        format!("{value:.1} {}", UNITS[unit])
    }
}

fn format_duration(seconds: u64) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let seconds = seconds % 60;

    if hours > 0 {
        format!("{hours}h {minutes}m {seconds}s")
    } else if minutes > 0 {
        format!("{minutes}m {seconds}s")
    } else {
        format!("{seconds}s")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::KillPlan;
    use crate::error::PortxError;

    #[test]
    fn truncates_long_values_with_ellipsis() {
        assert_eq!(truncate("abcdefghijkl", 8), "abcde...");
        assert_eq!(truncate("short", 8), "short");
    }

    #[test]
    fn formats_bytes_readably() {
        assert_eq!(format_bytes(512), "512 B");
        assert_eq!(format_bytes(1536), "1.5 KiB");
        assert_eq!(format_bytes(5 * 1024 * 1024), "5.0 MiB");
    }

    #[test]
    fn formats_duration_readably() {
        assert_eq!(format_duration(42), "42s");
        assert_eq!(format_duration(125), "2m 5s");
        assert_eq!(format_duration(3725), "1h 2m 5s");
    }

    #[test]
    fn builds_list_summary_with_scope() {
        assert_eq!(
            list_summary(1, 1, Some(ScopeArg::Local)),
            "Showing 1 listener (scope: local)"
        );
        assert_eq!(list_summary(3, 3, None), "Showing 3 listeners");
    }

    #[test]
    fn builds_process_summary_with_pid() {
        assert_eq!(
            process_summary("postgres", Some(1148)),
            "postgres (PID 1148)"
        );
        assert_eq!(process_summary("unknown", None), "unknown");
    }

    #[test]
    fn renders_header_and_rows_with_matching_width() {
        let row = GroupedListenerRow {
            port: 3000,
            protocol: crate::core::Protocol::Tcp,
            scope: crate::core::Scope::Local,
            pid: Some(4242),
            process_name: Some("node".to_string()),
            addresses: vec!["127.0.0.1".parse().unwrap()],
            warnings: Vec::new(),
        };

        assert_eq!(
            render_list_header().chars().count(),
            render_list_row(&row).chars().count()
        );
    }

    #[test]
    fn groups_ipv4_and_ipv6_listener_variants_together() {
        let rows = group_listener_rows(&[
            ListenerRecord {
                port: 5432,
                protocol: crate::core::Protocol::Tcp,
                bind_addr: "127.0.0.1".parse().unwrap(),
                scope: crate::core::Scope::Local,
                pid: Some(1148),
                process_name: Some("postgres".to_string()),
                command: None,
            },
            ListenerRecord {
                port: 5432,
                protocol: crate::core::Protocol::Tcp,
                bind_addr: "::1".parse().unwrap(),
                scope: crate::core::Scope::Local,
                pid: Some(1148),
                process_name: Some("postgres".to_string()),
                command: None,
            },
        ]);

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].address_summary(), "127.0.0.1, ::1");
    }

    #[test]
    fn includes_grouped_summary_when_socket_count_shrinks() {
        assert_eq!(
            list_summary(1, 2, None),
            "Showing 1 listener groups (from 2 sockets)"
        );
        assert_eq!(
            list_summary(1, 1, Some(ScopeArg::Local)),
            "Showing 1 listener (scope: local)"
        );
    }

    #[test]
    fn skip_confirmation_returns_ok() {
        let plan = KillPlan {
            port: 5432,
            pid: 1148,
            process_name: Some("postgres".to_string()),
            command: Some("postgres -D /tmp/data".to_string()),
            force: false,
        };

        assert!(confirm_kill(&plan, true).is_ok());
    }

    #[test]
    fn operation_cancelled_has_non_zero_exit_code() {
        assert_eq!(PortxError::OperationCancelled.exit_code(), 1);
    }

    #[test]
    fn builds_helpful_summary_for_missing_port() {
        let summary = error_summary(&PortxError::PortNotFound { port: 3000 });
        assert!(summary.contains("3000"));
        assert!(summary.contains("not currently listening"));
    }

    #[test]
    fn builds_hint_for_multi_pid_kill() {
        let hint = error_hint(&PortxError::MultiplePidsForPort { port: 3000 }).unwrap();
        assert!(hint.contains("portx info 3000"));
        assert!(hint.contains("portx kill 3000 --pid <pid>"));
    }
}
