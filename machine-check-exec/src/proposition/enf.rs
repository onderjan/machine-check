use super::{PropBi, PropU, PropUni, Proposition};

impl PropUni {
    #[must_use]
    pub fn enf(&self) -> Self {
        PropUni(Box::new(self.0.enf()))
    }

    pub fn new(prop: Proposition) -> Self {
        PropUni(Box::new(prop))
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

impl Proposition {
    #[must_use]
    pub fn enf(&self) -> Self {
        match self {
            Proposition::Const(_) => self.clone(),
            Proposition::Literal(_) => self.clone(),
            Proposition::Negation(inner) => Proposition::Negation(inner.enf()),
            Proposition::Or(v) => Proposition::Or(v.enf()),
            Proposition::And(v) => Proposition::And(v.enf()),
            Proposition::EX(inner) => Proposition::EX(inner.enf()),
            Proposition::EG(inner) => Proposition::EG(inner.enf()),
            Proposition::EU(inner) => Proposition::EU(inner.enf()),
            Proposition::EF(inner) => {
                // EF[p] = E[true U p]
                Proposition::EU(PropU {
                    hold: Box::new(Proposition::Const(true)),
                    until: Box::new(inner.0.enf()),
                })
            }
            Proposition::AX(inner) => {
                // AX[p] = !EX[!p]
                Proposition::Negation(PropUni::new(Proposition::EX(PropUni::new(
                    Proposition::Negation(inner.enf()),
                ))))
            }
            Proposition::AF(inner) => {
                // AF[p] = !EG[!p]
                Proposition::Negation(PropUni::new(Proposition::EG(PropUni::new(
                    Proposition::Negation(inner.enf()),
                ))))
            }
            Proposition::AG(inner) => {
                // AG[p] = !EF[!p] = !E[true U !p]
                Proposition::Negation(PropUni::new(Proposition::EU(PropU {
                    hold: Box::new(Proposition::Const(true)),
                    until: Box::new(Proposition::Negation(inner.enf())),
                })))
            }
            Proposition::AU(inner) => {
                let hold_enf = inner.hold.enf();
                let until_enf = inner.until.enf();

                // A[p U q] = !(E[!q U !(p or q)] or EG[!q])
                let eu_part = Proposition::EU(PropU {
                    hold: Box::new(Proposition::Negation(PropUni::new(until_enf.clone()))),
                    until: Box::new(Proposition::Negation(PropUni::new(Proposition::Or(
                        PropBi {
                            a: Box::new(hold_enf),
                            b: Box::new(until_enf.clone()),
                        },
                    )))),
                });
                let eg_part =
                    Proposition::EG(PropUni::new(Proposition::Negation(PropUni::new(until_enf))));
                Proposition::Negation(PropUni::new(Proposition::Or(PropBi {
                    a: Box::new(eu_part),
                    b: Box::new(eg_part),
                })))
            }
            Proposition::ER(inner) => {
                // E[p R q] = !A[!p U !q]
                let neg_hold_enf = Proposition::Negation(PropUni::new(inner.hold.enf()));
                let neg_release_enf = Proposition::Negation(PropUni::new(inner.release.enf()));
                Proposition::Negation(PropUni::new(Proposition::AU(PropU {
                    hold: Box::new(neg_hold_enf),
                    until: Box::new(neg_release_enf),
                })))
            }
            Proposition::AR(inner) => {
                // A[p R q] = !E[!p U !q]
                let neg_hold_enf = Proposition::Negation(PropUni::new(inner.hold.enf()));
                let neg_release_enf = Proposition::Negation(PropUni::new(inner.release.enf()));
                Proposition::Negation(PropUni::new(Proposition::EU(PropU {
                    hold: Box::new(neg_hold_enf),
                    until: Box::new(neg_release_enf),
                })))
            }
        }
    }
}
