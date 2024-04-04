#[derive(Debug)]
pub struct Next<T> {
    pub idx: usize,
    pub value: T,
}

#[derive(Debug)]
pub struct NextMany<T, Iter>
where
    Iter: Iterator<Item = T>,
{
    pub begin_idx: usize,
    pub values: Iter,
}

impl<T, Iter> From<NextMany<T, Iter>> for (usize, Iter)
where
    Iter: Iterator<Item = T>,
{
    fn from(value: NextMany<T, Iter>) -> Self {
        (value.begin_idx, value.values)
    }
}
