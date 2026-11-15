mod cli;
mod commit;
mod input;
mod render;
mod repo;

use std::collections::BTreeMap;
use std::fs;
use std::io::IsTerminal;

use anyhow::{Context, Result};

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
        for r in &repos {
            println!("{}", r.display());
        }
    }

    if args.stats {
        run_stats(&args, &repos, since, until)?;
        return Ok(());
    }

    let mut per_repo: Vec<BTreeMap<chrono::NaiveDate, u32>> = Vec::new();
    for r in &repos {
        let commits = commit::read_commits(r, since, until, &args.authors)?;
        if args.verbose {
            eprintln!("  {} commits", commits.len());
        }
        per_repo.push(commit::aggregate(&commits));
    }

    let merged = commit::merge_maps(per_repo);

    let html = render::render(&merged, since, until, &repos);

    fs::write(&args.output, html)
        .with_context(|| format!(
            "falha ao escrever {}", args.output.display()
        ))?;

    println!("escrito em {}", args.output.display());

    Ok(())
}

fn run_stats(
    args: &cli::Args,
    repos: &[std::path::PathBuf],
    since: chrono::NaiveDate,
    until: chrono::NaiveDate,
) -> Result<()> {
    let mut total: BTreeMap<(String, String), u32> = BTreeMap::new();

    for r in repos {
        let stats = commit::author_stats(r, since, until, &args.authors)?;
        for (email, name, n) in stats {
            if args.verbose {
                eprintln!("  {n:>6}  {name} <{email}>");
            }
            *total.entry((email, name)).or_insert(0) += n;
        }
    }

    let mut v: Vec<_> = total
        .into_iter()
        .map(|((e, n), c)| (e, n, c))
        .collect();

    v.sort_by(|a, b| b.2.cmp(&a.2).then_with(|| a.0.cmp(&b.0)));

    let total_commits: u32 = v.iter().map(|(_, _, c)| c).sum();

    let email_w = width_for(
        v.iter().map(|(e, _, _)| display_width(e)),
        display_width("email"),
        60,
    );
    let name_w = width_for(
        v.iter().map(|(_, n, _)| display_width(n)),
        display_width("name"),
        60,
    );
    let commits_w = 8;

    eprintln!();
    eprintln!("janela: {since} ..= {until}");
    eprintln!("repos: {}", repos.len());
    eprintln!("commits: {total_commits}");
    eprintln!();

    eprintln!(
        "{} {} {:>width$}",
        pad_or_truncate("email", email_w),
        pad_or_truncate("name", name_w),
        "commits",
        width = commits_w
    );

    eprintln!("{:-<email_w$} {:-<name_w$} {:->commits_w$}", "", "", "");

    for (email, name, c) in &v {
        eprintln!(
            "{} {} {:>width$}",
            pad_or_truncate(email, email_w),
            pad_or_truncate(name, name_w),
            c,
            width = commits_w
        );
    }

    Ok(())
}

fn display_width(s: &str) -> usize {
    s.chars().count()
}

fn width_for(
    items: impl Iterator<Item = usize>, header: usize, cap: usize
) -> usize {
    items
        .chain(std::iter::once(header))
        .max()
        .unwrap_or(0)
        .min(cap)
}

fn pad_or_truncate(s: &str, width: usize) -> String {
    let n = display_width(s);
    if n <= width {
        format!("{s}{}", " ".repeat(width - n))
    } else {
        let take = width.saturating_sub(1);
        let mut t: String = s.chars().take(take).collect();
        t.push('…');
        t
    }
}
