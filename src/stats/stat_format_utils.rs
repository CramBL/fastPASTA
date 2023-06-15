use owo_colors::OwoColorize;

/// Format a size in bytes to human readable.
pub(crate) fn format_data_size(size_bytes: u64) -> String {
    match size_bytes {
        0..=1024 => format!("{} B", size_bytes),
        1025..=1048576 => {
            format!("{:.2} KiB", size_bytes as f64 / 1024_f64)
        }
        1048577..=1073741824 => {
            format!("{:.2} MiB", size_bytes as f64 / 1048576_f64)
        }
        _ => format!("{:.2} GiB", size_bytes as f64 / 1073741824_f64),
    }
}

/// Sort and format links observed
pub(crate) fn format_links_observed(links_observed: &[u8]) -> String {
    links_observed
        .iter()
        .map(|x| x.to_string())
        .collect::<Vec<String>>()
        .join(", ")
}

/// Sort and format layers and staves seen
pub(crate) fn format_layers_and_staves(
    mut layers_staves_seen: Vec<(u8, u8)>,
    mut layers_stave_with_errors: Vec<(u8, u8)>,
) -> String {
    if layers_staves_seen.is_empty() {
        return "none".red().to_string();
    }
    layers_staves_seen.sort();
    layers_stave_with_errors.sort();

    layers_staves_seen
        .iter()
        .enumerate()
        .map(|(i, (layer, stave))| {
            if i > 0 && i % 7 == 0 {
                if layers_stave_with_errors.contains(&(*layer, *stave)) {
                    format!("L{layer}_{stave}\n").red().to_string()
                } else {
                    format!("L{layer}_{stave}\n").white().to_string()
                }
            } else if layers_stave_with_errors.contains(&(*layer, *stave)) {
                format!("L{layer}_{stave} ").red().to_string()
            } else {
                format!("L{layer}_{stave} ").white().to_string()
            }
        })
        .collect::<Vec<String>>()
        .join("")
}

pub(crate) fn format_fee_ids(fee_ids_seen: &[u16]) -> String {
    if fee_ids_seen.is_empty() {
        return "none".red().to_string();
    }
    let mut fee_ids_seen = fee_ids_seen.to_owned();
    fee_ids_seen.sort();
    format_nums_max_lines_width(40, Some(5), &fee_ids_seen)
}

pub(crate) fn format_error_codes(error_codes: &[u8]) -> String {
    error_codes
        .iter()
        .enumerate()
        .map(|(i, code)| {
            if i > 0 && i % 5 == 0 {
                format!("E{code}\n")
            } else {
                format!("E{code} ")
            }
        })
        .collect()
}

/// Generic function to format a list of numbers into a string with a max width and optional max lines.
pub fn format_nums_max_lines_width(max_width: u16, max_lines: Option<u16>, nums: &[u16]) -> String {
    let mut result = String::new();
    let mut num_chars = 0;
    let mut line_count = 0;
    for (i, id) in nums.iter().enumerate() {
        if max_lines.is_some_and(|max_lines| line_count >= max_lines) {
            result.push_str(&format!("... {} more", nums.len() - i).yellow().to_string());
            break;
        }
        // How many characters will this id take up?
        let tmp_num_chars: u16 = id.checked_ilog10().unwrap_or(0) as u16 + 2; // +1 for whitespace and +1 for the first character
        if num_chars + tmp_num_chars > max_width {
            result.push_str(&format!("\n{id} ", id = id).white().to_string());
            log::warn!("Width {}, line count {line_count}", num_chars);
            num_chars = tmp_num_chars;
            line_count += 1;
        } else {
            result.push_str(&format!("{id} ", id = id).white().to_string());
            num_chars += tmp_num_chars;
        }
    }
    result
}
