use mck::misc::MetaEq;
use std::fmt::Debug;
use std::hash::Hash;

/// Wrapper structure allowing to use meta-equality as equality.
///
/// Abstract states in meta wrap can be compared against each other.
/// Outside the meta-wrap, there is no danger of confusing equality
/// of abstract states and equality of concrete states.
#[derive(Clone)]
pub struct MetaWrap<E: MetaEq + Debug + Clone + Hash>(pub E);

impl<E: MetaEq + Debug + Clone + Hash> PartialEq for MetaWrap<E> {
    fn eq(&self, other: &Self) -> bool {
        self.0.meta_eq(&other.0)
    }
}
impl<E: MetaEq + Debug + Clone + Hash> Eq for MetaWrap<E> {}

impl<E: MetaEq + Debug + Clone + Hash> Hash for MetaWrap<E> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}
