pub trait FieldManipulate<T> {
    fn get(&self, name: &str) -> Option<&T>;
    fn get_mut(&mut self, name: &str) -> Option<&mut T>;
}

pub trait Meta: Sized {
    type Proto: Clone;

    fn proto_first(&self) -> Self::Proto;
    fn proto_increment(&self, proto: &mut Self::Proto) -> bool;

    fn into_proto_iter(self) -> ProtoIterator<Self> {
        ProtoIterator {
            meta: self,
            current_proto: None,
        }
    }
}

pub struct ProtoIterator<M: Meta> {
    meta: M,
    current_proto: Option<M::Proto>,
}

impl<M: Meta> Iterator for ProtoIterator<M> {
    type Item = M::Proto;

    fn next(&mut self) -> Option<M::Proto> {
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
