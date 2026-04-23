pub mod cli;
pub mod domain;
pub mod error;
pub mod output;
pub mod service;

use std::ffi::OsString;

use cli::{Cli, Command};
use error::Result;
use service::PortService;

pub fn run<I>(args: I) -> Result<()>
where
    I: IntoIterator<Item = OsString>,
{
    let cli = Cli::parse_normalized(args)?;
    let service = PortService;

    match cli.command.unwrap_or_default() {
        Command::List(args) => {
            let ports = service.list(args.scope.map(Into::into))?;
            output::print_list(&ports, args.json)?;
        }
        Command::Info(args) => {
            let details = service.info(args.port, args.pid)?;
            output::print_details(&details, args.json)?;
        }
        Command::Find(args) => {
            let ports = service.find(&args.process_name, args.scope.map(Into::into))?;
            output::print_list(&ports, args.json)?;
        }
        Command::Kill(args) => {
            service.kill(args.port, args.pid, args.force, args.yes)?;
            output::print_kill_placeholder(args.port, args.pid, args.force);
        }
        Command::Watch(args) => {
            service.watch(args.port, args.pid)?;
            output::print_watch_placeholder(args.port, args.pid);
        }
    }

    Ok(())
}
