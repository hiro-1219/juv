// src/main.rs
mod cli;
mod julia;

use clap::Parser;
use cli::{Cli, Commands};
use std::process::{Command, Stdio};
use std::env;
use std::fs;
use colored::Colorize;
use anyhow::{Result, Context};

fn main() -> Result<()> {
    let args = Cli::parse();

    match args.command {
        Commands::New { path } => {
            println!("{} `{}`", "Creating".green().bold(), path);
            julia::run_pkg_generate(&path)?;
            println!("{} Project successfully created inside `{}`", "Success".green().bold(), path);
        }
        Commands::Init => {
            let pwd = env::current_dir().context("Failed to get current directory")?;
            let project_toml = pwd.join("Project.toml");
            if project_toml.exists() {
                println!("{} Project.toml already exists", "Skipped".yellow().bold());
            } else {
                let dirname = pwd.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("Env");
                let toml_content = format!("name = \"{}\ 지\nauthors = [\"juv\"]\n", dirname);
                fs::write(&project_toml, toml_content).context("Failed to write Project.toml")?;
                println!("{} Project.toml in the current directory", "Initialized".green().bold());
            }
        }
        Commands::Add { package } => {
            println!("{} package `{}` via Pkg.jl", "Adding".green().bold(), package);
            let code = format!("using Pkg; Pkg.add(\"{}\")", package);
            julia::run_pkg_command(&code)?;
            println!("{} Package `{}` added", "Success".green().bold(), package);
        }
        Commands::Remove { package } => {
            println!("{} package `{}` via Pkg.jl", "Removing".green().bold(), package);
            let code = format!("using Pkg; Pkg.rm(\"{}\")", package);
            julia::run_pkg_command(&code)?;
            println!("{} Package `{}` removed", "Success".green().bold(), package);
        }
        Commands::Sync => {
            println!("{} environment (Pkg.instantiate)", "Syncing".green().bold());
            let code = "using Pkg; Pkg.instantiate()";
            julia::run_pkg_command(code)?;
            println!("{} Environment is in sync", "Success".green().bold());
        }
        Commands::Run { script, args } => {
            let mut cmd = Command::new("julia");
            cmd.arg("--project=@."); // Use local environment
            cmd.arg(&script);
            cmd.args(&args);
            
            let mut child = cmd
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .stdin(Stdio::inherit())
                .spawn()
                .context("Failed to start Julia process")?;

            let status = child.wait().context("Failed to wait on Julia process")?;
            if !status.success() {
                anyhow::bail!("Process exited with status: {}", status);
            }
        }
    }

    Ok(())
}