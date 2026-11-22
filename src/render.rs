use std::collections::BTreeMap;
use std::fmt::Write;
use std::path::PathBuf;

use chrono::{Datelike, Duration, NaiveDate};

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
    daily_commit_count: &BTreeMap<NaiveDate, u32>,
    since: NaiveDate,
    until: NaiveDate,
    repos: &[PathBuf],
) -> String {
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

    let repos_html = render_repos_panel(repos);

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
  .wrap {{ display: inline-block; }}
</style>
</head>
<body>
  <h1>arinn — {total_commits} commits</h1>
  {repos_html}<div class="wrap">{svg_content}</div>
</body>
</html>
"##
    )
}

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

fn render_repos_panel(repos: &[PathBuf]) -> String {
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

        writeln!(
            repo_items,
            r##"<li title="{escaped_path}">{escaped_name}</li>"##
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

            writeln!(
                svg_content,
                r##"<rect class="cell" x="{cell_x}" y="{cell_y}" width="{CELL_SIZE}" height="{CELL_SIZE}" rx="2" ry="2" fill="{cell_color}"><title>{commit_count} {commit_label} em {current_date}</title></rect>"##
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
