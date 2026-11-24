use orx_concurrent_iter::implementations::jagged_arrays::{
    AsRawSlice, JaggedIndex, JaggedIndexer, RawJaggedRef, Slices,
};
use orx_concurrent_iter::{ConcurrentIter, IntoConcurrentIter};
use orx_pseudo_default::PseudoDefault;

// Indexer for rows of fixed width `n`, same idea as `MatrixIndexer` in the crate's tests.
// The methods are `unsafe fn` only because the trait declares them so.
#[derive(Clone)]
struct RowIndexer {
    n: usize,
}

impl PseudoDefault for RowIndexer {
    fn pseudo_default() -> Self {
        Self { n: 1 }
    }
}

impl JaggedIndexer for RowIndexer {
    unsafe fn jagged_index_unchecked<'a, T: 'a>(
        &self,
        _: &impl Slices<'a, T>,
        flat: usize,
    ) -> JaggedIndex {
        JaggedIndex::new(flat / self.n, flat % self.n)
    }

    unsafe fn jagged_index_unchecked_from_slice<'a, T: 'a>(
        &self,
        _: &[impl AsRawSlice<T>],
        flat: usize,
    ) -> JaggedIndex {
        JaggedIndex::new(flat / self.n, flat % self.n)
    }
}

fn main() {
    let rows: Vec<Vec<u64>> = vec![vec![1, 2, 3, 4]]; // one row, 4 elements
    let jagged = RawJaggedRef::new(rows.as_slice(), RowIndexer { n: 8 }); // total_len says 8
    let con = jagged.into_con_iter();
    let mut sum = 0u64;
    while let Some(x) = con.next() {
        sum = sum.wrapping_add(*x); // items 4..8 are read past the end of the row
    }
    println!("sum = {sum}");
}
