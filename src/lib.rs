pub mod cli;
pub mod commands;
pub mod core;
pub mod error;
pub mod output;
pub mod platform;
pub mod tui;

use std::ffi::OsString;

use cli::Cli;
use error::Result;

pub fn run<I>(args: I) -> Result<()>
where
    I: IntoIterator<Item = OsString>,
{
    let cli = Cli::parse_normalized(args)?;
    commands::execute(cli)
}
