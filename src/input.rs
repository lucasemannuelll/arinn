use std::io::{BufRead, Read};
use std::path::PathBuf;

use anyhow::{bail, Result};

use crate::cli::Args;
use crate::repo;

/// Decide a fonte (args vs stdin) e devolve a lista de repos normalizada e deduplicada.
pub fn resolve_paths(args: &Args, stdin_is_pipe: bool) -> Result<Vec<PathBuf>> {
    let raw: Vec<String> = if !args.paths.is_empty() {
        args.paths.iter().map(|p| p.display().to_string()).collect()
    } else if stdin_is_pipe {
        read_stdin_lines()?
    } else {
        bail!("nenhum repositório fornecido (passe caminhos como argumentos ou via pipe)");
    };

    let mut out: Vec<PathBuf> = Vec::new();
    for line in raw {
        match repo::normalize_path(&line) {
            Some(p) => {
                if !repo::is_git_repo(&p) {
                    eprintln!("aviso: {} não parece um repositório git; ignorando", p.display());
                    continue;
                }
                let canon = p.canonicalize().unwrap_or(p.clone());
                if !out.iter().any(|e| e.canonicalize().ok().as_ref() == Some(&canon)) {
                    out.push(p);
                }
            }
            None => {
                let t = line.trim();
                if !t.is_empty() {
                    eprintln!("aviso: caminho inválido ou inexistente: {t}");
                }
            }
        }
    }
    Ok(out)
}


fn read_stdin_lines() -> Result<Vec<String>> {
    let stdin = std::io::stdin();
    let mut lock = stdin.lock();
    let mut buf = String::new();
    lock.read_to_string(&mut buf)?;
    Ok(buf.lines().map(str::to_string).collect())
}
