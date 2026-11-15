use std::collections::BTreeMap;
use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result};
use chrono::{DateTime, NaiveDate};


#[derive(Debug, Clone)]
pub struct Commit {
    pub email: String,
    pub name: String,
    pub date: NaiveDate,
}

pub fn read_commits(
    repo: &Path,
    since: NaiveDate,
    until: NaiveDate,
    authors: &[String],
) -> Result<Vec<Commit>> {
    let stdout = run_log(repo, since, until, authors)?;
    let mut commits = Vec::new();

    for line in stdout.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        match parse_commit_line(line) {
            Ok(c) => {
                if c.date >= since && c.date <= until {
                    commits.push(c);
                }
            }
            Err(e) => {
                eprintln!(
                    "aviso: linha de commit inválida em {}: {e}",
                    repo.display()
                );
            }
        }
    }
    Ok(commits)
}

fn run_log(
    repo: &Path, 
    since: NaiveDate, 
    until: NaiveDate, 
    authors: &[String]
) -> Result<String> {
    let mut cmd = Command::new("git");
    cmd.arg("-C")
        .arg(repo)
        .arg("log")
        .arg("HEAD")
        .arg(format!("--since={since}"))
        .arg(format!("--until={until}"))
        .arg("--pretty=format:%aE%x1f%aN%x1f%aI");

    for a in authors {
        cmd.arg(format!("--author={a}"));
    }

    let out = cmd.output().with_context(|| {
        format!("falha ao invocar git em {}", repo.display())
    })?;

    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        eprintln!(
            "aviso: git log falhou em {}: {}", repo.display(), stderr.trim()
        );
        return Ok(String::new());
    }

    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

fn parse_commit_line(line: &str) -> Result<Commit> {
    let mut parts = line.splitn(3, '\x1f');
    let email = parts.next().context("email ausente")?.to_string();
    let name = parts.next().context("name ausente")?.to_string();
    let iso = parts.next().context("data ausente")?;
    let dt = DateTime::parse_from_rfc3339(iso)?;
    
    Ok(Commit { 
        email, 
        name, 
        date: dt.date_naive() 
    })
}

pub fn author_stats(
    repo: &Path,
    since: NaiveDate,
    until: NaiveDate,
    authors: &[String],
) -> Result<Vec<(String, String, u32)>> {
    let stdout = run_log(repo, since, until, authors)?;
    let mut counts: BTreeMap<(String, String), u32> = BTreeMap::new();
    for line in stdout.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        match parse_commit_line(line) {
            Ok(c) => {
                if c.date >= since && c.date <= until {
                    *counts.entry((c.email, c.name)).or_insert(0) += 1;
                }
            }

            Err(_) => {}
        }
    }
    let mut v: Vec<_> = counts
    .into_iter()
    .map(|((email, name), n)| (email, name, n))
    .collect();

    v.sort_by(|a, b| b.2.cmp(&a.2));

    Ok(v)
}

pub fn aggregate(commits: &[Commit]) -> BTreeMap<NaiveDate, u32> {
    let mut map = BTreeMap::new();
    for c in commits {
        *map.entry(c.date).or_insert(0) += 1;
    }
    map
}

pub fn merge_maps(
    maps: impl IntoIterator<Item = BTreeMap<NaiveDate, u32>>
) -> BTreeMap<NaiveDate, u32> {
    let mut acc: BTreeMap<NaiveDate, u32> = BTreeMap::new();
    for m in maps {
        for (d, n) in m {
            *acc.entry(d).or_insert(0) += n;
        }
    }
    acc
}
