//! This module tests the standard examples: their inherent properties and ones
//! mentioned in the example comment, if any.
//!
//! If an example is a normal example but also in book, properties may be tested twice.

use crate::examples::harness::{test_example, TestConfig};

#[test]
fn conditional_panic() {
    // inherent property should NOT hold in conditional panic
    test_example(
        "conditional_panic",
        TestConfig::new_inherent(),
        r#"{"result":{"Ok":"False"},"stats":{"num_refinements":8,"num_generated_states":12,"num_final_states":3,"num_generated_transitions":514,"num_final_transitions":5,"inherent_panic_message":"Example panic 2"}}"#,
    );
}

#[test]
fn counter() {
    test_example(
        "counter",
        TestConfig::new_inherent(),
        r#"{"result":{"Ok":"True"},"stats":{"num_refinements":0,"num_generated_states":10,"num_final_states":9,"num_generated_transitions":10,"num_final_transitions":10,"inherent_panic_message":null}}"#,
    );
    // no properties specified within the example itself
}

#[test]
fn division() {
    // inherent property should NOT hold in division due to the possibility of division by zero
    test_example(
        "division",
        TestConfig::new_inherent(),
        r#"{"result":{"Ok":"False"},"stats":{"num_refinements":8,"num_generated_states":20,"num_final_states":11,"num_generated_transitions":522,"num_final_transitions":21,"inherent_panic_message":"attempt to divide by zero"}}"#,
    );
}

#[test]
fn mu_even() {
    // inherent property should hold
    test_example(
        "mu_even",
        TestConfig::new_inherent(),
        r#"{"result":{"Ok":"True"},"stats":{"num_refinements":0,"num_generated_states":3,"num_final_states":2,"num_generated_transitions":3,"num_final_transitions":3,"inherent_panic_message":null}}"#,
    );

    // example property given
    test_example(
        "mu_even",
        TestConfig::new_property("gfp![X, value == 0 && AX![AX![X]]]"),
        r#"{"result":{"Ok":"True"},"stats":{"num_refinements":0,"num_generated_states":3,"num_final_states":2,"num_generated_transitions":3,"num_final_transitions":3,"inherent_panic_message":null}}"#,
    );
}

#[test]
fn mu_infinitely_often() {
    // inherent property should hold
    test_example(
        "mu_infinitely_often",
        TestConfig::new_inherent(),
        r#"{"result":{"Ok":"True"},"stats":{"num_refinements":2,"num_generated_states":10,"num_final_states":4,"num_generated_transitions":12,"num_final_transitions":7,"inherent_panic_message":null}}"#,
    );

    // example properties given, these are also in the book

    // whether p == 1 holds eventually forever (should not hold)
    test_example(
        "mu_infinitely_often",
        TestConfig::new_property("AF![AG![p == 1]]"),
        r#"{"result":{"Ok":"False"},"stats":{"num_refinements":2,"num_generated_states":10,"num_final_states":4,"num_generated_transitions":12,"num_final_transitions":7,"inherent_panic_message":null}}"#,
    );

    // whether p == 1 holds infinitely often
    // TODO: THIS IS THE WRONG PROPERTY FOR AFG[p == 1], which is written there
    test_example(
        "mu_infinitely_often",
        TestConfig::new_property("gfp![Y, lfp![X, (p == 1 && EX![Y]) || EX![X]]]"),
        r#"{"result":{"Ok":"True"},"stats":{"num_refinements":2,"num_generated_states":10,"num_final_states":4,"num_generated_transitions":12,"num_final_transitions":7,"inherent_panic_message":null}}"#,
    );
}

#[test]
fn parametric() {
    // inherent property should hold
    test_example(
        "parametric",
        TestConfig::new_inherent(),
        r#"{"result":{"Ok":"True"},"stats":{"num_refinements":0,"num_generated_states":3,"num_final_states":2,"num_generated_transitions":3,"num_final_transitions":3,"inherent_panic_message":null}}"#,
    );

    // example properties given

    test_example(
        "parametric",
        TestConfig::new_property("AG![value == 0]"),
        r#"{"result":{"Ok":"Dependent"},"stats":{"num_refinements":5,"num_generated_states":69,"num_final_states":33,"num_generated_transitions":96,"num_final_transitions":50,"inherent_panic_message":null}}"#,
    );
    test_example(
        "parametric",
        TestConfig::new_property("AG![value == 1]"),
        r#"{"result":{"Ok":"False"},"stats":{"num_refinements":0,"num_generated_states":3,"num_final_states":2,"num_generated_transitions":3,"num_final_transitions":3,"inherent_panic_message":null}}"#,
    );
    test_example(
        "parametric",
        TestConfig::new_property("EF![as_unsigned(value) > 15]"),
        r#"{"result":{"Ok":"False"},"stats":{"num_refinements":0,"num_generated_states":3,"num_final_states":2,"num_generated_transitions":3,"num_final_transitions":3,"inherent_panic_message":null}}"#,
    );
    test_example(
        "parametric",
        TestConfig::new_property("EF![as_unsigned(value) > 8]"),
        r#"{"result":{"Ok":"Dependent"},"stats":{"num_refinements":30,"num_generated_states":115,"num_final_states":44,"num_generated_transitions":172,"num_final_transitions":88,"inherent_panic_message":null}}"#,
    );
    test_example(
        "parametric",
        TestConfig::new_property("EF![as_unsigned(value) >= 0]"),
        r#"{"result":{"Ok":"True"},"stats":{"num_refinements":0,"num_generated_states":3,"num_final_states":2,"num_generated_transitions":3,"num_final_transitions":3,"inherent_panic_message":null}}"#,
    );
    test_example(
        "parametric",
        TestConfig::new_property("AF![as_unsigned(value) > 0]"),
        r#"{"result":{"Ok":"False"},"stats":{"num_refinements":64,"num_generated_states":368,"num_final_states":151,"num_generated_transitions":784,"num_final_transitions":287,"inherent_panic_message":null}}"#,
    );
    test_example(
        "parametric",
        TestConfig::new_property("EX![AF![as_unsigned(value) > 8]]"),
        r#"{"result":{"Ok":"Dependent"},"stats":{"num_refinements":30,"num_generated_states":115,"num_final_states":44,"num_generated_transitions":172,"num_final_transitions":88,"inherent_panic_message":null}}"#,
    );
    test_example(
        "parametric",
        TestConfig::new_property("AF![as_unsigned(value) > 0]"),
        r#"{"result":{"Ok":"False"},"stats":{"num_refinements":64,"num_generated_states":368,"num_final_states":151,"num_generated_transitions":784,"num_final_transitions":287,"inherent_panic_message":null}}"#,
    );
    test_example(
        "parametric",
        TestConfig::new_property("EU![value == 0, as_unsigned(value) >= 5]"),
        r#"{"result":{"Ok":"Dependent"},"stats":{"num_refinements":21,"num_generated_states":109,"num_final_states":43,"num_generated_transitions":240,"num_final_transitions":70,"inherent_panic_message":null}}"#,
    );
    test_example(
        "parametric",
        TestConfig::new_property("AU![value == 0, as_unsigned(value) >= 5]"),
        r#"{"result":{"Ok":"False"},"stats":{"num_refinements":52,"num_generated_states":340,"num_final_states":149,"num_generated_transitions":656,"num_final_transitions":279,"inherent_panic_message":null}}"#,
    );
}

#[test]
fn recovery() {
    // first, test with disabled reset input

    // inherent property should hold
    test_example(
        "recovery",
        TestConfig::new_inherent().with_release(),
        r#"Reset input is disabled
{"result":{"Ok":"True"},"stats":{"num_refinements":0,"num_generated_states":18,"num_final_states":17,"num_generated_transitions":18,"num_final_transitions":18,"inherent_panic_message":null}}"#,
    );

    // recovery should not hold
    test_example(
        "recovery",
        TestConfig::new_property("AG![EF![max_value == 0]]").with_release(),
        r#"Reset input is disabled
{"result":{"Ok":"False"},"stats":{"num_refinements":5,"num_generated_states":100,"num_final_states":64,"num_generated_transitions":157,"num_final_transitions":96,"inherent_panic_message":null}}"#,
    );

    // then, test with enabled reset input

    // inherent property should hold
    test_example(
        "recovery",
        TestConfig::new_inherent()
            .with_arg("--system-enable-reset")
            .with_release(),
        r#"Reset input is enabled
{"result":{"Ok":"True"},"stats":{"num_refinements":0,"num_generated_states":18,"num_final_states":17,"num_generated_transitions":18,"num_final_transitions":18,"inherent_panic_message":null}}"#,
    );

    // recovery should hold
    test_example(
        "recovery",
        TestConfig::new_property("AG![EF![max_value == 0]]")
            .with_arg("--system-enable-reset")
            .with_release(),
        r#"Reset input is enabled
{"result":{"Ok":"True"},"stats":{"num_refinements":3078,"num_generated_states":4477,"num_final_states":513,"num_generated_transitions":66037,"num_final_transitions":8977,"inherent_panic_message":null}}"#,
    );
}

#[test]
fn simple_risc() {
    // inherent property should hold
    test_example(
        "simple_risc",
        TestConfig::new_inherent(),
        r#"{"result":{"Ok":"True"},"stats":{"num_refinements":0,"num_generated_states":12,"num_final_states":11,"num_generated_transitions":12,"num_final_transitions":12,"inherent_panic_message":null}}"#,
    );

    // example properties given that should hold
    test_example(
        "simple_risc",
        TestConfig::new_property("AF![reg[1] == 1 && as_unsigned(pc) < 3]"),
        r#"{"result":{"Ok":"True"},"stats":{"num_refinements":0,"num_generated_states":12,"num_final_states":11,"num_generated_transitions":12,"num_final_transitions":12,"inherent_panic_message":null}}"#,
    );

    test_example(
        "simple_risc",
        TestConfig::new_property("AF![reg[1] == 1 && as_unsigned(pc) < 3]").with_release(),
        r#"{"result":{"Ok":"True"},"stats":{"num_refinements":0,"num_generated_states":12,"num_final_states":11,"num_generated_transitions":12,"num_final_transitions":12,"inherent_panic_message":null}}"#,
    );

    test_example(
        "simple_risc",
        TestConfig::new_property("AG![as_unsigned(pc) <= 9]")
            .with_arg("--strategy")
            .with_arg("decay")
            .with_release(),
        r#"{"result":{"Ok":"True"},"stats":{"num_refinements":427,"num_generated_states":736,"num_final_states":11,"num_generated_transitions":751,"num_final_transitions":13,"inherent_panic_message":null}}"#,
    );
}

#[test]
fn timed_panic() {
    // inherent property should NOT hold in timed panic
    test_example(
        "timed_panic",
        TestConfig::new_inherent(),
        r#"{"result":{"Ok":"False"},"stats":{"num_refinements":2,"num_generated_states":61,"num_final_states":42,"num_generated_transitions":65,"num_final_transitions":46,"inherent_panic_message":"Value must not be 3"}}"#,
    );
}
