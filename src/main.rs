use clap::Parser;

use crate::sandbox::SandboxBuilder;

pub mod error;
pub mod sandbox;

#[derive(Parser, Debug)]
#[command(
    version,
    about,
    long_about = "Run a process in a sandboxxed environment with configurable isolation.",
    trailing_var_arg = true
)]
struct Args {
    /// The program to execute, will be resolved by $PATH
    process: String,

    /// The arguments to pass to the programe.
    args: Vec<String>,

    #[arg(short, long, default_value_t = true)]
    network: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let sandbox = SandboxBuilder::new(&args.process)?
        .add_args(&args.args)
        .network(args.network)
        .build()?;

    sandbox.run()?;
    return Ok(());
}
