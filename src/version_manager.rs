// src/version_manager.rs
use std::process::Command;
use anyhow::{Result, Context};
use semver::{Version, VersionReq};

pub fn check_and_get_julia_command(compat_str: &str) -> Result<Vec<String>> {
    // Julia's compat is slightly different from standard semver.
    // Bare version "1.10" means "^1.10" in Julia.
    let semver_req_str = if compat_str.chars().next().map_or(false, |c| c.is_ascii_digit()) {
        format!("^{}", compat_str)
    } else {
        compat_str.to_string()
    };

    let req = VersionReq::parse(&semver_req_str)
        .with_context(|| format!("Failed to parse julia compat version: {}", compat_str))?;

    // Try current default julia version
    if let Ok(current_ver) = get_julia_version("julia") {
        if req.matches(&current_ver) {
            return Ok(vec!["julia".to_string()]);
        }
    }

    if let Some(hint) = extract_version_hint(&semver_req_str) {
        let channel = format!("+{}", hint);
        if let Ok(ver) = get_julia_version_with_channel(&channel) {
            if req.matches(&ver) {
                return Ok(vec!["julia".to_string(), channel]);
            }
        } else {
            // Not installed? Try to add it
            println!("Julia version {} is required but not installed. Adding via juliaup...", hint);
            let status = Command::new("juliaup")
                .arg("add")
                .arg(&hint)
                .status()?;
            
            if status.success() {
                return Ok(vec!["julia".to_string(), channel]);
            }
        }
    }

    // Fallback to default julia if no match found but we can't decide
    Ok(vec!["julia".to_string()])
}

fn get_julia_version(cmd: &str) -> Result<Version> {
    let output = Command::new(cmd)
        .arg("--version")
        .output()?;
    
    parse_julia_version_output(&String::from_utf8_lossy(&output.stdout))
}

fn get_julia_version_with_channel(channel: &str) -> Result<Version> {
    let output = Command::new("julia")
        .arg(channel)
        .arg("--version")
        .output()?;
    
    parse_julia_version_output(&String::from_utf8_lossy(&output.stdout))
}

fn parse_julia_version_output(output: &str) -> Result<Version> {
    // "julia version 1.10.2"
    let parts: Vec<&str> = output.split_whitespace().collect();
    let ver_str = parts.get(2).context("Failed to parse julia version output")?;
    Version::parse(ver_str).context("Failed to parse version string")
}

fn extract_version_hint(req_str: &str) -> Option<String> {
    // Very naive extraction of "1.10" from "^1.10"
    let hint: String = req_str.chars()
        .skip_while(|c| !c.is_ascii_digit())
        .take_while(|c| c.is_ascii_digit() || *c == '.')
        .collect();
    
    if hint.is_empty() { None } else { Some(hint) }
}
