use std::collections::BTreeMap;
use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result};
use chrono::{DateTime, NaiveDate};


#[derive(Debug, Clone)]
pub struct Commit {
    pub email: String,
    pub name: String,
    pub commit_date: NaiveDate,
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
        let trimmed_line = line.trim();
        if trimmed_line.is_empty() {
            continue;
        }

        match parse_commit_line(trimmed_line) {
            Ok(commit) => {
                if commit.commit_date >= since && commit.commit_date <= until {
                    commits.push(commit);
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
    let mut git_cmd = Command::new("git");
    git_cmd.arg("-C")
        .arg(repo)
        .arg("log")
        .arg("HEAD")
        .arg(format!("--since={since}"))
        .arg(format!("--until={until}"))
        .arg("--pretty=format:%aE%x1f%aN%x1f%aI");

    for author in authors {
        git_cmd.arg(format!("--author={author}"));
    }

    let cmd_output = git_cmd.output().with_context(|| {
        format!("falha ao invocar git em {}", repo.display())
    })?;

    if !cmd_output.status.success() {
        let stderr = String::from_utf8_lossy(&cmd_output.stderr);
        eprintln!(
            "aviso: git log falhou em {}: {}", repo.display(), stderr.trim()
        );
        return Ok(String::new());
    }

    Ok(String::from_utf8_lossy(&cmd_output.stdout).into_owned())
}

fn parse_commit_line(line: &str) -> Result<Commit> {
    let mut fields = line.splitn(3, '\x1f');
    let email = fields
        .next()
        .context("email ausente")?
        .to_string();
    let name = fields
        .next()
        .context("name ausente")?
        .to_string();
    let commit_date = fields
        .next()
        .context("data ausente")?;
    let parsed_date = DateTime::parse_from_rfc3339(commit_date)?;
    
    Ok(Commit { 
        email, 
        name, 
        commit_date: parsed_date.date_naive() 
    })
}

pub fn author_stats(
    repo: &Path,
    since: NaiveDate,
    until: NaiveDate,
    authors: &[String],
) -> Result<Vec<(String, String, u32)>> {
    let stdout = run_log(repo, since, until, authors)?;
    let mut author_commit_count: BTreeMap<(String, String), u32> = BTreeMap::new();

    for line in stdout.lines() {
        let trimmed_line = line.trim();
        if trimmed_line.is_empty() {
            continue;
        }

        match parse_commit_line(trimmed_line) {
            Ok(commit) => {
                if commit.commit_date >= since && commit.commit_date <= until {
                    *author_commit_count
                        .entry((commit.email, commit.name))
                        .or_insert(0) += 1;
                }
            }

            Err(_) => {}
        }
    }
    let mut author_stats: Vec<_> = author_commit_count
    .into_iter()
    .map(|((email, name), commit_count)| (email, name, commit_count))
    .collect();

    author_stats.sort_by(|a, b| b.2.cmp(&a.2));

    Ok(author_stats)
}

pub fn aggregate(commits: &[Commit]) -> BTreeMap<NaiveDate, u32> {
    let mut daily_commit_counts = BTreeMap::new();
    for commit in commits {
        *daily_commit_counts
            .entry(commit.commit_date)
            .or_insert(0) += 1;
    }
    daily_commit_counts
}

pub fn merge_maps(
    daily_commit_maps: impl IntoIterator<Item = BTreeMap<NaiveDate, u32>>
) -> BTreeMap<NaiveDate, u32> {
    let mut merged_commit_counts: BTreeMap<NaiveDate, u32> = BTreeMap::new();

    for daily_commit_map in daily_commit_maps {
        for (commit_date, commit_count) in daily_commit_map {
            *merged_commit_counts
                .entry(commit_date)
                .or_insert(0) += commit_count;
        }
    }
    merged_commit_counts
}
