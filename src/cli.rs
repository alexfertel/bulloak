use clap::{Parser, Subcommand};

/// The main `bulloak` cli interface.
///
/// This is the entry point to the executable.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)] // Read from `Cargo.toml`
pub(crate) struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    #[command(name = "scaffold")]
    Scaffold(crate::scaffold::Scaffold),
    #[command(name = "check")]
    Check(crate::check::Check),
}

pub fn run() -> anyhow::Result<()> {
    let config = Cli::parse();
    match config.command {
        Commands::Scaffold(command) => command.run(),
        Commands::Check(command) => Ok(command.run()),
    }?;

    Ok(())
}
