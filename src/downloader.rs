// src/downloader.rs
use anyhow::Result;
use reqwest::Client;
use std::path::PathBuf;
use tokio::task;
use std::io::Cursor;
use flate2::read::GzDecoder;
use tar::Archive;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::sync::Arc;

pub struct DownloadJob {
    pub name: String,
    pub uuid: String,
    pub tree_sha: String,
    pub target_dir: PathBuf,
    pub repo_url: Option<String>,
    pub repo_rev: Option<String>,
}

fn extract_archive<R: std::io::Read>(
    tar: GzDecoder<R>,
    target: &std::path::Path,
    strip_first_component: bool,
) -> Result<()> {
    let mut archive = Archive::new(tar);
    if !target.exists() {
        std::fs::create_dir_all(target)?;
    }

    if !strip_first_component {
        archive.unpack(target)?;
    } else {
        for file in archive.entries()? {
            let mut file = file?;
            let path = file.path()?.to_path_buf();

            let mut components = path.components();
            components.next(); // Skip the root directory
            let stripped_path = components.as_path();

            if stripped_path.as_os_str().is_empty() {
                continue; // Skip the root directory entry itself
            }

            let dest = target.join(stripped_path);
            if let Some(parent) = dest.parent() {
                std::fs::create_dir_all(parent)?;
            }
            file.unpack(&dest)?;
        }
    }
    Ok(())
}

pub async fn download_and_extract_all(jobs: Vec<DownloadJob>) -> Result<()> {
    if jobs.is_empty() {
        return Ok(());
    }

    let client = Client::new();
    let multi_progress = Arc::new(MultiProgress::new());

    let pb_style = ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {msg} ({bytes}/{total_bytes})")
        .unwrap()
        .progress_chars("#>-");

    let mut handles = Vec::new();

    for job in jobs {
        let client_clone = client.clone();
        let name_clone = job.name.clone();
        let mp_clone = multi_progress.clone();
        let style_clone = pb_style.clone();

        let handle = task::spawn(async move {
            let pb = mp_clone.add(ProgressBar::new(0));
            pb.set_style(style_clone);
            pb.set_message(format!("Downloading {}", name_clone));

            // Determine URL and strategy
            let mut url = format!(
                "https://pkg.julialang.org/package/{}/{}",
                job.uuid, job.tree_sha
            );
            let mut strip_first = false;

            if let (Some(repo_url), Some(repo_rev)) = (&job.repo_url, &job.repo_rev) {
                if repo_url.contains("github.com") {
                    let mut base = repo_url.clone();
                    if base.ends_with(".git") {
                        base = base[0..base.len() - 4].to_string();
                    }
                    url = format!("{}/archive/{}.tar.gz", base, repo_rev);
                    strip_first = true;
                } else {
                    // Fallback to git clone for non-GitHub repos
                    pb.finish_with_message(format!("Cloning {} via Git", name_clone));
                    
                    let target = job.target_dir.clone();
                    let url_git = repo_url.clone();
                    let rev_git = repo_rev.clone();
                    
                    // Run git clone natively
                    let status = tokio::process::Command::new("git")
                        .arg("clone")
                        .arg("--depth")
                        .arg("1")
                        .arg("-b")
                        .arg(&rev_git)
                        .arg(&url_git)
                        .arg(&target)
                        .stdout(std::process::Stdio::null())
                        .stderr(std::process::Stdio::null())
                        .status()
                        .await?;

                    if !status.success() {
                        anyhow::bail!("Failed to git clone {} version {}", url_git, rev_git);
                    }

                    // For git clone, we don't proceed to download the tarball
                    return Ok::<(), anyhow::Error>(());
                }
            }

            let response = client_clone.get(&url).send().await?;
            if !response.status().is_success() {
                anyhow::bail!("Failed to download {}: HTTP {}", name_clone, response.status());
            }

            let total_size = response.content_length().unwrap_or(0);
            pb.set_length(total_size);

            let bytes = response.bytes().await?;
            pb.finish_with_message(format!("Extracting {}", name_clone));

            let bytes_vec = bytes.to_vec();
            let target = job.target_dir.clone();

            task::spawn_blocking(move || -> Result<()> {
                if target.exists() {
                    return Ok(());
                }
                let cursor = Cursor::new(bytes_vec);
                let tar = GzDecoder::new(cursor);
                extract_archive(tar, &target, strip_first)?;
                Ok(())
            }).await??;

            Ok::<(), anyhow::Error>(())
        });
        handles.push(handle);
    }

    // Wait for all
    for handle in handles {
        let _ = handle.await?;
    }

    Ok(())
}
