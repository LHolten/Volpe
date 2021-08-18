struct Inner<'a, T> {
    head: &'a T,
    tail: &'a StackList<'a, T>,
}

pub struct StackList<'a, T>(Option<Inner<'a, T>>);

impl<'a, T: PartialEq> StackList<'a, T> {
    pub fn find(&self, value: &T) -> Option<usize> {
        let inner = self.0.as_ref()?;
        if inner.head == value {
            Some(0)
        } else {
            inner.tail.find(value).map(|index| index + 1)
        }
    }

    pub fn push(&'a self, value: &'a T) -> Self {
        Self(Some(Inner {
            head: value,
            tail: self,
        }))
    }

    pub fn pop(&self) -> Option<Self> {
        Some(*self.0.as_ref()?.tail)
    }
}

impl<'a, T> Copy for Inner<'a, T> {}

impl<'a, T> Clone for Inner<'a, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, T> Copy for StackList<'a, T> {}

impl<'a, T> Clone for StackList<'a, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, T> Default for StackList<'a, T> {
    fn default() -> Self {
        Self(None)
    }
}
