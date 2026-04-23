use std::process::ExitCode;

fn main() -> ExitCode {
    match portx::run(std::env::args_os()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("portx: {error}");
            ExitCode::from(error.exit_code())
        }
    }
}
