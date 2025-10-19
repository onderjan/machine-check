use std::fmt::Debug;

use mck::abstr::AbstractValue;

use machine_check_common::KnownParamValuation;
use machine_check_common::ParamValuation;
use machine_check_common::StateId;

use crate::MetaWrap;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum CheckChoice {
    Next(StateId, Box<CheckChoice>),
    FixedPoint(Box<CheckChoice>),
    FixedVariable(u64),
    Func(Vec<(MetaWrap<AbstractValue>, Option<CheckChoice>)>),
}

#[derive(Clone, Hash, PartialEq, Eq)]
pub enum CheckValue {
    True,
    False,
    Dependent,
    Unknown(Box<CheckChoice>),
}

#[derive(Clone, Hash)]
pub struct TimedCheckValue {
    pub time: u64,
    pub value: CheckValue,
}

impl TimedCheckValue {
    pub fn new(time: u64, value: CheckValue) -> Self {
        TimedCheckValue { time, value }
    }
}

impl CheckValue {
    pub fn is_unknown(&self) -> bool {
        matches!(self, CheckValue::Unknown(_))
    }

    pub fn from_bool(value: bool) -> Self {
        match value {
            true => CheckValue::True,
            false => CheckValue::False,
        }
    }

    pub fn from_known(valuation: KnownParamValuation) -> Self {
        match valuation {
            KnownParamValuation::False => CheckValue::False,
            KnownParamValuation::True => CheckValue::True,
            KnownParamValuation::Dependent => CheckValue::Dependent,
        }
    }

    pub fn valuation(&self) -> ParamValuation {
        match self {
            CheckValue::False => ParamValuation::False,
            CheckValue::True => ParamValuation::True,
            CheckValue::Dependent => ParamValuation::Dependent,
            CheckValue::Unknown(_) => ParamValuation::Unknown,
        }
    }
}

impl Debug for CheckValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CheckValue::False => write!(f, "False"),
            CheckValue::True => write!(f, "True"),
            CheckValue::Dependent => write!(f, "Dependent"),
            CheckValue::Unknown(choice) => {
                write!(f, "Unknown ({:?})", choice)
            }
        }
    }
}

impl Debug for TimedCheckValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {:?})", self.time, self.value)
    }
}
