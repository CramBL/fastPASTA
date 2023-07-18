//! A convenience vector-like wrapper struct for CDPs. Contains a vector of [RDH]s, a vector of payloads and a vector of memory positions.
//!
//! [CdpChunk] can be treated similarly to a `std::vec::Vec::<T>` where `T` is a tuple of `(impl RDH, vec<u8>, u64)`
//!
//!  # Examples
//!
//! ```
//! # use alice_protocol_reader::data_wrapper::CdpChunk;
//! # use alice_protocol_reader::prelude::test_data::CORRECT_RDH_CRU_V7;
//! # use alice_protocol_reader::prelude::{RdhCru, V7};
//! let mut chunk = CdpChunk::<RdhCru<V7>>::new();
//! let cdp_tup = (CORRECT_RDH_CRU_V7, vec![0; 10], 0);
//!
//! // Push a tuple of (RDH, payload, mem_pos)
//! chunk.push_tuple(cdp_tup);
//!
//! // Push a tuple of (RDH, payload, mem_pos) using the push method
//! let (rdh, payload, mem_pos) = (CORRECT_RDH_CRU_V7, vec![0; 10], 0);
//! chunk.push(rdh, payload, mem_pos);
//!
//! // Get the length of the CdpChunk
//! let len = chunk.len();
//!
//! // Check if the CdpChunk is empty
//! let is_empty = chunk.is_empty();
//!
//! // Clear the CdpChunk
//! chunk.clear();
//!
//!
//! // Iterate over the CdpChunk using an immutable borrow (no copying)
//! for (rdh, payload, mem_pos) in &chunk {
//!   // Do something with the tuple
//! }
//!
//! // Get a borrowed slice of the RDHs
//! let rdh_slice = chunk.rdh_slice();
//!
//! // Iterate over the CdpChunk using a consuming iterator (no copying)
//! chunk.into_iter()
//!         .for_each(|(rdh, payload, mem_pos)| {
//!            // Do something requiring ownership of the tuple
//!        });
//!```

use super::rdh::RDH;

type CdpTuple<T> = (T, Vec<u8>, u64);

/// The vector-like wrapper struct for CDPs
pub struct CdpChunk<T: RDH> {
    rdhs: Vec<T>,
    payloads: Vec<Vec<u8>>,
    rdh_mem_pos: Vec<u64>,
}

impl<T: RDH> Default for CdpChunk<T> {
    #[inline(always)]
    fn default() -> Self {
        Self::new()
    }
}

impl<T: RDH> CdpChunk<T> {
    /// Construct a new, empty `CdpChunk<T: RDH>`.
    #[inline(always)]
    pub fn new() -> Self {
        Self {
            rdhs: Vec::new(),
            payloads: Vec::new(),
            rdh_mem_pos: Vec::new(),
        }
    }
    /// Construct a new, empty `CdpChunk<T: RDH>` with at least the specified capacity.
    ///
    /// The chunk will be able to hold at least `capacity` elements without reallocating.
    ///
    /// # Examples
    /// ```
    /// # use alice_protocol_reader::data_wrapper::CdpChunk;
    /// # use alice_protocol_reader::prelude::test_data::CORRECT_RDH_CRU_V7;
    /// # use alice_protocol_reader::prelude::{RdhCru, V7};
    /// let mut chunk = CdpChunk::<RdhCru<V7>>::with_capacity(10);
    /// assert!(chunk.len() == 0);
    /// ```
    ///
    #[inline(always)]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            rdhs: Vec::with_capacity(capacity),
            payloads: Vec::with_capacity(capacity),
            rdh_mem_pos: Vec::with_capacity(capacity),
        }
    }

    /// Appends an [RDH], payload, and memory position to the back of the CdpChunk
    #[inline(always)]
    pub fn push(&mut self, rdh: T, payload: Vec<u8>, mem_pos: u64) {
        self.rdhs.push(rdh);
        self.payloads.push(payload);
        self.rdh_mem_pos.push(mem_pos);
    }

    /// Convenience method to push a tuple of (RDH, payload, mem_pos)
    ///
    /// Removes the need to destructure the tuple before pushing
    #[inline(always)]
    pub fn push_tuple(&mut self, cdp_tuple: CdpTuple<T>) {
        self.rdhs.push(cdp_tuple.0);
        self.payloads.push(cdp_tuple.1);
        self.rdh_mem_pos.push(cdp_tuple.2);
    }

    /// Get the length of the CdpChunk, corresponding to the number of CDPs
    #[inline(always)]
    pub fn len(&self) -> usize {
        debug_assert!(self.rdhs.len() == self.payloads.len());
        debug_assert!(self.rdhs.len() == self.rdh_mem_pos.len());
        self.rdhs.len()
    }

    /// Check if the CdpChunk is empty
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        debug_assert!(self.rdhs.len() == self.payloads.len());
        debug_assert!(self.rdhs.len() == self.rdh_mem_pos.len());
        self.rdhs.is_empty()
    }

    /// Clear the CdpChunk, removing all elements.
    #[inline(always)]
    pub fn clear(&mut self) {
        self.rdhs.clear();
        self.payloads.clear();
        self.rdh_mem_pos.clear();
    }

    /// Get a borrowed slice of the [RDH]s
    #[inline(always)]
    pub fn rdh_slice(&self) -> &[T] {
        &self.rdhs
    }

    /// Get a borrowed slice of the memory positions
    #[inline(always)]
    pub fn rdh_mem_pos_slice(&self) -> &[u64] {
        &self.rdh_mem_pos
    }
}

/// Implementation of a consuming iterator for CdpChunk, with a helper struct
impl<T: RDH> IntoIterator for CdpChunk<T> {
    type Item = CdpTuple<T>; // (RDH, payload, mem_pos)
    type IntoIter = IntoIterHelper<T>;

    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        IntoIterHelper {
            iter: self
                .rdhs
                .into_iter()
                .zip(self.payloads.into_iter())
                .zip(self.rdh_mem_pos.into_iter())
                .map(|((rdh, payload), mem_pos)| (rdh, payload, mem_pos))
                .collect::<Vec<_>>()
                .into_iter(),
        }
    }
}
/// Helper struct for the implementation of a consuming iterator
pub struct IntoIterHelper<T: RDH> {
    iter: std::vec::IntoIter<CdpTuple<T>>,
}

impl<T: RDH> Iterator for IntoIterHelper<T> {
    type Item = CdpTuple<T>;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

type RefCdpTuple<'a, T> = (&'a T, &'a [u8], u64);
/// Implementation of a non-consuming iterator for CdpChunk, with a helper struct
impl<'a, T: RDH> IntoIterator for &'a CdpChunk<T> {
    type Item = RefCdpTuple<'a, T>;
    type IntoIter = CdpChunkIter<'a, T>;

    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        CdpChunkIter {
            cdp_chunk: self,
            index: 0,
        }
    }
}

/// Helper struct for the implementation of a non-consuming iterator
pub struct CdpChunkIter<'a, T: RDH> {
    cdp_chunk: &'a CdpChunk<T>,
    index: usize,
}

impl<'a, T: RDH> Iterator for CdpChunkIter<'a, T> {
    type Item = RefCdpTuple<'a, T>;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.cdp_chunk.rdhs.len() {
            let item = Some((
                self.cdp_chunk.rdhs.get(self.index)?,
                self.cdp_chunk.payloads.get(self.index)?.as_slice(),
                self.cdp_chunk.rdh_mem_pos.get(self.index)?.to_owned(),
            ));
            self.index += 1;
            item
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::prelude::test_data::CORRECT_RDH_CRU_V6;
    use super::super::prelude::test_data::CORRECT_RDH_CRU_V7;
    use super::super::prelude::RdhCru;
    use super::super::prelude::RDH;
    use super::super::prelude::V7;
    use super::CdpChunk;

    #[test]
    fn test_push() {
        let mut chunk = CdpChunk::<RdhCru<V7>>::new();
        chunk.push(CORRECT_RDH_CRU_V7, vec![0; 10], 0);
        chunk.push(CORRECT_RDH_CRU_V7, vec![0; 10], 1);

        assert_eq!(chunk.rdhs.len(), 2);
        assert_eq!(chunk.payloads.len(), 2);
        assert_eq!(chunk.rdh_mem_pos.len(), 2);
    }

    #[test]
    fn test_push_tup() {
        let mut chunk = CdpChunk::<RdhCru<V7>>::new();
        let tup = (CORRECT_RDH_CRU_V7, vec![0; 10], 0);
        chunk.push_tuple(tup);
        chunk.push_tuple((CORRECT_RDH_CRU_V7, vec![0; 10], 1));

        assert_eq!(chunk.rdhs.len(), 2);
        assert_eq!(chunk.payloads.len(), 2);
        assert_eq!(chunk.rdh_mem_pos.len(), 2);
    }

    #[test]
    fn test_clear() {
        let mut chunk = CdpChunk::<RdhCru<V7>>::new();
        chunk.push(CORRECT_RDH_CRU_V7, vec![0; 10], 0);
        chunk.push(CORRECT_RDH_CRU_V7, vec![0; 10], 1);

        assert_eq!(chunk.rdhs.len(), 2);
        assert_eq!(chunk.payloads.len(), 2);
        assert_eq!(chunk.rdh_mem_pos.len(), 2);

        chunk.clear();

        assert_eq!(chunk.rdhs.len(), 0);
        assert_eq!(chunk.payloads.len(), 0);
        assert_eq!(chunk.rdh_mem_pos.len(), 0);
    }

    #[test]
    fn test_len() {
        let mut chunk = CdpChunk::<RdhCru<V7>>::new();
        chunk.push(CORRECT_RDH_CRU_V7, vec![0; 10], 0);
        chunk.push(CORRECT_RDH_CRU_V7, vec![0; 10], 1);

        assert_eq!(chunk.len(), 2);
    }

    #[test]
    fn test_is_empty() {
        let mut chunk = CdpChunk::<RdhCru<V7>>::new();
        assert!(chunk.is_empty());

        chunk.push(CORRECT_RDH_CRU_V7, vec![0; 10], 0);
        assert!(!chunk.is_empty());
    }

    #[test]
    fn test_with_capacity() {
        let chunk = CdpChunk::<RdhCru<V7>>::with_capacity(10);
        assert_eq!(chunk.rdhs.capacity(), 10);
        assert_eq!(chunk.payloads.capacity(), 10);
        assert_eq!(chunk.rdh_mem_pos.capacity(), 10);
    }

    #[test]
    fn test_rdh_slice() {
        let mut chunk = CdpChunk::<RdhCru<V7>>::new();
        chunk.push(CORRECT_RDH_CRU_V7, vec![0; 10], 0);
        chunk.push(CORRECT_RDH_CRU_V7, vec![0; 10], 1);

        for rdh in chunk.rdh_slice() {
            println!("{rdh}");
        }

        assert_eq!(chunk.rdh_slice().len(), 2);
    }

    #[test]
    fn test_consuming_iterator_cdp_chunk_v7() {
        let cdp_chunk = CdpChunk {
            rdhs: vec![CORRECT_RDH_CRU_V7, CORRECT_RDH_CRU_V7],
            payloads: vec![vec![0; 10], vec![0; 10]],
            rdh_mem_pos: vec![0, 1],
        };

        cdp_chunk
            .into_iter()
            .enumerate()
            .for_each(|(idx, (rdh, payload, mem_pos))| {
                assert_eq!(rdh, CORRECT_RDH_CRU_V7);
                assert_eq!(payload.len(), 10);
                assert_eq!(mem_pos, idx as u64);
            });
    }

    #[test]
    fn test_non_consuming_iterator_cdp_chunk_v7() {
        let cdp_chunk = CdpChunk {
            rdhs: vec![CORRECT_RDH_CRU_V7, CORRECT_RDH_CRU_V7],
            payloads: vec![vec![0; 10], vec![0; 10]],
            rdh_mem_pos: vec![255, 255],
        };

        for (rdh, payload, mem_pos) in &cdp_chunk {
            assert_eq!(rdh, &CORRECT_RDH_CRU_V7);
            assert_eq!(payload.len(), 10);
            assert_eq!(mem_pos, 255);
        }
    }

    #[test]
    fn test_consuming_iterator_cdp_chunk_v6() {
        let cdp_chunk = CdpChunk {
            rdhs: vec![CORRECT_RDH_CRU_V6, CORRECT_RDH_CRU_V6],
            payloads: vec![vec![0; 10], vec![0; 10]],
            rdh_mem_pos: vec![0, 1],
        };

        cdp_chunk
            .into_iter()
            .enumerate()
            .for_each(|(idx, (rdh, payload, mem_pos))| {
                assert_eq!(rdh, CORRECT_RDH_CRU_V6);
                assert_eq!(payload.len(), 10);
                assert_eq!(mem_pos, idx as u64);
            });
    }

    #[test]
    fn test_non_consuming_iterator_cdp_chunk_v6() {
        let cdp_chunk = CdpChunk {
            rdhs: vec![CORRECT_RDH_CRU_V6, CORRECT_RDH_CRU_V6],
            payloads: vec![vec![0; 10], vec![0; 10]],
            rdh_mem_pos: vec![0xd, 0xd],
        };

        for (rdh, payload, mem_pos) in &cdp_chunk {
            assert_eq!(*rdh, CORRECT_RDH_CRU_V6);
            assert_eq!(payload.len(), 10);
            assert_eq!(mem_pos, 0xd);
        }

        let len = cdp_chunk.rdhs.len();
        assert_eq!(len, 2);
    }

    fn print_cdp_chunk<T: RDH>(cdp_chunk: &CdpChunk<T>) {
        for (rdh, payload, mem_pos) in cdp_chunk {
            println!("rdh: {rdh}, payload: {:?}, mem_pos: {:?}", payload, mem_pos);
        }
    }

    #[test]
    fn test_fn_borrows_cdp_chunk() {
        let cdp_chunk = CdpChunk {
            rdhs: vec![CORRECT_RDH_CRU_V6, CORRECT_RDH_CRU_V6],
            payloads: vec![vec![0; 10], vec![0; 10]],
            rdh_mem_pos: vec![0xd, 0xd],
        };

        print_cdp_chunk(&cdp_chunk);
    }

    fn consume_cdp_chunk<T: RDH>(cdp_chunk: CdpChunk<T>) {
        cdp_chunk.into_iter().for_each(|(rdh, payload, mem_pos)| {
            println!("rdh: {rdh}, payload: {:?}, mem_pos: {:?}", payload, mem_pos);
        });
    }

    #[test]
    fn test_fn_consume_cdp_chunk() {
        let cdp_chunk = CdpChunk {
            rdhs: vec![CORRECT_RDH_CRU_V6, CORRECT_RDH_CRU_V6],
            payloads: vec![vec![0; 10], vec![0; 10]],
            rdh_mem_pos: vec![0xd, 0xd],
        };

        consume_cdp_chunk(cdp_chunk);

        // 'Use of moved value' compiler error
        // println!("cdp_chunk: {:?}", cdp_chunk.rdhs);
    }
}
