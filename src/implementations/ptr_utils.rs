use core::mem::MaybeUninit;

pub(crate) unsafe fn take<T>(ptr: *mut T) -> T {
    let mut value = MaybeUninit::<T>::uninit();
    unsafe { value.as_mut_ptr().swap(ptr) };
    unsafe { value.assume_init() }
}

pub(crate) unsafe fn read<T>(ptr: *mut T) -> T {
    unsafe { core::ptr::read(ptr) }
}
