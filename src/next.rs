mod sealed {
    pub trait NextSealed<T> {
        fn new(begin_idx: usize, value: T) -> Self;
    }

    impl<T> NextSealed<T> for T {
        #[inline(always)]
        fn new(_: usize, value: T) -> Self {
            value
        }
    }

    impl<T> NextSealed<T> for (usize, T) {
        #[inline(always)]
        fn new(begin_idx: usize, value: T) -> Self {
            (begin_idx, value)
        }
    }
}

pub trait Next<T>: sealed::NextSealed<T> {}

impl<T> Next<T> for T {}

impl<T> Next<T> for (usize, T) {}
