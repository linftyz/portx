use std::{env, io::IsTerminal};

use colored::{ColoredString, Colorize};

use crate::core::PortWarning;

pub fn accent(value: &str) -> String {
    paint(value, |text| text.cyan().bold())
}

pub fn muted(value: &str) -> String {
    paint(value, |text| text.bright_black())
}

pub fn highlight(value: &str) -> String {
    paint(value, |text| text.bold())
}

pub fn success(value: &str) -> String {
    paint(value, |text| text.green().bold())
}

pub fn warning(value: &str) -> String {
    paint(value, |text| text.yellow().bold())
}

pub fn danger(value: &str) -> String {
    paint(value, |text| text.red().bold())
}

pub fn scope_value(scope: &str) -> String {
    match scope {
        "PUBLIC" => danger(scope),
        "LAN" => warning(scope),
        "LOCAL" => success(scope),
        _ => scope.to_string(),
    }
}

pub fn warning_value(warnings: &[PortWarning]) -> String {
    let value = warning_text(warnings);
    if warnings.is_empty() {
        muted(&value)
    } else {
        danger(&value)
    }
}

pub fn table_scope_cell(scope: &str, width: usize) -> String {
    let padded = pad_left(scope, width);
    scope_value(&padded)
}

pub fn table_warning_cell(warnings: &str, width: usize) -> String {
    let padded = pad_left(warnings, width);
    if warnings == "-" {
        muted(&padded)
    } else {
        danger(&padded)
    }
}

pub fn warning_text(warnings: &[PortWarning]) -> String {
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

fn color_enabled() -> bool {
    std::io::stdout().is_terminal()
        && env::var_os("NO_COLOR").is_none()
        && env::var("TERM").map(|term| term != "dumb").unwrap_or(true)
}

fn paint<F>(value: &str, painter: F) -> String
where
    F: FnOnce(&str) -> ColoredString,
{
    if color_enabled() {
        painter(value).to_string()
    } else {
        value.to_string()
    }
}

fn pad_left(value: &str, width: usize) -> String {
    let padding = width.saturating_sub(value.chars().count());
    format!("{value}{:padding$}", "", padding = padding)
}
