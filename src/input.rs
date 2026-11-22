use std::io::Read;
use std::path::PathBuf;

use anyhow::{bail, Result};

use crate::cli::Args;
use crate::repo;

/// Decide a fonte (args vs stdin) e
/// devolve a lista de repos normalizada e deduplicada.
pub fn resolve_paths(args: &Args, stdin_is_pipe: bool) -> Result<Vec<PathBuf>> {
    let input_paths: Vec<String> = if !args.paths.is_empty() {
        args.paths.iter().map(|p| p.display().to_string()).collect()
    } else if stdin_is_pipe {
        read_stdin_lines()?
    } else {
        bail!("nenhum repositório fornecido");
    };

    let mut resolved_paths: Vec<PathBuf> = Vec::new();

    for input_path in input_paths {
        match repo::normalize_path(&input_path) {
            Some(repo) => {
                if !repo::is_git_repo(&repo) {
                    eprintln!(
                        "{} não parece um repositório git; ignorando",
                        repo.display()
                    );
                    continue;
                }
                let canonical_path = repo.canonicalize().unwrap_or(repo.clone());

                if !resolved_paths
                    .iter()
                    .any(|e| e.canonicalize().ok().as_ref() == Some(&canonical_path))
                {
                    resolved_paths.push(repo);
                }
            }
            None => {
                let trimmed_path = input_path.trim();
                if !trimmed_path.is_empty() {
                    eprintln!("caminho inválido ou inexistente: {trimmed_path}");
                }
            }
        }
    }
    Ok(resolved_paths)
}

fn read_stdin_lines() -> Result<Vec<String>> {
    let stdin = std::io::stdin();
    let mut stdin_lock = stdin.lock();
    let mut stdin_buffer = String::new();

    stdin_lock.read_to_string(&mut stdin_buffer)?;

    Ok(stdin_buffer.lines().map(str::to_string).collect())
}
