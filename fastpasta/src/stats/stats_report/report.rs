/// The Report struct is used by the StatsController to structure the report printed at the end of execution
///
/// Report contains several StatSummary structs that are used to generate the report table
use tabled::{
    settings::{object::Rows, Alignment, Format, Modify, Panel},
    Table, Tabled,
};

use super::table_formatter_utils::format_global_stats_sub_table;
use super::table_formatter_utils::format_sub_table;
use super::table_formatter_utils::format_super_table;
use super::table_formatter_utils::SubtableColor;
use owo_colors::OwoColorize;

/// Describes the columns of the report table
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
/// Describes the columns of the detected attributes table
#[derive(Tabled)]
struct DetectedAttribute {
    pub attribute: String,
    pub detected: String,
}

/// The Report struct is used by the StatsController to structure the report printed at the end of execution
///
/// Contains convenience methods to add stats to the report, and to generate the report table
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
        format_global_stats_sub_table(&mut global_stats_table);
        let mut detected_attributes_table = Table::new(&self.detected_attributes);

        detected_attributes_table = format_sub_table(
            detected_attributes_table,
            "Detected Attributes".to_string(),
            SubtableColor::Yellow,
        );

        if self.filter_stats_table.is_some() {
            let filter_stats_table = format_sub_table(
                self.filter_stats_table.take().unwrap(),
                "Filter Stats".to_string(),
                SubtableColor::Purple,
            );
            let mut multi_table = tabled::col![
                global_stats_table,
                tabled::row![detected_attributes_table, filter_stats_table]
            ];
            let multi_table = multi_table.with(tabled::settings::Style::rounded());
            self.report_table = Some(format_super_table(multi_table, self.processing_time));
        } else {
            let mut multi_table =
                tabled::col![global_stats_table, tabled::row![detected_attributes_table]];
            let multi_table = multi_table.with(tabled::settings::Style::rounded());
            self.report_table = Some(format_super_table(multi_table, self.processing_time));
        }

        if self.fatal_error.is_some() {
            let mut error_table = self.report_table.clone().unwrap();
            let _ = error_table
                .with(Panel::header("FATAL ERROR - EARLY TERMINATION"))
                .with(Modify::new(Rows::single(0)).with(Alignment::center()).with(
                    Format::content(|x| {
                        let x = x.to_uppercase();
                        x.red().to_string()
                    }),
                ));
            self.report_table = Some(error_table);
        }
        println!(
            "{final_report}",
            final_report = self.report_table.as_ref().unwrap()
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! assert_stdout_contains {
        ($test:expr, $expected:literal) => {{
            use gag::BufferRedirect;
            use std::io::Read;

            let mut buf = BufferRedirect::stdout().unwrap();

            $test;

            let mut output = String::new();
            let _ = buf.read_to_string(&mut output).unwrap();
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
        assert_stdout_contains!(report.print(), "Filtered links");
        assert_stdout_contains!(report.print(), "Total RDHs");
        assert_stdout_contains!(report.print(), "725800");
    }

    #[test]
    fn test_summary_contains_filtered_links_rdhs_windows() {
        let filtered_links = StatSummary::new(
            "Filtered links".to_string(),
            String::from("0"),
            Some(String::from("Not found: 2")),
        );
        let observed_links =
            StatSummary::new("Links observed".to_string(), "1 7 8 9".to_string(), None);

        let mut filter_table = Table::new(vec![filtered_links, observed_links]);
        println!("before:\n{filter_table}");
        format_global_stats_sub_table(&mut filter_table);
        println!("After:\n{filter_table}");
        assert_stdout_contains!(println!("{filter_table}"), "Filtered links");
    }

    #[test]
    fn test_fatal_error_report() {
        let processing_time = std::time::Instant::now();
        let fatal_error = "Fatal Error happened";
        let mut report = Report::new(processing_time.elapsed());
        report.add_fatal_error(fatal_error.to_string());

        assert_stdout_contains!(report.print(), "FATAL ERROR");
    }

    #[test]
    fn stats_summar_default() {
        let stats_summary = StatSummary::default();

        assert_eq!(stats_summary.statistic, "");
        assert_eq!(stats_summary.value, "");
        assert_eq!(stats_summary.notes, "");
    }

    #[test]
    fn test_os() {
        println!("{}", std::env::consts::OS);
    }
}
