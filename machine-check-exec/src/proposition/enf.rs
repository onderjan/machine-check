use super::{PropBi, PropF, PropG, PropR, PropTemp, PropU, PropUni, Proposition};

impl Proposition {
    #[must_use]
    pub fn enf(&self) -> Self {
        match self {
            Proposition::Const(_) => self.clone(),
            Proposition::Literal(_) => self.clone(),
            Proposition::Negation(inner) => Proposition::Negation(inner.enf()),
            Proposition::Or(v) => Proposition::Or(v.enf()),
            Proposition::And(v) => Proposition::And(v.enf()),
            // only convert F and R from E
            Proposition::E(temporal) => Proposition::E(match temporal {
                PropTemp::X(inner) => PropTemp::X(inner.enf()),
                PropTemp::F(inner) => {
                    // rewrite as EF[p] = E[true U p]
                    PropTemp::U(inner.expanded().enf())
                }
                PropTemp::G(inner) => PropTemp::G(inner.enf()),
                PropTemp::U(inner) => PropTemp::U(inner.enf()),
                PropTemp::R(inner) => {
                    // for convenience, rewrite as E[p R q] = !A[!p U !q]
                    let au = make_negated(Proposition::A(PropTemp::U(inner.negated())));
                    // and perform ENF on that
                    return au.enf();
                }
            }),
            // convert all A to E
            // they will (except for AU) all start with !E, add it outside the match
            Proposition::A(temporal) => make_negated(Proposition::E(match temporal {
                PropTemp::X(inner) => {
                    // AX[p] = !EX[!p]
                    PropTemp::X(PropUni::new(make_negated_box(inner.enf().0)))
                }
                PropTemp::F(inner) => {
                    // AF[p] = !EG[!p]
                    PropTemp::G(inner.negated().enf())
                }

                PropTemp::G(inner) => {
                    // AG[p] = !EF[!p] = !E[true U !p]
                    PropTemp::U(inner.negated().expanded().enf())
                }
                PropTemp::U(inner) => {
                    // the most problematic case
                    // A[p U q] = !(E[!q U !(p or q)] or EG[!q])
                    let hold_enf = inner.hold.enf();
                    let until_enf = inner.until.enf();

                    let eu_part = Proposition::E(PropTemp::U(PropU {
                        hold: Box::new(Proposition::Negation(PropUni::new(until_enf.clone()))),
                        until: Box::new(Proposition::Negation(PropUni::new(Proposition::Or(
                            PropBi {
                                a: Box::new(hold_enf),
                                b: Box::new(until_enf.clone()),
                            },
                        )))),
                    }));
                    let eg_part =
                        Proposition::E(PropTemp::G(PropG(Box::new(make_negated(until_enf)))));
                    return Proposition::Negation(PropUni::new(Proposition::Or(PropBi {
                        a: Box::new(eu_part),
                        b: Box::new(eg_part),
                    })));
                }
                PropTemp::R(inner) => {
                    // A[p R q] = !E[!p U !q]
                    PropTemp::U(inner.negated().enf())
                }
            })),
        }
    }
}

impl PropUni {
    #[must_use]
    pub fn enf(&self) -> Self {
        PropUni(Box::new(self.0.enf()))
    }
}

impl PropF {
    #[must_use]
    pub fn negated(&self) -> PropG {
        PropG(Box::new(make_negated((*self.0).clone())))
    }

    #[must_use]
    pub fn expanded(&self) -> PropU {
        // F[p] expands to [1 U p]
        PropU {
            hold: Box::new(Proposition::Const(true)),
            until: Box::clone(&self.0),
        }
    }
}

impl PropG {
    #[must_use]
    pub fn enf(&self) -> Self {
        PropG(Box::new(self.0.enf()))
    }

    #[must_use]
    pub fn negated(&self) -> PropF {
        PropF(Box::new(make_negated((*self.0).clone())))
    }
}

impl PropBi {
    #[must_use]
    pub fn enf(&self) -> Self {
        PropBi {
            a: Box::new(self.a.enf()),
            b: Box::new(self.b.enf()),
        }
    }
}

impl PropU {
    #[must_use]
    pub fn enf(&self) -> Self {
        PropU {
            hold: Box::new(self.hold.enf()),
            until: Box::new(self.until.enf()),
        }
    }
}

impl PropR {
    #[must_use]
    pub fn negated(&self) -> PropU {
        PropU {
            hold: Box::new(make_negated((*self.hold).clone())),
            until: Box::new(make_negated((*self.release).clone())),
        }
    }
}

fn make_negated_box(prop: Box<Proposition>) -> Proposition {
    Proposition::Negation(PropUni(prop))
}

fn make_negated(prop: Proposition) -> Proposition {
    Proposition::Negation(PropUni::new(prop))
}
