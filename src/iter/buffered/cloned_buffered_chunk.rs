use super::buffered_chunk::BufferedChunk;
use crate::{iter::cloned::Cloned, NextChunk};
use std::marker::PhantomData;

pub struct ClonedBufferedChunk<'a, T, C>
where
    T: Send + Sync + Clone,
    C: BufferedChunk<&'a T>,
{
    chunk: C,
    phantom: PhantomData<&'a T>,
}

impl<'a, T, C> BufferedChunk<T> for ClonedBufferedChunk<'a, T, C>
where
    T: Send + Sync + Clone,
    C: BufferedChunk<&'a T>,
{
    type ConIter = Cloned<'a, T, C::ConIter>;

    fn new(chunk_size: usize) -> Self {
        Self {
            chunk: C::new(chunk_size),
            phantom: PhantomData,
        }
    }

    fn chunk_size(&self) -> usize {
        self.chunk.chunk_size()
    }

    fn pull(
        &mut self,
        iter: &Self::ConIter,
    ) -> Option<NextChunk<T, impl ExactSizeIterator<Item = T>>> {
        self.chunk.pull(iter.underlying_iter()).map(|x| x.cloned())
    }
}

// X

// pub struct ClonedBufferedChunkX<'a, T, C>
// where
//     T: Send + Sync + Clone,
//     C: BufferedChunkX<&'a T>,
// {
//     chunk: C,
//     phantom: PhantomData<&'a T>,
// }

// impl<'a, T, C> BufferedChunkX<T> for ClonedBufferedChunkX<'a, T, C>
// where
//     T: Send + Sync + Clone,
//     C: BufferedChunkX<&'a T>,
// {
//     type ConIter = Cloned<'a, T, C::ConIter>;

//     fn new(chunk_size: usize) -> Self {
//         Self {
//             chunk: C::new(chunk_size),
//             phantom: PhantomData,
//         }
//     }

//     fn chunk_size(&self) -> usize {
//         self.chunk.chunk_size()
//     }

//     fn pull(
//         &mut self,
//         iter: &Self::ConIter,
//     ) -> Option<NextChunk<T, impl ExactSizeIterator<Item = T>>> {
//         self.chunk.pull(iter.underlying_iter()).map(|x| x.cloned())
//     }
// }
