use crate::implementations::jagged_arrays::AsSlice;

pub trait Slices<'a, T, S>
where
    S: AsSlice<T>,
{
    //
}
