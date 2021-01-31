#[derive(Clone, Copy)]
struct EnvInner<'b, K, V> {
    key: K,
    value: V,
    tail: &'b Env<'b, K, V>,
}

#[derive(Clone, Copy)]
pub struct Env<'b, K, V>(Option<EnvInner<'b, K, V>>);

impl<'b, K: PartialEq, V: Copy> Env<'b, K, V> {
    pub fn new() -> Self {
        Self(None)
    }

    pub fn get(&'b self, key: K) -> Option<V> {
        let inner = self.0.as_ref()?;
        if key == inner.key {
            Some(inner.value)
        } else {
            inner.tail.get(key)
        }
    }

    pub fn insert(&'b self, key: K, value: V) -> Self {
        Self(Some(EnvInner {
            key,
            value,
            tail: self,
        }))
    }
}
#[derive(Clone, Copy)]
struct ArgInner<'b, V> {
    value: V,
    tail: &'b Arg<'b, V>,
}
#[derive(Clone, Copy)]
pub struct Arg<'b, V>(Option<ArgInner<'b, V>>);

impl<'b, V: Copy> Arg<'b, V> {
    pub fn new() -> Self {
        Self(None)
    }

    pub fn push(&'b self, value: V) -> Self {
        Self(Some(ArgInner { value, tail: self }))
    }

    pub fn pop(&'b self) -> Option<(&'b Self, V)> {
        let inner = self.0.as_ref()?;
        Some((&inner.tail, inner.value))
    }
}

impl<'b, V: Copy + PartialEq> Arg<'b, V> {
    pub fn find(&'b self, value: V) -> Option<usize> {
        let inner = self.0.as_ref()?;
        if inner.value == value {
            Some(0)
        } else {
            inner.tail.find(value).map(|index| index + 1)
        }
    }
}
