use super::{PropTemp, Proposition};

impl Proposition {
    pub(crate) fn contains_negation(&self) -> bool {
        match self {
            Proposition::Const(_) => false,
            Proposition::Literal(_) => false,
            Proposition::Negation(_) => true,
            Proposition::Or(bi) => bi.a.contains_negation() || bi.b.contains_negation(),
            Proposition::And(bi) => bi.a.contains_negation() || bi.b.contains_negation(),
            Proposition::E(temporal) => temporal.contains_negation(),
            Proposition::A(temporal) => temporal.contains_negation(),
        }
    }
}

impl PropTemp {
    fn contains_negation(&self) -> bool {
        match self {
            PropTemp::X(inner) => inner.0.contains_negation(),
            PropTemp::F(inner) => inner.0.contains_negation(),
            PropTemp::G(inner) => inner.0.contains_negation(),
            PropTemp::U(inner) => inner.hold.contains_negation() || inner.until.contains_negation(),
            PropTemp::R(inner) => {
                inner.releaser.contains_negation() || inner.releasee.contains_negation()
            }
        }
    }
}
