//! A convenience vector-like wrapper struct for CDPs. Contains a vector of [RDH]s, a vector of payloads and a vector of memory positions.
//!
//! [CdpVec] can be treated similarly to a [`Vec<T>`](std::vec::Vec::<T>) where `T` is a tuple of `(impl RDH, vec<u8>, u64)`
//!
//!  # Examples
//!
//! ```
//! # use alice_protocol_reader::cdp_wrapper::cdp_vec::CdpVec;
//! # use alice_protocol_reader::prelude::test_data::CORRECT_RDH_CRU_V7;
//! # use alice_protocol_reader::prelude::{RdhCru, V7};
//! let mut cdp_vec = CdpVec::<RdhCru<V7>>::new();
//! let cdp_tup = (CORRECT_RDH_CRU_V7, vec![0; 10], 0);
//!
//! // Push a tuple of (RDH, payload, mem_pos)
//! cdp_vec.push_tuple(cdp_tup);
//!
//! // Push a tuple of (RDH, payload, mem_pos) using the push method
//! let (rdh, payload, mem_pos) = (CORRECT_RDH_CRU_V7, vec![0; 10], 0);
//! cdp_vec.push(rdh, payload, mem_pos);
//!
//! // Get the length of the CdpVec
//! let len = cdp_vec.len();
//!
//! // Check if the CdpVec is empty
//! let is_empty = cdp_vec.is_empty();
//!
//! // Clear the CdpVec
//! cdp_vec.clear();
//!
//!
//! // Iterate over the CdpVec using an immutable borrow (no copying)
//! for (rdh, payload, mem_pos) in &cdp_vec {
//!   // Do something with the tuple
//! }
//!
//! // Get a borrowed slice of the RDHs
//! let rdh_slice = cdp_vec.rdh_slice();
//!
//! // Iterate over the CdpVec using a consuming iterator (no copying)
//! cdp_vec.into_iter()
//!         .for_each(|(rdh, payload, mem_pos)| {
//!            // Do something requiring ownership of the tuple
//!        });
//!```

use crate::rdh::RDH;

type CdpTuple<T> = (T, Vec<u8>, u64);

/// The vector-like wrapper struct for CDPs
#[derive(Debug, Clone, PartialEq)]
pub struct CdpVec<T: RDH> {
    rdhs: Vec<T>,
    payloads: Vec<Vec<u8>>,
    rdh_mem_pos: Vec<u64>,
}

impl<T: RDH> Default for CdpVec<T> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<T: RDH> CdpVec<T> {
    /// Construct a new, empty `CdpVec<T: RDH>`.
    #[inline]
    pub fn new() -> Self {
        Self {
            rdhs: Vec::new(),
            payloads: Vec::new(),
            rdh_mem_pos: Vec::new(),
        }
    }
    /// Construct a new, empty `CdpVec<T: RDH>` with at least the specified capacity.
    ///
    /// The cdp_vec will be able to hold at least `capacity` elements without reallocating.
    ///
    /// # Examples
    /// ```
    /// # use alice_protocol_reader::cdp_wrapper::cdp_vec::CdpVec;
    /// # use alice_protocol_reader::prelude::test_data::CORRECT_RDH_CRU_V7;
    /// # use alice_protocol_reader::prelude::{RdhCru, V7};
    /// let mut cdp_vec = CdpVec::<RdhCru<V7>>::with_capacity(10);
    /// assert!(cdp_vec.len() == 0);
    /// ```
    ///
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            rdhs: Vec::with_capacity(capacity),
            payloads: Vec::with_capacity(capacity),
            rdh_mem_pos: Vec::with_capacity(capacity),
        }
    }

    /// Appends an [RDH], payload, and memory position to the back of the CdpVec
    #[inline]
    pub fn push(&mut self, rdh: T, payload: Vec<u8>, mem_pos: u64) {
        self.rdhs.push(rdh);
        self.payloads.push(payload);
        self.rdh_mem_pos.push(mem_pos);
    }

    /// Convenience method to push a tuple of (RDH, payload, mem_pos)
    ///
    /// Removes the need to destructure the tuple before pushing
    #[inline]
    pub fn push_tuple(&mut self, cdp_tuple: CdpTuple<T>) {
        self.rdhs.push(cdp_tuple.0);
        self.payloads.push(cdp_tuple.1);
        self.rdh_mem_pos.push(cdp_tuple.2);
    }

    /// Get the length of the CdpVec, corresponding to the number of CDPs
    #[inline]
    pub fn len(&self) -> usize {
        debug_assert!(self.rdhs.len() == self.payloads.len());
        debug_assert!(self.rdhs.len() == self.rdh_mem_pos.len());
        self.rdhs.len()
    }

    /// Check if the CdpVec is empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        debug_assert!(self.rdhs.len() == self.payloads.len());
        debug_assert!(self.rdhs.len() == self.rdh_mem_pos.len());
        self.rdhs.is_empty()
    }

    /// Clear the CdpVec, removing all elements.
    #[inline]
    pub fn clear(&mut self) {
        self.rdhs.clear();
        self.payloads.clear();
        self.rdh_mem_pos.clear();
    }

    /// Get a borrowed slice of the [RDH]s
    #[inline]
    pub fn rdh_slice(&self) -> &[T] {
        &self.rdhs
    }

    /// Get a borrowed slice of the memory positions
    #[inline]
    pub fn rdh_mem_pos_slice(&self) -> &[u64] {
        &self.rdh_mem_pos
    }
}

/// Implementation of a consuming iterator for CdpVec, with a helper struct
impl<T: RDH> IntoIterator for CdpVec<T> {
    type Item = CdpTuple<T>; // (RDH, payload, mem_pos)
    type IntoIter = IntoIterHelper<T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        IntoIterHelper {
            iter: self
                .rdhs
                .into_iter()
                .zip(self.payloads)
                .zip(self.rdh_mem_pos)
                .map(|((rdh, payload), mem_pos)| (rdh, payload, mem_pos))
                .collect::<Vec<_>>()
                .into_iter(),
        }
    }
}
/// Helper struct for the implementation of a consuming iterator
#[derive(Debug)]
pub struct IntoIterHelper<T: RDH> {
    iter: std::vec::IntoIter<CdpTuple<T>>,
}

impl<T: RDH> Iterator for IntoIterHelper<T> {
    type Item = CdpTuple<T>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

type RefCdpTuple<'a, T> = (&'a T, &'a [u8], u64);
/// Implementation of a non-consuming iterator for CdpVec, with a helper struct
impl<'a, T: RDH> IntoIterator for &'a CdpVec<T> {
    type Item = RefCdpTuple<'a, T>;
    type IntoIter = CdpVecIter<'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        CdpVecIter {
            cdp_cdp_vec: self,
            index: 0,
        }
    }
}

/// Helper struct for the implementation of a non-consuming iterator
#[derive(Debug)]
pub struct CdpVecIter<'a, T: RDH> {
    cdp_cdp_vec: &'a CdpVec<T>,
    index: usize,
}

impl<'a, T: RDH> Iterator for CdpVecIter<'a, T> {
    type Item = RefCdpTuple<'a, T>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.cdp_cdp_vec.rdhs.len() {
            let item = Some((
                self.cdp_cdp_vec.rdhs.get(self.index)?,
                self.cdp_cdp_vec.payloads.get(self.index)?.as_slice(),
                self.cdp_cdp_vec.rdh_mem_pos.get(self.index)?.to_owned(),
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
    use super::CdpVec;
    use crate::prelude::test_data::CORRECT_RDH_CRU_V6;
    use crate::prelude::test_data::CORRECT_RDH_CRU_V7;
    use crate::prelude::RdhCru;
    use crate::prelude::RDH;
    use crate::prelude::V7;

    #[test]
    fn test_push() {
        let mut cdp_vec = CdpVec::<RdhCru<V7>>::new();
        cdp_vec.push(CORRECT_RDH_CRU_V7, vec![0; 10], 0);
        cdp_vec.push(CORRECT_RDH_CRU_V7, vec![0; 10], 1);

        assert_eq!(cdp_vec.rdhs.len(), 2);
        assert_eq!(cdp_vec.payloads.len(), 2);
        assert_eq!(cdp_vec.rdh_mem_pos.len(), 2);
    }

    #[test]
    fn test_push_tup() {
        let mut cdp_vec = CdpVec::<RdhCru<V7>>::new();
        let tup = (CORRECT_RDH_CRU_V7, vec![0; 10], 0);
        cdp_vec.push_tuple(tup);
        cdp_vec.push_tuple((CORRECT_RDH_CRU_V7, vec![0; 10], 1));

        assert_eq!(cdp_vec.rdhs.len(), 2);
        assert_eq!(cdp_vec.payloads.len(), 2);
        assert_eq!(cdp_vec.rdh_mem_pos.len(), 2);
    }

    #[test]
    fn test_clear() {
        let mut cdp_vec = CdpVec::<RdhCru<V7>>::new();
        cdp_vec.push(CORRECT_RDH_CRU_V7, vec![0; 10], 0);
        cdp_vec.push(CORRECT_RDH_CRU_V7, vec![0; 10], 1);

        assert_eq!(cdp_vec.rdhs.len(), 2);
        assert_eq!(cdp_vec.payloads.len(), 2);
        assert_eq!(cdp_vec.rdh_mem_pos.len(), 2);

        cdp_vec.clear();

        assert_eq!(cdp_vec.rdhs.len(), 0);
        assert_eq!(cdp_vec.payloads.len(), 0);
        assert_eq!(cdp_vec.rdh_mem_pos.len(), 0);
    }

    #[test]
    fn test_len() {
        let mut cdp_vec = CdpVec::<RdhCru<V7>>::new();
        cdp_vec.push(CORRECT_RDH_CRU_V7, vec![0; 10], 0);
        cdp_vec.push(CORRECT_RDH_CRU_V7, vec![0; 10], 1);

        assert_eq!(cdp_vec.len(), 2);
    }

    #[test]
    fn test_is_empty() {
        let mut cdp_vec = CdpVec::<RdhCru<V7>>::new();
        assert!(cdp_vec.is_empty());

        cdp_vec.push(CORRECT_RDH_CRU_V7, vec![0; 10], 0);
        assert!(!cdp_vec.is_empty());
    }

    #[test]
    fn test_with_capacity() {
        let cdp_vec = CdpVec::<RdhCru<V7>>::with_capacity(10);
        assert_eq!(cdp_vec.rdhs.capacity(), 10);
        assert_eq!(cdp_vec.payloads.capacity(), 10);
        assert_eq!(cdp_vec.rdh_mem_pos.capacity(), 10);
    }

    #[test]
    fn test_rdh_slice() {
        let mut cdp_vec = CdpVec::<RdhCru<V7>>::new();
        cdp_vec.push(CORRECT_RDH_CRU_V7, vec![0; 10], 0);
        cdp_vec.push(CORRECT_RDH_CRU_V7, vec![0; 10], 1);

        for rdh in cdp_vec.rdh_slice() {
            println!("{rdh}");
        }

        assert_eq!(cdp_vec.rdh_slice().len(), 2);
    }

    #[test]
    fn test_consuming_iterator_cdp_cdp_vec_v7() {
        let cdp_cdp_vec = CdpVec {
            rdhs: vec![CORRECT_RDH_CRU_V7, CORRECT_RDH_CRU_V7],
            payloads: vec![vec![0; 10], vec![0; 10]],
            rdh_mem_pos: vec![0, 1],
        };

        cdp_cdp_vec
            .into_iter()
            .enumerate()
            .for_each(|(idx, (rdh, payload, mem_pos))| {
                assert_eq!(rdh, CORRECT_RDH_CRU_V7);
                assert_eq!(payload.len(), 10);
                assert_eq!(mem_pos, idx as u64);
            });
    }

    #[test]
    fn test_non_consuming_iterator_cdp_cdp_vec_v7() {
        let cdp_cdp_vec = CdpVec {
            rdhs: vec![CORRECT_RDH_CRU_V7, CORRECT_RDH_CRU_V7],
            payloads: vec![vec![0; 10], vec![0; 10]],
            rdh_mem_pos: vec![255, 255],
        };

        for (rdh, payload, mem_pos) in &cdp_cdp_vec {
            assert_eq!(rdh, &CORRECT_RDH_CRU_V7);
            assert_eq!(payload.len(), 10);
            assert_eq!(mem_pos, 255);
        }
    }

    #[test]
    fn test_consuming_iterator_cdp_cdp_vec_v6() {
        let cdp_cdp_vec = CdpVec {
            rdhs: vec![CORRECT_RDH_CRU_V6, CORRECT_RDH_CRU_V6],
            payloads: vec![vec![0; 10], vec![0; 10]],
            rdh_mem_pos: vec![0, 1],
        };

        cdp_cdp_vec
            .into_iter()
            .enumerate()
            .for_each(|(idx, (rdh, payload, mem_pos))| {
                assert_eq!(rdh, CORRECT_RDH_CRU_V6);
                assert_eq!(payload.len(), 10);
                assert_eq!(mem_pos, idx as u64);
            });
    }

    #[test]
    fn test_non_consuming_iterator_cdp_cdp_vec_v6() {
        let cdp_cdp_vec = CdpVec {
            rdhs: vec![CORRECT_RDH_CRU_V6, CORRECT_RDH_CRU_V6],
            payloads: vec![vec![0; 10], vec![0; 10]],
            rdh_mem_pos: vec![0xd, 0xd],
        };

        for (rdh, payload, mem_pos) in &cdp_cdp_vec {
            assert_eq!(*rdh, CORRECT_RDH_CRU_V6);
            assert_eq!(payload.len(), 10);
            assert_eq!(mem_pos, 0xd);
        }

        let len = cdp_cdp_vec.rdhs.len();
        assert_eq!(len, 2);
    }

    fn print_cdp_cdp_vec<T: RDH>(cdp_cdp_vec: &CdpVec<T>) {
        for (rdh, payload, mem_pos) in cdp_cdp_vec {
            println!("rdh: {rdh}, payload: {:?}, mem_pos: {:?}", payload, mem_pos);
        }
    }

    #[test]
    fn test_fn_borrows_cdp_cdp_vec() {
        let cdp_cdp_vec = CdpVec {
            rdhs: vec![CORRECT_RDH_CRU_V6, CORRECT_RDH_CRU_V6],
            payloads: vec![vec![0; 10], vec![0; 10]],
            rdh_mem_pos: vec![0xd, 0xd],
        };

        print_cdp_cdp_vec(&cdp_cdp_vec);
    }

    fn consume_cdp_cdp_vec<T: RDH>(cdp_cdp_vec: CdpVec<T>) {
        cdp_cdp_vec.into_iter().for_each(|(rdh, payload, mem_pos)| {
            println!("rdh: {rdh}, payload: {:?}, mem_pos: {:?}", payload, mem_pos);
        });
    }

    #[test]
    fn test_fn_consume_cdp_cdp_vec() {
        let cdp_cdp_vec = CdpVec {
            rdhs: vec![CORRECT_RDH_CRU_V6, CORRECT_RDH_CRU_V6],
            payloads: vec![vec![0; 10], vec![0; 10]],
            rdh_mem_pos: vec![0xd, 0xd],
        };

        consume_cdp_cdp_vec(cdp_cdp_vec);

        // 'Use of moved value' compiler error
        // println!("cdp_cdp_vec: {:?}", cdp_cdp_vec.rdhs);
    }
}
