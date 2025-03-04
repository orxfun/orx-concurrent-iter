use crate::concurrent_iter::ConcurrentIter;
use core::{
    iter::{Cloned, Copied},
    marker::PhantomData,
};

pub struct ConIterCopied<'a, I, T>
where
    T: Send + Sync + Copy,
    I: ConcurrentIter<Item = &'a T>,
{
    con_iter: I,
    phantom: PhantomData<&'a T>,
}

impl<'a, I, T> ConIterCopied<'a, I, T>
where
    T: Send + Sync + Copy,
    I: ConcurrentIter<Item = &'a T>,
{
    pub(super) fn new(con_iter: I) -> Self {
        Self {
            con_iter,
            phantom: PhantomData,
        }
    }
}

// impl<'a, I, T> ConcurrentIter for ConIterCopied<'a, I, T>
// where
//     T: Send + Sync + Copy,
//     I: ConcurrentIter<Item = &'a T>,
// {
//     type Item = T;

//     type SequentialIter = Copied<I::SequentialIter>;

//     type ChunkPuller<'i>
//     where
//         Self: 'i;

//     fn into_seq_iter(self) -> Self::SequentialIter {
//         todo!()
//     }

//     fn skip_to_end(&self) {
//         todo!()
//     }

//     fn next(&self) -> Option<Self::Item> {
//         todo!()
//     }

//     fn next_with_idx(&self) -> Option<(usize, Self::Item)> {
//         todo!()
//     }

//     fn size_hint(&self) -> (usize, Option<usize>) {
//         todo!()
//     }
// }
