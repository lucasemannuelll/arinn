use std::{cell, collections::BTreeMap};
use std::fmt::Write;

use chrono::{Datelike, Duration, NaiveDate};

const CELL: i32 = 11;
const GAP: i32 = 2;
const STEP: i32 = CELL + GAP;
const MARGIN_LEFT: i32 = 30;
const MARGIN_TOP: i32 = 20;
const MARGIN_RIGHT: i32 = 10;
const MARGIN_BOTTOM: i32 = 30;

const COLORS: [&str; 5] = [
    "#ebedf0", // 0
    "#9be9a8", // 1–3
    "#40c463", // 4–9
    "#30a14e", // 10–19
    "#216e39", // 20+
];

const WEEKDAYS: [&str; 7] = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
const MONTHS: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun",
    "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

pub fn render (
    data: &BTreeMap<NaiveDate, u32>, 
    since: NaiveDate, 
    until: NaiveDate, 
    repo_count: usize
) -> String {
    let grid_start = since - Duration::days(
        since.weekday().num_days_from_sunday() as i64
    );
    let days_to_sat = 6 - until.weekday().num_days_from_sunday() as i64;
    let grid_end = until + Duration::days(days_to_sat);

    let total_days = (grid_end - grid_start).num_days() as i32 + 1;
    let weeks = (total_days + 5) / 7;

    let grid_w = weeks * STEP - GAP;
    let grid_h = 7 * STEP - GAP;

    let svg_w = MARGIN_LEFT + grid_w + MARGIN_RIGHT;
    let svg_h = MARGIN_TOP + grid_h + MARGIN_BOTTOM;

    let mut svg = String::new();

    writeln!(
        svg,
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="{svg_w}" height="{svg_h}" viewBox="0 0 {svg_w} {svg_h}" font-family="-apple-system, BlinkMacSystemFont, 'Segoe UI', Helvetica, Arial, sans-serif" font-size="10">"##
    ).unwrap();

    draw_month_labels(&mut svg, grid_start, weeks);

    draw_day_labels(&mut svg);

    draw_cells(&mut svg, data, grid_start, grid_end, since, until);

    draw_legend(&mut svg, svg_w, svg_h);
    draw_footer(&mut svg, since, until, repo_count, data, svg_w, svg_h);

    svg.push_str("</svg>\n");

    let total: u32 = data.values().sum();

    format!(
        r##"<!DOCTYPE html>
        <html lang="en">
        <head>
        <meta charset="utf-8">
        <title>arinn — {total} commits</title>
        <style>
            body {{ 
                margin: 0; 
                padding: 24px; 
                background: #fff; 
                color: #24292f; 
            }}

            h1 {{ 
                font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Helvetica, Arial, sans-serif; 
                font-size: 16px; 
                font-weight: 600; 
                margin: 0 0 16px; 
            }}

            .wrap {{ 
                display: inline-block; 
            }}
        </style>
        </head>
        <body>
            <h1>arinn — {total} commits</h1>
            <div class="wrap">{svg}</div>
        </body>
        </html>
        "##
    )
}

fn bucket(n: u32) -> usize {
    match n {
        0 => 0,
        1..=3 => 1,
        4..=9 => 2,
        10..=19 => 3,
        _ => 4,
    }
}

fn draw_month_labels(out: &mut String, grid_start: NaiveDate, weeks: i32) {
    let mut last_month: Option<u32> = None;
    for w in 0..weeks {
        let week_date = grid_start + Duration::days((w * 7) as i64);
        let m = week_date.month0();
        if Some(m) != last_month {
            let x = MARGIN_LEFT + w * STEP;
            let y = MARGIN_TOP - 6;

            writeln!(
                out, 
                r##"<text x="{x}" y="{y}" fill="#57606a">{}</text>"##,
                MONTHS[m as usize]
            ).unwrap();
            last_month = Some(m);
        }
    }
}

fn draw_day_labels(out: &mut String) {
    for (row, name) in [(1usize, "Mon"), (3, "Wed"), (5, "Fri")] {
        let x = MARGIN_LEFT - 6;
        let y = MARGIN_TOP + row as i32 * STEP + CELL - 2;

        writeln!(
            out,
            r##"<text x="{x}" y="{y}" fill="#57606a" text-anchor="end">{name}</text>"##
        ).unwrap();
    }
}

fn draw_cells(
    out: &mut String,
    data: &BTreeMap<NaiveDate, u32>,
    grid_start: NaiveDate,
    grid_end: NaiveDate,
    since: NaiveDate,
    until: NaiveDate,
) {
    let mut day = grid_start;
    let mut idx = 0i32;

    while day <= grid_end {
        let col = idx / 7;
        let row = idx % 7;
        let x = MARGIN_LEFT + col * STEP;
        let y = MARGIN_TOP + row * STEP;

        let in_window = day >= since && day <= until;
        let n = if in_window { *data.get(&day).unwrap_or(&0) } else { 0 };
        let fill = COLORS[bucket(n)];

        if in_window {
            let plural = if n == 1 { "commit" } else { "commits" };
            writeln!(
                out,
                r##"<rect x="{x}" y="{y}" width="{CELL}" height="{CELL}" rx="2" ry="2" fill="{fill}"><title>{n} {plural} em {day}</title></rect>"##
            ).unwrap();
        } else {
            writeln!(
                out, 
                r##"<rect x="{x}" y="{y}" width="{CELL}" height="{CELL}" rx="2" ry="2" fill="{fill}"/>"##
            ).unwrap();
        }

        day += Duration::days(1);
        idx += 1;
    }
}

fn draw_legend(out: &mut String, svg_w: i32, svg_h: i32) {
    let cell = CELL;
    let gap = 3;
    let legeng_w = 5 * (cell + gap) - gap;
    let x0 = svg_w - MARGIN_RIGHT - legeng_w - 50;
    let y0 = svg_h - MARGIN_BOTTOM + 8;

    writeln!(
        out,
        r##"<text x="{x0}" y="{}" fill="#57606a" text-anchor="end">Less</text>"##,
        y0 + cell - 2
    ).unwrap();

    for (i, color) in COLORS.iter().enumerate() {
        let x = x0 + 10 + i as i32 * (cell + gap);
        writeln!(
            out,
            r##"<rect x="{x}" y="{y0}" width="{cell}" height="{cell}" rx="2" ry="2" fill="{color}"/>"##
        ).unwrap();
    }

    let more_x = x0 + 10 + 5 * (cell + gap) + 6;
    writeln!(
        out,
        r##"<text x="{more_x}" y="{}" fill="#57606a">More</text>"##,
        y0 + cell + 2
    ).unwrap();
}

fn draw_footer(
    out: &mut String,
    since: NaiveDate,
    until: NaiveDate,
    repo_count: usize,
    data: &BTreeMap<NaiveDate, u32>,
    svg_w: i32,
    svg_h: i32,
) {
    let total: u32 = data.values().sum();
    let text = format!(
        "{since} → {until} · {repo_count} repos · {total} commits"
    );
    let x = MARGIN_LEFT;
    let y = svg_h - 6;
    writeln!(
        out,
        r##"<text x="{x}" y="{y}" fill="#57606a">{text}</text>"##
    ).unwrap();
    let _ = svg_w; // reservado pra alinhamento futuro
}
