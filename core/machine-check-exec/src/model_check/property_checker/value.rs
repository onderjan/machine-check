use std::fmt::Debug;

use machine_check_common::{KnownParamValuation, ParamValuation, StateId};

use crate::model_check::property_checker::BiChoice;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Reason {
    Atomic,
    Negation,
    BiLogic(BiChoice),
    Next(StateId),
    FixedPoint,
    FixedVariable,
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum CheckValue {
    False,
    True,
    Dependent,
    Unknown(Vec<Reason>),
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct TimedCheckValue {
    pub time: u64,
    pub value: CheckValue,
}

impl CheckValue {
    pub fn is_unknown(&self) -> bool {
        matches!(self, CheckValue::Unknown(_))
    }

    pub fn valuation(&self) -> ParamValuation {
        match self {
            CheckValue::False => ParamValuation::False,
            CheckValue::True => ParamValuation::True,
            CheckValue::Dependent => ParamValuation::Dependent,
            CheckValue::Unknown(_) => ParamValuation::Unknown,
        }
    }

    pub fn from_bool(value: bool) -> Self {
        if value {
            CheckValue::True
        } else {
            CheckValue::False
        }
    }

    pub fn from_known(value: KnownParamValuation) -> Self {
        match value {
            KnownParamValuation::False => CheckValue::False,
            KnownParamValuation::True => CheckValue::True,
            KnownParamValuation::Dependent => CheckValue::Dependent,
        }
    }
}

impl TimedCheckValue {
    pub fn new(time: u64, value: CheckValue) -> Self {
        TimedCheckValue { time, value }
    }
}

impl Debug for CheckValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::False => write!(f, "False"),
            Self::True => write!(f, "True"),
            Self::Dependent => write!(f, "Dependent"),
            Self::Unknown(reasons) => {
                write!(f, "Unknown [")?;

                for reason in reasons {
                    match reason {
                        Reason::Atomic => write!(f, "A"),
                        Reason::Negation => write!(f, "N"),
                        Reason::BiLogic(BiChoice::Left) => write!(f, "BL"),
                        Reason::BiLogic(BiChoice::Right) => write!(f, "BR"),
                        Reason::Next(state_id) => write!(f, "N{}", state_id),
                        Reason::FixedPoint => write!(f, "F"),
                        Reason::FixedVariable => write!(f, "V"),
                    }?;
                    write!(f, ", ")?;
                }

                write!(f, "]")
            }
        }
    }
}

impl Debug for TimedCheckValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {:?})", self.time, self.value)
    }
}
