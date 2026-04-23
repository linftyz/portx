use crate::{
    cli::{Cli, Command},
    core::PortService,
    error::Result,
    output,
};

pub fn execute(cli: Cli) -> Result<()> {
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
