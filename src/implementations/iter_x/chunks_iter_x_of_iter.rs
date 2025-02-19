use super::con_iter_x_of_iter::ConIterXOfIter;
use crate::chunk_puller::ChunkPuller;
use crate::enumeration::{Element, Enumeration, Regular};
use alloc::vec::Vec;
use core::ops::{Add, Range};

pub struct ChunksIterXOfIter<'i, I, T>
where
    T: Send + Sync,
    I: Iterator<Item = T>,
{
    con_iter: &'i ConIterXOfIter<I, T>,
    buffer: Vec<Option<T>>,
}

impl<'i, I, T> ChunksIterXOfIter<'i, I, T>
where
    T: Send + Sync,
    I: Iterator<Item = T>,
{
    pub(super) fn new(con_iter: &'i ConIterXOfIter<I, T>, chunk_size: usize) -> Self {
        Self {
            con_iter,
            buffer: (0..chunk_size).map(|_| None).collect(),
        }
    }
}

// impl<'i, I, T> ChunkPuller<Regular> for ChunksIterXOfIter<'i, I, T>
// where
//     T: Send + Sync,
//     I: Iterator<Item = T>,
// {
//     type ChunkItem = T;

//     type Iter;

//     fn chunk_size(&self) -> usize {
//         todo!()
//     }
// }
