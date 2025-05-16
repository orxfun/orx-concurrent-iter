use crate::implementations::jagged_arrays::JaggedIndexer;

pub trait AsRawJaggedRef<'a, T, X>: Default
where
    X: JaggedIndexer,
{
    fn slice(&self, f: usize, begin_i: usize, len: usize) -> Option<&'a [T]>;

    fn len_of(&self, f: usize) -> usize;
}
