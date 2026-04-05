// src/julia.rs
use std::process::{Command, Stdio};
use anyhow::{Context, Result};

pub fn run_pkg_command(jl_code: &str) -> Result<()> {
    let status = Command::new("julia")
        .arg("--project=@.") 
        .arg("-e")
        .arg(jl_code)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .context("Failed to execute `julia`.")?;

    if !status.success() {
        anyhow::bail!("Julia process exited with status: {}", status);
    }
    Ok(())
}

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

/// Computes the slug path (e.g. ~/.julia/packages/Name/SLUG) by querying Julia
pub fn get_slug_path(name: &str, uuid: &str, tree_sha: &str) -> Result<String> {
    let jl_code = format!(
        "using Base: UUID, SHA1; \
         println(joinpath(DEPOT_PATH[1], \"packages\", \"{}\", Base.version_slug(UUID(\"{}\"), SHA1(\"{}\"))))",
         name, uuid, tree_sha
    );
    let output = Command::new("julia")
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
