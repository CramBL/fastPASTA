use crate::util::*;

/// Manages the state of the analyzed readout frames.
///
/// When they start, when they end, and makes the call to analyze them after they end.
pub struct ItsReadoutFrameValidator<C: CustomChecksOpt + 'static> {
    custom_checks_config: &'static C,
    // Stores the ALPIDE data from an ITS readout frame, if the config is set to check ALPIDE data, and a filter for a stave is set.
    alpide_readout_frame: Option<AlpideReadoutFrame>,
    // Flag to start storing ALPIDE data,
    // Set to true when a TDH with internal trigger bit set is found.
    // Set to false when it's true and a TDT is found.
    is_readout_frame: bool,
    // The stave from which the data is collected.
    from_stave: Option<Stave>,
    // If any lane is in FATAL state (and correctly reported it through the ITS protocol), then store the lane ids here.
    fatal_lanes: Option<Vec<u8>>,
}

impl<C: CustomChecksOpt> ItsReadoutFrameValidator<C> {
    pub fn new(custom_checks: &'static C) -> Self {
        Self {
            custom_checks_config: custom_checks,
            alpide_readout_frame: None,
            is_readout_frame: false,
            from_stave: None,
            fatal_lanes: None,
        }
    }

    pub fn stave(&self) -> Option<&Stave> {
        self.from_stave.as_ref()
    }

    pub fn set_stave(&mut self, from_stave: Stave) {
        debug_assert!(self.from_stave.is_none());
        self.from_stave = Some(from_stave);
    }

    pub fn add_fatal_lanes(&mut self, new_fatal_lanes: Vec<u8>) {
        if let Some(current_fatal_lanes) = &mut self.fatal_lanes {
            current_fatal_lanes.extend(new_fatal_lanes);
        } else {
            self.fatal_lanes = Some(new_fatal_lanes);
        }
    }

    pub fn fatal_lanes(&self) -> Option<&[u8]> {
        self.fatal_lanes.as_deref()
    }

    pub fn is_in_frame(&self) -> bool {
        self.is_readout_frame
    }

    pub fn new_frame(&mut self, current_mem_pos: u64) {
        self.alpide_readout_frame = Some(AlpideReadoutFrame::new(current_mem_pos));
        self.is_readout_frame = true;
    }

    pub fn store_lane_data(&mut self, data_word: &[u8]) {
        self.alpide_readout_frame
            .as_mut()
            .unwrap()
            .store_lane_data(data_word, Layer::from_stave(&self.from_stave.unwrap()));
    }

    pub fn try_close_frame(&mut self, end_mem_pos: u64) -> Result<(), ()> {
        self.is_readout_frame = false;
        if let Some(frame) = self.alpide_readout_frame.as_mut() {
            frame.close_frame(end_mem_pos);
            Ok(())
        } else {
            Err(())
        }
    }

    /// Process ALPIDE data for a readout frame. Includes checks across lanes and within a lane.
    pub fn process_frame(
        &mut self,
        err_chan: &flume::Sender<StatType>,
        // These are supplied just to be able to add more context to error messages
        status_words: &StatusWordContainer,
        current_rdh: &impl RDH,
    ) {
        // First close/end the frame by setting the end memory position to the current word position

        let frame = self.alpide_readout_frame.take().unwrap();

        debug_assert!(!self.is_readout_frame);
        debug_assert!(frame.start_mem_pos() != 0, "Frame start mem pos not set");

        let mem_pos_start = frame.start_mem_pos();
        let mem_pos_end = frame.end_mem_pos();

        if frame.is_empty() {
            // No data in a full readout frame is a protocol error unless lanes in error has been reported by the TDT/DDW.
            self.report_empty_alpide_frame_error(&frame, err_chan, status_words, current_rdh);
            // No data, nothing to process (and erroneous to do so) early return
            return;
        }

        let is_ib = frame.from_layer() == Layer::Inner;

        // Process the data frame
        let (lanes_in_error_ids, lane_error_msgs, alpide_stats, fatal_lanes) =
            alpide::check_alpide_data_frame(&frame, self.custom_checks_config);

        // Add the fatal lanes to the running list of fatal lanes
        if let Some(new_fatal_lanes) = fatal_lanes {
            self.add_fatal_lanes(new_fatal_lanes);
        }

        // Check if the frame is valid in terms of lanes in the data.
        if let Err(err_msg) = frame.check_frame_lanes_valid(self.fatal_lanes()) {
            // Format and send error message
            let err_code = if is_ib { "E72" } else { "E73" };
            let err_msg = format!(
                "{mem_pos_start:#X}: [{err_code}] FEE ID:{feeid} ALPIDE data frame ending at {mem_pos_end:#X} {err_msg}. Lanes: {lanes:?}",
                feeid=current_rdh.fee_id(),
                lanes = frame.lane_data_frames_as_slice().iter().map(|lane|
                    lane_id_to_lane_number(lane.id(), is_ib)).collect::<Vec<u8>>(),
            );
            err_chan
                .send(StatType::Error(err_msg.into()))
                .expect("Failed to send error to stats channel");
        }

        err_chan
            .send(StatType::AlpideStats(alpide_stats))
            .expect("Failed to send error to stats channel");

        // Format and send all errors
        if !lane_error_msgs.is_empty() {
            let err_code = if is_ib { "E74" } else { "E75" };
            let lane_error_numbers = lanes_in_error_ids
                .iter()
                .map(|lane_id| lane_id_to_lane_number(*lane_id, is_ib))
                .collect_vec();
            let mut error_string = format!(
                "{mem_pos_start:#X}: [{err_code}] FEE ID:{feeid} ALPIDE data frame ending at {mem_pos_end:#X} has errors in lane {lane_error_numbers:?}:", feeid=current_rdh.fee_id()
            );
            // Don't add the error messages to the error string if the config is set to mute errors
            // (these error messages are context so it doesn't change the amount of errors reported)
            if !Cfg::global().mute_errors() {
                lane_error_msgs.into_iter().for_each(|lane_error_msg| {
                    error_string.push_str(&lane_error_msg);
                });
            }
            err_chan
                .send(StatType::Error(error_string.into()))
                .expect("Failed to send error to stats channel");
        }
    }

    fn report_empty_alpide_frame_error(
        &self,
        frame: &AlpideReadoutFrame,
        err_chan: &flume::Sender<StatType>,
        // These are supplied just to give more context to error messages
        status_words: &StatusWordContainer,
        current_rdh: &impl RDH,
    ) {
        // No data in a full readout frame is a protocol error unless lanes in error has been reported by the TDT/DDW.
        let (mem_pos_start, mem_pos_end) = (frame.start_mem_pos(), frame.end_mem_pos());
        log::warn!("ALPIDE data frame at {mem_pos_start:#X} - {mem_pos_end:#X} is empty",);
        // TODO: Check lane errors in TDT and DDW
        let ddw_lane_status_str = if let Some(ddw0) = status_words.ddw() {
            format!("Last DDW [{ddw0}] lane status: {:#X}", ddw0.lane_status())
        } else {
            "No DDW seen yet".to_string()
        };

        let tdt_lane_status_str = {
            let curr_tdt = status_words.tdt().unwrap();
            format!("\
            Frame closing TDT [{curr_tdt}] lane status: 0:15={lane_0_15:#X} 16:23={lane_16_23:#X} 24:27={lane_24_27:#X}\
            ",
            lane_0_15 = curr_tdt.lane_status_15_0(),
            lane_16_23 = curr_tdt.lane_status_23_16(),
            lane_24_27 = curr_tdt.lane_status_27_24(),
        )
        };

        let error_string = format!(
            "\
        {mem_pos_start:#X}: [E701] FEE ID:{feeid} ALPIDE data frame ending at {mem_pos_end:#X} has no data words.\
        \n\t Additional information:\
        \n\t\t - Lanes in error (as indicated by APEs): {fatal_lanes:?}\
        \n\t\t - {ddw_lane_status_str}\
        \n\t\t - {tdt_lane_status_str}",
            feeid = current_rdh.fee_id(),
            fatal_lanes = self.fatal_lanes()
        );
        err_chan
            .send(StatType::Error(error_string.into()))
            .expect("Failed to send error to stats channel");
    }
}
