//! A convenience vector-like wrapper struct for CDPs. Contains a vector of [RDH]s, a vector of payloads and a vector of memory positions.

use itertools::multiunzip;

type CdpTuple<T> = (T, Vec<u8>, u64);

/// The vector-like wrapper struct for CDPs
#[derive(Debug, Clone, PartialEq)]
pub struct CdpChunkBoxed<T> {
    rdhs: Box<[T]>,
    payloads: Box<[Box<[u8]>]>,
    rdh_mem_pos: Box<[u64]>,
}

impl<T> CdpChunkBoxed<T> {
    /// Create a new [CdpChunkBox] from a Boxed [CdpChunk]
    pub fn new(rdhs: Box<[T]>, payloads: Box<[Box<[u8]>]>, rdh_mem_pos: Box<[u64]>) -> Self {
        Self {
            rdhs,
            payloads,
            rdh_mem_pos,
        }
    }

    /// Create a new CdpChunkBoxed from a vector of RDHs, a vector of payloads and a vector of memory positions
    pub fn from_vecs(rdhs: Vec<T>, payloads: Vec<Vec<u8>>, rdh_mem_pos: Vec<u64>) -> Self {
        Self {
            rdhs: rdhs.into_boxed_slice(),
            payloads: payloads
                .into_iter()
                .map(|pl| pl.into_boxed_slice())
                .collect(),
            rdh_mem_pos: rdh_mem_pos.into_boxed_slice(),
        }
    }

    /// Make a boxed CdpChunk from a vector of CDPs
    pub fn from_cdp_vec(cdp_chunk: Vec<CdpTuple<T>>) -> Self {
        let (rdhs, payloads, rdh_mem_pos): (Vec<T>, Vec<Vec<u8>>, Vec<u64>) = multiunzip(cdp_chunk);
        Self::from_vecs(rdhs, payloads, rdh_mem_pos)
    }

    /// Get the number of CDPs in the chunk
    pub fn len(&self) -> usize {
        self.rdhs.len()
    }

    /// Check if the chunk is empty
    pub fn is_empty(&self) -> bool {
        self.rdhs.is_empty()
    }

    /// Get a borrowed slice of the [RDH]s
    #[inline]
    pub fn rdh_slice(&self) -> &[T] {
        &self.rdhs
    }
}

type CdpBoxTuple<T> = (T, Box<[u8]>, u64);
/// A helper struct for the consuming iterator
#[derive(Debug, Clone)]
pub struct IntoIterHelper<T> {
    iter: std::vec::IntoIter<CdpBoxTuple<T>>,
}

impl<T> Iterator for IntoIterHelper<T> {
    type Item = CdpBoxTuple<T>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.iter.len();
        (len, Some(len))
    }
}

/// A consuming iterator over a [CdpChunkBoxed]
#[derive(Debug, Clone, PartialEq)]
pub struct CdpChunkBoxedIntoIterator<T> {
    cdp_chunk_boxed: CdpChunkBoxed<T>,
    index: usize,
}

impl<T> IntoIterator for CdpChunkBoxed<T> {
    type Item = CdpBoxTuple<T>; // (RDH, payload, mem_pos)
    type IntoIter = IntoIterHelper<T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let rdh_iter = self.rdhs.into_vec().into_iter();
        let payload_iter = self.payloads.into_vec().into_iter();
        let rdh_mem_pos_iter = self.rdh_mem_pos.into_vec().into_iter();
        IntoIterHelper {
            iter: rdh_iter
                .zip(payload_iter)
                .zip(rdh_mem_pos_iter)
                .map(|((rdh, pl), rdh_mem_pos)| (rdh, pl, rdh_mem_pos))
                .collect::<Vec<_>>()
                .into_iter(),
        }
    }
}

// Borrowing iterator
impl<'a, T> IntoIterator for &'a CdpChunkBoxed<T> {
    type Item = (&'a T, &'a Box<[u8]>, u64);
    type IntoIter = CdpChunkBoxedIterator<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        CdpChunkBoxedIterator {
            cdp_chunk_boxed: self,
            index: 0,
        }
    }
}

/// An iterator over a [CdpChunkBoxed]
#[derive(Debug, Clone, PartialEq)]
pub struct CdpChunkBoxedIterator<'a, T> {
    cdp_chunk_boxed: &'a CdpChunkBoxed<T>,
    index: usize,
}

impl<'a, T> Iterator for CdpChunkBoxedIterator<'a, T> {
    type Item = (&'a T, &'a Box<[u8]>, u64);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let index = self.index;
        self.index += 1;
        self.cdp_chunk_boxed.rdhs.get(index).and_then(|rdh| {
            self.cdp_chunk_boxed.payloads.get(index).and_then(|pl| {
                self.cdp_chunk_boxed
                    .rdh_mem_pos
                    .get(index)
                    .map(|rdh_mem_pos| (rdh, pl, *rdh_mem_pos))
            })
        })
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.cdp_chunk_boxed.rdhs.len();
        (len, Some(len))
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::CdpChunk;

    use super::super::prelude::test_data::CORRECT_RDH_CRU_V6;
    use super::super::prelude::test_data::CORRECT_RDH_CRU_V7;
    use super::super::prelude::RdhCru;
    use super::super::prelude::RDH;
    use super::super::prelude::V7;

    use super::CdpChunkBoxed;

    #[test]
    fn test_new() {
        let rdhs = vec![CORRECT_RDH_CRU_V7, CORRECT_RDH_CRU_V7, CORRECT_RDH_CRU_V7];
        let payloads = vec![vec![0; 100], vec![0; 100], vec![0; 100]];
        let cdps = CdpChunkBoxed::<RdhCru<V7>>::from_vecs(rdhs, payloads, vec![2, 2003, 2]);
    }
    #[test]
    fn test_iterators() {
        let mut chunk = CdpChunk::<RdhCru<V7>>::new();
        chunk.push(CORRECT_RDH_CRU_V7, vec![0; 10], 0);
        chunk.push(CORRECT_RDH_CRU_V7, vec![0; 10], 1);

        let boxed = chunk.into_boxed();

        (&boxed)
            .into_iter()
            .enumerate()
            .for_each(|(i, (_rdh, payload, rdh_mem_pos))| {
                assert_eq!(rdh_mem_pos, i as u64);
                assert_eq!(payload.len(), 10);
            });

        boxed
            .into_iter()
            .enumerate()
            .for_each(|(i, (_rdh, payload, rdh_mem_pos))| {
                assert_eq!(rdh_mem_pos, i as u64);
                assert_eq!(payload.len(), 10);
            });
    }
}
