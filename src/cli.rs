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

    /// Add packages to the project
    Add {
        /// Package names or URLs to add
        #[arg(required = true)]
        packages: Vec<String>,
    },

    /// Remove packages from the project
    Remove {
        /// Package names to remove
        #[arg(required = true)]
        packages: Vec<String>,
    },

    /// Instantiate/sync the project environment from Project.toml / Manifest.toml
    Sync,

    /// Pure Rust concurrent artifact synchronization (bypasses Pkg resolver)
    SyncOnly,

    /// Run a script in the project environment
    Run {
        /// Script path to run
        script: String,
        /// Additional arguments to pass to the script
        #[arg(last = true)]
        args: Vec<String>,
    },

    /// Build the project into an executable or sysimage
    Build {
        /// Build an executable app (calls PackageCompiler.create_app)
        #[arg(long)]
        app: bool,
        /// Build a sysimage (calls PackageCompiler.create_sysimage)
        #[arg(long)]
        sysimage: bool,
        /// Entry point file (e.g. main.jl). Default is src/ProjectName.jl or main.jl
        #[arg(short, long)]
        entry: Option<String>,
        /// Output directory
        #[arg(short, long)]
        output: Option<String>,
    },
}
