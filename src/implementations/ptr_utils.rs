use core::mem::MaybeUninit;

pub(crate) unsafe fn take<T>(ptr: *mut T) -> T {
    let mut value = MaybeUninit::<T>::uninit();
    value.as_mut_ptr().swap(ptr);
    value.assume_init()
}
