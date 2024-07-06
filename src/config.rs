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
    /// Return a cloned `Config` instance with `with_vm_skip` set to the passed
    /// value.
    #[must_use]
    pub fn with_vm_skip(mut self, with_vm_skip: bool) -> Self {
        if let Commands::Scaffold(s) = &mut self.command {
            s.with_vm_skip = with_vm_skip;
        }
        self
    }

    pub(crate) fn scaffold(&self) -> crate::scaffold::Scaffold {
        self.command.clone().into()
    }

    pub(crate) fn check(&self) -> crate::check::Check {
        self.command.clone().into()
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

impl From<Commands> for crate::scaffold::Scaffold {
    fn from(command: Commands) -> Self {
        match command {
            Commands::Scaffold(s) => s,
            _ => Default::default(),
        }
    }
}

impl From<Commands> for crate::check::Check {
    fn from(command: Commands) -> Self {
        match command {
            Commands::Check(c) => c,
            _ => Default::default(),
        }
    }
}
