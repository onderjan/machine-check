//! Testing for miscellaneous examples.
//!
//! This is mostly to prevent regressions of known bugs.

use crate::examples::harness::{test_example, TestConfig};

#[test]
fn misc_freerun() {
    // This uncovers a bug in version 0.6.0 where double-checking fails.
    //
    // The problem was that the incremental model checking sometimes
    // led to spurious computations retained, not discarding them. It seems
    // that the computations were not wrong, but led to more computation
    // and history entries than when computing fresh. In any case, this
    // should not happen.
    //
    // This did not impact soundness as machine-check panicked due to the
    // failed double-checking instead of returning a result. This should be
    // fixed from 0.6.1 onwards, as witnessed by this test.
    test_example(
        "misc_freerun",
        TestConfig::new_property("AG![lfp![X,gfp![Y, AX![X] || (value == 0 && AX![Y])]]]")
            .with_arg("--strategy")
            .with_arg("decay"),
        r#"{"result":{"Ok":"False"},"stats":{"num_refinements":12,"num_generated_states":22,"num_final_states":4,"num_generated_transitions":22,"num_final_transitions":5,"inherent_panic_message":null}}"#,
    );
}
