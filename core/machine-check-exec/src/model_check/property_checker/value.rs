use std::fmt::Debug;

use machine_check_common::{ParamValuation, StateId};

use crate::MetaWrap;
use machine_check_common::KnownParamValuation;
use mck::abstr::AbstractValue;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum CheckChoice {
    Next(Option<StateId>),
    FixedVariable(u64),
    Func(Vec<MetaWrap<AbstractValue>>),
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

impl CheckValue {
    pub fn is_unknown(&self) -> bool {
        matches!(self.valuation, ParamValuation::Unknown)
    }

    pub fn fixed_from_bool(value: bool) -> Self {
        let valuation = if value {
            ParamValuation::True
        } else {
            ParamValuation::False
        };

        CheckValue {
            valuation,
            choice: CheckChoice::FixedVariable(0),
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

impl TimedCheckValue {
    pub fn new(time: u64, value: CheckValue) -> Self {
        TimedCheckValue { time, value }
    }
}

impl Debug for CheckValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.valuation {
            ParamValuation::False => write!(f, "False"),
            ParamValuation::True => write!(f, "True"),
            ParamValuation::Dependent => write!(f, "Dependent"),
            ParamValuation::Unknown => {
                write!(f, "Unknown [")?;

                match &self.choice {
                    CheckChoice::Next(state_id) => {
                        write!(
                            f,
                            "N{}",
                            state_id.expect("Next state should be present when unknown")
                        )
                    }
                    CheckChoice::FixedVariable(time) => write!(f, "V({})", time),
                    CheckChoice::Func(abstract_values) => write!(f, "F({:?})", abstract_values),
                }?;

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
