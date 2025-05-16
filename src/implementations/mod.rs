mod empty;
mod iter;
/// Generic implementations of jagged arrays or slice of slices, etc.
pub mod jagged;
/// Generic implementations of jagged arrays or slice of slices, etc.
pub mod jagged_arrays;
mod ptr_utils;
mod range;
mod slice;
mod vec;

pub use empty::ConIterEmpty;
pub use iter::ConIterOfIter;
pub use range::ConIterRange;
pub use slice::ConIterSlice;
pub use vec::ConIterVec;
