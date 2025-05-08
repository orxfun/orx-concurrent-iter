use super::raw_jagged::RawJagged;

pub struct RawJaggedSlice<'a, T, X> {
    jagged: &'a RawJagged<T, X>,
    begin: [usize; 2],
    end: [usize; 2],
}
