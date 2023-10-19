//! Contains the [ValidatorDispatcher], that manages [LinkValidator]s and iterates over and consumes a [`CdpArray<T>`], dispatching the data to the correct thread based on the Link ID running an instance of [LinkValidator].
use std::fmt::Display;

use super::link_validator::LinkValidator;
use crate::config::prelude::*;
use crate::stats::StatType;
use alice_protocol_reader::{cdp_wrapper::cdp_array::CdpArray, prelude::RDH};

type CdpTuple<T> = (T, Vec<u8>, u64);

/// The [ValidatorDispatcher] is responsible for creating and managing the [LinkValidator] threads.
///
/// It receives a [`CdpArray<T>`] and dispatches the data to the correct thread running an instance of [LinkValidator].
pub struct ValidatorDispatcher<T: RDH, C: Config + 'static> {
    processors: Vec<DispatchId>,
    process_channels: Vec<crossbeam_channel::Sender<CdpTuple<T>>>,
    validator_thread_handles: Vec<std::thread::JoinHandle<()>>,
    stats_sender: flume::Sender<StatType>,
    global_config: &'static C,
    dispatch_by: DispatchId,
}

#[derive(PartialEq, Clone, Copy)]
enum DispatchId {
    FeeId(u16),
    GbtLink(u16),
}

impl DispatchId {
    /// Returns the integer value of the ID
    pub fn number(&self) -> u16 {
        match self {
            DispatchId::FeeId(x) | DispatchId::GbtLink(x) => *x,
        }
    }
}

impl Display for DispatchId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DispatchId::FeeId(id) => write!(f, "FEE ID {id}"),
            DispatchId::GbtLink(id) => write!(f, "GBT Link {id}"),
        }
    }
}

impl<T: RDH + 'static, C: Config + 'static> ValidatorDispatcher<T, C> {
    /// Create a new ValidatorDispatcher from a Config and a stats sender channel
    pub fn new(global_config: &'static C, stats_sender: flume::Sender<StatType>) -> Self {
        // Dispatch by FEE ID if system targeted for checks is ITS Stave (gonna be a lot of data to parse for each stave!)
        let dispatch_by = if global_config.check().is_some_and(|c| {
            if let CheckCommands::All(arg) = c {
                arg.target.is_some_and(|s| s == System::ITS_Stave)
            } else {
                false
            }
        }) {
            DispatchId::FeeId(0)
        } else {
            DispatchId::GbtLink(0)
        };

        Self {
            processors: Vec::new(),
            process_channels: Vec::new(),
            validator_thread_handles: Vec::new(),
            stats_sender,
            global_config,
            dispatch_by,
        }
    }

    /// Iterates over and consumes a [`CdpArray<T>`], dispatching the data to the correct thread running an instance of [LinkValidator].
    ///
    /// If a link validator thread does not exist for the link id of the current rdh, a new one is spawned
    pub fn dispatch_cdp_batch<const CAP: usize>(&mut self, cdp_array: CdpArray<T, CAP>) {
        // Iterate over the CDP array
        cdp_array.into_iter().for_each(|(rdh, data, mem_pos)| {
            // Dispatch by FEE ID if system targeted for checks is ITS Stave (gonna be a lot of data to parse for each stave!)
            let id = match self.dispatch_by {
                DispatchId::FeeId(_) => DispatchId::FeeId(rdh.fee_id()),
                DispatchId::GbtLink(_) => DispatchId::GbtLink(rdh.link_id() as u16),
            };

            self.dispatch_by_id(rdh, data, mem_pos, id);
        });
    }

    fn init_validator(&mut self, id: DispatchId) -> LinkValidator<T, C> {
        // Add a new ID to the list of processors
        self.processors.push(id);
        // The first channel will have this capacity, and then exponential backoff will be used
        const INITIAL_CHAN_CAP: usize = 128;
        // Once we've backed off 7 times, create any new channels with the upper capacity
        const MAX_BACKOFF: usize = 7;
        const UPPER_CHAN_CAP: usize = INITIAL_CHAN_CAP << MAX_BACKOFF; // At this point use the max for the rest of the channels

        // Create a new link validator thread to handle a new ID that should be processed
        let (link_validator, send_chan) = if self.processors.len() == 1 {
            // Create the first 2 link validators with a channel capacity of 1000
            LinkValidator::<T, C>::with_chan_capacity(
                self.global_config,
                self.stats_sender.clone(),
                Some(INITIAL_CHAN_CAP),
            )
        } else {
            // Create the rest of the link validators using exponential backoff for the channel capacity
            // Or use the max capacity if the backoff would exceed it
            LinkValidator::<T, C>::with_chan_capacity(
                self.global_config,
                self.stats_sender.clone(),
                if self.processors.len() < MAX_BACKOFF {
                    Some(INITIAL_CHAN_CAP << self.processors.len())
                } else {
                    Some(UPPER_CHAN_CAP)
                },
            )
        };

        // Add the send channel to the new link validator
        self.process_channels.push(send_chan);

        link_validator
    }

    fn dispatch_by_id(&mut self, rdh: T, data: Vec<u8>, mem_pos: u64, id: DispatchId) {
        // Check if the ID to dispatch by is already in the list of processors
        if let Some(index) = self.processors.iter().position(|&proc_id| proc_id == id) {
            // If the ID was found, use its index to send the data through the correct link validator's channel
            unsafe {
                self.process_channels
                    .get_unchecked(index)
                    .send((rdh, data, mem_pos))
                    .unwrap_or_else(|_|
                        self.stats_sender.send(
                            StatType::Fatal(
                            format!("Validator #{id} has prematurely disconnected from the receiver channel and is no longer processing data from {id_desc}", id  = id.number(), id_desc = id)
                            .into_boxed_str()))
                            .unwrap());
            }
        } else {
            // If the ID wasn't found, make a new validator to handle that ID
            let mut validator = self.init_validator(id);

            // Spawn a thread where the newly created link validator will run
            self.validator_thread_handles.push(
                std::thread::Builder::new()
                    .name(format!("Validator #{}", id.number()))
                    .spawn({
                        move || {
                            validator.run();
                        }
                    })
                    .expect("Failed to spawn link validator thread"),
            );
            // Send the data through the newly created link validator's channel, by taking the last element of the vector
            unsafe {
                self.process_channels
                    .last()
                    .unwrap_unchecked()
                    .send((rdh, data, mem_pos))
                    .unwrap_or_else(|_|
                        self.stats_sender.send(
                            StatType::Fatal(
                            format!("Validator #{id} has prematurely disconnected from the receiver channel and is no longer processing data from {id_desc}", id  = id.number(), id_desc = id)
                            .into_boxed_str()))
                            .unwrap());
            }
        }
    }

    /// Disconnects all the link validator's receiver channels and joins all link validator threads
    pub fn join(&mut self) {
        self.process_channels.clear();
        self.validator_thread_handles.drain(..).for_each(|handle| {
            handle.join().expect("Failed to join a validator thread");
        });
    }
}

#[cfg(test)]
mod tests {
    use crate::config::check::{CheckCommands, CheckModeArgs};
    use crate::config::test_util::MockConfig;
    use alice_protocol_reader::prelude::test_data::CORRECT_RDH_CRU_V7;
    use alice_protocol_reader::prelude::*;
    use std::sync::OnceLock;

    use super::*;

    static CFG_TEST_DISPACTER: OnceLock<MockConfig> = OnceLock::new();

    #[test]
    fn test_dispacter() {
        let mut cfg = MockConfig::new();
        cfg.check = Some(CheckCommands::Sanity(CheckModeArgs::default()));
        CFG_TEST_DISPACTER.set(cfg).unwrap();

        let mut disp: ValidatorDispatcher<RdhCru, MockConfig> =
            ValidatorDispatcher::new(CFG_TEST_DISPACTER.get().unwrap(), flume::unbounded().0);

        let cdp_tuple: CdpTuple<RdhCru> = (CORRECT_RDH_CRU_V7, vec![0; 100], 0);

        let mut cdp_array = CdpArray::new();
        cdp_array.push_tuple(cdp_tuple);

        disp.dispatch_cdp_batch::<1>(cdp_array);

        disp.join();
    }
}
