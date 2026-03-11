use clap::Parser;

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
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    sandbox::start_sandbox(&args.process, &args.args)?;
    return Ok(());
}
