use core::fmt::Write;
use std::collections::BTreeMap;
use time::Date;

use crate::summary::{SeriesSummary, Summary, WeeklyEpisode};

macro_rules! write_safe {
    ($dst:expr, $($arg:tt)*) => {
        if let Err(oops) = write!($dst, $($arg)*) {
            panic!("{}", oops);
        }
    };
}

pub(crate) fn to_markdown_github(summary: &Summary) -> String {
    let mut output = String::new();
    output.push_str(&to_markdown_this_week(&summary.this_week));
    output.push_str(&to_markdown_all_table(&summary.all));
    output
}

fn to_markdown_this_week(this_week: &BTreeMap<Date, Vec<WeeklyEpisode>>) -> String {
    let mut output = String::from("# This Week\n\n");
    for (date, eps) in this_week {
        output.push_str(&to_markdown_day(date, eps));
    }
    output.push('\n');
    output
}

fn to_markdown_day(date: &Date, eps: &Vec<WeeklyEpisode>) -> String {
    let mut output = format!("## {} ({})\n", date.weekday(), date);
    for ep in eps {
        write_safe!(
            output,
            "- {} {}x{}\n",
            ep.series_name,
            ep.season,
            ep.episode
        )
    }
    output
}

fn to_markdown_all_table(all: &Vec<SeriesSummary>) -> String {
    let mut output = String::from("# Currently Watching\n\n");
    output.push_str("| Series | Last | Next |\n");
    output.push_str("| --- | --- | --- |\n");
    for series in all {
        write_safe!(
            output,
            "| {} | {} | {} |\n",
            series.name,
            series
                .last_air
                .map(|d| d.to_string())
                .unwrap_or_else(|| "?".to_string()),
            series
                .next_air
                .map(|d| d.to_string())
                .unwrap_or_else(|| "?".to_string()),
        )
    }
    output
}
