use std::{
    io::{self, IsTerminal, Write},
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{
    core::{KillPlan, KillResult, ListenerRecord, PortDetails, PortWarning, warnings_for_listener},
    error::{PortxError, Result},
};

pub fn print_list(records: &[ListenerRecord]) {
    if records.is_empty() {
        println!("No listening ports found.");
        return;
    }

    println!(
        "{:<8} {:<6} {:<7} {:<8} {:<24} {:<18} WARNINGS",
        "PORT", "PROTO", "SCOPE", "PID", "ADDRESS", "PROCESS"
    );
    for record in records {
        let warnings = format_warnings(&warnings_for_listener(record));
        println!(
            "{:<8} {:<6} {:<7} {:<8} {:<24} {:<18} {}",
            record.port,
            record.protocol,
            record.scope,
            record
                .pid
                .map_or_else(|| "N/A".to_string(), |pid| pid.to_string()),
            record.bind_addr,
            record.process_name.as_deref().unwrap_or("N/A"),
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
                .map_or_else(|| "N/A".to_string(), |memory| memory.to_string())
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
                .map_or_else(|| "N/A".to_string(), |uptime| format!("{uptime}s"))
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
    println!("Updated: {}", unix_timestamp_seconds());
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

fn unix_timestamp_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default()
}
