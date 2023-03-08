/// The Report struct is used by the StatsController to structure the report printed at the end of execution
///
/// Report contains several StatSummary structs that are used to generate the report table
use owo_colors::OwoColorize;
use tabled::{
    format::Format,
    object::{Columns, Rows},
    Alignment, Modify, Panel, Table, Tabled,
};
#[derive(Tabled)]
pub struct StatSummary {
    pub statistic: String,
    pub value: String,
    pub notes: String,
}

impl StatSummary {
    pub fn new(statistic: String, value: String, notes: Option<String>) -> Self {
        Self {
            statistic,
            value,
            notes: if let Some(notes) = notes {
                notes
            } else {
                "".to_string()
            },
        }
    }
}
impl std::default::Default for StatSummary {
    fn default() -> Self {
        Self {
            statistic: "".to_string(),
            value: "".to_string(),
            notes: "".to_string(),
        }
    }
}

pub struct Report {
    pub(crate) stats: Vec<StatSummary>,
    filter_stats_table: Option<Table>,
    processing_time: std::time::Duration,
}
impl Report {
    pub fn new(processing_time: std::time::Duration) -> Self {
        Self {
            stats: Vec::new(),
            processing_time,
            filter_stats_table: None,
        }
    }
    pub fn add_filter_stats(&mut self, filter_stats_table: Table) {
        self.filter_stats_table = Some(filter_stats_table);
    }
    pub fn add_stat(&mut self, stat: StatSummary) {
        self.stats.push(stat);
    }
    pub fn print(&self) {
        let table = Table::new(&self.stats);
        let table = format_global_stats_sub_table(&table);

        if self.filter_stats_table.is_some() {
            let filter_stats_table = format_sub_table(
                self.filter_stats_table.as_ref().unwrap(),
                "Filter Stats".to_string(),
            );

            let mut multi_table = tabled::col![table, filter_stats_table];
            multi_table = format_super_table(&multi_table, self.processing_time);
            eprintln!("{multi_table}");
        } else {
            eprintln!("{table}");
        }
    }
}

fn format_super_table(super_table: &Table, processing_time: std::time::Duration) -> Table {
    let mut modded_table = super_table.clone();
    let style = tabled::Style::modern()
        .horizontals([tabled::style::HorizontalLine::new(
            1,
            tabled::Style::modern().get_horizontal(),
        )
        .main(Some('═'))
        .intersection(None)])
        .verticals([tabled::style::VerticalLine::new(
            1,
            tabled::Style::modern().get_vertical(),
        )]);
    modded_table.with(style).with(Panel::header("Report")).with(
        Modify::new(Rows::single(0))
            .with(Alignment::center())
            .with(Format::new(|x| {
                let x = x.to_uppercase();
                x.green().to_string()
            })),
    );
    let height = modded_table.count_rows();
    modded_table
        .with(Panel::footer(format!("Processed in {processing_time:?}")))
        .with(
            Modify::new(Rows::single(height))
                .with(Alignment::center())
                .with(Format::new(|x| x.dimmed().to_string())),
        );
    modded_table
}

fn format_global_stats_sub_table(global_stats_table: &Table) -> Table {
    let mut modded_table = global_stats_table.clone();
    let style = tabled::Style::modern()
        .off_left()
        .off_right()
        .off_top()
        .off_bottom()
        .off_vertical()
        .horizontals([tabled::style::HorizontalLine::new(
            1,
            tabled::Style::modern().get_horizontal(),
        )
        .main(Some('═'))
        .intersection(None)]);

    modded_table
        .with(style)
        .with(Modify::new(Rows::single(0)).with(Format::new(|x| x.to_uppercase())))
        .with(Modify::new(Columns::single(0)).with(Format::new(|s| s.blue().to_string())))
        .with(Modify::new(Columns::single(1)).with(Format::new(|s| s.bright_cyan().to_string())))
        .with(Modify::new(Columns::new(2..)).with(Format::new(|s| s.yellow().to_string())))
        .with(Panel::header("Global Stats"))
        .with(
            Modify::new(Rows::single(0))
                .with(Alignment::center())
                .with(Format::new(|x| {
                    let x = x.to_uppercase();
                    x.bright_yellow().to_string()
                })),
        );
    modded_table
}

/// Formats a subtable to use the same style as the main table
/// Adds a header to the subtable in all caps, purple, and aligned center
fn format_sub_table(subtable: &Table, header: String) -> Table {
    let mut modded_subtable = subtable.clone();
    let style = tabled::Style::modern()
        .off_left()
        .off_right()
        .off_top()
        .off_bottom()
        .off_vertical()
        .horizontals([tabled::style::HorizontalLine::new(
            1,
            tabled::Style::modern().get_horizontal(),
        )
        .main(Some('═'))
        .intersection(None)]);
    modded_subtable.with(style);
    modded_subtable.with(Panel::header(header)).with(
        Modify::new(Rows::single(0))
            .with(Alignment::center())
            .with(Format::new(|x| {
                let x = x.to_uppercase();
                x.purple().to_string()
            })),
    );
    modded_subtable
        .with(Modify::new(Rows::single(1)).with(Format::new(|x| x.bright_purple().to_string())));

    modded_subtable
}

#[cfg(test)]
mod tests {
    use super::*;
    use owo_colors::OwoColorize;
    use tabled::{
        format::Format,
        object::{Columns, Rows},
    };

    #[test]
    fn test_print_filtered_links() {
        let sum = StatSummary::new(
            "Filtered links".to_string(),
            String::from("0"),
            Some(String::from("Not found: 2")),
        );
        let sum2 = StatSummary::new(
            "Links observed".to_string(),
            "1 32 4 5 6 7 8 9".to_string(),
            None,
        );
        let sum_vec = vec![sum, sum2];
        let mut table = Table::new(sum_vec);

        table
            .with(Panel::header("Report"))
            .with(Panel::footer("3 elements"));

        table
            //.with(style)
            .with(Modify::new(Rows::single(1)).with(Format::new(|x| x.to_uppercase())))
            .with(Modify::new(Columns::single(0)).with(Format::new(|s| s.blue().to_string())))
            .with(
                Modify::new(Columns::single(1)).with(Format::new(|s| s.bright_cyan().to_string())),
            )
            .with(Modify::new(Columns::new(2..)).with(Format::new(|s| s.yellow().to_string())));
    }

    #[test]
    fn test_summary() {
        let processing_time = std::time::Instant::now();
        let mut report = Report::new(processing_time.elapsed());
        report.add_stat(StatSummary::new(
            "Total errors".to_string(),
            "0".to_string(),
            None,
        ));
        report.add_stat(StatSummary::new(
            "Total RDHs".to_string(),
            "725800".to_string(),
            None,
        ));
        let filtered_links = StatSummary::new(
            "Filtered links".to_string(),
            String::from("0"),
            Some(String::from("Not found: 2")),
        );
        let observed_links =
            StatSummary::new("Links observed".to_string(), "1 7 8 9".to_string(), None);

        let filter_table = Table::new(vec![filtered_links, observed_links]);
        report.add_filter_stats(filter_table);

        report.print();
    }
}
