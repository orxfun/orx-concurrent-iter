use super::raw_slice::RawSlice;
use alloc::vec::Vec;

pub struct RawSlices<'a, T> {
    slices: Vec<RawSlice<'a, T>>,
}
