//! This module tests the examples from the chapters of the book.
//!
//! If an example is a normal example but also in book, properties may be tested twice.

use crate::examples::harness::{test_example, test_example_expect_panic, TestConfig};

#[test]
fn book_ch1_quickstart() {
    // chapter 1: recovery property
    test_example(
        "book_quickstart",
        TestConfig::new_property("AG![EF![value == 0]]"),
        r#"{"result":{"Ok":"True"},"stats":{"num_refinements":16,"num_generated_states":48,"num_final_states":16,"num_generated_transitions":64,"num_final_transitions":33,"inherent_panic_message":null}}"#,
    );
}

#[test]
fn book_ch2_properties() {
    // chapter 2a: is the value zero at the start of the computation?
    test_example(
        "book_quickstart",
        TestConfig::new_property("value == 0"),
        r#"{"result":{"Ok":"True"},"stats":{"num_refinements":0,"num_generated_states":6,"num_final_states":5,"num_generated_transitions":6,"num_final_transitions":6,"inherent_panic_message":null}}"#,
    );

    // chapter 2b: with AX
    test_example(
        "book_quickstart",
        TestConfig::new_property("AX![value == 0]"),
        r#"{"result":{"Ok":"False"},"stats":{"num_refinements":1,"num_generated_states":8,"num_final_states":5,"num_generated_transitions":9,"num_final_transitions":7,"inherent_panic_message":null}}"#,
    );

    // chapter 2c: with EX
    test_example(
        "book_quickstart",
        TestConfig::new_property("EX![value == 0]"),
        r#"{"result":{"Ok":"True"},"stats":{"num_refinements":1,"num_generated_states":8,"num_final_states":5,"num_generated_transitions":9,"num_final_transitions":7,"inherent_panic_message":null}}"#,
    );

    // chapter 2d: globally, on all paths, value (unsigned) is at most 15
    test_example(
        "book_quickstart",
        TestConfig::new_property("AG![as_unsigned(value) <= 15]"),
        r#"{"result":{"Ok":"True"},"stats":{"num_refinements":0,"num_generated_states":6,"num_final_states":5,"num_generated_transitions":6,"num_final_transitions":6,"inherent_panic_message":null}}"#,
    );

    // chapter 2e: there exists a path where value is globally zero
    test_example(
        "book_quickstart",
        TestConfig::new_property("EG![value == 0]"),
        r#"{"result":{"Ok":"True"},"stats":{"num_refinements":1,"num_generated_states":8,"num_final_states":5,"num_generated_transitions":9,"num_final_transitions":7,"inherent_panic_message":null}}"#,
    );

    // chapter 2f: there exists a path where value is globally 3
    test_example(
        "book_quickstart",
        TestConfig::new_property("EG![value == 3]"),
        r#"{"result":{"Ok":"False"},"stats":{"num_refinements":0,"num_generated_states":6,"num_final_states":5,"num_generated_transitions":6,"num_final_transitions":6,"inherent_panic_message":null}}"#,
    );

    // chapter 2g: there exists a path where value is eventually 3
    test_example(
        "book_quickstart",
        TestConfig::new_property("EF![as_unsigned(value) == 3]"),
        r#"{"result":{"Ok":"True"},"stats":{"num_refinements":3,"num_generated_states":13,"num_final_states":6,"num_generated_transitions":16,"num_final_transitions":10,"inherent_panic_message":null}}"#,
    );

    // chapter 2h: there exists a path that eventually reaches a state
    // where there exists a path s.t. the value is globally 3
    test_example(
        "book_quickstart",
        TestConfig::new_property("EF![EG![value == 3]]"),
        r#"{"result":{"Ok":"True"},"stats":{"num_refinements":4,"num_generated_states":17,"num_final_states":8,"num_generated_transitions":21,"num_final_transitions":13,"inherent_panic_message":null}}"#,
    );

    // chapter 2i: until
    test_example(
        "book_quickstart",
        TestConfig::new_property("EU![as_unsigned(value) < 3, value == 3]"),
        r#"{"result":{"Ok":"True"},"stats":{"num_refinements":3,"num_generated_states":13,"num_final_states":6,"num_generated_transitions":16,"num_final_transitions":10,"inherent_panic_message":null}}"#,
    );

    // chapter 2j: release
    test_example(
        "book_quickstart",
        TestConfig::new_property("AR![value == 3, as_unsigned(value) <= 3]"),
        r#"{"result":{"Ok":"True"},"stats":{"num_refinements":3,"num_generated_states":13,"num_final_states":6,"num_generated_transitions":16,"num_final_transitions":10,"inherent_panic_message":null}}"#,
    );

    // chapter 2k: implication using logical operators
    test_example(
        "book_quickstart",
        TestConfig::new_property("AG![!(value == 5) || AX![value == 5 || value == 6]]"),
        r#"{"result":{"Ok":"True"},"stats":{"num_refinements":16,"num_generated_states":48,"num_final_states":16,"num_generated_transitions":64,"num_final_transitions":33,"inherent_panic_message":null}}"#,
    );

    // chapter 2l: recovery property
    test_example(
        "book_quickstart",
        TestConfig::new_property("AG![EF![value == 0]]"),
        r#"{"result":{"Ok":"True"},"stats":{"num_refinements":16,"num_generated_states":48,"num_final_states":16,"num_generated_transitions":64,"num_final_transitions":33,"inherent_panic_message":null}}"#,
    );
}

#[test]
fn book_ch3() {
    // chapter 3.1a: checking existence of path where eventually value is 10, field max_value set to 5
    test_example(
        "book_instances_input",
        TestConfig::new_property("EF![value == 10]").with_input("5"),
        r#"Write the maximum system value: {"result":{"Ok":"False"},"stats":{"num_refinements":6,"num_generated_states":19,"num_final_states":6,"num_generated_transitions":25,"num_final_transitions":13,"inherent_panic_message":null}}"#,
    );

    // chapter 3.1b: checking existence of path where eventually value is 10, field max_value set to 12
    test_example(
        "book_instances_input",
        TestConfig::new_property("EF![value == 10]").with_input("12"),
        r#"Write the maximum system value: {"result":{"Ok":"True"},"stats":{"num_refinements":10,"num_generated_states":33,"num_final_states":13,"num_generated_transitions":43,"num_final_transitions":24,"inherent_panic_message":null}}"#,
    );

    // chapter 3.1c: checking existence of path where eventually value is 10, field max_value set to 20
    // this should panic
    test_example_expect_panic(
        "book_instances_input",
        TestConfig::new_property("EF![value == 10]").with_input("20"),
    );

    // chapter 3.1d: checking inherent where field max_value is set to 10, inherent should not hold
    test_example(
        "book_instances_todo",
        TestConfig::new_inherent().with_input("10"),
        r#"Write the maximum system value: {"result":{"Ok":"False"},"stats":{"num_refinements":11,"num_generated_states":36,"num_final_states":14,"num_generated_transitions":47,"num_final_transitions":26,"inherent_panic_message":"not yet implemented: Zero the next value when it is greater than max value"}}"#,
    );

    // chapter 3.1e: checking inherent where field max_value is set to 15, inherent should hold
    test_example(
        "book_instances_todo",
        TestConfig::new_inherent().with_input("15"),
        r#"Write the maximum system value: {"result":{"Ok":"True"},"stats":{"num_refinements":0,"num_generated_states":6,"num_final_states":5,"num_generated_transitions":6,"num_final_transitions":6,"inherent_panic_message":null}}"#,
    );
}

#[test]
fn book_ch9_parametric() {
    // chapter 9a: whether the property holds depends on parameters
    test_example(
        "parametric",
        TestConfig::new_property("AG![value == 0]"),
        r#"{"result":{"Ok":"Dependent"},"stats":{"num_refinements":5,"num_generated_states":69,"num_final_states":33,"num_generated_transitions":96,"num_final_transitions":50,"inherent_panic_message":null}}"#,
    );

    // chapter 9b: a property that does not hold independently of parameters
    test_example(
        "parametric",
        TestConfig::new_property("AF![as_unsigned(value) > 0]"),
        r#"{"result":{"Ok":"False"},"stats":{"num_refinements":64,"num_generated_states":368,"num_final_states":151,"num_generated_transitions":784,"num_final_transitions":287,"inherent_panic_message":null}}"#,
    );
}

#[test]
fn book_ch10_mu_calculus() {
    // chapter 10a: holds in even positions
    test_example(
        "mu_even",
        TestConfig::new_property("gfp![Z, value == 0 && AX![AX![Z]]]"),
        r#"{"result":{"Ok":"True"},"stats":{"num_refinements":0,"num_generated_states":3,"num_final_states":2,"num_generated_transitions":3,"num_final_transitions":3,"inherent_panic_message":null}}"#,
    );

    // chapter 10b: whether holds eventually forever
    test_example(
        "mu_infinitely_often",
        TestConfig::new_property("AF![AG![p == 1]]"),
        r#"{"result":{"Ok":"False"},"stats":{"num_refinements":2,"num_generated_states":10,"num_final_states":4,"num_generated_transitions":12,"num_final_transitions":7,"inherent_panic_message":null}}"#,
    );

    // chapter 10c: whether holds infinitely often
    // TODO: THIS IS THE WRONG PROPERTY FOR AFG[p == 1], which we want to verify
    // ... but this property gives the result as in the book ...
    test_example(
        "mu_infinitely_often",
        TestConfig::new_property("gfp![Y, lfp![X, (p == 1 && EX![Y]) || EX![X]]]"),
        r#"{"result":{"Ok":"True"},"stats":{"num_refinements":2,"num_generated_states":10,"num_final_states":4,"num_generated_transitions":12,"num_final_transitions":7,"inherent_panic_message":null}}"#,
    );
}
