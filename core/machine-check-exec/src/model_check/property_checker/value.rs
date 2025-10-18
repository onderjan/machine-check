use std::fmt::Debug;

use mck::abstr::AbstractValue;

use machine_check_common::KnownParamValuation;
use machine_check_common::ParamValuation;
use machine_check_common::StateId;

use crate::MetaWrap;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum CheckChoice {
    Atomic(MetaWrap<AbstractValue>),
    Next(Option<(StateId, Box<CheckChoice>)>),
    FixedVariable(u64),
    Func(Vec<(MetaWrap<AbstractValue>, CheckChoice)>),
}

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct CheckValue {
    pub valuation: ParamValuation,
    pub choice: CheckChoice,
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
        matches!(self.valuation, ParamValuation::Unknown)
    }

    pub fn fixed_from_bool(value: bool, time: u64) -> Self {
        let valuation = if value {
            ParamValuation::True
        } else {
            ParamValuation::False
        };

        CheckValue {
            valuation,
            choice: CheckChoice::FixedVariable(time),
        }
    }

    pub fn next_from_bool(value: bool) -> Self {
        let valuation = if value {
            ParamValuation::True
        } else {
            ParamValuation::False
        };

        CheckValue {
            valuation,
            choice: CheckChoice::Next(None),
        }
    }

    pub fn next_from_known(valuation: KnownParamValuation) -> Self {
        let valuation = match valuation {
            KnownParamValuation::False => ParamValuation::False,
            KnownParamValuation::True => ParamValuation::True,
            KnownParamValuation::Dependent => ParamValuation::Dependent,
        };

        CheckValue {
            valuation,
            choice: CheckChoice::Next(None),
        }
    }

    /*pub fn from_bool(value: bool) -> Self {
        if value {
            CheckValue::True
        } else {
            CheckValue::False
        }
    }

    */
}

impl Debug for CheckValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.valuation {
            ParamValuation::False => write!(f, "False"),
            ParamValuation::True => write!(f, "True"),
            ParamValuation::Dependent => write!(f, "Dependent"),
            ParamValuation::Unknown => {
                write!(f, "Unknown ({:?})", self.choice)
            }
        }
    }
}

impl Debug for TimedCheckValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {:?})", self.time, self.value)
    }
}
