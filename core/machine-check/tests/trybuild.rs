//! Tests here test whether the machine_description macro parses
//! or fails to parse various constructs.
//!
//! The tests for this must be here, not in machine-check-macros,
//! as the namespace machine_check is used within the constructs.

#[test]
fn machine_description() {
    let t = trybuild::TestCases::new();
    t.pass("tests/cases/01-items.rs");
    t.compile_fail("tests/cases/02-unsupported-item.rs");
    t.compile_fail("tests/cases/03-struct-generics.rs");
    t.pass("tests/cases/04-struct-accept-attributes.rs");
    t.compile_fail("tests/cases/05-struct-arbitrary-attribute.rs");
    t.compile_fail("tests/cases/06-struct-members.rs");
    t.compile_fail("tests/cases/07-impl-unsupported.rs");
    t.compile_fail("tests/cases/08-fn-unsupported.rs");
    t.compile_fail("tests/cases/09-stmt-unsupported.rs");
    t.compile_fail("tests/cases/10-expr-unsupported.rs");
    t.compile_fail("tests/cases/11-type-unsupported.rs");
    t.compile_fail("tests/cases/12-use-unsupported.rs");
    t.pass("tests/cases/13-nested-if-with-array.rs");
}
