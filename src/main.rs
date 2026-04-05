// src/main.rs
mod cli;
mod julia;
mod manifest;
mod downloader;
mod project;
mod version_manager;

use clap::Parser;
use cli::{Cli, Commands};
use std::process::{Command, Stdio};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use colored::Colorize;
use anyhow::{Result, Context};
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<()> {
    let args = Cli::parse();
    let julia_cmd = resolve_julia_command()?;

    match args.command {
        Commands::New { path } => {
            println!("{} `{}` using {:?}", "Creating".green().bold(), path, julia_cmd);
            julia::run_pkg_generate(&julia_cmd, &path)?;
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
                let new_uuid = Uuid::new_v4().to_string();
                let toml_content = format!("name = \"{}\"\nuuid = \"{}\"\nauthors = [\"juv\"]\n", dirname, new_uuid);
                fs::write(&project_toml, toml_content).context("Failed to write Project.toml")?;
                println!("{} Project.toml in the current directory (with name and uuid)", "Initialized".green().bold());
            }
        }
        Commands::Add { packages } => {
            println!("{} packages `{:?}` via Pkg.jl resolver", "Adding".green().bold(), packages);
            let specs: Vec<String> = packages.iter().map(|p| {
                if p.starts_with("http://") || p.starts_with("https://") || p.starts_with("git@") {
                    format!("Pkg.PackageSpec(url=\"{}\")", p)
                } else {
                    format!("Pkg.PackageSpec(name=\"{}\")", p)
                }
            }).collect();
            let code = format!("using Pkg; Pkg.add([{}])", specs.join(", "));
            
            julia::run_pkg_command(&julia_cmd, &code)?;
            println!("{} Packages `{:?}` added to Manifest", "Resolved".green().bold(), packages);
            
            // Execute parallel sync after adding
            run_parallel_sync(&julia_cmd).await?;
        }
        Commands::Remove { packages } => {
            println!("{} packages `{:?}` via Pkg.jl", "Removing".green().bold(), packages);
            let pkg_list = packages.iter().map(|p| format!("\"{}\"", p)).collect::<Vec<_>>().join(", ");
            let code = format!("using Pkg; Pkg.rm([{}])", pkg_list);
            julia::run_pkg_command(&julia_cmd, &code)?;
            println!("{} Packages `{:?}` removed", "Success".green().bold(), packages);
        }
        Commands::Sync => {
            println!("{} environment (Resolving dependencies)", "Syncing".green().bold());
            let code = "using Pkg; Pkg.resolve()";
            julia::run_pkg_command(&julia_cmd, code)?;
            
            if let Err(e) = run_parallel_sync(&julia_cmd).await {
                println!("{} {:?}", "Error during sync:".red().bold(), e);
            } else {
                println!("{} Environment is in sync", "Success".green().bold());
            }
        }
        Commands::SyncOnly => {
            println!("{} packages concurrently", "Syncing".green().bold());
            if let Err(e) = run_parallel_sync(&julia_cmd).await {
                println!("{} {:?}", "Error during parallel sync:".red().bold(), e);
            } else {
                println!("{} Artifact syncing complete", "Success".green().bold());
            }
        }
        Commands::Run { script, args } => {
            let mut cmd = Command::new(&julia_cmd[0]);
            if julia_cmd.len() > 1 {
                cmd.args(&julia_cmd[1..]);
            }
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
        Commands::Build { app, sysimage, entry, output } => {
            run_build(&julia_cmd, app, sysimage, entry, output).await?;
        }
    }

    Ok(())
}

fn resolve_julia_command() -> Result<Vec<String>> {
    let pwd = env::current_dir()?;
    let project_toml = pwd.join("Project.toml");
    
    if project_toml.exists() {
        let content = fs::read_to_string(&project_toml)?;
        if let Ok(proj) = project::Project::parse(&content) {
            if let Some(compat) = proj.get_julia_compat() {
                return version_manager::check_and_get_julia_command(compat);
            }
        }
    }
    
    Ok(vec!["julia".to_string()])
}

async fn run_build(julia_cmd: &[String], app: bool, sysimage: bool, entry: Option<String>, output: Option<String>) -> Result<()> {
    if !app && !sysimage {
        anyhow::bail!("Please specify either --app or --sysimage for build.");
    }

    // Ensure name and uuid exist in Project.toml (required by PackageCompiler)
    let project_toml_path = Path::new("Project.toml");
    if project_toml_path.exists() {
        let content = fs::read_to_string(project_toml_path)?;
        let mut toml_val: toml::Value = toml::from_str(&content)?;
        let mut changed = false;
        
        if let Some(table) = toml_val.as_table_mut() {
            if !table.contains_key("name") {
                let dirname = env::current_dir()?
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("App")
                    .to_string();
                table.insert("name".to_string(), toml::Value::String(dirname));
                println!("{} field to Project.toml", "Adding missing `name`".yellow().bold());
                changed = true;
            }
            if !table.contains_key("uuid") {
                table.insert("uuid".to_string(), toml::Value::String(Uuid::new_v4().to_string()));
                println!("{} field to Project.toml", "Adding missing `uuid`".yellow().bold());
                changed = true;
            }
        }
        
        if changed {
            let new_content = toml::to_string_pretty(&toml_val)?;
            fs::write(project_toml_path, new_content)?;
        }
    }

    println!("{} Ensuring PackageCompiler.jl is installed...", "Step 1/3".blue().bold());
    let ensure_pkg = "using Pkg; if !haskey(Pkg.dependencies(), Base.UUID(\"9b29e061-f09b-5136-be59-e93540c49f8b\")) \
                      Pkg.add(\"PackageCompiler\") end";
    julia::run_pkg_command(julia_cmd, ensure_pkg)?;

    let output_path = output.unwrap_or_else(|| "build".to_string());
    let entry_point = entry.unwrap_or_else(|| {
        if Path::new("main.jl").exists() {
            "main.jl".to_string()
        } else {
            "src/main.jl".to_string() // Rough guess
        }
    });

    if app {
        println!("{} Building application in `{}` using entry `{}`", "Step 2/3".blue().bold(), output_path, entry_point);
        let jl_code = format!(
            "using PackageCompiler; create_app(\".\", \"{}\"; force=true, incremental=false)",
            output_path
        );
        julia::run_pkg_command(julia_cmd, &jl_code)?;
    } else if sysimage {
        println!("{} Building sysimage in `{}`", "Step 2/3".blue().bold(), output_path);
        let jl_code = format!(
            "using PackageCompiler; create_sysimage(sysimage_path=\"{}/sysimage.so\")",
            output_path
        );
        fs::create_dir_all(&output_path)?;
        julia::run_pkg_command(julia_cmd, &jl_code)?;
    }

    println!("{} Build complete!", "Step 3/3".green().bold());
    Ok(())
}

async fn run_parallel_sync(julia_cmd: &[String]) -> Result<()> {
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
                let target_path_str = match julia::get_slug_path(julia_cmd, &name, &pkg.uuid, &sha) {
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