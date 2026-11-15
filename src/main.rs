mod cli;
mod commit;
mod input;
mod render;
mod repo;

use std::collections::BTreeMap;
use std::fs;
use std::io::IsTerminal;
use std::path::PathBuf;

use anyhow::{Context, Result};
use chrono::NaiveDate;

fn main() -> Result<()> {
    let args = cli::parse();

    let stdin_is_pipe = !std::io::stdin().is_terminal();
    let repos = input::resolve_paths(&args, stdin_is_pipe)?;

    if repos.is_empty() {
        anyhow::bail!("nenhum repositório válido encontrado");
    }

    let (since, until) = cli::resolve_window(&args);

    if args.verbose {
        eprintln!("janela: {since} ..= {until}");
        for repo in &repos {
            println!("{}", repo.display());
        }
    }

    if args.stats {
        run_stats(&args, &repos, since, until)?;
        return Ok(());
    }

    let mut repo_commit_count: Vec<BTreeMap<NaiveDate, u32>> = Vec::new();

    for repo in &repos {
        let commits = commit::read_commits(repo, since, until, &args.authors)?;

        if args.verbose {
            eprintln!("  {} commits", commits.len());
        }

        repo_commit_count.push(commit::aggregate(&commits));
    }

    let merged_commit_count = commit::merge_maps(repo_commit_count);

    let html_render = render::render(
        &merged_commit_count, since, until, &repos
    );

    fs::write(&args.output, html_render)
        .with_context(|| format!(
            "falha ao escrever {}", args.output.display()
        ))?;

    println!("escrito em {}", args.output.display());

    Ok(())
}

fn run_stats(
    args: &cli::Args,
    repos: &[PathBuf],
    since: NaiveDate,
    until: NaiveDate,
) -> Result<()> {
    let mut author_commit_total: 
        BTreeMap<(String, String), u32> = BTreeMap::new();

    for repo in repos {
        let author_stats = commit::author_stats(
            repo, since, until, &args.authors
        )?;

        for (email, name, commit_count) in author_stats {
            if args.verbose {
                eprintln!("  {commit_count:>6}  {name} <{email}>");
            }
            *author_commit_total
                .entry((email, name))
                .or_insert(0) += commit_count;
        }
    }

    let mut sorted_author_stats: Vec<_> = author_commit_total
        .into_iter()
        .map(|((e, n), c)| (e, n, c))
        .collect();

    sorted_author_stats
        .sort_by(|a, b| b.2.cmp(&a.2).then_with(|| a.0.cmp(&b.0)));

    let total_commits: u32 = sorted_author_stats
        .iter()
        .map(|(_, _, c)| c)
        .sum();

    let email_width = calculate_column_width(
        sorted_author_stats.iter().map(|(e, _, _)| display_width(e)),
        display_width("email"),
        60,
    );

    let name_width = calculate_column_width(
        sorted_author_stats.iter().map(|(_, n, _)| display_width(n)),
        display_width("name"),
        60,
    );

    let commits_width = 8;

    eprintln!();
    eprintln!("janela: {since} ..= {until}");
    eprintln!("repos: {}", repos.len());
    eprintln!("commits: {total_commits}");
    eprintln!();

    eprintln!(
        "{} {} {:>width$}",
        pad_or_truncate("email", email_width),
        pad_or_truncate("name", name_width),
        "commits",
        width = commits_width
    );

    eprintln!(
        "{:-<email_width$} {:-<name_width$} {:->commits_width$}", "", "", ""
    );

    for (email, name, commit_count) in &sorted_author_stats {
        eprintln!(
            "{} {} {:>width$}",
            pad_or_truncate(email, email_width),
            pad_or_truncate(name, name_width),
            commit_count,
            width = commits_width
        );
    }

    Ok(())
}

fn display_width(text: &str) -> usize {
    text.chars().count()
}

fn calculate_column_width(
    items_width: impl Iterator<Item = usize>,
    header_width: usize,
    max_width: usize
) -> usize {
    items_width
        .chain(std::iter::once(header_width))
        .max()
        .unwrap_or(0)
        .min(max_width)
}

fn pad_or_truncate(text: &str, column_width: usize) -> String {
    let text_width = display_width(text);

    if text_width <= column_width {
        format!("{text}{}", " ".repeat(column_width - text_width))
    } else {
        let truncated_width = column_width.saturating_sub(1);

        let mut truncated_text: String = text
            .chars()
            .take(truncated_width)
            .collect();

        truncated_text.push('…');

        truncated_text
    }
}
