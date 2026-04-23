use std::{ffi::OsString, str::FromStr};

use clap::{Args, Parser, Subcommand, ValueEnum};

use crate::{domain::Scope, error::Result};

#[derive(Debug, Parser)]
#[command(
    name = "portx",
    version,
    about = "A modern port and process management tool",
    long_about = None
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}

impl Cli {
    pub fn parse_normalized<I>(args: I) -> Result<Self>
    where
        I: IntoIterator<Item = OsString>,
    {
        let args = normalize_args(args);
        Ok(Self::try_parse_from(args)?)
    }
}

#[derive(Debug, Clone, Subcommand)]
pub enum Command {
    /// List active listening ports.
    List(ListArgs),
    /// Show detailed information for a port.
    Info(InfoArgs),
    /// Find listening ports by process name.
    Find(FindArgs),
    /// Kill the process bound to a port.
    Kill(KillArgs),
    /// Watch a port and refresh usage metrics.
    Watch(WatchArgs),
}

impl Default for Command {
    fn default() -> Self {
        Self::List(ListArgs::default())
    }
}

#[derive(Debug, Clone, Default, Args)]
pub struct ListArgs {
    /// Filter ports by exposure scope.
    #[arg(long, value_enum)]
    pub scope: Option<ScopeArg>,
    /// Print machine-readable JSON.
    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, Clone, Args)]
pub struct InfoArgs {
    /// Port number to inspect.
    pub port: u16,
    /// Restrict details to a specific PID when a port has multiple owners.
    #[arg(long)]
    pub pid: Option<u32>,
    /// Print machine-readable JSON.
    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, Clone, Args)]
pub struct FindArgs {
    /// Process name fragment to search for.
    pub process_name: String,
    /// Filter ports by exposure scope.
    #[arg(long, value_enum)]
    pub scope: Option<ScopeArg>,
    /// Print machine-readable JSON.
    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, Clone, Args)]
pub struct KillArgs {
    /// Port number whose owning process should be terminated.
    pub port: u16,
    /// Required when a port has multiple owning PIDs.
    #[arg(long)]
    pub pid: Option<u32>,
    /// Use a forceful kill instead of graceful termination.
    #[arg(long)]
    pub force: bool,
    /// Skip interactive confirmation.
    #[arg(long)]
    pub yes: bool,
}

#[derive(Debug, Clone, Args)]
pub struct WatchArgs {
    /// Port number to watch.
    pub port: u16,
    /// Restrict watch output to a specific PID.
    #[arg(long)]
    pub pid: Option<u32>,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum ScopeArg {
    Public,
    Lan,
    Local,
}

impl From<ScopeArg> for Scope {
    fn from(value: ScopeArg) -> Self {
        match value {
            ScopeArg::Public => Self::Public,
            ScopeArg::Lan => Self::Lan,
            ScopeArg::Local => Self::Local,
        }
    }
}

fn normalize_args<I>(args: I) -> Vec<OsString>
where
    I: IntoIterator<Item = OsString>,
{
    let mut args: Vec<OsString> = args.into_iter().collect();

    if let Some(first_user_arg) = args.get(1)
        && let Some(first_user_arg) = first_user_arg.to_str()
        && u16::from_str(first_user_arg).is_ok()
    {
        args.insert(1, OsString::from("info"));
    }

    args
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_to_list_command() {
        let cli = Cli::parse_normalized(["portx"].map(OsString::from)).unwrap();

        assert!(matches!(cli.command.unwrap_or_default(), Command::List(_)));
    }

    #[test]
    fn normalizes_bare_port_to_info_command() {
        let cli = Cli::parse_normalized(["portx", "3000"].map(OsString::from)).unwrap();

        let Command::Info(args) = cli.command.unwrap() else {
            panic!("expected info command");
        };

        assert_eq!(args.port, 3000);
    }

    #[test]
    fn parses_list_scope_and_json_flags() {
        let cli = Cli::parse_normalized(
            ["portx", "list", "--scope", "local", "--json"].map(OsString::from),
        )
        .unwrap();

        let Command::List(args) = cli.command.unwrap() else {
            panic!("expected list command");
        };

        assert!(matches!(args.scope, Some(ScopeArg::Local)));
        assert!(args.json);
    }
}
