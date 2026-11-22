use std::collections::BTreeMap;
use std::fmt::Write;
use std::path::PathBuf;

use chrono::{Datelike, Duration, NaiveDate};

use crate::commit::{total_by_day, totals_by_repo, DayBucket, RepoStat};

const CELL_SIZE: i32 = 11;
const GAP_SIZE: i32 = 2;
const STEP: i32 = CELL_SIZE + GAP_SIZE;
const MARGIN_LEFT: i32 = 30;
const MARGIN_TOP: i32 = 20;
const MARGIN_RIGHT: i32 = 10;
const MARGIN_BOTTOM: i32 = 30;

// Paleta GitHub dark
const BG: &str = "#0d1117";
const FG: &str = "#c9d1d9";
const FG_MUTED: &str = "#8b949e";
const BORDER_COLOR: &str = "#30363d";

const CONTRIBUTION_COLORS: [&str; 5] = [
    "#161b22", // 0 — célula vazia
    "#0e4429", // 1–3
    "#006d32", // 4–9
    "#26a641", // 10–19
    "#39d353", // 20+
];

const WEEKDAYS_LABELS: [(usize, &str); 3] = [(1, "Mon"), (3, "Wed"), (5, "Fri")];

const MONTHS: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

pub fn render(
    day_buckets: &[DayBucket],
    since: NaiveDate,
    until: NaiveDate,
    repos: &[PathBuf],
) -> String {
    let daily_commit_count = total_by_day(day_buckets);
    let repo_totals = totals_by_repo(day_buckets);

    let daily_commit_count = &daily_commit_count;
    let repo_totals = &repo_totals;

    let grid_start_date = since - Duration::days(since.weekday().num_days_from_sunday() as i64);
    let days_until_saturday = 6 - until.weekday().num_days_from_sunday() as i64;
    let grid_end_date = until + Duration::days(days_until_saturday);

    let grid_total_days = (grid_end_date - grid_start_date).num_days() as i32 + 1;

    let week_count = (grid_total_days + 6) / 7;

    let grid_width = week_count * STEP - GAP_SIZE;
    let grid_height = 7 * STEP - GAP_SIZE;

    let svg_width = MARGIN_LEFT + grid_width + MARGIN_RIGHT;
    let svg_height = MARGIN_TOP + grid_height + MARGIN_BOTTOM;

    let mut svg_content = String::new();

    writeln!(
        svg_content,
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="{svg_width}" height="{svg_height}" viewBox="0 0 {svg_width} {svg_height}" font-family="-apple-system, BlinkMacSystemFont, 'Segoe UI', Helvetica, Arial, sans-serif" font-size="10">"##
    ).unwrap();

    writeln!(
        svg_content,
        r##"<style>
rect.cell {{ 
  transition: stroke .1s, stroke-width .1s;
}}
rect.cell:hover {{ 
  stroke: #ffffff; 
  stroke-width: 1.5; 
}}
</style>"##
    )
    .unwrap();

    draw_month_labels(&mut svg_content, grid_start_date, week_count);
    draw_day_labels(&mut svg_content);

    draw_cells(
        &mut svg_content,
        daily_commit_count,
        grid_start_date,
        grid_end_date,
        since,
        until,
    );

    draw_legend(
        &mut svg_content,
        svg_width,
        svg_height,
        repos.is_empty(),
        daily_commit_count,
    );

    draw_footer(
        &mut svg_content,
        since,
        until,
        daily_commit_count,
        svg_height,
    );

    svg_content.push_str("</svg>\n");

    let total_commits: u32 = daily_commit_count.values().sum();

    let repos_html = render_repos_panel(repos, repo_totals);

    let panel_css = PANEL_CSS;
    let panel_js = PANEL_JS;
    let data_script_html = render_data_script(day_buckets);

    format!(
        r##"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<title>arinn — {total_commits} commits</title>
<style>
  body {{
    margin: 0;
    padding: 24px;
    background: {BG};
    color: {FG};
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Helvetica, Arial, sans-serif;
  }}
  h1 {{
    font-size: 16px;
    font-weight: 600;
    margin: 0 0 12px;
    color: {FG};
  }}
  .repos {{
    margin: 0 0 20px;
    font-size: 13px;
    color: {FG_MUTED};
  }}
  .repos summary {{
    cursor: pointer;
    color: {FG_MUTED};
    user-select: none;
  }}
  .repos summary:hover {{ color: {FG}; }}
  .repos ul {{
    margin: 8px 0 0;
    padding-left: 20px;
    list-style: disc;
  }}
  .repos li {{
    font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
    font-size: 12px;
    color: {FG_MUTED};
    margin: 2px 0;
  }}
  .repos li .diff {{ color: {FG}; }}
  .wrap {{ display: inline-block; }}
{panel_css}
</style>
</head>
<body>
  <h1>arinn — {total_commits} commits</h1>
  {repos_html}<div class="wrap">{svg_content}</div>
  {data_script_html}
  <aside id="arinn-panel" aria-hidden="true">
    <div class="panel-head">
      <div class="panel-title" id="arinn-panel-title"></div>
      <button id="arinn-panel-close" title="Fechar">×</button>
    </div>
    <div id="arinn-panel-body"></div>
  </aside>
  <script>
{panel_js}
  </script>
</body>
</html>
"##
    )
}

/// Cores do painel lateral (mesma paleta dark do heatmap).
const PANEL_CSS: &str = r##"
  rect.cell.has-commits { cursor: pointer; }
  rect.cell.active { stroke: #ffffff; stroke-width: 1.5; }
  #arinn-panel {
    position: fixed;
    top: 0;
    right: 0;
    height: 100%;
    width: 380px;
    max-width: 90vw;
    background: #161b22;
    border-left: 1px solid #30363d;
    padding: 0 16px 16px;
    box-sizing: border-box;
    overflow-y: auto;
    z-index: 10;
    font-size: 13px;
    transform: translateX(100%);
    visibility: hidden;
    transition: transform .15s ease, visibility .15s;
  }
  #arinn-panel.open {
    transform: translateX(0);
    visibility: visible;
  }
  .panel-head {
    position: sticky;
    top: 0;
    background: #161b22;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    padding: 14px 0 10px;
    border-bottom: 1px solid #30363d;
    margin-bottom: 10px;
  }
  .panel-title {
    font-weight: 600;
    color: #c9d1d9;
    text-transform: capitalize;
  }
  #arinn-panel-close {
    background: none;
    border: 1px solid #30363d;
    color: #8b949e;
    border-radius: 6px;
    width: 24px;
    height: 24px;
    cursor: pointer;
    font-size: 14px;
    line-height: 1;
    flex-shrink: 0;
  }
  #arinn-panel-close:hover { color: #c9d1d9; border-color: #8b949e; }
  .panel-total { color: #8b949e; margin: 0 0 12px; }
  .repo-sub {
    font-weight: 600;
    color: #c9d1d9;
    margin: 12px 0 6px;
    padding-bottom: 4px;
    border-bottom: 1px solid #21262d;
  }
  .repo-lines { color: #8b949e; font-weight: 400; font-size: 12px; }
  .commit-row {
    display: flex;
    gap: 8px;
    padding: 2px 0;
    font-size: 12px;
    color: #c9d1d9;
  }
  .commit-time {
    color: #8b949e;
    font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
    flex-shrink: 0;
  }
  .commit-hash {
    color: #8b949e;
    font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
    flex-shrink: 0;
  }
  .commit-msg { overflow-wrap: anywhere; }
  .commit-more {
    color: #8b949e;
    font-size: 12px;
    font-style: italic;
    padding: 4px 0;
  }
"##;

const PANEL_JS: &str = r##"(function () {
  var data_element = document.getElementById('arinn-data');
  var panel = document.getElementById('arinn-panel');
  if (!data_element || !panel) return;

  var buckets = {};
  JSON.parse(data_element.textContent).forEach(function (bucket) {
    buckets[bucket.date] = bucket;
  });

  var title_element = document.getElementById('arinn-panel-title');
  var body_element = document.getElementById('arinn-panel-body');
  var close_button = document.getElementById('arinn-panel-close');
  var active_cell = null;

  function esc(text) {
    return String(text)
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;')
      .replace(/"/g, '&quot;');
  }

  function truncate(text, max_length) {
    return text.length > max_length
      ? text.slice(0, max_length) + '…'
      : text;
  }

  function close() {
    panel.classList.remove('open');
    panel.setAttribute('aria-hidden', 'true');
    if (active_cell) {
      active_cell.classList.remove('active');
      active_cell = null;
    }
  }

  function open(bucket) {
    var commit_label = bucket.total_commits === 1 ? 'commit' : 'commits';
    var html = '<div class="panel-total">' +
      bucket.total_commits + ' ' + commit_label + ' · +' +
      bucket.total_insertions + '/−' + bucket.total_deletions +
      '</div>';

    Object.keys(bucket.per_repo).forEach(function (repo_name) {
      var repo_stat = bucket.per_repo[repo_name];
      html += '<div class="repo-sub">' + esc(repo_name) +
        ' <span class="repo-lines">+' + repo_stat.insertions +
        '/−' + repo_stat.deletions + '</span></div>';

      repo_stat.commits_detail.slice(0, 50).forEach(function (commit) {
        html += '<div class="commit-row">' +
          '<span class="commit-time">' + esc(commit.time) + '</span>' +
          '<span class="commit-hash">' + esc(commit.hash.slice(0, 7)) + '</span>' +
          '<span class="commit-msg">' + esc(truncate(commit.message, 120)) + '</span>' +
          '</div>';
      });

      var remaining = repo_stat.commits_detail.length - 50;
      if (remaining > 0) {
        html += '<div class="commit-more">+' + remaining + ' commits restantes</div>';
      }
    });

    title_element.textContent = new Date(bucket.date + 'T00:00:00')
      .toLocaleDateString(undefined, {
        weekday: 'long',
        year: 'numeric',
        month: 'long',
        day: 'numeric'
      });
    body_element.innerHTML = html;
    panel.setAttribute('aria-hidden', 'false');
    panel.classList.add('open');
  }

  close_button.addEventListener('click', close);

  document.addEventListener('keydown', function (event) {
    if (event.key === 'Escape') close();
  });

  document.addEventListener('click', function (event) {
    if (!panel.classList.contains('open')) return;
    if (panel.contains(event.target)) return;
    if (event.target.classList && event.target.classList.contains('cell')) return;
    close();
  });

  Array.prototype.forEach.call(
    document.querySelectorAll('rect.cell.has-commits'),
    function (cell) {
      cell.addEventListener('click', function (event) {
        event.stopPropagation();
        if (active_cell) active_cell.classList.remove('active');
        active_cell = cell;
        cell.classList.add('active');
        var bucket = buckets[cell.getAttribute('data-date')];
        if (bucket) open(bucket);
      });
    }
  );
})();"##;

fn html_escape(input: &str) -> String {
    let mut escaped = String::with_capacity(input.len());
    for char in input.chars() {
        match char {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&#39;"),
            _ => escaped.push(char),
        }
    }
    escaped
}

fn render_repos_panel(repos: &[PathBuf], repo_totals: &BTreeMap<String, RepoStat>) -> String {
    if repos.is_empty() {
        return String::new();
    }
    let repo_count = repos.len();
    let summary = if repo_count == 1 {
        "1 repo".to_string()
    } else {
        format!("{repo_count} repos")
    };

    let open_attribute = if repo_count == 1 { " open" } else { "" };

    let mut repo_items = String::new();
    for repo in repos {
        let repo_name = repo
            .file_name()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| repo.display().to_string());

        let escaped_name = html_escape(&repo_name);
        let escaped_path = html_escape(&repo.display().to_string());

        let detail_html = repo_totals
            .get(&repo_name)
            .map(|repo_stat| {
                let commit_label = if repo_stat.commits == 1 {
                    "commit"
                } else {
                    "commits"
                };

                format!(
                    " — {} {commit_label}, <span class=\"diff\">+{}/−{}</span>",
                    repo_stat.commits, repo_stat.insertions, repo_stat.deletions
                )
            })
            .unwrap_or_default();

        writeln!(
            repo_items,
            r##"<li title="{escaped_path}">{escaped_name}{detail_html}</li>"##
        )
        .unwrap();
    }

    format!(
        r##"
<div class="repos">
  <details{open_attribute}>
    <summary>{summary}</summary>
    <ul>{repo_items}</ul>
  </details>
</div>
"##
    )
}

fn contribution_level(commit_count: u32) -> usize {
    match commit_count {
        0 => 0,
        1..=3 => 1,
        4..=9 => 2,
        10..=19 => 3,
        _ => 4,
    }
}

fn draw_month_labels(svg_content: &mut String, grid_start_date: NaiveDate, week_count: i32) {
    let mut previous_month: Option<u32> = None;
    for week in 0..week_count {
        let week_start_date = grid_start_date + Duration::days((week * 7) as i64);

        let month = week_start_date.month0();

        if Some(month) != previous_month {
            let label_x = MARGIN_LEFT + week * STEP;
            let label_y = MARGIN_TOP - 6;
            writeln!(
                svg_content,
                r##"<text x="{label_x}" y="{label_y}" fill="{FG_MUTED}">{}</text>"##,
                MONTHS[month as usize]
            )
            .unwrap();
            previous_month = Some(month);
        }
    }
}

fn draw_day_labels(svg_content: &mut String) {
    for (row, weekday) in WEEKDAYS_LABELS {
        let label_x = MARGIN_LEFT - 6;
        let label_y = MARGIN_TOP + row as i32 * STEP + CELL_SIZE - 2;
        writeln!(
            svg_content,
            r##"<text x="{label_x}" y="{label_y}" fill="{FG_MUTED}" text-anchor="end">{weekday}</text>"##
        )
        .unwrap();
    }
}

fn draw_cells(
    svg_content: &mut String,
    daily_commit_count: &BTreeMap<NaiveDate, u32>,
    grid_start_date: NaiveDate,
    grid_end_date: NaiveDate,
    since: NaiveDate,
    until: NaiveDate,
) {
    let mut current_date = grid_start_date;
    let mut day_offset = 0i32;

    while current_date <= grid_end_date {
        let col = day_offset / 7;
        let row = day_offset % 7;
        let cell_x = MARGIN_LEFT + col * STEP;
        let cell_y = MARGIN_TOP + row * STEP;

        let is_in_date_range = current_date >= since && current_date <= until;

        let commit_count = if is_in_date_range {
            *daily_commit_count.get(&current_date).unwrap_or(&0)
        } else {
            0
        };

        let cell_color = CONTRIBUTION_COLORS[contribution_level(commit_count)];

        if is_in_date_range {
            let commit_label = if commit_count == 1 {
                "commit"
            } else {
                "commits"
            };

            // Dias com commits > 0 são clicáveis (painel lateral);
            // dias vazios não recebem data nem classe de clique.
            let cell_class = if commit_count > 0 {
                format!(r#" class="cell has-commits" data-date="{current_date}""#)
            } else {
                r#" class="cell""#.to_string()
            };

            writeln!(
                svg_content,
                r##"<rect{cell_class} x="{cell_x}" y="{cell_y}" width="{CELL_SIZE}" height="{CELL_SIZE}" rx="2" ry="2" fill="{cell_color}"><title>{commit_count} {commit_label} em {current_date}</title></rect>"##
            ).unwrap();
        } else {
            writeln!(
                svg_content,
                r##"<rect x="{cell_x}" y="{cell_y}" width="{CELL_SIZE}" height="{CELL_SIZE}" rx="2" ry="2" fill="{}"/>"##, CONTRIBUTION_COLORS[0]).unwrap();
        }

        current_date += Duration::days(1);
        day_offset += 1;
    }
}

fn draw_legend(
    svg_content: &mut String,
    svg_width: i32,
    svg_height: i32,
    repos_empty: bool,
    daily_commit_count: &BTreeMap<NaiveDate, u32>,
) {
    let cell_size = CELL_SIZE;
    let cell_gap = 3;
    let legend_width = 5 * (cell_size + cell_gap) - cell_gap;
    let legend_x = svg_width - MARGIN_RIGHT - legend_width - 50;
    let legend_y = svg_height - MARGIN_BOTTOM + 8;

    writeln!(
        svg_content,
        r##"<text x="{legend_x}" y="{}" fill="{FG_MUTED}" text-anchor="end">Less</text>"##,
        legend_y + cell_size - 2
    )
    .unwrap();

    for (i, color) in CONTRIBUTION_COLORS.iter().enumerate() {
        let cell_x = legend_x + 10 + i as i32 * (cell_size + cell_gap);
        writeln!(
            svg_content,
            r##"<rect x="{cell_x}" y="{legend_y}" width="{cell_size}" height="{cell_size}" rx="2" ry="2" fill="{color}" stroke="{BORDER_COLOR}" stroke-width="1"/>"##
        ).unwrap();
    }

    let more_label_x = legend_x + 10 + 5 * (cell_size + cell_gap) + 6;
    writeln!(
        svg_content,
        r##"<text x="{more_label_x}" y="{}" fill="{FG_MUTED}">More</text>"##,
        legend_y + cell_size - 2
    )
    .unwrap();

    let _ = (repos_empty, daily_commit_count);
}

fn draw_footer(
    svg_content: &mut String,
    since: NaiveDate,
    until: NaiveDate,
    daily_commit_count: &BTreeMap<NaiveDate, u32>,
    svg_height: i32,
) {
    let total_commits: u32 = daily_commit_count.values().sum();
    let footer_text = format!("{since} → {until} · {total_commits} commits");
    let footer_x = MARGIN_LEFT;
    let footer_y = svg_height - 6;

    writeln!(
        svg_content,
        r##"<text x="{footer_x}" y="{footer_y}" fill="{FG_MUTED}">{footer_text}</text>"##
    )
    .unwrap();
}

/// Embute os DayBuckets como JSON em `<script type="application/json">`
/// para o JS do painel lateral. O HTML continua 100% auto-contido
/// (sem servidor, sem fetch, funcional offline).
fn render_data_script(day_buckets: &[DayBucket]) -> String {
    if day_buckets.is_empty() {
        return String::new();
    }

    let mut json = String::from("[");

    for (i, bucket) in day_buckets.iter().enumerate() {
        if i > 0 {
            json.push(',');
        }

        json.push_str(&format!(
            "{{\"date\":\"{}\",\"total_commits\":{},\
             \"total_insertions\":{},\"total_deletions\":{},\
             \"per_repo\":{{",
            bucket.date, bucket.total_commits, bucket.total_insertions, bucket.total_deletions
        ));

        for (j, (repo_name, repo_stat)) in bucket.per_repo.iter().enumerate() {
            if j > 0 {
                json.push(',');
            }

            json.push_str(&format!(
                "\"{}\":{{\"commits\":{},\"insertions\":{},\
                 \"deletions\":{},\"commits_detail\":[",
                json_escape(repo_name),
                repo_stat.commits,
                repo_stat.insertions,
                repo_stat.deletions
            ));

            for (k, commit_ref) in repo_stat.commits_detail.iter().enumerate() {
                if k > 0 {
                    json.push(',');
                }

                json.push_str(&format!(
                    "{{\"hash\":\"{}\",\"time\":\"{}\",\
                     \"message\":\"{}\",\"author_email\":\"{}\",\
                     \"author_name\":\"{}\"}}",
                    json_escape(&commit_ref.hash),
                    json_escape(&commit_ref.time),
                    json_escape(&commit_ref.message),
                    json_escape(&commit_ref.author_email),
                    json_escape(&commit_ref.author_name)
                ));
            }

            json.push_str("]}");
        }

        json.push_str("}}");
    }

    json.push(']');

    format!(r##"<script type="application/json" id="arinn-data">{json}</script>"##)
}

/// Escapa uma string para embutir em JSON dentro de um `<script>`:
/// aspas, barras e controles viram sequências `\u…`; `<` e `>` viram
/// `\u003c`/`\u003e` para impedir fechamento antecipado da tag.
fn json_escape(input: &str) -> String {
    let mut escaped = String::with_capacity(input.len());
    for char in input.chars() {
        match char {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            '<' => escaped.push_str("\\u003c"),
            '>' => escaped.push_str("\\u003e"),
            c if (c as u32) < 0x20 => {
                let _ = write!(escaped, "\\u{:04x}", c as u32);
            }
            c => escaped.push(c),
        }
    }
    escaped
}
