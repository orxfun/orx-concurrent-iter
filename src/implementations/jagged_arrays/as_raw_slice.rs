use crate::implementations::jagged_arrays::RawSlice;
use alloc::vec::Vec;

/// A type that can be represented as a slice.
pub trait AsRawSlice<T> {
    /// Beginning of the slice.
    fn ptr(&self) -> *const T;

    /// Length of the slice.
    fn length(&self) -> usize;

    /// Creates a slice from this slice with `len` elements starting from the `begin`.
    fn raw_slice(&self, begin: usize, len: usize) -> RawSlice<T>;

    // provided

    /// True if length is zero; false otherwise.
    fn is_empty(&self) -> bool {
        self.length() == 0
    }

    /// Returns pointers to the first and last, (len-1)-th, element of the slice.
    ///
    /// If the slice is empty, both pointers are null.
    fn first_and_last_ptrs(&self) -> [*const T; 2] {
        match self.length() {
            0 => [core::ptr::null(), core::ptr::null()],
            n => [self.ptr(), unsafe { self.ptr_at(n - 1) }],
        }
    }

    /// Returns the pointer to the `position`-th element of the slice.
    ///
    /// # SAFETY
    ///
    /// Must ensure that `position` is in bounds; otherwise,
    /// the method accesses out of bounds memory if `position >= self.len()`.
    unsafe fn ptr_at(&self, position: usize) -> *const T {
        debug_assert!(position < self.length());
        unsafe { self.ptr().add(position) }
    }
}

/// A type that can be represented as a slice which also owns the data, such as the std vec.
pub trait AsOwningSlice<T>: AsRawSlice<T> {
    /// Capacity of the allocation.
    fn capacity(&self) -> usize;

    /// Drops the `position`-th element of the slice.
    ///
    /// # SAFETY
    ///
    /// Must ensure that `position` is in bounds; otherwise,
    /// the method accesses out of bounds memory if `position >= self.len()`.
    ///
    /// After this call, the corresponding element of the slice must be considered as
    /// uninitialized memory and should not be accessed.
    ///
    /// Further, it must not be attempted to drop the second time.
    unsafe fn drop_in_place(&self, position: usize) {
        let ptr = unsafe { self.ptr_at(position) } as *mut T;
        unsafe { ptr.drop_in_place() };
    }

    /// Drops the allocation and releases the memory used by the owning slice.
    ///
    /// # SAFETY
    ///
    /// Once allocation is dropped, it is not safe to use any of the methods except for `len` and
    /// `capacity`, since the memory that the pointers point to does not belong to the slice now.
    unsafe fn drop_allocation(&self) {
        let _vec_to_drop = unsafe { Vec::from_raw_parts(self.ptr() as *mut T, 0, self.capacity()) };
    }
}

// implementations

impl<'a, T> AsRawSlice<T> for &'a [T] {
    fn ptr(&self) -> *const T {
        self.as_ptr()
    }

    fn length(&self) -> usize {
        self.len()
    }

    fn raw_slice(&self, begin: usize, len: usize) -> RawSlice<T> {
        debug_assert!(begin < self.len());
        let ptr = unsafe { self.as_ptr().add(begin) };
        RawSlice::new(ptr, len)
    }
}

impl<T> AsRawSlice<T> for Vec<T> {
    fn ptr(&self) -> *const T {
        self.as_ptr()
    }

    fn length(&self) -> usize {
        self.len()
    }

    fn raw_slice(&self, begin: usize, len: usize) -> RawSlice<T> {
        debug_assert!(begin < self.len());
        let ptr = unsafe { self.as_ptr().add(begin) };
        RawSlice::new(ptr, len)
    }
}
