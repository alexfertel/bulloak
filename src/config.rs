//! `bulloak`'s configuration.
use clap::{Parser, Subcommand};
use figment::{providers::Serialized, Figment};
use serde::{Deserialize, Serialize};

/// `bulloak`'s configuration.
#[derive(Parser, Debug, Clone, Default, Serialize, Deserialize)]
#[command(author, version, about, long_about = None)] // Read from `Cargo.toml`
pub struct Config {
    /// `bulloak`'s commands.
    #[clap(subcommand)]
    pub command: Commands,
}

impl Config {
    pub(crate) fn scaffold(&self) -> crate::scaffold::Scaffold {
        match self.command.clone() {
            Commands::Scaffold(s) => s,
            Commands::Check(_) => Default::default(),
        }
    }

    /// Return a cloned `Config` instance with `with_vm_skip` set to the passed
    /// value.
    #[must_use] pub fn with_vm_skip(&self, with_vm_skip: bool) -> Config {
        if let Commands::Scaffold(ref s) = self.command {
            return Config {
                command: Commands::Scaffold(crate::scaffold::Scaffold {
                    with_vm_skip,
                    ..s.clone()
                }),
            };
        }

        self.clone()
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

/// Main entrypoint of `bulloak`'s execution.
pub fn run() -> anyhow::Result<()> {
    let config: Config = Figment::new()
        .merge(Serialized::defaults(Config::parse()))
        .extract()?;

    match &config.command {
        Commands::Scaffold(command) => command.run(&config),
        Commands::Check(command) => command.run(&config),
    }
}
