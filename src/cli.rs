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
// o Parser é um trait que em conjunto derive pede pro 
// compilador gerar o codigo que faz a leitura, tratamento
// das flags e valida os tipos e cria valores default,
// ele tambem mostra os comentarios dos acima dos args
// de maneira amigavel, sem isso eu teria que escrever tudo manualmente
use clap::Parser;

/// arinn — gera um grafo estilo GitHub a partir dos  seus repositórios locais.
// ----------------------------------
// comentario de explicaçao de syntax:
// 'value_name' é o que printado quando o user chama --help.
// 'long' cria a flag com nome completo (--exemplo)
// 'short' cria flag com redutivo (-e)
// sem 'short'/long significa argumento repetivel
#[derive(Parser, Debug)]
#[command(name = "arinn", version, about)]
pub struct Args {
    /// Caminhos para repositórios (aceita também via pipe no stdin).
    #[arg(value_name = "PATHS")]
    pub paths: Vec<PathBuf>, // Vetor de paths, ou seja da para colocar varios caminhos

    /// Início da janela (YYYY-MM-DD). Padrão: hoje - 365 dias.
    #[arg(long, value_name = "YYYY-MM-DD")]
    pub since: Option<NaiveDate>, // Option<...> = Pode ou nao ter alguma coisa

    /// Fim da janela (YYYY-MM-DD). Padrão: hoje.
    #[arg(long, value_name = "YYYY-MM-DD")]
    pub until: Option<NaiveDate>,

    /// Arquivo HTML de saída.
    #[arg(short, long, value_name = "FILE", default_value = "arinn.html")]
    pub output: PathBuf, // default_value faz com sempre tenha um valor, por isso nao tem Option<...>

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

// acessa Args por referecia/endereço
// a funcao retorna uma tupla 
// de NaiveDate, oq é o fim e 
// o inicio sem informaçao de fuso
pub fn resolve_window(args: &Args) -> (NaiveDate, NaiveDate) {
    // acessa o campo until de args e checa com
    // unwrap... se ele tem valor, ou seja se o user 
    // passou o argumento until, se sim ele atribui a variavel
    // se nao, ele usa uma clojure (uma funca anonima usado uma vez)
    // essa funcao nao tem param (||) e corpo significa:
    // Local::now() - pega data e hora atual com fuso local
    // .data_naive() descarta hora/min/seg e fica com data tornando
    // em NaiveDate par poder retornar
    let until = args.until.unwrap_or_else(|| Local::now().date_naive());

    // mesma logica inicial ate chegar na clojure
    // mas se no caso since no for informada
    // chrono::Duration::days(365) cria um perido de 365 dias
    // e a gente subtrai isso de until ou seja
    // a gente pega sempre um periodo de 1 ano caso por padrao
    // pra criar o grafo
    let since = args
        .since
        .unwrap_or_else(|| until - chrono::Duration::days(365));

    // em rust a ultima expressao em ';'
    // é o return da funcao
    (since, until)
}
