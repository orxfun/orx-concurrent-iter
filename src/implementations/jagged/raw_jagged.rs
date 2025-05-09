use super::{jagged_index::JaggedIndex, raw_jagged_slice::RawJaggedSlice, raw_vec::RawVec};
use alloc::vec::Vec;
use core::cmp::Ordering;
use std::fmt::Debug;

pub struct RawJagged<T, X>
where
    X: Fn(usize) -> [usize; 2],
    T: Debug,
{
    vectors: Vec<RawVec<T>>,
    len: usize,
    indexer: X,
    num_taken: usize,
}

impl<'a, T, X> RawJagged<T, X>
where
    X: Fn(usize) -> [usize; 2],
    T: Debug,
{
    pub fn new<I, V>(iter: I, indexer: X) -> Self
    where
        V: Into<RawVec<T>>,
        I: Iterator<Item = V>,
    {
        let mut len = 0;
        let mut vectors = match iter.size_hint() {
            (lb, Some(ub)) if lb == ub => Vec::with_capacity(lb),
            _ => Vec::new(),
        };

        for v in iter {
            let v = v.into();
            len += v.len();
            vectors.push(v);
        }

        Self {
            vectors,
            len,
            indexer,
            num_taken: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn num_slices(&self) -> usize {
        self.vectors.len()
    }

    pub fn slice(&self, begin: usize, end: usize) -> RawJaggedSlice<T> {
        let begin = self.jagged_index(begin).expect("index-out-of-bounds");
        let end = self.jagged_index(end).expect("index-out-of-bounds");
        RawJaggedSlice::new(&self.vectors, begin, end)
    }

    pub fn jagged_index(&self, flat_index: usize) -> Option<JaggedIndex> {
        match flat_index.cmp(&self.len) {
            Ordering::Equal => {
                let f = self.vectors.len() - 1;
                let i = self.vectors[f].len();
                Some(JaggedIndex::new(f, i))
            }
            Ordering::Less => {
                let [f, i] = (self.indexer)(flat_index);
                Some(JaggedIndex::new(f, i))
            }
            Ordering::Greater => None,
        }
    }

    pub fn set_num_taken(&mut self, num_taken: usize) {
        debug_assert!(num_taken >= self.num_taken);
        self.num_taken = num_taken;
    }
}

impl<T, X> Drop for RawJagged<T, X>
where
    X: Fn(usize) -> [usize; 2],
    T: Debug,
{
    fn drop(&mut self) {
        // drop elements
        if let Some(begin) = self.jagged_index(self.num_taken) {
            let [f, i] = [begin.f, begin.i];
            unsafe { self.vectors[f].drop_elements_in_place(i) };

            for f in (f + 1)..self.vectors.len() {
                unsafe { self.vectors[f].drop_elements_in_place(0) };
            }
        }

        // drop allocation
        for vec in &self.vectors {
            unsafe { vec.drop_allocation() };
        }
    }
}
