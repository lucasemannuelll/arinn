use std::path::{Path, PathBuf};

/// Normaliza um caminho vindo de arg ou stdin.
/// - Trim de whitespace.
/// - Se o último componente é `.git`, sobe um nível.
/// - Retorna None se o caminho não existir ou não for diretório.
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

/// Um diretório é um repo se contém `.git` como diretório (repo normal)
/// ou como arquivo (worktree ou submódulo com gitdir).
pub fn is_git_repo(p: &Path) -> bool {
    p.join(".git").exists()
}
