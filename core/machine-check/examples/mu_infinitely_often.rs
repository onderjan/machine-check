//! A system demonstrating "infinitely often" expressible in mu-calculus and LTL but not CTL.
//!
//! This is the classic standard example of a system where the CTL property 'AF![AG![p == 1]]'
//! ("eventually always p") does not hold but the LTL property FG[p == 1] ("infinitely often p") does.
//!
//! In machine-check, we can express "infinitely often p" in mu-calculus as
//! 'gfp![Y, lfp![X, (p == 1 && EX![Y]) || EX![X]]]'
//! and we can verify that it holds. On the other hand, we can verify 'AF![AG![p == 1]]' does not.
//!
//! See the Handbook of Model Checking (2018) pp. 63-67 for a formal treatment why the aforementioned
//! property and this example contribute to the result that CTL and LTL have different expressiveness.
//! Mu-calculus can express both, although the translation can be awkward, especially for LTL properties.
//! This example system corresponds to the system N in Figure 6, pp. 65.
//!
//! See also the easier-to-understand discussion of this example in
//! https://cs.stackexchange.com/questions/98916/distinguishing-between-ctl-formulas-afg-p-and-afag-p-using-transition-sys
//!
//! The general gist is that "eventually always p" does not hold since we can stay in state index 0
//! as long as we like, being threatened by the possibility of going through state index 1 where p does not hold.
//! On the other hand, "infinitely often p" holds since going through state index 1 is only a temporary setback
//! and we will necessarily go to and stay in state index 2 where p holds again.
//!

#[machine_check::machine_description]
mod machine_module {
    use ::machine_check::{Ext, Unsigned};
    use ::std::{
        clone::Clone,
        cmp::{Eq, PartialEq},
        fmt::Debug,
        hash::Hash,
    };

    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct Input {
        choice: Unsigned<1>,
    }

    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct Param {}

    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct State {
        index: Unsigned<2>,
        p: Unsigned<1>,
    }

    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct System {}

    impl ::machine_check::Machine for System {
        type Input = Input;
        type State = State;
        type Param = Param;

        fn init(&self, _input: &Input, _param: &Param) -> State {
            // the initial state has state index 0 and p == 1
            State {
                index: Unsigned::<2>::new(0),
                p: Unsigned::<1>::new(0),
            }
        }

        fn next(&self, state: &State, input: &Input, _param: &Param) -> State {
            let mut next_index = state.index;

            if state.index == Unsigned::<2>::new(0) {
                // we can transition either to state index 0 or 1
                next_index = Ext::<2>::ext(input.choice);
            }
            if state.index == Unsigned::<2>::new(1) {
                // we must transition to state index 2
                next_index = Unsigned::<2>::new(2);
            }
            if state.index == Unsigned::<2>::new(2) {
                // do nothing and stay in state index 2
            }
            if state.index == Unsigned::<2>::new(3) {
                ::std::panic!("This state should not be reachable");
            }

            // p holds in state index 0 and 2
            let mut p = Unsigned::<1>::new(0);
            if (next_index == Unsigned::<2>::new(0)) | (next_index == Unsigned::<2>::new(2)) {
                p = Unsigned::<1>::new(1);
            }

            State {
                index: next_index,
                p,
            }
        }
    }
}

/// Main entry point of the executable.
fn main() {
    // Construct the system. This one has no unchanging data.
    let system = machine_module::System {};
    // Run machine-check with the constructed system.
    machine_check::run(system);
}
