use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};

use anyhow::{Context, Result};

pub fn extract_poster(input: &Path, poster_out: &Path) -> Result<()> {
    if let Some(dir) = poster_out.parent() { fs::create_dir_all(dir)?; }

    // Force single frame and overwrite. This removes the "image2 pattern" warning.
    let args = [
        "-y", "-hide_banner", "-loglevel", "error",
        "-i", input.to_str().unwrap(),
        "-frames:v", "1",
        "-q:v", "2",
        poster_out.to_str().unwrap(),
    ];

    let out = Command::new("ffmpeg")
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .with_context(|| "spawn ffmpeg (poster)")?;

    if !out.status.success() {
        return Err(anyhow::anyhow!(
            "ffmpeg poster failed (code {:?}): {}",
            out.status.code(),
            String::from_utf8_lossy(&out.stderr)
        ));
    }
    Ok(())
}
