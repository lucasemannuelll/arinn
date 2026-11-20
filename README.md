# arinn

Ferramenta de linha de comando (CLI) escrita em Rust que lê o histórico de commits de um ou mais repositórios Git locais e gera um gráfico de contribuições estilo GitHub em HTML/SVG (tema dark) ou exibe estatísticas por autor no terminal.

## O que é

O **arinn** agrega commits de múltiplos repositórios Git dentro de um intervalo de datas e oferece dois modos principais de saída:

- **Grafo HTML/SVG (padrão):** gera um arquivo HTML (padrão `arinn.html`) contendo um mapa de calor semanal (alinhado de domingo a sábado) com escala de cores em tema dark, tooltips por dia, legenda e painel recolhível (`<details>`) com os repositórios analisados.
- **Estatísticas de autores (`--stats`):** imprime no `stderr` uma tabela com `email`, `name` e contagem de `commits` ordenada por volume, respeitando o `.mailmap` (`%aE` e `%aN` no `git log`) e ajustando dinamicamente a largura das colunas (com limite de 60 caracteres e truncamento com `…`).

Principais recursos observados no projeto:
- Recebe caminhos de repositórios via argumentos posicionais ou via `stdin` (pipe), normalizando caminhos (inclusive removendo sufixo `.git`), validando diretórios/repositórios Git e deduplicando por caminho canônico.
- Filtra commits por janela de datas (`--since` e `--until`, padrão dos últimos 365 dias até hoje) e por autor (`--authors`, repetível).

## Como instalar

### Pré-requisitos

- **Rust** e **Cargo**
- **Git** disponível no `PATH` do sistema

### Compilando a partir do código-fonte

```bash
git clone <url-do-repositorio>
cd arinn
cargo build --release

```

O binário otimizado ficará disponível em `target/release/arinn`. Você também pode executá-lo diretamente com `cargo run -- ...` ou instalá-lo localmente com:

```bash
cargo install --path .

```

## Como usar

### Gerar o gráfico HTML

Passe um ou mais caminhos de repositórios:

```bash
arinn /caminho/para/repo1 /caminho/para/repo2

```

Ou envie a lista de caminhos via pipe (`stdin`):

```bash
find ~/projetos -maxdepth 2 -name ".git" | arinn

```

### Filtrar por período, autor ou alterar o arquivo de saída

```bash
arinn ~/projetos/meu-repo \
  --since 2026-01-01 \
  --until 2026-11-20 \
  --authors "Lucas" \
  --authors "alice@exemplo.com" \
  --output relatorio.html \
  --verbose

```

### Ver estatísticas de autores no terminal (`--stats`)

```bash
arinn ~/projetos/repo1 ~/projetos/repo2 --stats

```

Para ver detalhes de cada repositório antes do total consolidado, combine com `--verbose`:

```bash
arinn ~/projetos/repo1 ~/projetos/repo2 --stats --verbose

```

### Opções da CLI

* `<PATHS>...`: caminhos para repositórios Git (também aceitos via pipe no `stdin`).
* `--since <YYYY-MM-DD>`: início da janela de datas (padrão: `hoje - 365 dias`).
* `--until <YYYY-MM-DD>`: fim da janela de datas (padrão: `hoje`).
* `-o, --output <FILE>`: arquivo HTML de saída (padrão: `arinn.html`).
* `--authors <PATTERN>`: filtra commits por autor (substring ou regex suportada pelo `git log --author`; pode ser repetida).
* `--stats`: exibe estatísticas de autores no `stderr` e encerra sem gerar HTML.
* `-v, --verbose`: lista repositórios descobertos, janela de datas e contagens durante a execução.
* `-h, --help` e `-V, --version`: exibem ajuda e versão.
