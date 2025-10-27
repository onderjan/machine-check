//! An example of how hierarchy of systems can be checked.
//!
//! Consider a that the specification is a system that performs unsigned division,
//! with the result of division of zero either zero or maximal value. In essence,
//! the specification is a parametric system, parametrised by the choice of zero
//! or max value. We want to check whether an implementation is correct.
//!
//! The result of the specification division is in 'specified_value', the result
//! of implementation division is in 'impl_value'. We can check the following:
//! 'AG![specified_value == impl_value]'
//!
//! The implementation result for division by zero is normally 0, which is allowed.
//! The result of checking will be "depends on parameters", as the implementation
//! instantiates the specification system with the division-by-zero parameter set to 0.
//! The implementation system is hierarchically 'under' the specification system.
//!
//! On the other hand, when the command-line argument '--system-wrong-div-by-zero'
//! is used, the result will be that the property does not hold, as there is no
//! way to instantiate the specification system by the implementation.
//! The implementation and specification systems are not hierarchically related.
//!

#[machine_check::machine_description]
mod machine_module {
    use ::machine_check::{Ext, Signed, Unsigned};
    use ::std::{
        clone::Clone,
        cmp::{Eq, PartialEq},
        convert::Into,
        fmt::Debug,
        hash::Hash,
    };

    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct Input {
        dividend: Unsigned<4>,
        divisor: Unsigned<4>,
    }

    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct Param {
        unspecified: Signed<1>,
    }

    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct State {
        specified_value: Unsigned<4>,
        impl_value: Unsigned<4>,
    }

    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct System {
        pub division_by_zero_result: Unsigned<4>,
    }

    impl ::machine_check::Machine for System {
        type Input = Input;
        type State = State;
        type Param = Param;

        fn init(&self, input: &Input, param: &Param) -> State {
            let specified_value;
            let impl_value;
            if input.divisor != Unsigned::<4>::new(0) {
                specified_value = input.dividend / input.divisor;
                impl_value = input.dividend / input.divisor;
            } else {
                specified_value = Into::<Unsigned<4>>::into(Ext::<4>::ext(param.unspecified));
                impl_value = self.division_by_zero_result;
            }

            State {
                specified_value,
                impl_value,
            }
        }

        fn next(&self, state: &State, _input: &Input, _param: &Param) -> State {
            State {
                specified_value: state.specified_value,
                impl_value: state.impl_value,
            }
        }
    }
}

use clap::Args;
use machine_check::Unsigned;

#[derive(Args)]
struct SystemArgs {
    #[arg(long = "system-wrong-div-by-zero")]
    wrong_div_by_zero: bool,
}

fn main() {
    let (run_args, system_args) = machine_check::parse_args::<SystemArgs>(std::env::args());
    let division_by_zero_result = Unsigned::<4>::new(system_args.wrong_div_by_zero as u64);
    machine_check::execute(
        machine_module::System {
            division_by_zero_result,
        },
        run_args,
    );
}
