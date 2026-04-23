mod json;
mod text;

use crate::{
    cli::ScopeArg,
    core::{KillPlan, KillResult, ListenerRecord, PortDetails},
    error::Result,
};

pub fn print_list(records: &[ListenerRecord], scope: Option<ScopeArg>, json: bool) -> Result<()> {
    if json {
        return self::json::print_list(records, scope);
    }

    text::print_list(records, scope);
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

    text::print_find(records, process_name, scope);
    Ok(())
}

pub fn confirm_kill(plan: &KillPlan, skip_confirmation: bool) -> Result<()> {
    text::confirm_kill(plan, skip_confirmation)
}

pub fn print_kill_result(result: &KillResult) {
    text::print_kill_result(result);
}

pub fn print_watch_placeholder(port: u16, pid: Option<u32>) {
    text::print_watch_placeholder(port, pid);
}

pub fn print_watch_snapshot(port: u16, pid: Option<u32>, details: &[PortDetails]) -> Result<()> {
    text::print_watch_snapshot(port, pid, details)
}
