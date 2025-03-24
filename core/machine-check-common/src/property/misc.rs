use super::{Property, TemporalOperator};

impl Property {
    pub(crate) fn contains_negation(&self) -> bool {
        match self {
            Property::Const(_) => false,
            Property::Atomic(_) => false,
            Property::Negation(_) => true,
            Property::Or(bi) => bi.a.contains_negation() || bi.b.contains_negation(),
            Property::And(bi) => bi.a.contains_negation() || bi.b.contains_negation(),
            Property::E(temporal) => temporal.contains_negation(),
            Property::A(temporal) => temporal.contains_negation(),
        }
    }
}

impl TemporalOperator {
    fn contains_negation(&self) -> bool {
        match self {
            TemporalOperator::X(inner) => inner.0.contains_negation(),
            TemporalOperator::F(inner) => inner.0.contains_negation(),
            TemporalOperator::G(inner) => inner.0.contains_negation(),
            TemporalOperator::U(inner) => {
                inner.hold.contains_negation() || inner.until.contains_negation()
            }
            TemporalOperator::R(inner) => {
                inner.releaser.contains_negation() || inner.releasee.contains_negation()
            }
        }
    }
}
