use std::{
    io::{self, IsTerminal, Write},
    time::SystemTime,
};

use chrono::{DateTime, Local};

use crate::{
    core::{KillPlan, KillResult, ListenerRecord, PortDetails, PortWarning, warnings_for_listener},
    error::{PortxError, Result},
};

const ADDRESS_WIDTH: usize = 30;
const PROCESS_WIDTH: usize = 20;
const WARNINGS_WIDTH: usize = 24;

pub fn print_list(records: &[ListenerRecord]) {
    if records.is_empty() {
        println!("No listening ports found.");
        return;
    }

    println!(
        "{:<8} {:<6} {:<7} {:<8} {:<30} {:<20} {:<24}",
        "PORT", "PROTO", "SCOPE", "PID", "ADDRESS", "PROCESS", "WARNINGS"
    );
    for record in records {
        let warnings = truncate(
            &format_warnings(&warnings_for_listener(record)),
            WARNINGS_WIDTH,
        );
        println!(
            "{:<8} {:<6} {:<7} {:<8} {:<30} {:<20} {:<24}",
            record.port,
            record.protocol,
            record.scope,
            record
                .pid
                .map_or_else(|| "N/A".to_string(), |pid| pid.to_string()),
            truncate(&record.bind_addr.to_string(), ADDRESS_WIDTH),
            truncate(
                record.process_name.as_deref().unwrap_or("N/A"),
                PROCESS_WIDTH
            ),
            warnings
        );
    }
}

pub fn print_details(details: &[PortDetails]) {
    if details.is_empty() {
        println!("No details found for this port.");
        return;
    }

    for (index, detail) in details.iter().enumerate() {
        if index > 0 {
            println!();
            println!("{}", "-".repeat(72));
        }

        if details.len() > 1 {
            println!("Entry: {}/{}", index + 1, details.len());
        }
        println!("Port: {}", detail.listener.port);
        println!("Protocol: {}", detail.listener.protocol);
        println!("Scope: {}", detail.listener.scope);
        println!("Address: {}", detail.listener.bind_addr);
        println!(
            "PID: {}",
            detail
                .listener
                .pid
                .map_or_else(|| "N/A".to_string(), |pid| pid.to_string())
        );
        println!(
            "Process: {}",
            detail.listener.process_name.as_deref().unwrap_or("N/A")
        );
        println!(
            "Command: {}",
            detail.listener.command.as_deref().unwrap_or("N/A")
        );
        println!(
            "CWD: {}",
            detail
                .cwd
                .as_ref()
                .map_or_else(|| "N/A".to_string(), |cwd| cwd.display().to_string())
        );
        println!("User: {}", detail.user.as_deref().unwrap_or("N/A"));
        println!(
            "CPU: {}",
            detail
                .cpu_percent
                .map_or_else(|| "N/A".to_string(), |cpu| format!("{cpu:.1}%"))
        );
        println!(
            "Memory: {}",
            detail
                .memory_bytes
                .map_or_else(|| "N/A".to_string(), format_bytes)
        );
        println!(
            "Threads: {}",
            detail
                .thread_count
                .map_or_else(|| "N/A".to_string(), |threads| threads.to_string())
        );
        println!(
            "Uptime: {}",
            detail
                .uptime_seconds
                .map_or_else(|| "N/A".to_string(), format_duration)
        );
        println!(
            "Connections: {}",
            detail
                .connection_count
                .map_or_else(|| "N/A".to_string(), |connections| connections.to_string())
        );
        println!("Warnings: {}", format_warnings(&detail.warnings));
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
    println!("Port: {port}");
    println!(
        "PID filter: {}",
        pid.map_or_else(|| "-".to_string(), |pid| pid.to_string())
    );
    println!("Updated: {}", local_timestamp_string());
    println!("Press Ctrl-C to stop.");
    println!();

    if details.is_empty() {
        println!("Port {port} is not currently listening.");
        return Ok(());
    }

    print_details(details);
    Ok(())
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
    let mut chars = value.chars();
    let collected = chars.by_ref().take(width).collect::<String>();
    if chars.next().is_some() && width > 1 {
        let mut truncated = collected.chars().take(width - 1).collect::<String>();
        truncated.push('…');
        truncated
    } else {
        collected
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

    #[test]
    fn truncates_long_values_with_ellipsis() {
        assert_eq!(truncate("abcdefghijkl", 8), "abcdefg…");
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
}
