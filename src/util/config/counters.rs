pub trait CountersOpt {
    fn cdps(&self) -> u16;
}

impl<T> CountersOpt for &T
where
    T: CountersOpt,
{
    fn cdps(&self) -> u16 {
        (*self).cdps()
    }
}

impl<T> CountersOpt for Box<T>
where
    T: CountersOpt,
{
    fn cdps(&self) -> u16 {
        (**self).cdps()
    }
}

impl<T> CountersOpt for std::sync::Arc<T>
where
    T: CountersOpt,
{
    fn cdps(&self) -> u16 {
        (**self).cdps()
    }
}
