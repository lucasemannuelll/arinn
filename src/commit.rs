use std::collections::BTreeMap;
use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result};
use chrono::{DateTime, NaiveDate, NaiveDateTime};

#[derive(Debug, Clone)]
pub struct Commit {
    pub email: String,
    pub name: String,
    pub commit_date: NaiveDate,
    pub commit_time: NaiveDateTime,
    pub hash: String,
    pub message: String,
    pub parents: Vec<String>,
    pub insertions: u64,
    pub deletions: u64,
}

impl Commit {
    pub fn is_merge(&self) -> bool {
        self.parents.len() > 1
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CommitRef {
    pub hash: String,
    pub time: String,
    pub message: String,
    pub author_email: String,
    pub author_name: String,
}

#[derive(Debug, Clone, Default)]
pub struct RepoStat {
    pub commits: u32,
    pub insertions: u64,
    pub deletions: u64,
    pub commits_detail: Vec<CommitRef>,
}

#[derive(Debug, Clone)]
pub struct DayBucket {
    pub date: NaiveDate,
    pub total_commits: u32,
    pub total_insertions: u64,
    pub total_deletions: u64,
    pub per_repo: BTreeMap<String, RepoStat>,
}

pub fn read_commits(
    repo: &Path,
    since: NaiveDate,
    until: NaiveDate,
    authors: &[String],
) -> Result<Vec<Commit>> {
    let stdout = run_log(repo, since, until, authors)?;

    let commits: Vec<Commit> = parse_log_output(repo, &stdout)
        .into_iter()
        .filter(|commit| {
            commit.commit_date >= since && commit.commit_date <= until
        })
        .collect();

    Ok(commits)
}

fn run_log(
    repo: &Path, since: NaiveDate, until: NaiveDate, authors: &[String]
) -> Result<String> {
    let mut git_cmd = Command::new("git");
    git_cmd
        .arg("-C")
        .arg(repo)
        .arg("log")
        .arg("HEAD")
        .arg(format!("--since={since}"))
        .arg(format!("--until={until}"))
        .arg("-M")
        .arg("--numstat")
        .arg("--pretty=format:%aE%x1f%aN%x1f%aI%x1f%H%x1f%P%x1f%s");

    for author in authors {
        git_cmd.arg(format!("--author={author}"));
    }

    let cmd_output = git_cmd
        .output()
        .with_context(|| format!(
            "falha ao invocar git em {}", repo.display())
        )?;

    if !cmd_output.status.success() {
        let stderr = String::from_utf8_lossy(&cmd_output.stderr);
        eprintln!(
            "git log falhou em {}: {}", repo.display(), stderr.trim()
        );
        return Ok(String::new());
    }

    Ok(String::from_utf8_lossy(&cmd_output.stdout).into_owned())
}

fn parse_commit_line(line: &str) -> Result<Commit> {
    let mut fields = line.splitn(6, '\x1f');
    let email = fields.next().context("email ausente")?.to_string();
    let name = fields.next().context("name ausente")?.to_string();
    let commit_date = fields.next().context("data ausente")?;
    let hash = fields.next().context("hash ausente")?.to_string();
    let parents = fields.next().context("parents ausente")?.to_string();
    let message = fields.next().context("mensagem ausente")?.to_string();
    let parsed_date = DateTime::parse_from_rfc3339(commit_date)?;

    Ok(Commit {
        email,
        name,
        commit_date: parsed_date.date_naive(),
        commit_time: parsed_date.naive_local(),
        hash,
        message,
        parents: parents
            .split_whitespace()
            .map(str::to_string)
            .collect(),
        insertions: 0,
        deletions: 0,
    })
}

fn parse_numstat_line(line: &str) -> Option<(u64, u64)> {
    let mut columns = line.split('\t');
    let insertions_str = columns.next()?;
    let deletions_str = columns.next()?;

    let insertions = insertions_str.parse().ok()?;
    let deletions = deletions_str.parse().ok()?;

    Some((insertions, deletions))
}

fn parse_log_output(repo: &Path, stdout: &str) -> Vec<Commit> {
    let mut commits = Vec::new();

    for line in stdout.lines() {
        let trimmed_line = line.trim();
        if trimmed_line.is_empty() {
            continue;
        }

        if trimmed_line.contains('\x1f') {
            match parse_commit_line(trimmed_line) {
                Ok(commit) => commits.push(commit),
                Err(e) => {
                    eprintln!(
                        "linha de commit inválida em {}: {e}",repo.display()
                    );
                }
            }
        } else if let Some((insertions, deletions)) =
            parse_numstat_line(trimmed_line)
        {
            if let Some(last_commit) = commits.last_mut() {
                last_commit.insertions += insertions;
                last_commit.deletions += deletions;
            }
        }
    }

    commits
}

pub fn author_stats(
    repo: &Path,
    since: NaiveDate,
    until: NaiveDate,
    authors: &[String],
) -> Result<Vec<(String, String, u32, u64, u64)>> {
    let stdout = run_log(repo, since, until, authors)?;

    let mut author_totals: BTreeMap<
        (String, String), (u32, u64, u64)
    > = BTreeMap::new();

    for commit in parse_log_output(repo, &stdout) {
        if commit.commit_date >= since
            && commit.commit_date <= until {
            // merges não somam linhas
            let lines = if commit.is_merge() {
                (0, 0)
            } else {
                (commit.insertions, commit.deletions)
            };

            let author_total = author_totals
                .entry((commit.email, commit.name))
                .or_insert((0, 0, 0));

            author_total.0 += 1;
            author_total.1 += lines.0;
            author_total.2 += lines.1;
        }
    }
    let mut author_stats: Vec<_> = author_totals
        .into_iter()
        .map(|((email, name), (commit_count, add, del))| {
            (email, name, commit_count, add, del)
        })
        .collect();

    author_stats.sort_by(|a, b| b.2.cmp(&a.2));

    Ok(author_stats)
}

pub fn repo_day_stats(
    commits: &[Commit],
) -> BTreeMap<NaiveDate, RepoStat> {
    let mut day_stats: BTreeMap<NaiveDate, RepoStat> = BTreeMap::new();
    for commit in commits {
        let repo_stat = day_stats.entry(commit.commit_date).or_default();
        repo_stat.commits += 1;

        if !commit.is_merge() {
            repo_stat.insertions += commit.insertions;
            repo_stat.deletions += commit.deletions;
        }

        repo_stat.commits_detail.push(CommitRef {
            hash: commit.hash.clone(),
            time: commit.commit_time.format("%H:%M").to_string(),
            message: commit.message.clone(),
            author_email: commit.email.clone(),
            author_name: commit.name.clone(),
        });
    }
    day_stats
}

pub fn merge_day_buckets(
    repo_day_stats: impl IntoIterator<
        Item = (String, BTreeMap<NaiveDate, RepoStat>)
    >,
) -> Vec<DayBucket> {
    let mut day_buckets: BTreeMap<NaiveDate, DayBucket> = BTreeMap::new();

    for (repo_name, day_stats) in repo_day_stats {
        for (date, repo_stat) in day_stats {
            let bucket = day_buckets
                .entry(date)
                .or_insert_with(|| DayBucket {
                    date,
                    total_commits: 0,
                    total_insertions: 0,
                    total_deletions: 0,
                    per_repo: BTreeMap::new(),
                });

            bucket.total_commits += repo_stat.commits;
            bucket.total_insertions += repo_stat.insertions;
            bucket.total_deletions += repo_stat.deletions;
            bucket.per_repo.insert(repo_name.clone(), repo_stat);
        }
    }

    day_buckets.into_values().collect()
}

pub fn total_by_day(
    day_buckets: &[DayBucket],
) -> BTreeMap<NaiveDate, u32> {
    day_buckets
        .iter()
        .map(|bucket| (bucket.date, bucket.total_commits))
        .collect()
}

pub fn totals_by_repo(day_buckets: &[DayBucket]) -> BTreeMap<String, RepoStat> {
    let mut totals: BTreeMap<String, RepoStat> = BTreeMap::new();
    for bucket in day_buckets {
        for (repo_name, repo_stat) in &bucket.per_repo {
            let total = totals.entry(repo_name.clone()).or_default();
            total.commits += repo_stat.commits;
            total.insertions += repo_stat.insertions;
            total.deletions += repo_stat.deletions;
            total.commits_detail
                .extend(repo_stat.commits_detail.iter().cloned());
        }
    }
    totals
}
