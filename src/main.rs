mod cli;
mod repo;
mod input;
mod commit;

use std::io::IsTerminal;

use anyhow::Result;

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

    let mut per_repo: Vec<commit::BTreeMap<chrono::NaiveDate, u32>> = Vec::new();
    for r in &repos {
        let commits = commit::read_commits(r, since, until)?;
        if args.verbose {
            eprintln!("  {} commits", commits.len());
        }
        per_repo.push(commit::aggregate(&commits));
    }

    let merged = commit::merge_maps(per_repo);

    let total: u32 = merged.values().sum();
    println!("Total: {total} commits em {} dias distintos", merged.len());
    if args.verbose {
        for (d, n) in &merged {
            println!("{d}: {n}")
        }
    }

    Ok(())
}
