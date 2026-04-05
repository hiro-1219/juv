// src/julia.rs
use std::process::{Command, Stdio};
use anyhow::{Context, Result};

/// A utility to execute a snippet of Julia code invoking Pkg.
/// It uses the current directory as the project (`--project=@.`).
pub fn run_pkg_command(jl_cmd: &[String], jl_code: &str) -> Result<()> {
    let mut cmd = Command::new(&jl_cmd[0]);
    if jl_cmd.len() > 1 {
        cmd.args(&jl_cmd[1..]);
    }
    let status = cmd
        .arg("--project=@.") 
        .arg("-e")
        .arg(jl_code)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .context("Failed to execute Julia.")?;

    if !status.success() {
        anyhow::bail!("Julia process exited with status: {}", status);
    }
    Ok(())
}

/// A utility to generate a new package/project directory.
pub fn run_pkg_generate(jl_cmd: &[String], name: &str) -> Result<()> {
    let jl_code = format!("using Pkg; Pkg.generate(\"{}\")", name);
    let mut cmd = Command::new(&jl_cmd[0]);
    if jl_cmd.len() > 1 {
        cmd.args(&jl_cmd[1..]);
    }
    let status = cmd
        .arg("-e")
        .arg(&jl_code)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .context("Failed to execute Julia.")?;

    if !status.success() {
        anyhow::bail!("Julia process exited with status: {}", status);
    }
    Ok(())
}

/// Computes the slug path (e.g. ~/.julia/packages/Name/SLUG) by querying Julia
pub fn get_slug_path(jl_cmd: &[String], name: &str, uuid: &str, tree_sha: &str) -> Result<String> {
    let jl_code = format!(
        "using Base: UUID, SHA1; \
         println(joinpath(DEPOT_PATH[1], \"packages\", \"{}\", Base.version_slug(UUID(\"{}\"), SHA1(\"{}\"))))",
         name, uuid, tree_sha
    );
    let mut cmd = Command::new(&jl_cmd[0]);
    if jl_cmd.len() > 1 {
        cmd.args(&jl_cmd[1..]);
    }
    let output = cmd
        .arg("-e")
        .arg(&jl_code)
        .output()
        .context("Failed to compute slug")?;
        
    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Failed to get slug path: {}", err);
    }
    
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}
