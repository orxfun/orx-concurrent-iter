use super::{
    jagged_index::JaggedIndex, raw_jagged_iter_owned::RawJaggedIterOwned,
    raw_jagged_slice::RawJaggedSlice, raw_slice::RawSlice, raw_vec::RawVec,
};
use alloc::vec::Vec;
use core::cmp::Ordering;

pub struct RawJagged<T, X>
where
    X: Fn(usize) -> [usize; 2],
{
    vectors: Vec<RawVec<T>>,
    len: usize,
    indexer: X,
    num_taken: Option<usize>,
}

impl<'a, T, X> RawJagged<T, X>
where
    X: Fn(usize) -> [usize; 2],
{
    pub fn new<I, V>(iter: I, indexer: X, droppable: bool) -> Self
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

        let num_taken = droppable.then_some(0);

        Self {
            vectors,
            len,
            indexer,
            num_taken,
        }
    }

    pub fn into_iter_owned(mut self) -> RawJaggedIterOwned<T, X> {
        match self.num_taken {
            Some(num_taken) => {
                self.num_taken = Some(self.len);
                RawJaggedIterOwned::new(self, num_taken)
            }
            None => {
                // we don't need to drop elements, return an empty iterator
                let len_as_num_taken = self.len;
                RawJaggedIterOwned::new(self, len_as_num_taken)
            }
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn vectors(&self) -> &[RawVec<T>] {
        &self.vectors
    }

    pub fn num_slices(&self) -> usize {
        self.vectors.len()
    }

    pub fn slice(&self, begin: usize, end: usize) -> RawJaggedSlice<T> {
        let begin = self.jagged_index(begin).expect("index-out-of-bounds");
        let end = self.jagged_index(end).expect("index-out-of-bounds");
        RawJaggedSlice::new(&self.vectors, begin, end)
    }

    pub fn slice_from(&self, begin: usize) -> RawJaggedSlice<T> {
        self.slice(begin, self.len)
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
        debug_assert!(num_taken >= self.num_taken.unwrap());
        if let Some(x) = self.num_taken.as_mut() {
            *x = num_taken;
        }
    }

    pub fn get_raw_slice(&self, f: usize) -> Option<RawSlice<T>> {
        match f < self.vectors.len() {
            true => {
                let slice = &self.vectors[f];
                Some(slice.raw_slice(0, slice.len()))
            }
            false => None,
        }
    }
}

impl<T, X> Drop for RawJagged<T, X>
where
    X: Fn(usize) -> [usize; 2],
{
    fn drop(&mut self) {
        let abc = 12;
        if let Some(num_taken) = self.num_taken {
            // drop elements
            if let Some(begin) = self.jagged_index(num_taken) {
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
}
