use crate::{
    concurrent_iter::ConcurrentIter,
    enumeration::{Element, Enumerated, Enumeration, IsEnumerated, IsNotEnumerated, Regular},
};
use alloc::vec::Vec;
use core::{
    iter::Skip,
    marker::PhantomData,
    sync::atomic::{AtomicUsize, Ordering},
};

use super::chunks_iter_vec::ChunksIterVec;

pub struct ConIterVec<T, E = Regular>
where
    T: Send + Sync,
    E: Enumeration,
{
    vec: Vec<T>,
    counter: AtomicUsize,
    phantom: PhantomData<E>,
}

impl<T, E> Default for ConIterVec<T, E>
where
    T: Send + Sync,
    E: Enumeration,
{
    fn default() -> Self {
        Self {
            vec: Default::default(),
            counter: 0.into(),
            phantom: PhantomData,
        }
    }
}

impl<T, E> ConcurrentIter<E> for ConIterVec<T, E>
where
    T: Send + Sync,
    E: Enumeration,
{
    type Item = T;

    type SeqIter = Skip<alloc::vec::IntoIter<T>>;

    type ChunkPuller<'i>
        = ChunksIterVec<'i, T, E>
    where
        Self: 'i;

    type Regular = ConIterVec<T, Regular>;

    type Enumerated = ConIterVec<T, Enumerated>;

    fn into_seq_iter(self) -> Self::SeqIter {
        todo!()
    }

    fn enumerated(self) -> Self::Enumerated
    where
        E: IsNotEnumerated,
    {
        todo!()
    }

    fn not_enumerated(self) -> Self::Regular
    where
        E: IsEnumerated,
    {
        todo!()
    }

    fn skip_to_end(&self) {
        todo!()
    }

    fn next(&self) -> Option<<<E as Enumeration>::Element as Element>::ElemOf<Self::Item>> {
        todo!()
    }

    fn chunks_iter(&self, chunk_size: usize) -> Self::ChunkPuller<'_> {
        todo!()
    }
}
