// essa crate é parte do standad library e é para trabalhar com 
// "caminho" de arquivo. Com ele eu nao
// preciso lidar com separadores de arquivo
// ou seja eu nao preciso lidar com a diferenca
// de '/' no linux e '\' no windows, o negoço só
// funciona e tem metodos proprios para lidar com
// erros 
use std::path::PathBuf;

// o chrono é uma dep para trabalhar com
// data/hora. O 'Local' sendo importado 
// serve para lidar com fuso horario
// automaticamente e 'NaiveDate' é o formato de
// data pura sem fuso horario
use chrono::{Local, NaiveDate};

// outra dep que serve pra criar CLI
// o Parser é um trait que em conjunto derive pede pro compilador gerar o codigo que faz a leitura, tratamento
// das flags e valida os tipos e cria valores default,
// ele tambem mostra os comentarios dos acima dos args
// de maneira amigavel, sem isso eu teria que escrever tudo manualmente
use clap::Parser;

/// arinn — gera um grafo estilo GitHub a partir dos seus repositórios locais.
// comentario de explicaçao de syntax:
// 'value_name' é o que printado quando o user chama --help.
// 'long' cria a flag com nome completo (--exemplo)
// 'short' cria flag com redutivo (-e)
//
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

    /// Filtra commits por autor (substring ou regex). Repetível.
    #[arg(long, value_name = "PATTERN")]
    pub authors: Vec<String>,

    /// Mostra stats de autores por repo (em stderr) e sai sem gerar HTML.
    #[arg(long)]
    pub stats: bool,

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
