use std::num::NonZeroU8;

use serde::{Deserialize, Serialize};

use crate::{
    abstr,
    backward::Bitwise,
    concr::BoolConvert,
    misc::{Meta, MetaEq},
    refin::RBitvector,
};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub struct Boolean(pub(crate) RBitvector);

impl Boolean {
    pub fn new_unmarked() -> Self {
        Self(RBitvector::new_unmarked(1))
    }

    pub fn new_marked(importance: NonZeroU8) -> Self {
        Self(RBitvector::new_marked(importance, 1))
    }

    pub fn new_marked_unimportant() -> Self {
        Self(RBitvector::new_marked_unimportant(1))
    }

    pub fn importance(&self) -> u8 {
        self.0.importance()
    }

    pub fn apply_refin(&mut self, offer: &Self) -> bool {
        self.0.apply_refin(&offer.0)
    }

    pub fn apply_join(&mut self, other: &Self) {
        self.0.apply_join(&other.0)
    }

    pub fn to_condition(&self) -> Boolean {
        self.0.to_condition()
    }

    pub fn force_decay(&self, target: &mut super::abstr::Boolean) {
        let mut runtime = target.0.as_runtime_bitvector();
        self.0.force_decay(&mut runtime);
        target.0 = crate::abstr::Bitvector::from_runtime_bitvector(runtime);
    }

    pub fn to_runtime_bitvector(self) -> RBitvector {
        self.0
    }

    pub fn limit(self, abstr: &abstr::Boolean) -> Self {
        let runtime = abstr.0.as_runtime_bitvector();
        Self(self.0.limit(&runtime))
    }
}

impl Meta<super::abstr::Boolean> for Boolean {
    fn proto_first(&self) -> super::abstr::Boolean {
        super::abstr::Boolean(crate::abstr::Bitvector::from_runtime_bitvector(
            self.0.proto_first(),
        ))
    }

    fn proto_increment(&self, proto: &mut super::abstr::Boolean) -> bool {
        let mut runtime = proto.0.as_runtime_bitvector();
        let result = self.0.proto_increment(&mut runtime);
        proto.0 = abstr::Bitvector::from_runtime_bitvector(runtime);
        result
    }
}

impl Bitwise for abstr::Boolean {
    type Mark = Boolean;

    fn bit_not(normal_input: (Self,), mark_later: Self::Mark) -> (Self::Mark,) {
        let mark_earlier =
            Bitwise::bit_not((normal_input.0 .0.as_runtime_bitvector(),), mark_later.0);
        (Boolean(mark_earlier.0),)
    }

    fn bit_and(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark) {
        let out = Bitwise::bit_and(
            (
                normal_input.0 .0.as_runtime_bitvector(),
                normal_input.1 .0.as_runtime_bitvector(),
            ),
            mark_later.0,
        );
        (Boolean(out.0), Boolean(out.1))
    }

    fn bit_or(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark) {
        let out = Bitwise::bit_or(
            (
                normal_input.0 .0.as_runtime_bitvector(),
                normal_input.1 .0.as_runtime_bitvector(),
            ),
            mark_later.0,
        );
        (Boolean(out.0), Boolean(out.1))
    }

    fn bit_xor(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark) {
        let out = Bitwise::bit_xor(
            (
                normal_input.0 .0.as_runtime_bitvector(),
                normal_input.1 .0.as_runtime_bitvector(),
            ),
            mark_later.0,
        );
        (Boolean(out.0), Boolean(out.1))
    }
}

impl MetaEq for Boolean {
    fn meta_eq(&self, other: &Self) -> bool {
        self.0.meta_eq(&other.0)
    }
}

impl BoolConvert<RBitvector> for Boolean {
    fn bool_from(value: RBitvector) -> Self {
        assert_eq!(value.width(), 1);

        Self(value)
    }

    fn bool_into(value: Self) -> RBitvector {
        value.0
    }
}
