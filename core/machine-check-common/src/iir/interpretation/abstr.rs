#[derive(Clone, Debug)]
pub enum IAbstractValue {
    Bitvector(mck::abstr::RBitvector),
    Bool(mck::abstr::Boolean),
    PanicResult(mck::abstr::PanicResult<mck::abstr::RBitvector>),
    Absent,
}

impl IAbstractValue {
    pub fn expect_bitvector(&self) -> mck::abstr::RBitvector {
        let IAbstractValue::Bitvector(bitvec) = self else {
            panic!("Value is not a bitvector");
        };
        *bitvec
    }

    pub fn expect_bool(&self) -> mck::abstr::Boolean {
        let IAbstractValue::Bool(boolean) = self else {
            panic!("Value is not a boolean");
        };
        *boolean
    }

    pub fn join(&self, right: &Self) -> Self {
        match (self, right) {
            (_, IAbstractValue::Absent) => self.clone(),
            (IAbstractValue::Absent, _) => right.clone(),
            (IAbstractValue::Bitvector(left), IAbstractValue::Bitvector(right)) => {
                IAbstractValue::Bitvector(left.join(*right))
            }
            (IAbstractValue::Bool(left), IAbstractValue::Bool(right)) => {
                IAbstractValue::Bool(left.join(*right))
            }
            (IAbstractValue::PanicResult(_), _) | (_, IAbstractValue::PanicResult(_)) => {
                panic!("Panic result should never be joined")
            }
            _ => panic!(
                "Unjoinable combination of values {:?} and {:?}",
                self, right
            ),
        }
    }
}
