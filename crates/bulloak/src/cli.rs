//! `bulloak`'s CLI config.
use clap::{Parser, Subcommand};
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
        let cwd = std::env::current_dir().unwrap_or_default();
        let fmt_config = bulloak_foundry::config::resolve_fmt_config(&cwd);

        match &cli.command {
            Commands::Scaffold(cmd) => Self {
                files: cmd.files.clone(),
                solidity_version: cmd.solidity_version.clone(),
                emit_vm_skip: cmd.with_vm_skip,
                skip_modifiers: cmd.skip_modifiers,
                format_descriptions: cmd.format_descriptions,
                fmt_config,
            },
            Commands::Check(cmd) => Self {
                files: cmd.files.clone(),
                emit_vm_skip: cmd.with_vm_skip,
                skip_modifiers: cmd.skip_modifiers,
                format_descriptions: cmd.format_descriptions,
                fmt_config,
                ..Self::default()
            },
        }
    }
}

impl From<&Cli> for bulloak_noir::Config {
    fn from(cli: &Cli) -> Self {
        if match &cli.command {
            Commands::Check(x) => x.format_descriptions,
            Commands::Scaffold(x) => x.format_descriptions,
        } {
            unimplemented!(
                "description formatting is not yet implemented in noir backend"
            )
        }
        if matches!(&cli.command, Commands::Check(x) if x.fix) {
            unimplemented!("--fix is not yet implemented in noir backend")
        }

        match &cli.command {
            Commands::Scaffold(cmd) => Self {
                files: cmd.files.clone(),
                skip_setup_hooks: cmd.skip_modifiers,
                format_descriptions: cmd.format_descriptions,
                ..Self::default()
            },
            Commands::Check(cmd) => Self {
                files: cmd.files.clone(),
                skip_setup_hooks: cmd.skip_modifiers,
                format_descriptions: cmd.format_descriptions,
                ..Self::default()
            },
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
