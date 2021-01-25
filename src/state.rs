#[derive(Clone)]
struct EnvInner<'b, K, V> {
    key: K,
    value: V,
    tail: &'b Env<'b, K, V>,
}

#[derive(Clone)]
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

struct ArgInner<'b, V> {
    value: V,
    tail: &'b Arg<'b, V>,
}

pub struct Arg<'b, V>(Option<ArgInner<'b, V>>);

impl<'b, V: Copy> Arg<'b, V> {
    pub fn new() -> Self {
        Self(None)
    }

    pub fn pop(&'b self) -> Option<(&'b Self, V)> {
        let inner = self.0.as_ref()?;
        Some((&inner.tail, inner.value))
    }

    pub fn push(&'b self, value: V) -> Self {
        Self(Some(ArgInner { value, tail: self }))
    }
}
