use std::{
    cell::Cell,
    rc::{Rc, Weak},
};

pub trait WithInternal<T> {
    fn with<R>(&self, func: impl FnOnce(&mut T) -> R) -> R;
}

impl<T: Default> WithInternal<T> for Cell<T> {
    fn with<R>(&self, func: impl FnOnce(&mut T) -> R) -> R {
        let mut temp = self.take();
        let res = func(&mut temp);
        self.set(temp);
        res
    }
}

pub trait UpgradeInternal<T> {
    fn upgrade(&self) -> Option<Rc<T>>;
}

impl<T> UpgradeInternal<T> for Cell<Weak<T>> {
    fn upgrade(&self) -> Option<Rc<T>> {
        self.with(|inner| inner.upgrade())
    }
}
