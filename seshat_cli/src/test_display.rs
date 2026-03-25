use clap::Parser;

#[derive(Parser)]
#[command(arg_required_else_help = true)]
struct Cli {
    #[command(subcommand)]
    cmd: Option<Cmd>,
}

#[derive(clap::Subcommand)]
enum Cmd {
    Run,
}

fn main() {
    let err = Cli::try_parse_from(["seshat"]).unwrap_err();
    println!("KIND: {:?}", err.kind());
    println!("ERR: {}", err);
}
