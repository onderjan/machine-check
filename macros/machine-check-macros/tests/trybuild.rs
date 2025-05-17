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
}
