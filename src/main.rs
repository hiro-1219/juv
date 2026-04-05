// src/main.rs
mod cli;
mod julia;
mod manifest;
mod downloader;

use clap::Parser;
use cli::{Cli, Commands};
use std::process::{Command, Stdio};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use colored::Colorize;
use anyhow::{Result, Context};

#[tokio::main]
async fn main() -> Result<()> {
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
                let toml_content = format!("name = \"{}\"\nauthors = [\"juv\"]\n", dirname);
                fs::write(&project_toml, toml_content).context("Failed to write Project.toml")?;
                println!("{} Project.toml in the current directory", "Initialized".green().bold());
            }
        }
        Commands::Add { package } => {
            println!("{} package `{}` via Pkg.jl resolver", "Adding".green().bold(), package);
            let code = if package.starts_with("http://") || package.starts_with("https://") || package.starts_with("git@") {
                format!("using Pkg; Pkg.add(url=\"{}\")", package)
            } else {
                format!("using Pkg; Pkg.add(\"{}\")", package)
            };
            julia::run_pkg_command(&code)?;
            println!("{} Package `{}` added to Manifest", "Resolved".green().bold(), package);
            
            // Execute parallel sync after adding
            run_parallel_sync().await?;
        }
        Commands::Remove { package } => {
            println!("{} package `{}` via Pkg.jl", "Removing".green().bold(), package);
            let code = format!("using Pkg; Pkg.rm(\"{}\")", package);
            julia::run_pkg_command(&code)?;
            println!("{} Package `{}` removed", "Success".green().bold(), package);
        }
        Commands::Sync => {
            println!("{} environment (Resolving dependencies)", "Syncing".green().bold());
            let code = "using Pkg; Pkg.resolve()";
            julia::run_pkg_command(code)?;
            
            if let Err(e) = run_parallel_sync().await {
                println!("{} {:?}", "Error during sync:".red().bold(), e);
            } else {
                println!("{} Environment is in sync", "Success".green().bold());
            }
        }
        Commands::SyncOnly => {
            println!("{} packages concurrently", "Syncing".green().bold());
            if let Err(e) = run_parallel_sync().await {
                println!("{} {:?}", "Error during parallel sync:".red().bold(), e);
            } else {
                println!("{} Artifact syncing complete", "Success".green().bold());
            }
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

async fn run_parallel_sync() -> Result<()> {
    let pwd = env::current_dir()?;
    let manifest_path = pwd.join("Manifest.toml");
    
    if !manifest_path.exists() {
        println!("{} No Manifest.toml found, skipping download phase", "Skipped".yellow().bold());
        return Ok(());
    }
    
    let content = fs::read_to_string(&manifest_path)?;
    let manifest = manifest::Manifest::parse(&content)?;
    
    let mut jobs = Vec::new();
    
    for (name, pkgs) in manifest.deps.into_iter() {
        for pkg in pkgs {
            if let Some(sha) = pkg.git_tree_sha1.clone() {
                // Get local target path
                let target_path_str = match julia::get_slug_path(&name, &pkg.uuid, &sha) {
                    Ok(p) => p,
                    Err(e) => {
                        println!("{} {} for {} ({})", "Warning".yellow().bold(), e, name, pkg.uuid);
                        continue;
                    }
                };
                let target_dir = PathBuf::from(&target_path_str);
                
                // If it already exists, skip it
                if target_dir.exists() {
                    continue;
                }
                
                jobs.push(downloader::DownloadJob {
                    name: name.clone(),
                    uuid: pkg.uuid.clone(),
                    tree_sha: sha,
                    target_dir,
                    repo_url: pkg.repo_url.clone(),
                    repo_rev: pkg.repo_rev.clone(),
                });
            }
        }
    }
    
    if !jobs.is_empty() {
        println!("{} {} packages concurrently in Rust", "Downloading".cyan().bold(), jobs.len());
        downloader::download_and_extract_all(jobs).await?;
        println!("{} All artifacts downloaded and extracted", "Success".green().bold());
    } else {
        println!("{} All dependencies are already downloaded", "Up-to-date".green().bold());
    }
    
    Ok(())
}