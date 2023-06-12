use crate::{
    analyze::validators::{its::lib::ItsPayloadWord, lib::preprocess_payload},
    analyze::view::lib::format_word_slice,
    input,
    words::lib::RDH,
};

pub(crate) fn its_readout_frame_data_view<T: RDH>(
    cdp_chunk: input::data_wrapper::CdpChunk<T>,
) -> Result<(), Box<dyn std::error::Error>> {
    todo!("ITS readout frame data view")
}
