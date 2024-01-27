pub trait FieldManipulate<T> {
    #[must_use]
    fn get(&self, name: &str) -> Option<&T>;
    #[must_use]
    fn get_mut(&mut self, name: &str) -> Option<&mut T>;
}

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
