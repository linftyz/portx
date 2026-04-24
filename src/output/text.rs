use std::{
    io::{self, IsTerminal, Write},
    time::SystemTime,
};

use chrono::{DateTime, Local};

use crate::{
    cli::ScopeArg,
    core::{KillPlan, KillResult, ListenerRecord, PortDetails, PortWarning, warnings_for_listener},
    error::{PortxError, Result},
};

const PORT_WIDTH: usize = 6;
const PROTOCOL_WIDTH: usize = 6;
const SCOPE_WIDTH: usize = 7;
const PID_WIDTH: usize = 8;
const ADDRESS_WIDTH: usize = 30;
const PROCESS_WIDTH: usize = 20;
const WARNINGS_WIDTH: usize = 24;
const DETAIL_LABEL_WIDTH: usize = 12;

pub fn print_list(records: &[ListenerRecord], scope: Option<ScopeArg>) {
    if records.is_empty() {
        println!("No listening ports found.");
        return;
    }

    println!("{}", list_summary(records.len(), scope));
    println!();
    print_list_table(records);
}

pub fn print_find(records: &[ListenerRecord], process_name: &str, scope: Option<ScopeArg>) {
    println!("Query: {}", process_name);
    println!("{}", list_summary(records.len(), scope));
    println!();

    if records.is_empty() {
        println!("No listeners matched the query.");
        return;
    }

    print_list_table(records);
}

pub fn print_details(details: &[PortDetails]) {
    if details.is_empty() {
        println!("No details found for this port.");
        return;
    }

    let port = details[0].listener.port;
    println!("Port {port}");
    println!("{}", detail_count_label(details.len()));
    println!();

    for (index, detail) in details.iter().enumerate() {
        if index > 0 {
            println!();
            println!("{}", "=".repeat(80));
            println!();
        }

        println!("{}", listener_heading(index, details.len(), detail));
        println!();
        print_section("Network");
        print_detail_line("Bind", detail.listener.bind_addr.to_string());
        print_detail_line("Scope", detail.listener.scope.to_string());
        print_detail_line("Risk", format_warnings(&detail.warnings));
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
        return Ok(());
    }

    if !is_tty() {
        return Err(PortxError::ConfirmationRequired);
    }

    let mode = if plan.force { "SIGKILL" } else { "SIGTERM" };
    println!("Kill PID {} on port {} with {}?", plan.pid, plan.port, mode);
    println!("Process: {}", plan.process_name.as_deref().unwrap_or("N/A"));
    println!("Command: {}", plan.command.as_deref().unwrap_or("N/A"));
    print!("Continue? [y/N]: ");
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
        Err(PortxError::ConfirmationRequired)
    }
}

pub fn print_kill_result(result: &KillResult) {
    let mode = if result.force { "SIGKILL" } else { "SIGTERM" };
    println!(
        "Sent {} to PID {} on port {} ({})",
        mode,
        result.pid,
        result.port,
        result.process_name.as_deref().unwrap_or("N/A")
    );
}

pub fn print_watch_placeholder(port: u16, pid: Option<u32>) {
    match pid {
        Some(pid) => println!("Watch placeholder: monitoring port {port} for PID {pid}."),
        None => println!("Watch placeholder: monitoring port {port}."),
    }
}

pub fn print_watch_snapshot(port: u16, pid: Option<u32>, details: &[PortDetails]) -> Result<()> {
    clear_screen();
    println!("portx watch");
    println!("{}", "=".repeat(80));
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
    println!("{}", "=".repeat(80));
    println!("Press Ctrl-C to stop.");
    println!();

    if details.is_empty() {
        println!("Port {port} is not currently listening.");
        return Ok(());
    }

    print_details(details);
    Ok(())
}

fn print_list_table(records: &[ListenerRecord]) {
    println!("{}", render_list_header());
    println!("{}", render_list_separator());

    for record in records {
        println!("{}", render_list_row(record));
    }
}

fn render_list_header() -> String {
    render_table_row(&[
        ("PORT", PORT_WIDTH, Alignment::Right),
        ("PROTO", PROTOCOL_WIDTH, Alignment::Left),
        ("SCOPE", SCOPE_WIDTH, Alignment::Left),
        ("PID", PID_WIDTH, Alignment::Right),
        ("ADDRESS", ADDRESS_WIDTH, Alignment::Left),
        ("PROCESS", PROCESS_WIDTH, Alignment::Left),
        ("WARNINGS", WARNINGS_WIDTH, Alignment::Left),
    ])
}

fn render_list_separator() -> String {
    render_table_row(&[
        (&"-".repeat(PORT_WIDTH), PORT_WIDTH, Alignment::Left),
        (&"-".repeat(PROTOCOL_WIDTH), PROTOCOL_WIDTH, Alignment::Left),
        (&"-".repeat(SCOPE_WIDTH), SCOPE_WIDTH, Alignment::Left),
        (&"-".repeat(PID_WIDTH), PID_WIDTH, Alignment::Left),
        (&"-".repeat(ADDRESS_WIDTH), ADDRESS_WIDTH, Alignment::Left),
        (&"-".repeat(PROCESS_WIDTH), PROCESS_WIDTH, Alignment::Left),
        (&"-".repeat(WARNINGS_WIDTH), WARNINGS_WIDTH, Alignment::Left),
    ])
}

fn render_list_row(record: &ListenerRecord) -> String {
    let port = record.port.to_string();
    let protocol = record.protocol.to_string();
    let scope = record.scope.to_string();
    let pid = record
        .pid
        .map_or_else(|| "N/A".to_string(), |pid| pid.to_string());
    let address = record.bind_addr.to_string();
    let process = record.process_name.as_deref().unwrap_or("N/A").to_string();
    let warnings = format_warnings(&warnings_for_listener(record));

    render_table_row(&[
        (&port, PORT_WIDTH, Alignment::Right),
        (&protocol, PROTOCOL_WIDTH, Alignment::Left),
        (&scope, SCOPE_WIDTH, Alignment::Left),
        (&pid, PID_WIDTH, Alignment::Right),
        (&address, ADDRESS_WIDTH, Alignment::Left),
        (&process, PROCESS_WIDTH, Alignment::Left),
        (&warnings, WARNINGS_WIDTH, Alignment::Left),
    ])
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

fn list_summary(count: usize, scope: Option<ScopeArg>) -> String {
    let noun = if count == 1 { "listener" } else { "listeners" };
    match scope {
        Some(scope) => format!("Showing {count} {noun} (scope: {})", scope_label(scope)),
        None => format!("Showing {count} {noun}"),
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

fn print_section(title: &str) {
    println!("{title}");
    println!("{}", "-".repeat(title.len()));
}

fn print_detail_line(label: &str, value: String) {
    println!("{label:>width$} : {value}", width = DETAIL_LABEL_WIDTH);
}

fn watch_field(label: &str, value: String) -> String {
    format!("{label}: {value}")
}

fn format_warnings(warnings: &[PortWarning]) -> String {
    if warnings.is_empty() {
        "-".to_string()
    } else {
        warnings
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(", ")
    }
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
            list_summary(1, Some(ScopeArg::Local)),
            "Showing 1 listener (scope: local)"
        );
        assert_eq!(list_summary(3, None), "Showing 3 listeners");
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
        let record = ListenerRecord {
            port: 3000,
            protocol: crate::core::Protocol::Tcp,
            bind_addr: "127.0.0.1".parse().unwrap(),
            scope: crate::core::Scope::Local,
            pid: Some(4242),
            process_name: Some("node".to_string()),
            command: None,
        };

        assert_eq!(
            render_list_header().chars().count(),
            render_list_row(&record).chars().count()
        );
    }
}
