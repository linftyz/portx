mod json;
mod text;

use crate::{
    cli::ScopeArg,
    core::{ListenerRecord, PortDetails},
    error::Result,
};

pub fn print_list(records: &[ListenerRecord], scope: Option<ScopeArg>, json: bool) -> Result<()> {
    if json {
        return self::json::print_list(records, scope);
    }

    text::print_list(records);
    Ok(())
}

pub fn print_details(
    details: &[PortDetails],
    port: u16,
    pid: Option<u32>,
    json: bool,
) -> Result<()> {
    if json {
        return self::json::print_info(details, port, pid);
    }

    text::print_details(details);
    Ok(())
}

pub fn print_find(
    records: &[ListenerRecord],
    process_name: &str,
    scope: Option<ScopeArg>,
    json: bool,
) -> Result<()> {
    if json {
        return self::json::print_find(records, process_name, scope);
    }

    text::print_list(records);
    Ok(())
}

pub fn print_kill_placeholder(port: u16, pid: Option<u32>, force: bool) {
    text::print_kill_placeholder(port, pid, force);
}

pub fn print_watch_placeholder(port: u16, pid: Option<u32>) {
    text::print_watch_placeholder(port, pid);
}
