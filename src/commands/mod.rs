use std::{thread, time::Duration};

use crate::{
    cli::{Cli, Command},
    core::{PortService, build_kill_plan, execute_kill},
    error::Result,
    output, tui,
};

pub fn execute(cli: Cli) -> Result<()> {
    let service = PortService;

    match cli.command.unwrap_or_default() {
        Command::List(args) => {
            let ports = service.list(args.scope.map(Into::into))?;
            output::print_list(&ports, args.scope, args.json)?;
        }
        Command::Info(args) => {
            let details = service.info(args.port, args.pid)?;
            output::print_details(&details, args.port, args.pid, args.json)?;
        }
        Command::Find(args) => {
            let ports = service.find(&args.process_name, args.scope.map(Into::into))?;
            output::print_find(&ports, &args.process_name, args.scope, args.json)?;
        }
        Command::Kill(args) => {
            let plan = build_kill_plan(&service, args.port, args.pid, args.force)?;
            output::confirm_kill(&plan, args.yes)?;
            let result = execute_kill(&service, plan)?;
            output::print_kill_result(&result);
        }
        Command::Watch(args) => loop {
            let details = service.watch(args.port, args.pid)?;
            output::print_watch_snapshot(args.port, args.pid, &details)?;
            thread::sleep(Duration::from_secs(1));
        },
        Command::Tui(_) => {
            tui::run(&service)?;
        }
    }

    Ok(())
}
