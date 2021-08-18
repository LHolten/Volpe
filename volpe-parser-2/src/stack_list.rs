#[derive(Clone, Copy)]
struct Inner<'b, T> {
    head: &'b T,
    tail: &'b StackList<'b, T>,
}

#[derive(Clone, Copy)]
pub struct StackList<'b, T>(Option<Inner<'b, T>>);

impl<'b, T> Default for StackList<'b, T> {
    fn default() -> Self {
        Self(None)
    }
}

impl<'b, T: PartialEq> StackList<'b, T> {
    pub fn find(&self, value: &T) -> Option<usize> {
        let inner = self.0.as_ref()?;
        if inner.head == value {
            Some(0)
        } else {
            inner.tail.find(value).map(|index| index + 1)
        }
    }

    pub fn push(&'b self, value: &'b T) -> Self {
        Self(Some(Inner {
            head: value,
            tail: self,
        }))
    }

    // pub fn pop(&self) -> Option<&'b Self> {
    //     let inner = self.0.as_ref()?;
    //     Some(inner.tail)
    // }
}
