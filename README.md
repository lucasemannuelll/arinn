# arinn

Uma CLI em Rust que lê o histórico de commits de um ou mais repositórios Git locais e gera um gráfico de contribuições no estilo GitHub em HTML/SVG (tema escuro), ou mostra estatísticas por autor direto no terminal.

## O que é

O **arinn** junta commits de vários repositórios Git dentro de um intervalo de datas e oferece dois modos de saída:

- **Grafo HTML/SVG (padrão):** gera um arquivo HTML (por padrão `arinn.html`) com um mapa de calor semanal (domingo a sábado) em tema escuro, tooltip por dia (só para dias dentro da janela), legenda e um painel recolhível (`<details>`) com os repositórios analisados e seus totais.
- **Estatísticas de autores (`--stats`):** imprime no `stderr` uma tabela com `email`, `name`, `commits`, `+add` e `−del`, ordenada por número de commits (desempate por email). Respeita o `.mailmap` (`%aE` e `%aN` no `git log`) e ajusta a largura das colunas dinamicamente (limite de 60 caracteres para email/nome, com truncamento por `…`).

### Recursos do grafo HTML

- **Mapa de calor semanal** de domingo a sábado, com escala de 5 níveis baseada na contagem de commits do dia: `0`, `1–3`, `4–9`, `10–19`, `20+` (paleta GitHub dark).
- **Tooltip por dia** (`<title>`) mostrando a contagem de commits e a data. Só é gerado para dias dentro da janela `--since..=--until`; os dias de preenchimento do grid (fora da janela) não têm tooltip e não são clicáveis.
- **Painel lateral fixo** (`#arinn-panel`) que abre ao clicar em qualquer dia com commits, listando os commits daquele dia agrupados por repositório: hora, hash curto (7 caracteres), mensagem (truncada em 120 caracteres) e autor. Mostra no máximo 50 commits por repo e indica quantos sobraram. Fecha com o botão `×`, com `Esc` ou clicando fora.
- **Painel recolhível de repositórios** (`<details>`) listando cada repo com contagem de commits e diff acumulado (`+add/−del`).
- **HTML auto-contido:** os `DayBucket`s vão embutidos como JSON em `<script type="application/json" id="arinn-data">`, então o arquivo funciona offline, sem servidor e sem `fetch`.

### Regras de agregação

- Commits de **merge** (`parents.len() > 1`) contam para o total de commits, mas **não somam linhas** (`insertions`/`deletions`) no `--stats` nem no grafo.
- A janela de datas é aplicada duas vezes: no `git log` (`--since`/`--until`) e de novo no filtro em Rust sobre `commit_date`, garantindo que só entrem commits dentro de `since..=until`.

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

O binário otimizado fica em `target/release/arinn`. Você também pode rodar direto com `cargo run -- ...` ou instalar localmente com:

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

Os caminhos são normalizados (inclusive removendo o sufixo `.git`), validados como diretórios/repositórios Git e deduplicados por caminho canônico.

### Filtrar por período, autor ou mudar o arquivo de saída

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

A saída vai para o `stderr` neste formato:

```
janela: 2025-11-23 ..= 2026-11-23
repos: 2
commits: 1234

email                          name                   commits     +add     −del
------------------------------ ---------------------- -------- -------- --------
alice@exemplo.com              Alice                    800    12000     3000
lucas@exemplo.com              Lucas                    434     5000     1200
```

Para ver o detalhamento por repositório antes do total consolidado, combine com `--verbose`:

```bash
arinn ~/projetos/repo1 ~/projetos/repo2 --stats --verbose
```

No modo `--stats --verbose`, cada repositório tem suas linhas por autor impressas no `stderr` antes da tabela consolidada.

### Opções da CLI

- `<PATHS>...`: caminhos para repositórios Git (também aceitos via pipe no `stdin`).
- `--since <YYYY-MM-DD>`: início da janela de datas (padrão: `hoje - 365 dias`).
- `--until <YYYY-MM-DD>`: fim da janela de datas (padrão: `hoje`).
- `-o, --output <FILE>`: arquivo HTML de saída (padrão: `arinn.html`).
- `--authors <PATTERN>`: filtra commits por autor; o padrão é repassado ao `git log --author` (regex suportada pelo Git). Pode ser repetida; múltiplas ocorrências viram filtros do `git log`.
- `--stats`: mostra estatísticas de autores no `stderr` e encerra sem gerar HTML.
- `-v, --verbose`: lista repositórios descobertos, janela de datas e contagens durante a execução; no modo `--stats`, também imprime as linhas por repositório antes do total consolidado.
- `-h, --help` e `-V, --version`: exibem ajuda e versão.

## Comportamento em caso de falha

Se o `git log` falhar em um repositório (por exemplo, diretório que não é repositório Git ou erro de invocação), o arinn imprime a mensagem de erro no `stderr` e **trata aquele repositório como tendo zero commits**, seguindo com os demais. Se nenhum repositório válido for encontrado, o programa encerra com erro.
