// src/project.rs
use serde::Deserialize;
use std::collections::HashMap;
use anyhow::{Result, Context};

#[derive(Debug, Deserialize)]
pub struct Project {
    pub name: Option<String>,
    pub uuid: Option<String>,
    pub authors: Option<Vec<String>>,
    pub version: Option<String>,
    pub compat: Option<HashMap<String, String>>,
}

impl Project {
    pub fn parse(content: &str) -> Result<Self> {
        toml::from_str(content).context("Failed to parse Project.toml")
    }

    pub fn get_julia_compat(&self) -> Option<&str> {
        self.compat.as_ref()?.get("julia").map(|s| s.as_str())
    }
}
