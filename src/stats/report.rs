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
#[derive(Tabled)]
struct DetectedAttribute {
    pub attribute: String,
    pub detected: String,
}

pub struct Report {
    pub(crate) stats: Vec<StatSummary>,
    filter_stats_table: Option<Table>,
    detected_attributes: Vec<DetectedAttribute>,
    processing_time: std::time::Duration,
    fatal_error: Option<String>,
    report_table: Option<Table>,
}
impl Report {
    pub fn new(processing_time: std::time::Duration) -> Self {
        Self {
            stats: Vec::new(),
            detected_attributes: Vec::new(),
            processing_time,
            filter_stats_table: None,
            fatal_error: None,
            report_table: None,
        }
    }
    pub fn add_filter_stats(&mut self, filter_stats_table: Table) {
        self.filter_stats_table = Some(filter_stats_table);
    }
    pub fn add_stat(&mut self, stat: StatSummary) {
        self.stats.push(stat);
    }
    pub fn add_detected_attribute(&mut self, attribute: String, detected: String) {
        self.detected_attributes.push(DetectedAttribute {
            attribute,
            detected,
        });
    }
    pub fn add_fatal_error(&mut self, error: String) {
        self.fatal_error = Some(error);
    }
    pub fn print(&mut self) {
        let mut global_stats_table = Table::new(&self.stats);
        global_stats_table = format_global_stats_sub_table(&global_stats_table);
        let mut detected_attributes_table = Table::new(&self.detected_attributes);
        detected_attributes_table = format_sub_table(
            &detected_attributes_table,
            "Detected Attributes".to_string(),
            SubtableColor::Yellow,
        );

        if self.filter_stats_table.is_some() {
            let filter_stats_table = format_sub_table(
                self.filter_stats_table.as_ref().unwrap(),
                "Filter Stats".to_string(),
                SubtableColor::Purple,
            );
            let multi_table = tabled::col![
                global_stats_table,
                tabled::row![detected_attributes_table, filter_stats_table]
            ];
            self.report_table = Some(format_super_table(&multi_table, self.processing_time));
        } else {
            let multi_table =
                tabled::col![global_stats_table, tabled::row![detected_attributes_table]];
            self.report_table = Some(format_super_table(&multi_table, self.processing_time));
        }
        if self.fatal_error.is_some() {
            let mut error_table = self.report_table.clone().unwrap();
            error_table
                .with(Panel::header("FATAL ERROR - EARLY TERMINATION"))
                .with(
                    Modify::new(Rows::single(0))
                        .with(Alignment::center())
                        .with(Format::new(|x| {
                            let x = x.to_uppercase();
                            x.red().to_string()
                        })),
                );
            self.report_table = Some(error_table);
        }
        eprintln!("{}", self.report_table.as_ref().unwrap());
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

#[allow(dead_code)]
enum SubtableColor {
    Purple,
    Green,
    Blue,
    Yellow,
    Red,
}
/// Formats a subtable to use the same style as the main table
/// Adds a header to the subtable in all caps, purple, and aligned center
fn format_sub_table(subtable: &Table, header: String, color: SubtableColor) -> Table {
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
                match color {
                    SubtableColor::Purple => x.bright_purple().to_string(),
                    SubtableColor::Green => x.green().to_string(),
                    SubtableColor::Blue => x.blue().to_string(),
                    SubtableColor::Yellow => x.yellow().to_string(),
                    SubtableColor::Red => x.red().to_string(),
                }
            })),
    );
    modded_subtable.with(
        Modify::new(Rows::single(1)).with(Format::new(|x| match color {
            SubtableColor::Purple => x.bright_purple().to_string(),
            SubtableColor::Green => x.green().to_string(),
            SubtableColor::Blue => x.blue().to_string(),
            SubtableColor::Yellow => x.yellow().to_string(),
            SubtableColor::Red => x.red().to_string(),
        })),
    );

    modded_subtable
}

#[cfg(test)]
mod tests {
    use super::*;

    // macro_rules! assert_stdout_eq {
    //     ($test:expr, $expected:literal) => {{
    //         use gag::BufferRedirect;
    //         use std::io::Read;

    //         let mut buf = BufferRedirect::stdout().unwrap();

    //         $test;

    //         let mut output = String::new();
    //         buf.read_to_string(&mut output).unwrap();
    //         drop(buf);

    //         assert_eq!(&output[..], $expected);
    //     }};
    // }

    macro_rules! assert_stderr_contains {
        ($test:expr, $expected:literal) => {{
            use gag::BufferRedirect;
            use std::io::Read;

            let mut buf = BufferRedirect::stderr().unwrap();

            $test;

            let mut output = String::new();
            buf.read_to_string(&mut output).unwrap();
            drop(buf);

            assert!(output.contains($expected));
        }};
    }

    #[test]
    fn test_summary_contains_filtered_links_rdhs() {
        let processing_time = std::time::Instant::now();
        let mut report = Report::new(processing_time.elapsed());
        let statistic_tot_errs = "Total errors".to_string();
        report.add_stat(StatSummary::new(statistic_tot_errs, "0".to_string(), None));
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
        assert_stderr_contains!(report.print(), "Filtered links");
        assert_stderr_contains!(report.print(), "Total RDHs");
        assert_stderr_contains!(report.print(), "725800");
    }

    #[test]
    fn test_fatal_error_report() {
        let processing_time = std::time::Instant::now();
        let fatal_error = "Fatal Error happened";
        let mut report = Report::new(processing_time.elapsed());
        report.add_fatal_error(fatal_error.to_string());

        assert_stderr_contains!(report.print(), "FATAL ERROR");
    }
}
