use clap::{Parser, Subcommand};
use std::ffi::OsString;

#[derive(Parser, Debug)]
#[command(name = "seshat", disable_colored_help = true)]
struct ClapCli {
    #[command(subcommand)]
    subcommand: Option<ClapSubcommandEnum>,
}

#[derive(Subcommand, Debug)]
enum ClapSubcommandEnum {
    #[command(name = "valid-command")]
    ValidCommand,
    #[command(name = "simulate-failure")]
    SimulateFailure,
    #[command(name = "complex-state")]
    ComplexState {
        #[arg(long, allow_negative_numbers = true)]
        depth: i32,
    },
}

fn main() {
    let args1: Vec<OsString> = vec!["seshat".into(), "unrecognized-cmd".into()];
    let args2: Vec<OsString> = vec!["seshat".into(), "--unknown-flag".into()];
    
    let mut args3: Vec<OsString> = vec!["seshat".into()];
    for _ in 0..10 { args3.push("--help".into()); }

    let args4: Vec<OsString> = vec!["seshat".into(), "complex-state".into(), "--depth".into(), "-2147483649".into()];

    for args in [args1, args2, args3, args4] {
        match ClapCli::try_parse_from(args) {
            Ok(cli) => println!("OK:\n{:?}", cli),
            Err(e) => {
                println!("ERR:\n{}", e);
                println!("KIND: {:?}", e.kind());
            }
        }
        println!("---------------------");
    }
}
