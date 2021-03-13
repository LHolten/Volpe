use std::cell::Cell;

pub trait WithInternal<T> {
    fn with<R>(&self, func: impl FnOnce(&mut T) -> R) -> R;
}

impl<T: Default> WithInternal<T> for Cell<T> {
    fn with<R>(&self, func: impl FnOnce(&mut T) -> R) -> R {
        let mut temp = self.replace(T::default());
        let res = func(&mut temp);
        self.set(temp);
        res
    }
}