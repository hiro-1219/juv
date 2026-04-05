// src/julia.rs
use std::process::{Command, Stdio};
use anyhow::{Context, Result};

/// A utility to execute a snippet of Julia code invoking Pkg.
/// It uses the current directory as the project (`--project=.`).
pub fn run_pkg_command(jl_code: &str) -> Result<()> {
    let status = Command::new("julia")
        .arg("--project=@.") // Use the environment of the nearest Project.toml
        .arg("-e")
        .arg(jl_code)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .context("Failed to execute `julia`. Make sure Julia is installed and in your PATH.")?;

    if !status.success() {
        anyhow::bail!("Julia process exited with status: {}", status);
    }
    Ok(())
}

/// A utility to generate a new package/project directory.
/// Note: Pkg.generate does not use `--project=@.` as it creates a new project.
pub fn run_pkg_generate(name: &str) -> Result<()> {
    let jl_code = format!("using Pkg; Pkg.generate(\"{}\")", name);
    let status = Command::new("julia")
        .arg("-e")
        .arg(&jl_code)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .context("Failed to execute `julia`.")?;

    if !status.success() {
        anyhow::bail!("Julia process exited with status: {}", status);
    }
    Ok(())
}
