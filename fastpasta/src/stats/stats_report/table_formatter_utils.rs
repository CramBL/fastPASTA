use owo_colors::OwoColorize;
use tabled::settings::object::{Columns, Rows};
use tabled::settings::{Alignment, Format, Modify, Panel};
use tabled::Table;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub(crate) enum SubtableColor {
    Purple,
    Green,
    Blue,
    BrightBlue,
    Yellow,
    Red,
}

/// The super table is the table that contains all the other tables
pub(crate) fn format_super_table(
    super_table: &Table,
    processing_time: std::time::Duration,
) -> Table {
    let mut modded_table = super_table.clone();

    let _ = modded_table.with(Panel::header("Report")).with(
        Modify::new(Rows::single(0))
            .with(Alignment::center())
            .with(Format::content(|x| x.to_uppercase().green().to_string())),
    );

    let row_count = modded_table.count_rows();
    let _ = modded_table
        .with(Panel::footer(format!(
            "Processed in {processing_time:.02?}"
        )))
        .with(
            Modify::new(Rows::single(row_count))
                .with(Alignment::center())
                .with(Format::content(|x| x.dimmed().to_string())),
        );
    modded_table
}

pub(crate) fn format_global_stats_sub_table(global_stats_table: &mut Table) {
    let style = tabled::settings::Style::rounded()
        .remove_left()
        .remove_right()
        .remove_top()
        .remove_bottom()
        .remove_vertical()
        .horizontals([tabled::settings::style::HorizontalLine::new(
            1,
            tabled::settings::Style::rounded().get_horizontal(),
        )
        .main(Some('═'))
        .intersection(None)]);

    let _ = global_stats_table
        .with(style.clone())
        .with(Modify::new(Rows::single(0)).with(Format::content(|x| x.to_uppercase())))
        .with(
            Modify::new(Columns::single(0)).with(Format::content(|s| s.bright_blue().to_string())),
        )
        .with(
            Modify::new(Columns::single(1)).with(Format::content(|s| s.bright_cyan().to_string())),
        )
        .with(Modify::new(Columns::new(2..)).with(Format::content(|s| s.yellow().to_string())))
        .with(Panel::header("Global Stats"))
        .with(
            Modify::new(Rows::single(0))
                .with(Alignment::center())
                .with(Format::content(|x| {
                    let x = x.to_uppercase();
                    x.bright_yellow().to_string()
                })),
        );
    let _ = global_stats_table
        .with(style)
        .with(Modify::new(Columns::single(1)).with(Format::content(|s| s.green().to_string())))
        .with(Modify::new(Rows::single(1)).with(Format::content(|s| s.red().to_string())));
}

/// Formats a subtable to use the same style as the main table
/// Adds a header to the subtable in all caps, aligned center, and with the chosen color
pub(crate) fn format_sub_table(subtable: Table, header: String, color: SubtableColor) -> Table {
    let mut modded_subtable = subtable;
    let style = tabled::settings::Style::rounded()
        .remove_left()
        .remove_right()
        .remove_top()
        .remove_bottom()
        .remove_vertical()
        .horizontals([tabled::settings::style::HorizontalLine::new(
            1,
            tabled::settings::Style::rounded().get_horizontal(),
        )
        .main(Some('═'))
        .intersection(None)]);
    let _ = modded_subtable.with(style);
    let _ = modded_subtable.with(Panel::header(header)).with(
        Modify::new(Rows::single(0))
            .with(Alignment::center())
            .with(Format::content(|x| {
                let x = x.to_uppercase();
                match color {
                    SubtableColor::Purple => x.bright_purple().to_string(),
                    SubtableColor::Green => x.green().to_string(),
                    SubtableColor::Blue => x.blue().to_string(),
                    SubtableColor::BrightBlue => x.bright_blue().to_string(),
                    SubtableColor::Yellow => x.yellow().to_string(),
                    SubtableColor::Red => x.red().to_string(),
                }
            })),
    );
    let _ = modded_subtable.with(Modify::new(Rows::single(1)).with(Format::content(
        |x| match color {
            SubtableColor::Purple => x.bright_purple().to_string(),
            SubtableColor::Green => x.green().to_string(),
            SubtableColor::Blue => x.blue().to_string(),
            SubtableColor::BrightBlue => x.bright_blue().to_string(),
            SubtableColor::Yellow => x.yellow().to_string(),
            SubtableColor::Red => x.red().to_string(),
        },
    )));

    modded_subtable
}
