use std::collections::BTreeMap;
use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result};
use chrono::{DateTime, FixedOffset, NaiveDate};


#[derive(Debug, Clone)]
pub struct Commit {
    pub date: NaiveDate,
}

pub fn read_commits(repo: &Path, since: NaiveDate, until: NaiveDate) -> Result<Vec<Commit>> {
    let out = Command::new("git")
        .arg("-C")
        .arg(repo)
        .arg("log")
        .arg("HEAD")
        .arg(format!("--since={since}"))
        .arg(format!("--until={until}"))
        .arg("--pretty=format:%aI")
        .output()
        .with_context(|| format!("falha ao invocar git em {}", repo.display()))?;

    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        eprintln!("aviso: git log falhou em {}: {}", repo.display(), stderr.trim());
        return Ok(Vec::new());
    }

    let stdout = String::from_utf8_lossy(&out.stdout);
    let mut commits = Vec::new();
    for line in stdout.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        match DateTime::parse_from_rfc3339(line) {
            Ok(dt) => {
                let date = dt.date_naive();

                if date >= since && date <= until {
                    commits.push(Commit { date });
                }
            }
            Err(e) => {
                eprintln!("aviso: linha de commit inválida em {}: {e}", repo.display());
            }
        }
    }
    Ok(commits)
}

pub fn aggregate(commits: &[Commit]) -> BTreeMap<NaiveDate, u32> {
    let mut map = BTreeMap::new();
    for c in commits {
        *map.entry(c.date).or_insert(0) += 1;
    }
    map
}

pub fn merge_maps(maps: impl IntoIterator<Item = BTreeMap<NaiveDate, u32>>) -> BTreeMap<NaiveDate, u32> {
    let mut acc: BTreeMap<NaiveDate, u32> = BTreeMap::new();
    for m in maps {
        for (d, n) in m {
            *acc.entry(d).or_insert(0) += n;
        }
    }
    acc
}
