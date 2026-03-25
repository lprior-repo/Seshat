use std::process::ExitCode;

fn main() -> ExitCode {
    let args = std::env::args_os();
    match seshat_cli::parse_args(args) {
        Ok(seshat_cli::Cli::Help) => {
            print!("{}", seshat_cli::get_help());
            ExitCode::SUCCESS
        }
        Ok(seshat_cli::Cli::Version) => {
            print!("{}", seshat_cli::get_version());
            ExitCode::SUCCESS
        }
        Ok(cli @ (seshat_cli::Cli::Run(_) | seshat_cli::Cli::Bare)) => {
            match seshat_cli::execute(cli) {
                Ok(()) => ExitCode::SUCCESS,
                Err(e) => {
                    eprintln!("{e}");
                    ExitCode::FAILURE
                }
            }
        }
        Err(e) => {
            eprintln!("{e}");
            ExitCode::from(2)
        }
    }
}
