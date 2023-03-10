use crate::words::rdh::RDH;

type CdpTuple<T> = (T, Vec<u8>, u64);

pub struct CdpChunk<T: RDH> {
    rdhs: Vec<T>,
    payloads: Vec<Vec<u8>>,
    mem_positions: Vec<u64>,
}

impl<T: RDH> Default for CdpChunk<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: RDH> CdpChunk<T> {
    pub fn new() -> Self {
        Self {
            rdhs: Vec::new(),
            payloads: Vec::new(),
            mem_positions: Vec::new(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            rdhs: Vec::with_capacity(capacity),
            payloads: Vec::with_capacity(capacity),
            mem_positions: Vec::with_capacity(capacity),
        }
    }

    pub fn push(&mut self, rdh: T, payload: Vec<u8>, mem_pos: u64) {
        self.rdhs.push(rdh);
        self.payloads.push(payload);
        self.mem_positions.push(mem_pos);
    }

    /// Convenience method to push a tuple of (RDH, payload, mem_pos)
    ///
    /// Removes the need to destructure the tuple before pushing
    pub fn push_tuple(&mut self, cdp_tuple: CdpTuple<T>) {
        self.rdhs.push(cdp_tuple.0);
        self.payloads.push(cdp_tuple.1);
        self.mem_positions.push(cdp_tuple.2);
    }

    pub fn len(&self) -> usize {
        debug_assert!(self.rdhs.len() == self.payloads.len());
        debug_assert!(self.rdhs.len() == self.mem_positions.len());
        self.rdhs.len()
    }

    pub fn is_empty(&self) -> bool {
        debug_assert!(self.rdhs.len() == self.payloads.len());
        debug_assert!(self.rdhs.len() == self.mem_positions.len());
        self.rdhs.is_empty()
    }

    pub fn clear(&mut self) {
        self.rdhs.clear();
        self.payloads.clear();
        self.mem_positions.clear();
    }

    pub fn rdh_slice(&self) -> &[T] {
        &self.rdhs
    }
}

/// Implementation of a consuming iterator for CdpChunk, with a helper struct
impl<T: RDH> IntoIterator for CdpChunk<T> {
    type Item = CdpTuple<T>; // (RDH, payload, mem_pos)
    type IntoIter = IntoIterHelper<T>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIterHelper {
            iter: self
                .rdhs
                .into_iter()
                .zip(self.payloads.into_iter())
                .zip(self.mem_positions.into_iter())
                .map(|((rdh, payload), mem_pos)| (rdh, payload, mem_pos))
                .collect::<Vec<_>>()
                .into_iter(),
        }
    }
}
pub struct IntoIterHelper<T: RDH> {
    iter: std::vec::IntoIter<CdpTuple<T>>,
}

impl<T: RDH> Iterator for IntoIterHelper<T> {
    type Item = CdpTuple<T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

type RefCdpTuple<'a, T> = (&'a T, &'a [u8], u64);
/// Implementation of a non-consuming iterator for CdpChunk, with a helper struct
impl<'a, T: RDH> IntoIterator for &'a CdpChunk<T> {
    type Item = RefCdpTuple<'a, T>;
    type IntoIter = CdpChunkIter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        CdpChunkIter {
            cdp_chunk: self,
            index: 0,
        }
    }
}

pub struct CdpChunkIter<'a, T: RDH> {
    cdp_chunk: &'a CdpChunk<T>,
    index: usize,
}

impl<'a, T: RDH> Iterator for CdpChunkIter<'a, T> {
    type Item = RefCdpTuple<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.cdp_chunk.rdhs.len() {
            let item = Some((
                self.cdp_chunk.rdhs.get(self.index)?,
                self.cdp_chunk.payloads.get(self.index)?.as_slice(),
                self.cdp_chunk.mem_positions.get(self.index)?.to_owned(),
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
    use super::CdpChunk;
    use crate::words::rdh::{RdhCRUv7, CORRECT_RDH_CRU_V6, CORRECT_RDH_CRU_V7, RDH};

    #[test]
    fn test_push() {
        let mut chunk = CdpChunk::<RdhCRUv7>::new();
        chunk.push(CORRECT_RDH_CRU_V7, vec![0; 10], 0);
        chunk.push(CORRECT_RDH_CRU_V7, vec![0; 10], 1);

        assert_eq!(chunk.rdhs.len(), 2);
        assert_eq!(chunk.payloads.len(), 2);
        assert_eq!(chunk.mem_positions.len(), 2);
    }

    #[test]
    fn test_push_tup() {
        let mut chunk = CdpChunk::<RdhCRUv7>::new();
        let tup = (CORRECT_RDH_CRU_V7, vec![0; 10], 0);
        chunk.push_tuple(tup);
        chunk.push_tuple((CORRECT_RDH_CRU_V7, vec![0; 10], 1));

        assert_eq!(chunk.rdhs.len(), 2);
        assert_eq!(chunk.payloads.len(), 2);
        assert_eq!(chunk.mem_positions.len(), 2);
    }

    #[test]
    fn test_clear() {
        let mut chunk = CdpChunk::<RdhCRUv7>::new();
        chunk.push(CORRECT_RDH_CRU_V7, vec![0; 10], 0);
        chunk.push(CORRECT_RDH_CRU_V7, vec![0; 10], 1);

        assert_eq!(chunk.rdhs.len(), 2);
        assert_eq!(chunk.payloads.len(), 2);
        assert_eq!(chunk.mem_positions.len(), 2);

        chunk.clear();

        assert_eq!(chunk.rdhs.len(), 0);
        assert_eq!(chunk.payloads.len(), 0);
        assert_eq!(chunk.mem_positions.len(), 0);
    }

    #[test]
    fn test_len() {
        let mut chunk = CdpChunk::<RdhCRUv7>::new();
        chunk.push(CORRECT_RDH_CRU_V7, vec![0; 10], 0);
        chunk.push(CORRECT_RDH_CRU_V7, vec![0; 10], 1);

        assert_eq!(chunk.len(), 2);
    }

    #[test]
    fn test_is_empty() {
        let mut chunk = CdpChunk::<RdhCRUv7>::new();
        assert!(chunk.is_empty());

        chunk.push(CORRECT_RDH_CRU_V7, vec![0; 10], 0);
        assert!(!chunk.is_empty());
    }

    #[test]
    fn test_with_capacity() {
        let chunk = CdpChunk::<RdhCRUv7>::with_capacity(10);
        assert_eq!(chunk.rdhs.capacity(), 10);
        assert_eq!(chunk.payloads.capacity(), 10);
        assert_eq!(chunk.mem_positions.capacity(), 10);
    }

    #[test]
    fn test_rdh_slice() {
        let mut chunk = CdpChunk::<RdhCRUv7>::new();
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
            mem_positions: vec![0, 1],
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
            mem_positions: vec![255, 255],
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
            mem_positions: vec![0, 1],
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
            mem_positions: vec![0xd, 0xd],
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
            mem_positions: vec![0xd, 0xd],
        };

        print_cdp_chunk(&cdp_chunk);
    }

    fn consume_cdp_chunk<T: RDH>(cdp_chunk: CdpChunk<T>) {
        for (rdh, payload, mem_pos) in cdp_chunk {
            println!("rdh: {rdh}, payload: {:?}, mem_pos: {:?}", payload, mem_pos);
        }
    }

    #[test]
    fn test_fn_consume_cdp_chunk() {
        let cdp_chunk = CdpChunk {
            rdhs: vec![CORRECT_RDH_CRU_V6, CORRECT_RDH_CRU_V6],
            payloads: vec![vec![0; 10], vec![0; 10]],
            mem_positions: vec![0xd, 0xd],
        };

        consume_cdp_chunk(cdp_chunk);

        // 'Use of moved value' compiler error
        // println!("cdp_chunk: {:?}", cdp_chunk.rdhs);
    }
}
