//! Testing for miscellaneous examples.
//!
//! This is mostly to prevent regressions of known bugs.

use crate::examples::harness::{test_example, TestConfig};

#[test]
fn misc_freerun() {
    // This uncovers a bug in version 0.6.0 where double-checking fails.
    //
    // This did not impact soundness as machine-check panicked due to the
    // failed double-checking instead of returning a result.
    test_example(
        "misc_freerun",
        TestConfig::new_property("AG![lfp![X,gfp![Y, AX![X] || (value == 0 && AX![Y])]]]")
            .with_arg("--strategy")
            .with_arg("decay"),
        // TODO: this should contain the correct result after this is unbroken
        r#""#,
    );
}
