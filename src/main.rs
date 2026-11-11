mod cli;
mod repo;
mod input;

use std::io::IsTerminal;

use anyhow::Result;

fn main() -> Result<()> {
    let args = cli::parse();

    let stdin_is_pipe = !std::io::stdin().is_terminal();
    let repos = input::resolve_paths(&args, stdin_is_pipe)?;

    if repos.is_empty() {
        anyhow::bail!("nenhum repositório válido encontrado");
    }

    if args.verbose {
        for r in &repos {
            println!("{}", r.display());
        }
    }

    println!("Encontrados {} repos.", repos.len());
    Ok(())
}
