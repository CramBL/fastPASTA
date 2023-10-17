use crate::stats::StatType;

/// Helper function to format and report an error in ITS protocol
///
/// Takes in the error string slice and the word slice
/// Adds the memory position to the error string
/// Sends the error to the stats channel
#[inline]
pub(super) fn report_error(
    mem_pos: u64,
    err: &str,
    word_slice: &[u8],
    sender: &flume::Sender<StatType>,
) {
    sender
            .send(StatType::Error(format!(
                "{mem_pos:#X}: {err} [{:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X}]",
                word_slice[0],
                word_slice[1],
                word_slice[2],
                word_slice[3],
                word_slice[4],
                word_slice[5],
                word_slice[6],
                word_slice[7],
                word_slice[8],
                word_slice[9],
                            ).into()))
            .expect("Failed to send error to stats channel");
}
