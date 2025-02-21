use super::con_iter_x_of_iter::ConIterXOfIter;
use crate::chunk_puller::ChunkPuller;
use crate::enumeration::{Element, Enumeration, Regular};
use alloc::vec::Vec;
use core::iter::FusedIterator;

pub struct ChunkPullerXOfIter<'i, I, T>
where
    T: Send + Sync,
    I: Iterator<Item = T>,
{
    con_iter: &'i ConIterXOfIter<I, T>,
    buffer: Vec<Option<T>>,
}

impl<'i, I, T> ChunkPullerXOfIter<'i, I, T>
where
    T: Send + Sync,
    I: Iterator<Item = T>,
{
    pub(super) fn new(con_iter: &'i ConIterXOfIter<I, T>, chunk_size: usize) -> Self {
        let mut buffer = Vec::with_capacity(chunk_size);
        for _ in 0..chunk_size {
            buffer.push(None);
        }
        Self { con_iter, buffer }
    }
}

impl<'i, I, T> ChunkPuller<Regular> for ChunkPullerXOfIter<'i, I, T>
where
    T: Send + Sync,
    I: Iterator<Item = T>,
{
    type ChunkItem = T;

    type Iter<'c>
        = ChunksIterXOfIter<'c, T>
    where
        Self: 'c;

    fn chunk_size(&self) -> usize {
        self.buffer.len()
    }

    fn pull(
        &mut self,
    ) -> Option<<<Regular as Enumeration>::Element as Element>::IterOf<Self::Iter<'_>>> {
        self.con_iter.get_handle().and_then(|mut handle| {
            let iter = unsafe { &mut *self.con_iter.iter.get() };
            let mut slice_len = self.buffer.len();
            for i in 0..self.buffer.len() {
                let next = iter.next();
                match next.is_some() {
                    true => self.buffer[i] = next,
                    false => {
                        slice_len = i;
                        handle.set_target_to_completed();
                        break;
                    }
                }
            }

            match slice_len {
                0 => None,
                n => {
                    let buffer = &mut self.buffer[0..n];
                    Some(ChunksIterXOfIter { buffer, current: 0 })
                }
            }
        })
    }
}

pub struct ChunksIterXOfIter<'i, T>
where
    T: Send + Sync,
{
    buffer: &'i mut [Option<T>],
    current: usize,
}

impl<'i, T> Default for ChunksIterXOfIter<'i, T>
where
    T: Send + Sync,
{
    fn default() -> Self {
        Self {
            buffer: &mut [],
            current: 0,
        }
    }
}

impl<'i, T> Iterator for ChunksIterXOfIter<'i, T>
where
    T: Send + Sync,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.buffer.get_mut(self.current).and_then(|x| {
            self.current += 1;
            x.take()
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.buffer.len().saturating_sub(self.current);
        (len, Some(len))
    }
}

impl<'i, T> ExactSizeIterator for ChunksIterXOfIter<'i, T>
where
    T: Send + Sync,
{
    fn len(&self) -> usize {
        self.buffer.len().saturating_sub(self.current)
    }
}

impl<'i, T> FusedIterator for ChunksIterXOfIter<'i, T> where T: Send + Sync {}

// TESTTESTTESTTESTTESTTESTTESTTESTTESTTESTTESTTESTTESTTESTTESTTESTTESTTESTTESTTESTTESTTESTTESTTESTTESTTESTTESTTESTTESTTESTTEST

trait Puller {
    type ChunkItem;
    type Iter<'c>: Iterator<Item = Self::ChunkItem>
    where
        Self: 'c;
    fn pull<'c>(&'c mut self) -> Option<Self::Iter<'c>>;
}

struct ChunkIter<'c, P>
where
    P: Puller + 'c,
    Self: 'c,
{
    puller: P,
    current: P::Iter<'c>,
}
impl<'c, P> ChunkIter<'c, P>
where
    P: Puller + 'c,
    Self: 'c,
{
    // fn next_chunk(&mut self) -> Option<P::ChunkItem> {
    //     match self.puller.pull() {
    //         None => None,
    //         Some(x) => {
    //             self.current = x;
    //             self.next()
    //         }
    //     }
    // }
}
impl<'c, P> Iterator for ChunkIter<'c, P>
where
    P: Puller + 'c,
    Self: 'c,
{
    type Item = P::ChunkItem;

    fn next<'d>(&'d mut self) -> Option<Self::Item> {
        let p = &mut self.puller as *mut P;
        let puller = unsafe { &mut *p };
        // let mut puller = unsafe { &mut self.puller as *mut P };
        self.current = puller.pull().unwrap();
        None
        // match self.current.next() {
        //     Some(next) => Some(next),
        //     None => {
        //         match self.puller.pull() {
        //             Some(chunk) => {
        //                 self.current = chunk;
        //                 // asdf
        //                 self.current.next()
        //             }
        //             None => None,
        //         }
        //     }
        // }
        // let next = self.current.next();
        // match next.is_some() {
        //     true => next,
        //     false => {
        //         // self.next_chunk()
        //         None
        //     }
        // }
    }
}

struct MyPuller {
    vec: Vec<usize>,
}
impl Puller for MyPuller {
    type ChunkItem = usize;

    type Iter<'c> = MyPullerIter<'c>;

    fn pull<'c>(&'c mut self) -> Option<Self::Iter<'c>> {
        Some(MyPullerIter {
            slice: self.vec.as_slice(),
        })
    }
}
// impl Iterator for MyPuller {
//     type Item = MyPullerIter;

//     fn next(&mut self) -> Option<Self::Item> {
//         Some(Self::Item {
//             slice: self.vec.as_slice(),
//         })
//     }
// }
struct MyPullerIter<'a> {
    slice: &'a [usize],
}
impl<'a> Iterator for MyPullerIter<'a> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        self.slice.get(0).cloned()
    }
}
