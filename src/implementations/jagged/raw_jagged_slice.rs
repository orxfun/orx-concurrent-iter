use super::raw_jagged::RawJagged;

pub struct RawJaggedSlice<'a, T, X>
where
    X: Fn(usize) -> [usize; 2],
{
    jagged: &'a RawJagged<T, X>,
    begin: [usize; 2],
    end: [usize; 2],
}
