mod cli;
mod commit;
mod input;
mod render;
mod repo;

use std::collections::BTreeMap;
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

    std::fs::write(&args.output, html)
        .with_context(|| format!("falha ao escrever {}", args.output.display()))?;

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

    let mut v: Vec<_> = total.into_iter().map(|((e, n), c)| (e, n, c)).collect();
    v.sort_by(|a, b| b.2.cmp(&a.2).then_with(|| a.0.cmp(&b.0)));

    let total_commits: u32 = v.iter().map(|(_, _, c)| c).sum();

    eprintln!();
    eprintln!("janela: {since} ..= {until}");
    eprintln!("repos: {}", repos.len());
    eprintln!("commits: {total_commits}");
    eprintln!();
    eprintln!("{:<32} {:<24} {:>8}", "email", "name", "commits");
    eprintln!("{:-<32} {:-<24} {:->8}", "", "", "");
    for (email, name, c) in &v {
        eprintln!("{:<32} {:<24} {:>8}", email, name, c);
    }

    Ok(())
}
