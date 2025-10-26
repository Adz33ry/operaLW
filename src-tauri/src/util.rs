use std::env;
use std::path::{Path, PathBuf};

// Resolve a binary path reliably when GUI apps lack the interactive shell PATH.
// Priority: explicit env override -> current PATH -> common install locations.
pub fn resolve_bin(name: &str, env_override: Option<&str>) -> Option<PathBuf> {
    if let Some(var) = env_override {
        if let Ok(val) = env::var(var) {
            if !val.is_empty() {
                let p = PathBuf::from(&val);
                if p.exists() {
                    return Some(p);
                }
            }
        }
    }

    if let Ok(path) = env::var("PATH") {
        for p in path.split(':') {
            let candidate = Path::new(p).join(name);
            if candidate.exists() {
                return Some(candidate);
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        for p in ["/opt/homebrew/bin", "/usr/local/bin", "/usr/bin"] {
            let candidate = Path::new(p).join(name);
            if candidate.exists() {
                return Some(candidate);
            }
        }
    }

    #[cfg(not(target_os = "macos"))]
    {
        for p in ["/usr/local/bin", "/usr/bin", "/bin"] {
            let candidate = Path::new(p).join(name);
            if candidate.exists() {
                return Some(candidate);
            }
        }
    }

    None
}

