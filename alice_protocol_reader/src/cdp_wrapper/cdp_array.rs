//! A convenience vector-like wrapper struct for CDPs. Contains a vector of [RDH]s, a vector of payloads and a vector of memory positions.

use crate::rdh::RDH;
use arrayvec::ArrayVec;

type CdpTuple<T> = (T, Vec<u8>, u64);

/// The vector-like wrapper struct for CDPs
#[derive(Debug, Clone, PartialEq)]
pub struct CdpArray<T: RDH, const CAP: usize> {
    rdhs: ArrayVec<T, CAP>,
    payloads: ArrayVec<Vec<u8>, CAP>,
    rdh_mem_pos: ArrayVec<u64, CAP>,
}

impl<T: RDH, const CAP: usize> Default for CdpArray<T, CAP> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<T: RDH, const CAP: usize> CdpArray<T, CAP> {
    /// Construct a new, empty `CdpArray<T: RDH>`.
    #[inline]
    pub fn new() -> Self {
        Self {
            rdhs: ArrayVec::new(),
            payloads: ArrayVec::new(),
            rdh_mem_pos: ArrayVec::new(),
        }
    }
    /// Construct a new, empty `CdpArray<T: RDH>` with at least the specified capacity.
    ///
    /// The array will be able to hold at least `capacity` elements.
    ///
    /// # Examples
    /// ```
    /// # use alice_protocol_reader::cdp_wrapper::cdp_array::CdpArray;
    /// # use alice_protocol_reader::prelude::test_data::CORRECT_RDH_CRU_V7;
    /// # use alice_protocol_reader::prelude::RdhCru;
    /// let mut arrvec = CdpArray::<RdhCru, 10>::new_const();
    /// assert!(arrvec.len() == 0);
    /// ```
    ///
    #[inline]
    pub const fn new_const() -> Self {
        Self {
            rdhs: ArrayVec::new_const(),
            payloads: ArrayVec::new_const(),
            rdh_mem_pos: ArrayVec::new_const(),
        }
    }

    /// Appends an [RDH], payload, and memory position to the back of the CdpArray
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

    /// Get the length of the CdpArray, corresponding to the number of CDPs
    #[inline]
    pub fn len(&self) -> usize {
        debug_assert!(self.rdhs.len() == self.payloads.len());
        debug_assert!(self.rdhs.len() == self.rdh_mem_pos.len());
        self.rdhs.len()
    }

    /// Check if the CdpArray is empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        debug_assert!(self.rdhs.len() == self.payloads.len());
        debug_assert!(self.rdhs.len() == self.rdh_mem_pos.len());
        self.rdhs.is_empty()
    }

    /// Clear the CdpArray, removing all elements.
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

/// Implementation of a consuming iterator for CdpArray, with a helper struct
impl<T: RDH, const CAP: usize> IntoIterator for CdpArray<T, CAP> {
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
/// Implementation of a non-consuming iterator for CdpArray, with a helper struct
impl<'a, T: RDH, const CAP: usize> IntoIterator for &'a CdpArray<T, CAP> {
    type Item = RefCdpTuple<'a, T>;
    type IntoIter = CdpArrayIter<'a, T, CAP>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        CdpArrayIter {
            cdp_array: self,
            index: 0,
        }
    }
}

/// Helper struct for the implementation of a non-consuming iterator
#[derive(Debug)]
pub struct CdpArrayIter<'a, T: RDH, const CAP: usize> {
    cdp_array: &'a CdpArray<T, CAP>,
    index: usize,
}

impl<'a, T: RDH, const CAP: usize> Iterator for CdpArrayIter<'a, T, CAP> {
    type Item = RefCdpTuple<'a, T>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.cdp_array.rdhs.len() {
            let item = Some((
                self.cdp_array.rdhs.get(self.index)?,
                self.cdp_array.payloads.get(self.index)?.as_slice(),
                self.cdp_array.rdh_mem_pos.get(self.index)?.to_owned(),
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
    use super::CdpArray;
    use super::*;
    use crate::prelude::test_data::CORRECT_RDH_CRU_V6;
    use crate::prelude::test_data::CORRECT_RDH_CRU_V7;
    use crate::prelude::RdhCru;
    use crate::prelude::RDH;

    #[test]
    fn test_push() {
        let mut arrvec = CdpArray::<RdhCru, 10>::new();
        arrvec.push(CORRECT_RDH_CRU_V7, vec![0; 10], 0);
        arrvec.push(CORRECT_RDH_CRU_V7, vec![0; 10], 1);

        assert_eq!(arrvec.rdhs.len(), 2);
        assert_eq!(arrvec.payloads.len(), 2);
        assert_eq!(arrvec.rdh_mem_pos.len(), 2);
    }

    #[test]
    fn test_push_tup() {
        let mut arrvec = CdpArray::<RdhCru, 10>::new();
        let tup = (CORRECT_RDH_CRU_V7, vec![0; 10], 0);
        arrvec.push_tuple(tup);
        arrvec.push_tuple((CORRECT_RDH_CRU_V7, vec![0; 10], 1));

        assert_eq!(arrvec.rdhs.len(), 2);
        assert_eq!(arrvec.payloads.len(), 2);
        assert_eq!(arrvec.rdh_mem_pos.len(), 2);
    }

    #[test]
    fn test_clear() {
        let mut arrvec = CdpArray::<RdhCru, 10>::new();
        arrvec.push(CORRECT_RDH_CRU_V7, vec![0; 10], 0);
        arrvec.push(CORRECT_RDH_CRU_V7, vec![0; 10], 1);

        assert_eq!(arrvec.rdhs.len(), 2);
        assert_eq!(arrvec.payloads.len(), 2);
        assert_eq!(arrvec.rdh_mem_pos.len(), 2);

        arrvec.clear();

        assert_eq!(arrvec.rdhs.len(), 0);
        assert_eq!(arrvec.payloads.len(), 0);
        assert_eq!(arrvec.rdh_mem_pos.len(), 0);
    }

    #[test]
    fn test_len() {
        let mut arrvec = CdpArray::<RdhCru, 2>::new();
        arrvec.push(CORRECT_RDH_CRU_V7, vec![0; 10], 0);
        arrvec.push(CORRECT_RDH_CRU_V7, vec![0; 10], 1);

        assert_eq!(arrvec.len(), 2);
    }

    #[test]
    fn test_is_empty() {
        let mut arrvec = CdpArray::<RdhCru, 1>::new();
        assert!(arrvec.is_empty());

        arrvec.push(CORRECT_RDH_CRU_V7, vec![0; 10], 0);
        assert!(!arrvec.is_empty());
    }

    #[test]
    fn test_with_capacity() {
        let arrvec = CdpArray::<RdhCru, 10>::new_const();
        assert_eq!(arrvec.rdhs.capacity(), 10);
        assert_eq!(arrvec.payloads.capacity(), 10);
        assert_eq!(arrvec.rdh_mem_pos.capacity(), 10);
    }

    #[test]
    fn test_rdh_slice() {
        let mut arrvec = CdpArray::<RdhCru, 2>::new();
        arrvec.push(CORRECT_RDH_CRU_V7, vec![0; 10], 0);
        arrvec.push(CORRECT_RDH_CRU_V7, vec![0; 10], 1);

        for rdh in arrvec.rdh_slice() {
            println!("{rdh}");
        }

        assert_eq!(arrvec.rdh_slice().len(), 2);
    }

    #[test]
    fn test_consuming_iterator_cdp_array_v7() {
        let cdp_array = CdpArray::<RdhCru, 2> {
            rdhs: {
                let mut a = ArrayVec::new_const();
                a.push(CORRECT_RDH_CRU_V7);
                a.push(CORRECT_RDH_CRU_V7);
                a
            },
            payloads: ArrayVec::from([vec![0; 10], vec![0; 10]]),
            rdh_mem_pos: ArrayVec::from([0, 1]),
        };

        cdp_array
            .into_iter()
            .enumerate()
            .for_each(|(idx, (rdh, payload, mem_pos))| {
                assert_eq!(rdh, CORRECT_RDH_CRU_V7);
                assert_eq!(payload.len(), 10);
                assert_eq!(mem_pos, idx as u64);
            });
    }

    #[test]
    fn test_non_consuming_iterator_cdp_array_v7() {
        let mut cdp_array = CdpArray::<RdhCru, 2> {
            rdhs: ArrayVec::new(),
            payloads: ArrayVec::from([vec![0; 10], vec![0; 10]]),
            rdh_mem_pos: ArrayVec::from([255, 255]),
        };
        [CORRECT_RDH_CRU_V7, CORRECT_RDH_CRU_V7]
            .into_iter()
            .for_each(|rdh| cdp_array.rdhs.push(rdh));

        for (rdh, payload, mem_pos) in &cdp_array {
            assert_eq!(rdh, &CORRECT_RDH_CRU_V7);
            assert_eq!(payload.len(), 10);
            assert_eq!(mem_pos, 255);
        }
    }

    #[test]
    fn test_consuming_iterator_cdp_array_v6() {
        let mut cdp_array = CdpArray::<RdhCru, 2> {
            rdhs: ArrayVec::new_const(),
            payloads: ArrayVec::from([vec![0; 10], vec![0; 10]]),
            rdh_mem_pos: ArrayVec::from([0, 1]),
        };
        [CORRECT_RDH_CRU_V6, CORRECT_RDH_CRU_V6]
            .into_iter()
            .for_each(|rdh| cdp_array.rdhs.push(rdh));

        cdp_array
            .into_iter()
            .enumerate()
            .for_each(|(idx, (rdh, payload, mem_pos))| {
                assert_eq!(rdh, CORRECT_RDH_CRU_V6);
                assert_eq!(payload.len(), 10);
                assert_eq!(mem_pos, idx as u64);
            });
    }

    #[test]
    fn test_non_consuming_iterator_cdp_array_v6() {
        let cdp_array = CdpArray {
            rdhs: {
                let mut a = ArrayVec::<RdhCru, 2>::new_const();
                a.push(CORRECT_RDH_CRU_V6);
                a.push(CORRECT_RDH_CRU_V6);
                a
            },
            payloads: ArrayVec::from([vec![0; 10], vec![0; 10]]),
            rdh_mem_pos: ArrayVec::from([0xd, 0xd]),
        };

        for (rdh, payload, mem_pos) in &cdp_array {
            assert_eq!(*rdh, CORRECT_RDH_CRU_V6);
            assert_eq!(payload.len(), 10);
            assert_eq!(mem_pos, 0xd);
        }

        let len = cdp_array.rdhs.len();
        assert_eq!(len, 2);
    }

    fn print_cdp_array<T: RDH, const CAP: usize>(cdp_array: &CdpArray<T, CAP>) {
        for (rdh, payload, mem_pos) in cdp_array {
            println!("rdh: {rdh}, payload: {:?}, mem_pos: {:?}", payload, mem_pos);
        }
    }

    #[test]
    fn test_fn_borrows_cdp_array() {
        let cdp_array = CdpArray {
            rdhs: {
                let mut a = ArrayVec::<RdhCru, 2>::new_const();
                a.push(CORRECT_RDH_CRU_V6);
                a.push(CORRECT_RDH_CRU_V6);
                a
            },
            payloads: ArrayVec::from([vec![0; 10], vec![0; 10]]),
            rdh_mem_pos: ArrayVec::from([0xd, 0xd]),
        };

        print_cdp_array(&cdp_array);
    }

    fn consume_cdp_array<T: RDH, const CAP: usize>(cdp_array: CdpArray<T, CAP>) {
        cdp_array.into_iter().for_each(|(rdh, payload, mem_pos)| {
            println!("rdh: {rdh}, payload: {:?}, mem_pos: {:?}", payload, mem_pos);
        });
    }

    #[test]
    fn test_fn_consume_cdp_array() {
        let cdp_array = CdpArray {
            rdhs: {
                let mut a = ArrayVec::<RdhCru, 3>::new_const();
                a.push(CORRECT_RDH_CRU_V6);
                a.push(CORRECT_RDH_CRU_V6);
                a
            },
            payloads: ArrayVec::from([vec![0; 10], vec![0; 10], vec![0; 10]]),
            rdh_mem_pos: ArrayVec::from([0xd, 0xe, 0xf]),
        };

        consume_cdp_array(cdp_array);

        // 'Use of moved value' compiler error
        // println!("cdp_array: {:?}", cdp_array.rdhs);
    }
}
