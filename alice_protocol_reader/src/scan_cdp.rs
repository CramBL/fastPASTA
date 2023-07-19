//! Contains the [ScanCDP] trait for reading CDPs from a file or stdin (readable instance)
use crate::rdh::RDH;
use crate::config::filter::FilterTarget;

type CdpTuple<T> = (T, Vec<u8>, u64);



/// Trait for a scanner that reads CDPs from a file or stdin
pub trait ScanCDP {

    /// Loads the next [RDH] from the input and returns it
    fn load_rdh_cru<T: RDH>(&mut self) -> Result<T, std::io::Error>;

    /// Loads the payload in the form of raw bytes from the input and returns it
    ///
    /// The size of the payload is given as an argument.
    fn load_payload_raw(&mut self, payload_size: usize) -> Result<Vec<u8>, std::io::Error>;

    /// Loads the next CDP ([RDH] and payload) from the input and returns it as a ([RDH], [`Vec<u8>`], [u64]) tuple.
    #[inline(always)]
    fn load_cdp<T: RDH>(&mut self) -> Result<CdpTuple<T>, std::io::Error> {
        let rdh: T = self.load_rdh_cru()?;
        let payload = self.load_payload_raw(rdh.payload_size() as usize)?;
        let mem_pos = self.current_mem_pos();

        Ok((rdh, payload, mem_pos))
    }

    /// Loads the next [RDH] that matches the user specified filter target from the input and returns it
    fn load_next_rdh_to_filter<T: RDH>(
        &mut self,
        offset_to_next: u16,
        target: FilterTarget,
    ) -> Result<T, std::io::Error>;

    /// Convenience function to return the current memory position in the input stream
    fn current_mem_pos(&self) -> u64;
}