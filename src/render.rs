use std::collections::BTreeMap;
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

// pub fn render (data: &BTreeMap<NaiveDate, u32>, since: NaiveDate, until: NaiveDate, repo_count: usize) -> String { ... }

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
                r#"<text x="{x}" y="{y}" fill="#57606a">{}</text>"#,
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
            r#"<text x="{x}" y="{y}" fill="#57606a" text-anchor="end">{name}</text>"#
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
            let plural = if n = 1 { "commit" } else { "commits" };
            writeln!(
                out,
                r#"<rect x="{x}" y="{y}" width="{CELL}" height="{CELL}" rx="2" ry="2" fill="{fill}"><title>{n} {plural} em {day}</title></rect>"#
            ).unwrap();
        } else {
            writeln!(
                out, 
                r#"<rect x="{x}" y="{y}" width="{CELL}" height="{CELL}" rx="2" ry="2" fill="{fill}"/>"#
            ).unwrap();
        }

        day += Duration::days(1);
        idx += 1;
    }
}
