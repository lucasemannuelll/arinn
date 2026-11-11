use std::path::{Path, PathBuf};

pub fn normalize_path(raw: &str) -> Option<PathBuf> {
    let trimmed = raw.trim();

    if trimmed.is_empty() {
        return None;
    }

    let mut p = PathBuf::from(trimmed);

    if p.file_name().and_then(|s| s.to_str()) == Some(".git") {
        p = p.parent()?.to_path_buf();
    }

    if !p.is_dir() {
        return None;
    }

    Some(p)
}

pub fn is_git_repo(p: &Path) -> bool {
    p.join(".git").exists()
}
