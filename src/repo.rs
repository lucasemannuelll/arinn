use std::path::{Path, PathBuf};

/// Normaliza um caminho vindo de arg ou stdin.
/// - Trim de whitespace.
/// - Se o último componente é `.git`, sobe um nível.
/// - Retorna None se o caminho não existir ou não for diretório.
pub fn normalize_path(raw_path: &str) -> Option<PathBuf> {
    let trimmed_path = raw_path.trim();

    if trimmed_path.is_empty() {
        return None;
    }

    let mut normalized_path = PathBuf::from(trimmed_path);

    if normalized_path.file_name().and_then(|s| s.to_str()) == Some(".git") {
        normalized_path = normalized_path
            .parent()?
            .to_path_buf();
    }

    if !normalized_path.is_dir() {
        return None;
    }

    Some(normalized_path)
}

/// Um diretório é um repo se contém `.git` como diretório (repo normal)
/// ou como arquivo (worktree ou submódulo com gitdir).
pub fn is_git_repo(path: &Path) -> bool {
    path.join(".git").exists()
}
