// src/manifest.rs
use std::collections::HashMap;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Manifest {
    pub deps: HashMap<String, Vec<PackageData>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PackageData {
    pub uuid: String,
    pub version: Option<String>,
    #[serde(rename = "git-tree-sha1")]
    pub git_tree_sha1: Option<String>,
    #[serde(rename = "repo-url")]
    pub repo_url: Option<String>,
    #[serde(rename = "repo-rev")]
    pub repo_rev: Option<String>,
}

impl Manifest {
    pub fn parse(content: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_manifest() {
        let content = r#"
            found_version = "1.12.5"
            manifest_format = "2.0"

            [[deps.JSON]]
            git-tree-sha1 = "67c6f1f085cb2671c93fe34244c9cccde30f7a26"
            uuid = "682c06a0-de6a-54ab-a142-c8b1cf79cde6"
            version = "1.5.0"
        "#;
        let manifest = Manifest::parse(content).unwrap();
        assert!(manifest.deps.contains_key("JSON"));
        let pkg = &manifest.deps["JSON"][0];
        assert_eq!(pkg.uuid, "682c06a0-de6a-54ab-a142-c8b1cf79cde6");
        assert_eq!(pkg.git_tree_sha1.as_deref(), Some("67c6f1f085cb2671c93fe34244c9cccde30f7a26"));
    }

    #[test]
    fn test_parse_git_manifest() {
        let content = r#"
            [[deps.Example]]
            git-tree-sha1 = "84ec67045e0af90908e775e4d7a3782b9f27e98a"
            repo-rev = "master"
            repo-url = "https://github.com/JuliaLang/Example.jl"
            uuid = "7876af07-990d-54b4-ab0e-23690620f79a"
            version = "0.5.5"
        "#;
        let manifest = Manifest::parse(content).unwrap();
        assert!(manifest.deps.contains_key("Example"));
        let pkg = &manifest.deps["Example"][0];
        assert_eq!(pkg.repo_url.as_deref(), Some("https://github.com/JuliaLang/Example.jl"));
        assert_eq!(pkg.repo_rev.as_deref(), Some("master"));
    }
}
