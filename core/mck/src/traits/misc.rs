use std::fmt::Debug;
use std::hash::Hash;

pub trait MetaEq {
    fn meta_eq(&self, other: &Self) -> bool;
}

pub trait Meta<P: Clone>: Sized {
    fn proto_first(&self) -> P;
    fn proto_increment(&self, proto: &mut P) -> bool;

    fn into_proto_iter(self) -> ProtoIterator<Self, P> {
        ProtoIterator {
            meta: self,
            current_proto: None,
        }
    }
}

/// Wrapper structure allowing to use meta-equality as equality.
///
/// Types in meta wrap can be compared against each other.
/// Outside the meta-wrap, there is no danger of confusing
/// meta-equality with equality.
pub struct MetaWrap<E: MetaEq>(pub E);

impl<E: MetaEq> PartialEq for MetaWrap<E> {
    fn eq(&self, other: &Self) -> bool {
        self.0.meta_eq(&other.0)
    }
}
impl<E: MetaEq> Eq for MetaWrap<E> {}

impl<E: MetaEq + Debug> Debug for MetaWrap<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // just debug the field
        self.0.fmt(f)
    }
}

impl<E: MetaEq + Clone> Clone for MetaWrap<E> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<E: MetaEq + Copy> Copy for MetaWrap<E> {}

impl<E: MetaEq + Hash> Hash for MetaWrap<E> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

pub struct ProtoIterator<M: Meta<P>, P: Clone> {
    meta: M,
    current_proto: Option<P>,
}

impl<M: Meta<P>, P: Clone> Iterator for ProtoIterator<M, P> {
    type Item = P;

    fn next(&mut self) -> Option<P> {
        let current = self.current_proto.take();
        self.current_proto = if let Some(mut current) = current {
            if self.meta.proto_increment(&mut current) {
                Some(current)
            } else {
                None
            }
        } else {
            Some(self.meta.proto_first())
        };
        self.current_proto.clone()
    }
}

pub trait PanicMessage {
    fn panic_message(panic_id: u32) -> &'static str;
}
