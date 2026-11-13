use std::collections::BTreeMap;
use std::path::Path;
use std::process::Command;

use anyhow::{Context, Ok, Result};
use chrono::{DateTime, NaiveDate};


#[derive(Debug, Clone)]
pub struct Commit {
    pub email: String,
    pub name: String,
    pub date: NaiveDate,
}

// pub fn read_commits - apaguei para adicionar os proximo commandos (--author e --stats)

fn run_log(repo: &Path, since: NaiveDate, until: NaiveDate, authors: &[String]) -> Result<String> {
    let mut cmd = Command::new("git");
    cmd.arg("-C")
        .arg(repo)
        .arg("log")
        .arg("HEAD")
        .arg(format!("--since={since}"))
        .arg(format!("--until={until}"))
        .arg("--pretty=format:%ae%x1f%an%x1f%aI");

    for a in authors {
        cmd.arg(format!("--author={a}"));
    }
    let out = cmd.output().with_context(|| format!("falha ao invocar git em {}", repo.display()))?;

    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        eprintln!("aviso: git log falhou em {}: {}", repo.display(), stderr.trim());
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

// pub fn author_stats(...) { ... };

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
