use super::{PropBi, PropU, Proposition};

impl Proposition {
    pub fn apply_enf(&mut self) {
        match self {
            Proposition::Const(_) => return,
            Proposition::Literal(_) => return,
            Proposition::Negation(inner) => {
                inner.apply_enf();
                return;
            }
            Proposition::Or(PropBi { a, b }) => {
                a.apply_enf();
                b.apply_enf();
                return;
            }
            Proposition::And(PropBi { a, b }) => {
                // p and q = !(!p or !q)
                *self = Proposition::Negation(Box::new(Proposition::Or(PropBi {
                    a: Box::new(Proposition::Negation(Box::clone(a))),
                    b: Box::new(Proposition::Negation(Box::clone(b))),
                })));
            }
            Proposition::EX(inner) => {
                inner.apply_enf();
                return;
            }
            Proposition::AX(inner) => {
                // AX[p] = !EX[!p]
                *self = Proposition::Negation(Box::new(Proposition::EX(Box::new(
                    Proposition::Negation(Box::clone(inner)),
                ))));
            }
            Proposition::AF(inner) => {
                // AF[p] = A[true U p] = !EG[!p]
                *self = Proposition::Negation(Box::new(Proposition::EG(Box::new(
                    Proposition::Negation(Box::clone(inner)),
                ))));
            }
            Proposition::EF(inner) => {
                // EF[p] = E[true U p]
                *self = Proposition::EU(PropU {
                    hold: Box::new(Proposition::Const(true)),
                    until: Box::clone(inner),
                });
            }
            Proposition::EG(_) => return,
            Proposition::AG(inner) => {
                // AG[p] = !EF[!p] = !E[true U !p]
                *self = Proposition::Negation(Box::new(Proposition::EU(PropU {
                    hold: Box::new(Proposition::Const(true)),
                    until: Box::new(Proposition::Negation(Box::clone(inner))),
                })));
            }
            Proposition::EU(inner) => {
                inner.hold.apply_enf();
                inner.until.apply_enf();
                return;
            }
            Proposition::AU(inner) => {
                // A[p U q] = !(E[!q U !(p or q)] or EG[!q])
                let eu_part = Proposition::EU(PropU {
                    hold: Box::new(Proposition::Negation(Box::clone(&inner.until))),
                    until: Box::new(Proposition::Negation(Box::new(Proposition::Or(PropBi {
                        a: Box::clone(&inner.hold),
                        b: Box::clone(&inner.until),
                    })))),
                });
                let eg_part =
                    Proposition::EG(Box::new(Proposition::Negation(Box::clone(&inner.until))));
                *self = Proposition::Negation(Box::new(Proposition::Or(PropBi {
                    a: Box::new(eu_part),
                    b: Box::new(eg_part),
                })));
            }
            Proposition::ER(inner) => {
                // E[p R q] = !A[!p U !q]
                let neg_hold = Proposition::Negation(inner.hold.clone());
                let neg_release = Proposition::Negation(inner.release.clone());
                *self = Proposition::Negation(Box::new(Proposition::AU(PropU {
                    hold: Box::new(neg_hold),
                    until: Box::new(neg_release),
                })));
            }
            Proposition::AR(inner) => {
                // A[p R q] = !E[!p U !q]
                let neg_hold = Proposition::Negation(inner.hold.clone());
                let neg_release = Proposition::Negation(inner.release.clone());
                *self = Proposition::Negation(Box::new(Proposition::EU(PropU {
                    hold: Box::new(neg_hold),
                    until: Box::new(neg_release),
                })));
            }
        }
        // minimize the new expression
        self.apply_enf();
    }
}
