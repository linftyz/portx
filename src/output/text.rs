use crate::core::{ListenerRecord, PortDetails};

pub fn print_list(records: &[ListenerRecord]) {
    if records.is_empty() {
        println!("No listening ports found.");
        return;
    }

    println!(
        "{:<8} {:<6} {:<7} {:<8} {:<24} PROCESS",
        "PORT", "PROTO", "SCOPE", "PID", "ADDRESS"
    );
    for record in records {
        println!(
            "{:<8} {:<6} {:<7} {:<8} {:<24} {}",
            record.port,
            record.protocol,
            record.scope,
            record
                .pid
                .map_or_else(|| "N/A".to_string(), |pid| pid.to_string()),
            record.bind_addr,
            record.process_name.as_deref().unwrap_or("N/A")
        );
    }
}

pub fn print_details(details: &[PortDetails]) {
    if details.is_empty() {
        println!("No details found for this port.");
        return;
    }

    for detail in details {
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
        println!("Warnings: {:?}", detail.warnings);
    }
}

pub fn print_kill_placeholder(port: u16, pid: Option<u32>, force: bool) {
    let mode = if force { "force" } else { "graceful" };
    match pid {
        Some(pid) => println!("Kill placeholder: {mode} termination for PID {pid} on port {port}."),
        None => println!("Kill placeholder: {mode} termination for port {port}."),
    }
}

pub fn print_watch_placeholder(port: u16, pid: Option<u32>) {
    match pid {
        Some(pid) => println!("Watch placeholder: monitoring port {port} for PID {pid}."),
        None => println!("Watch placeholder: monitoring port {port}."),
    }
}
