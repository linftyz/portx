mod json;
mod text;

use crate::{
    core::{ListenerRecord, PortDetails},
    error::Result,
};

pub fn print_list(records: &[ListenerRecord], json: bool) -> Result<()> {
    if json {
        return self::json::print(records);
    }

    text::print_list(records);
    Ok(())
}

pub fn print_details(details: &[PortDetails], json: bool) -> Result<()> {
    if json {
        return self::json::print(details);
    }

    text::print_details(details);
    Ok(())
}

pub fn print_kill_placeholder(port: u16, pid: Option<u32>, force: bool) {
    text::print_kill_placeholder(port, pid, force);
}

pub fn print_watch_placeholder(port: u16, pid: Option<u32>) {
    text::print_watch_placeholder(port, pid);
}
