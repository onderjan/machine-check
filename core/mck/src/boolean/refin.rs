use std::num::NonZeroU8;

use serde::{Deserialize, Serialize};

use crate::{
    abstr,
    backward::Bitwise,
    misc::{Meta, MetaEq},
    three_valued::ThreeValued,
};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub struct Boolean(u8);

impl Boolean {
    pub fn new_unmarked() -> Self {
        Self(0)
    }

    pub fn new_marked(importance: NonZeroU8) -> Self {
        Self(importance.get())
    }

    pub fn new_marked_unimportant() -> Self {
        Self(1)
    }

    pub fn importance(&self) -> u8 {
        self.0
    }

    pub fn apply_refin(&mut self, offer: &Self) -> bool {
        // offer should be marked and ours unmarked
        if offer.0 != 0 && self.0 == 0 {
            self.0 = offer.0;
            true
        } else {
            false
        }
    }

    pub fn apply_join(&mut self, other: &Self) {
        self.0 = self.0.max(other.0);
    }

    pub fn to_condition(&self) -> Boolean {
        *self
    }

    pub fn force_decay(&self, target: &mut super::abstr::Boolean) {
        // unmarked becomes unknown
        if self.0 == 0 {
            *target = super::abstr::Boolean::from_three_valued(ThreeValued::Unknown);
        }
    }

    pub fn limit(self, abstr: &abstr::Boolean) -> Self {
        // lose mark if abstr is not unknown
        if !abstr.is_unknown() {
            Self(0)
        } else {
            self
        }
    }
}

impl Meta<super::abstr::Boolean> for Boolean {
    fn proto_first(&self) -> super::abstr::Boolean {
        let result = if self.0 == 0 {
            // unmarked, return unknown
            ThreeValued::Unknown
        } else {
            // marked, start with false
            ThreeValued::False
        };

        super::abstr::Boolean::from_three_valued(result)
    }

    fn proto_increment(&self, proto: &mut super::abstr::Boolean) -> bool {
        if proto.into_three_valued().is_false() {
            // move to true
            *proto = super::abstr::Boolean::from_three_valued(ThreeValued::True);
            true
        } else {
            // end
            false
        }
    }
}

impl Bitwise for abstr::Boolean {
    type Mark = Boolean;

    fn bit_not(_normal_input: (Self,), mark_later: Self::Mark) -> (Self::Mark,) {
        (mark_later,)
    }

    fn bit_and(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark) {
        (
            mark_later.limit(&normal_input.0),
            mark_later.limit(&normal_input.1),
        )
    }

    fn bit_or(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark) {
        (
            mark_later.limit(&normal_input.0),
            mark_later.limit(&normal_input.1),
        )
    }

    fn bit_xor(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark) {
        (
            mark_later.limit(&normal_input.0),
            mark_later.limit(&normal_input.1),
        )
    }
}

impl MetaEq for Boolean {
    fn meta_eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

/*impl BoolConvert<RBitvector> for Boolean {
    fn bool_from(value: RBitvector) -> Self {
        assert_eq!(value.width(), 1);

        Self(value.importance())
    }

    fn bool_into(value: Self) -> RBitvector {
        value.to_runtime_bitvector()
    }
}*/
