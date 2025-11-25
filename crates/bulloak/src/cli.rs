//! `bulloak`'s CLI config.
use clap::{Parser, Subcommand, ValueEnum};
use figment::{providers::Serialized, Figment};
use serde::{Deserialize, Serialize};

/// `bulloak`'s configuration.
#[derive(Parser, Debug, Clone, Default, Serialize, Deserialize)]
#[command(author, version, about, long_about = None)] // Read from `Cargo.toml`
pub struct Cli {
    /// `bulloak`'s commands.
    #[clap(subcommand)]
    pub command: Commands,
}

/// Available backend types for CLI argument parsing.
#[derive(Debug, Serialize, Deserialize, Clone, ValueEnum)]
pub enum BackendKind {
    /// original Foundry backend.
    Solidity,
    Noir,
}

impl BackendKind {
    /// Creates a boxed Backend trait object from this BackendKind.
    ///
    /// This is the factory method that instantiates the correct backend
    /// implementation with its configuration baked in.
    pub fn into_backend(self, cli: &Cli) -> Box<dyn bulloak_backend::Backend> {
        match self {
            Self::Solidity => {
                Box::new(bulloak_foundry::FoundryBackend::new(cli))
            }
            Self::Noir => Box::new(bulloak_noir::NoirBackend::new(cli)),
        }
    }
}

/// `bulloak`'s commands.
#[derive(Debug, Clone, Subcommand, Serialize, Deserialize)]
pub enum Commands {
    /// `bulloak scaffold`.
    #[command(name = "scaffold")]
    Scaffold(crate::scaffold::Scaffold),
    /// `bulloak check`.
    #[command(name = "check")]
    Check(crate::check::Check),
}

impl Default for Commands {
    fn default() -> Self {
        Self::Scaffold(Default::default())
    }
}

impl From<&Cli> for bulloak_foundry::config::Config {
    fn from(cli: &Cli) -> Self {
        match &cli.command {
            Commands::Scaffold(cmd) => Self {
                files: cmd.files.clone(),
                solidity_version: cmd.solidity_version.clone(),
                emit_vm_skip: cmd.with_vm_skip,
                skip_modifiers: cmd.skip_modifiers,
                format_descriptions: cmd.format_descriptions,
                ..Self::default()
            },
            Commands::Check(cmd) => Self {
                files: cmd.files.clone(),
                skip_modifiers: cmd.skip_modifiers,
                format_descriptions: cmd.format_descriptions,
                ..Self::default()
            },
        }
    }
}

impl From<&Cli> for bulloak_noir::Config {
    fn from(cli: &Cli) -> Self {
        match &cli.command {
            Commands::Scaffold(cmd) => Self {
                files: cmd.files.clone(),
                skip_setup_hooks: cmd.skip_modifiers,
                format_descriptions: cmd.format_descriptions,
                ..Self::default()
            },
            Commands::Check(_cmd) => {
                todo!();
            }
        }
    }
}

/// Main entrypoint of `bulloak`'s execution.
pub(crate) fn run() -> anyhow::Result<()> {
    let config: Cli =
        Figment::new().merge(Serialized::defaults(Cli::parse())).extract()?;

    match &config.command {
        Commands::Scaffold(command) => command.run(&config),
        Commands::Check(command) => command.run(&config),
    };

    Ok(())
}
