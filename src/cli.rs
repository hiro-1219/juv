// src/cli.rs
use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "juv")]
#[command(about = "A fast and seamless package/project manager for Julia, written in Rust.")]
#[command(version, author)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Create a new Julia project in a new directory
    New {
        /// Project name/path
        path: String,
    },
    /// Initialize a Julia project in the current directory
    Init,

    /// Add a package to the project
    Add {
        /// Package name to add
        package: String,
    },

    /// Remove a package from the project
    Remove {
        /// Package name to remove
        package: String,
    },

    /// Instantiate/sync the project environment from Project.toml / Manifest.toml
    Sync,

    /// Run a script in the project environment
    Run {
        /// Script path to run
        script: String,
        /// Additional arguments to pass to the script
        #[arg(last = true)]
        args: Vec<String>,
    },
}
