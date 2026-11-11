use std::path::PathBuf;

use chrono::{Local, NaiveDate};
use clap::Parser;

/// arinn — gera um grafo estilo GitHub a partir dos seus repositórios locais.
#[derive(Parser, Debug)]
#[command(name = "arinn", version, about)]
pub struct Args {
    /// Caminhos para repositórios (aceita também via pipe no stdin).
    #[arg(value_name = "PATHS")]
    pub paths: Vec<PathBuf>,

    /// Início da janela (YYYY-MM-DD). Padrão: hoje - 365 dias.
    #[arg(long, value_name = "YYYY-MM-DD")]
    pub since: Option<NaiveDate>,

    /// Fim da janela (YYYY-MM-DD). Padrão: hoje.
    #[arg(long, value_name = "YYYY-MM-DD")]
    pub until: Option<NaiveDate>,

    /// Arquivo HTML de saída.
    #[arg(short, long, value_name = "FILE", default_value = "arinn.html")]
    pub output: PathBuf,

    /// Lista cada repositório descoberto.
    #[arg(short, long)]
    pub verbose: bool,
}

pub fn parse() -> Args {
    Args::parse()
}

pub fn resolve_window(args: &Args) -> (NaiveDate, NaiveDate) {
    let until = args.until.unwrap_or_else(|| Local::now().date_naive());
    let since = args
        .since
        .unwrap_or_else(|| until - chrono::Duration::days(365));
    (since, until)
}
